use zed_extension_api as zed;

use crate::binary::Cached;
use crate::lsp::Lsp;
use crate::plantuml::Plantuml;
use crate::preview::Preview;

mod binary;
mod lsp;
mod plantuml;
mod preview;

struct PlantUMLExtension {
    lsp: Cached<Lsp>,
    plantuml: Cached<Plantuml>,
    preview: Cached<Preview>,
}

impl zed::Extension for PlantUMLExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            lsp: Default::default(),
            plantuml: Default::default(),
            preview: Default::default(),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        match language_server_id.as_ref() {
            Lsp::ID => Ok(zed::Command {
                command: self.lsp.get(language_server_id, worktree)?,
                args: vec![format!(
                    "--exec-path={}",
                    self.plantuml.get(language_server_id, worktree)?
                )],
                env: Vec::new(),
            }),
            Preview::ID => Ok(zed::Command {
                command: self.preview.get(language_server_id, worktree)?,
                args: vec![
                    "--plantuml".to_string(),
                    self.plantuml.get(language_server_id, worktree)?,
                ],
                env: Vec::new(),
            }),
            id => Err(format!("unknown language server `{id}`"))?,
        }
    }
}

zed::register_extension!(PlantUMLExtension);
