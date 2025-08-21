use std::fmt::Debug;

use tiger_parse::{NullString, PackageManagerExt, Pointer, TigerReadable, tiger_tag};
use tiger_pkg::{TagHash, package_manager};
use tiger_tag::{WideHash, WideTag};
use tiger_text::{FNV1_PRIME, Language, SLocalizedStrings};

use crate::global_instance::investment_manager;

#[tiger_tag(id = 0x80805a09, size = 0x18)]
pub struct SIndexedLocalizedStrings {
    pub file_size: u64,
    pub localized_strings: Vec<S80805A0E>,
}

#[tiger_tag(id = 0x80805a0e, size = 0x20)]
pub struct S80805A0E {
    pub container_hash: u32,
    #[tag(offset = 0x8)]
    pub localized_tag: WideHash,
    pub index: u16, // index into 26BA8080 if wide tag isnt correct
}

#[derive(Clone)]
#[tiger_tag(id = 0x80804401, size = 0x1C0)]
pub struct S80804401 {
    pub unk0: u32,
    pub unk4: u32,
    pub unk8: WideHash,
    pub localized_strings: Vec<()>,
    #[tag(offset = 0x4c)]
    pub unk4c: f32,
    pub unk50: f32,
    pub unk54: f32,
    pub unk58: Pointer<NullString>,
    pub unk60: WideHash,
    #[tag(offset = 0x78)]
    pub unk78: Pointer<()>,
    pub unk80: WideHash,
    pub unk90: Pointer<()>,
    pub unk98: WideHash,
    pub unka8: Pointer<NullString>,
    pub unkb0: WideHash,
    pub unkc0: Pointer<NullString>,
    pub unkc8: WideHash,
    pub unkd8: Pointer<NullString>,
    pub unke0: WideHash,
    #[tag(offset = 0x120)]
    pub unk_morestrings: TagHash,
    pub unk128: Vec<()>,
    pub unk138: Vec<()>,
    pub unk148: Vec<()>,
    pub unk158: Vec<()>,
    pub unk168: Vec<()>,
    pub unk178: Vec<()>,
    pub unk188: Vec<()>,
    pub unk198: Vec<()>,
}

#[derive(Debug, Clone)]
pub struct IndexedString(pub u32, pub u32);

impl TigerReadable for IndexedString {
    fn read_ds_endian<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: tiger_parse::Endian,
    ) -> tiger_parse::Result<Self> {
        Ok(Self(
            TigerReadable::read_ds_endian(reader, endian)?,
            TigerReadable::read_ds_endian(reader, endian)?,
        ))
    }

    const SIZE: usize = 8;
}

impl IndexedString {
    pub fn valid(&self) -> bool {
        self.0 != 0xFF_FF || self.1 == FNV1_PRIME
    }

    pub fn get(&self, language: Language) -> Option<String> {
        if !self.valid() {
            return None;
        }
        investment_manager()
            .strings()
            .get_indexed_string(language, self.0, self.1)
    }
}
// TODO: implement display and debug (move lang to investment manager?)
