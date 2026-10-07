use std::fs;

use zed_extension_api as zed;

#[derive(Default)]
pub struct Lsp {
    cached_binary: Option<String>,
}

impl Lsp {
    pub const ID: &str = "ptdewey-plantuml-lsp";
    const DOWNLOAD_REPO: &str = "someone13574/zed-plantuml-ext"; // no upstream builds at ptdewey/plantuml-lsp
    const DOWNLOAD_TAG: &str = "v0.0.1";

    pub fn get_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        if let Some(path) = &self.cached_binary {
            if !fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
                self.cached_binary = None;
            } else {
                return Ok(path.to_string());
            }
        }

        let (os, arch) = zed::current_platform();
        if let Some(path) = worktree.which(Self::binary_name(os)) {
            self.cached_binary = Some(path);
        } else {
            self.cached_binary = Some(Self::download_binary(language_server_id, os, arch)?);
        }

        Ok(self.cached_binary.clone().unwrap())
    }

    fn download_binary(
        language_server_id: &zed::LanguageServerId,
        os: zed::Os,
        arch: zed::Architecture,
    ) -> zed::Result<String> {
        let version_dir = Self::version_dir();
        let binary_path = format!("{version_dir}/{}", Self::binary_name(os));
        if fs::metadata(&binary_path).is_ok_and(|metadata| metadata.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );

            return Ok(binary_path);
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::github_release_by_tag_name(Self::DOWNLOAD_REPO, Self::DOWNLOAD_TAG)
            .map_err(|err| format!("failed to find release tag for `{}`: {err}", Self::ID))?;
        let asset_name = Self::asset_name(os, arch)?;
        let asset = release
            .assets
            .into_iter()
            .find(|asset| asset.name == asset_name)
            .ok_or(format!("no asset found matching `{asset_name}`"))?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );
        zed::download_file(&asset.download_url, &version_dir, Self::asset_type(os)?)
            .map_err(|err| format!("failed to download file `{}`: {err}", asset.download_url))?;
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        // Remove binaries from previous versions
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("plantuml-lsp-") && name != version_dir {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        Ok(binary_path)
    }

    fn version_dir() -> String {
        format!("plantuml-lsp-{}", Self::DOWNLOAD_TAG)
    }

    fn binary_name(os: zed::Os) -> &'static str {
        match os {
            zed::Os::Mac | zed::Os::Linux => "plantuml-lsp",
            zed::Os::Windows => "plantuml-lsp.exe",
        }
    }

    fn asset_name(os: zed::Os, arch: zed::Architecture) -> zed::Result<&'static str> {
        Ok(match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "plantuml-lsp-macos-arm64.tar.gz",
            (zed::Os::Mac, zed::Architecture::X8664) => "plantuml-lsp-macos-x64.tar.gz",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "plantuml-lsp-linux-arm64.tar.gz",
            (zed::Os::Linux, zed::Architecture::X8664) => "plantuml-lsp-linux-x64.tar.gz",
            (zed::Os::Windows, zed::Architecture::X8664) => "plantuml-lsp-windows-x64.zip",
            (os, arch) => {
                return Err(format!("architecture {arch:?} not supported on os {os:?}"));
            }
        })
    }

    fn asset_type(os: zed::Os) -> zed::Result<zed::DownloadedFileType> {
        Ok(match os {
            zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
            zed::Os::Windows => zed::DownloadedFileType::Zip,
        })
    }
}
