use zed_extension_api as zed;

use crate::lsp::Lsp;

mod lsp;

struct PlantUMLExtension {
    lsp: Lsp,
}

impl zed::Extension for PlantUMLExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            lsp: Default::default(),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        match language_server_id.as_ref() {
            Lsp::ID => Ok(zed::Command {
                command: self.lsp.get_binary(language_server_id, worktree)?,
                args: Vec::new(),
                env: Vec::new(),
            }),
            id => Err(format!("unknown language server `{id}`"))?,
        }
    }
}

zed::register_extension!(PlantUMLExtension);
