use std::{
    collections::HashMap,
    env, fs,
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    thread,
    time::Instant,
};

use crossbeam_channel::{Receiver, Sender};
use lsp_server::Message;
use lsp_types::MessageType;

use crate::log;

pub struct Job {
    pub path: PathBuf,
    pub text: String,
}

struct Outcome {
    written: Vec<PathBuf>,
    unchanged: usize,
    status: ExitStatus,
    stderr: String,
}

pub fn run(plantuml: PathBuf, jobs: Receiver<Job>, logger: Sender<Message>) {
    while let Ok(job) = jobs.recv() {
        let mut received = 1;
        let mut latest = HashMap::from([(job.path, job.text)]);
        for job in jobs.try_iter() {
            received += 1;
            latest.insert(job.path, job.text);
        }
        if received > latest.len() {
            log(
                &logger,
                MessageType::LOG,
                format!("skipped {} superseded render(s)", received - latest.len()),
            );
        }

        for (path, text) in latest {
            let start = Instant::now();
            match render(&plantuml, &path, text) {
                Ok(outcome) => {
                    log(
                        &logger,
                        MessageType::INFO,
                        format!(
                            "rendered `{}` in {} ms: {} written, {} unchanged",
                            path.display(),
                            start.elapsed().as_millis(),
                            outcome.written.len(),
                            outcome.unchanged,
                        ),
                    );
                    for svg in outcome.written {
                        log(
                            &logger,
                            MessageType::LOG,
                            format!("wrote `{}`", svg.display()),
                        );
                    }
                    if !outcome.status.success() {
                        log(
                            &logger,
                            MessageType::WARNING,
                            format!(
                                "plantuml exited with {} for `{}`: {}",
                                outcome.status,
                                path.display(),
                                outcome.stderr.trim(),
                            ),
                        );
                    }
                }
                Err(err) => log(
                    &logger,
                    MessageType::ERROR,
                    format!("failed to render `{}`: {err}", path.display()),
                ),
            }
        }
    }
}

pub fn png_path(source: &Path, idx: usize) -> Option<PathBuf> {
    let stem = source.file_stem()?.to_string_lossy();
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);

    let name = match idx {
        0 => format!("{stem}.png"),
        idx => format!("{stem}_{idx:03}.png"),
    };
    Some(
        env::temp_dir()
            .join("plantuml-preview")
            .join(format!("{:016x}", hasher.finish()))
            .join(name),
    )
}

fn render(plantuml: &Path, path: &Path, text: String) -> io::Result<Outcome> {
    const DELIMITER: &str = "___PLANTUML_PREVIEW_END___";

    let dir = path
        .parent()
        .ok_or_else(|| io::Error::other("not a file path"))?;

    let mut command = Command::new(plantuml);
    command
        .args([
            "-pipe",
            "-tpng",
            "-charset",
            "UTF-8",
            "-pipedelimitor",
            DELIMITER,
        ])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = command.spawn()?;
    let mut stdin = child.stdin.take().expect("stdin is piped");
    let writer = thread::spawn(move || stdin.write_all(text.as_bytes()));
    let output = child.wait_with_output()?;
    writer.join().expect("stdin writer panicked")?;

    let mut outcome = Outcome {
        written: Vec::new(),
        unchanged: 0,
        status: output.status,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    };
    let mut rest = output.stdout.as_slice();
    let mut pngs = Vec::new();
    while let Some(end) = rest
        .windows(DELIMITER.len())
        .position(|window| window == DELIMITER.as_bytes())
    {
        pngs.push(&rest[..end]);
        rest = &rest[end + DELIMITER.len()..];
    }
    pngs.push(rest);

    let pngs = pngs
        .into_iter()
        .map(<[u8]>::trim_ascii)
        .filter(|png| !png.is_empty());
    for (idx, png) in pngs.enumerate() {
        let png_path = png_path(path, idx).ok_or_else(|| io::Error::other("not a file path"))?;
        if write_if_changed(&png_path, png)? {
            outcome.written.push(png_path);
        } else {
            outcome.unchanged += 1;
        }
    }

    Ok(outcome)
}

fn write_if_changed(path: &Path, contents: &[u8]) -> io::Result<bool> {
    if fs::read(path).is_ok_and(|existing| existing == contents) {
        return Ok(false);
    }

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, contents)?;
    Ok(true)
}
