use zed_extension_api as zed;

use crate::binary::Binary;

#[derive(Default)]
pub struct Plantuml {
    cached_binary: Option<String>,
}

impl Binary for Plantuml {
    const DOWNLOAD_REPO: &str = "plantuml/plantuml";
    const DOWNLOAD_TAG: &str = "v1.2026.8";
    const DIR_PREFIX: &str = "plantuml-native-";

    fn get_cached_binary(&self) -> Option<String> {
        self.cached_binary.clone()
    }

    fn set_cached_binary(&mut self, cached_binary: Option<String>) {
        self.cached_binary = cached_binary;
    }

    fn binary_name(os: zed::Os) -> &'static str {
        match os {
            zed::Os::Mac | zed::Os::Linux => "plantuml",
            zed::Os::Windows => "plantuml.exe",
        }
    }

    fn asset_name(version: &str, os: zed::Os, arch: zed::Architecture) -> zed::Result<String> {
        let platform = match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "macos-arm64",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "linux-arm64",
            (zed::Os::Linux, zed::Architecture::X8664) => "linux-amd64",
            (zed::Os::Windows, zed::Architecture::X8664) => "windows-amd64",
            (os, arch) => {
                return Err(format!("architecture {arch:?} not supported on os {os:?}"));
            }
        };

        Ok(format!(
            "native-plantuml-{platform}-{}.zip",
            version.trim_start_matches('v')
        ))
    }

    fn asset_type(_os: zed::Os) -> zed::DownloadedFileType {
        zed::DownloadedFileType::Zip
    }
}
