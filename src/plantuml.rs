use zed_extension_api as zed;

use crate::binary::Binary;

pub struct Plantuml;

impl Binary for Plantuml {
    const NAME: &str = "plantuml";
    const DIR_PREFIX: &str = "plantuml-native-";
    const DOWNLOAD_REPO: &str = "plantuml/plantuml";
    const DOWNLOAD_TAG: &str = "v1.2026.8";

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
