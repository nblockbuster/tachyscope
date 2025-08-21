use tiger_parse::{FnvHash, tiger_tag};
use tiger_pkg::TagHash;

use crate::data::text::IndexedString;

#[tiger_tag(id = 0x80805767, size = 0x18)]
pub struct SActivityTypes {
    pub file_size: u64,
    pub activity_types: Vec<SActivityType>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x8080576B, size = 0x80)]
pub struct SActivityType {
    pub hash: u32,
    #[tag(offset = 0x8)]
    pub name: FnvHash,
    pub unkc: IndexedString,
    pub description: IndexedString,
}
