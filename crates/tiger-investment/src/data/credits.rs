use tiger_parse::{FnvHash, NullString, Pointer, tiger_tag};
use tiger_tag::{WideHash, WideTag};

#[derive(Clone)]
#[tiger_tag(id = 0x80807BF2, size = 0x28)]
pub struct S80807BF2 {
    pub file_size: u64,
    pub stringtag_1: WideHash,
    pub strings: Vec<WideHash>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803AFA, size = 0x28)]
pub struct SCreditsContainer {
    pub file_size: u64,
    pub companies: Vec<SCreditsCompany>,
    pub string_containers: WideTag<S80807BF2>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803AFC, size = 0x18)]
pub struct SCreditsCompany {
    pub name: FnvHash,
    pub name_2: FnvHash,
    pub categories: Vec<SCreditsCategory>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803AFE, size = 0x18)]
pub struct SCreditsCategory {
    pub name: FnvHash,
    pub name_2: FnvHash,
    pub names: Vec<Pointer<NullString>>,
}
