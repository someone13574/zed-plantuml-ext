use crate::binary::Binary;

pub struct Preview;

impl Preview {
    pub const ID: &str = "plantuml-preview";
}

impl Binary for Preview {
    const NAME: &str = "plantuml-preview";
    const DIR_PREFIX: &str = "plantuml-preview-";
}
