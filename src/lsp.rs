use zed_extension_api as zed;

use crate::binary::Binary;

#[derive(Default)]
pub struct Lsp {
    cached_binary: Option<String>,
}

impl Lsp {
    pub const ID: &str = "ptdewey-plantuml-lsp";
}

impl Binary for Lsp {
    const DOWNLOAD_REPO: &str = "someone13574/zed-plantuml-ext"; // no upstream builds at ptdewey/plantuml-lsp
    const DOWNLOAD_TAG: &str = "test";
    const DIR_PREFIX: &str = "plantuml-lsp-";

    fn get_cached_binary(&self) -> Option<String> {
        self.cached_binary.clone()
    }

    fn set_cached_binary(&mut self, cached_binary: Option<String>) {
        self.cached_binary = cached_binary;
    }

    fn binary_name(os: zed::Os) -> &'static str {
        match os {
            zed::Os::Mac | zed::Os::Linux => "plantuml-lsp",
            zed::Os::Windows => "plantuml-lsp.exe",
        }
    }

    fn asset_name(_version: &str, os: zed::Os, arch: zed::Architecture) -> zed::Result<String> {
        Ok(match (os, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "plantuml-lsp-macos-arm64.tar.gz",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "plantuml-lsp-linux-arm64.tar.gz",
            (zed::Os::Linux, zed::Architecture::X8664) => "plantuml-lsp-linux-x64.tar.gz",
            (zed::Os::Windows, zed::Architecture::X8664) => "plantuml-lsp-windows-x64.zip",
            (os, arch) => {
                return Err(format!("architecture {arch:?} not supported on os {os:?}"));
            }
        }
        .to_string())
    }

    fn asset_type(os: zed::Os) -> zed::DownloadedFileType {
        match os {
            zed::Os::Mac | zed::Os::Linux => zed::DownloadedFileType::GzipTar,
            zed::Os::Windows => zed::DownloadedFileType::Zip,
        }
    }
}
