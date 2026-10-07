use crate::binary::Binary;

pub struct Lsp;

impl Lsp {
    pub const ID: &str = "ptdewey-plantuml-lsp";
}

// no upstream builds at ptdewey/plantuml-lsp
impl Binary for Lsp {
    const NAME: &str = "plantuml-lsp";
    const DIR_PREFIX: &str = "plantuml-lsp-";
}
