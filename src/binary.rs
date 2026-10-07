use std::{env, fs, marker::PhantomData};

use zed_extension_api as zed;

pub trait Binary {
    const NAME: &'static str;
    const DIR_PREFIX: &'static str;
    const DOWNLOAD_REPO: &'static str = "someone13574/zed-plantuml-ext";
    const DOWNLOAD_TAG: &'static str = "test5";

    fn binary_name(os: zed::Os) -> String {
        match os {
            zed::Os::Mac | zed::Os::Linux => Self::NAME.to_string(),
            zed::Os::Windows => format!("{}.exe", Self::NAME),
        }
    }

    fn asset_name(_version: &str, os: zed::Os, arch: zed::Architecture) -> zed::Result<String> {
        let platform = match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "macos-arm64.tar.gz",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "linux-arm64.tar.gz",
            (zed::Os::Linux, zed::Architecture::X8664) => "linux-x64.tar.gz",
            (zed::Os::Windows, zed::Architecture::X8664) => "windows-x64.zip",
            (os, arch) => {
                return Err(format!("architecture {arch:?} not supported on os {os:?}"));
            }
        };

        Ok(format!("{}-{platform}", Self::NAME))
    }

    fn asset_type(os: zed::Os) -> zed::DownloadedFileType {
        match os {
            zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
            zed::Os::Windows => zed::DownloadedFileType::Zip,
        }
    }

    fn version_dir() -> String {
        format!("{}{}", Self::DIR_PREFIX, Self::DOWNLOAD_TAG)
    }

    fn download_binary(
        language_server_id: &zed::LanguageServerId,
        os: zed::Os,
        arch: zed::Architecture,
    ) -> zed::Result<String> {
        let version_dir = Self::version_dir();
        let binary_path = format!("{version_dir}/{}", Self::binary_name(os));
        if !fs::metadata(&binary_path).is_ok_and(|metadata| metadata.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::CheckingForUpdate,
            );

            let release = zed::github_release_by_tag_name(Self::DOWNLOAD_REPO, Self::DOWNLOAD_TAG)
                .map_err(|err| {
                    format!(
                        "failed to find release `{}` of `{}`: {err}",
                        Self::DOWNLOAD_TAG,
                        Self::DOWNLOAD_REPO
                    )
                })?;
            let asset_name = Self::asset_name(&release.version, os, arch)?;
            let asset = release
                .assets
                .into_iter()
                .find(|asset| asset.name == asset_name)
                .ok_or(format!("no asset found matching `{asset_name}`"))?;

            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(&asset.download_url, &version_dir, Self::asset_type(os)).map_err(
                |err| format!("failed to download file `{}`: {err}", asset.download_url),
            )?;
            if os != zed::Os::Windows {
                zed::make_file_executable(&binary_path)?;
            }

            // Remove binaries from previous versions
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with(Self::DIR_PREFIX) && name != version_dir {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        let work_dir = env::current_dir()
            .map_err(|err| format!("failed to get extension directory: {err}"))?;
        Ok(work_dir.join(binary_path).to_string_lossy().to_string())
    }
}

pub struct Cached<B> {
    path: Option<String>,
    binary: PhantomData<B>,
}

impl<B> Default for Cached<B> {
    fn default() -> Self {
        Self {
            path: None,
            binary: PhantomData,
        }
    }
}

impl<B: Binary> Cached<B> {
    pub fn get(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        if let Some(path) = &self.path {
            if fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
                return Ok(path.clone());
            }
        }

        let (os, arch) = zed::current_platform();
        let path = match worktree.which(&B::binary_name(os)) {
            Some(path) => path,
            None => B::download_binary(language_server_id, os, arch)?,
        };
        self.path = Some(path.clone());

        Ok(path)
    }
}
