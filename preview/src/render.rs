use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use crossbeam_channel::Receiver;

pub struct Job {
    pub path: PathBuf,
    pub text: String,
}

pub fn run(plantuml: PathBuf, jobs: Receiver<Job>) {
    while let Ok(job) = jobs.recv() {
        let mut latest = HashMap::from([(job.path, job.text)]);
        for job in jobs.try_iter() {
            latest.insert(job.path, job.text);
        }

        for (path, text) in latest {
            if let Err(err) = render(&plantuml, &path, text) {
                eprintln!("failed to render `{}`: {err}", path.display());
            }
        }
    }
}

fn render(plantuml: &Path, path: &Path, text: String) -> io::Result<()> {
    const DELIMITER: &str = "___PLANTUML_PREVIEW_END___";

    let (Some(dir), Some(stem)) = (path.parent(), path.file_stem()) else {
        return Err(io::Error::other("not a file path"));
    };
    let stem = stem.to_string_lossy();

    let mut command = Command::new(plantuml);
    command
        .args([
            "-pipe",
            "-tsvg",
            "-charset",
            "UTF-8",
            "-pipedelimitor",
            DELIMITER,
        ])
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let svgs = stdout
        .split(DELIMITER)
        .map(str::trim)
        .filter(|svg| !svg.is_empty());
    for (idx, svg) in svgs.enumerate() {
        let name = match idx {
            0 => format!("{stem}.svg"),
            idx => format!("{stem}_{idx:03}.svg"),
        };
        write_if_changed(&dir.join(name), svg)?;
    }

    Ok(())
}

fn write_if_changed(path: &Path, contents: &str) -> io::Result<()> {
    if fs::read(path).is_ok_and(|existing| existing == contents.as_bytes()) {
        return Ok(());
    }

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    let temp = path.with_file_name(format!(".{file_name}.tmp"));
    fs::write(&temp, contents)?;
    fs::rename(&temp, path)
}
