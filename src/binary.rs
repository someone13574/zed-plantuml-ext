use std::{env, fs};

use zed_extension_api as zed;

pub trait Binary {
    const DOWNLOAD_REPO: &'static str;
    const DOWNLOAD_TAG: &'static str;
    const DIR_PREFIX: &'static str;

    fn get_cached_binary(&self) -> Option<String>;
    fn set_cached_binary(&mut self, cached_binary: Option<String>);

    fn binary_name(os: zed::Os) -> &'static str;
    fn asset_name(version: &str, os: zed::Os, arch: zed::Architecture) -> zed::Result<String>;
    fn asset_type(os: zed::Os) -> zed::DownloadedFileType;

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

    fn get_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        if let Some(path) = self.get_cached_binary() {
            if fs::metadata(&path).is_ok_and(|metadata| metadata.is_file()) {
                return Ok(path);
            }
            self.set_cached_binary(None);
        }

        let (os, arch) = zed::current_platform();
        let path = match worktree.which(Self::binary_name(os)) {
            Some(path) => path,
            None => Self::download_binary(language_server_id, os, arch)?,
        };
        self.set_cached_binary(Some(path.clone()));

        Ok(path)
    }
}
