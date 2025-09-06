use std::{
    collections::HashMap,
    default,
    fmt::Display,
    io::{Cursor, Read, Seek, SeekFrom},
};

use log::{error, info};
use rustc_hash::FxHashMap;
use tiger_parse::{
    FnvHash, PackageManagerExt, Padding, Pointer, ResourcePointer, ResourcePointerWithClass,
    TigerReadable, VariantPointer, tiger_tag, tiger_variant_enum,
};
use tiger_pkg::TagHash;

pub const FNV1_PRIME: u32 = 0x811c9dc5;

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    strum::FromRepr,
    strum::EnumIter,
)]
#[repr(u16)]
pub enum Language {
    #[default]
    English,
    Japanese,
    German,
    French,
    Spanish,
    SpanishLatAm,
    Italian,
    Korean,
    TraditionalChinese,
    SimplifiedChinese,
    Portuguese,
    Polish,
    Russian,
}

impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

#[derive(Clone)]
#[tiger_tag(id = 0x808099EF)]
pub struct SLocalizedStrings {
    pub file_size: u64,
    pub string_hashes: Vec<FnvHash>,
    pub languages: [TagHash; 13],
}

#[derive(Clone)]
#[tiger_tag(id = 0x808099F1, size = 0x48)]
pub struct SStringData {
    pub file_size: u64,
    pub string_parts: Vec<SStringPart>,
    // 2 vecs,
    // colors
    // unk i16
    #[tag(offset = 0x28)]
    pub string_characters: Vec<i8>,
    pub string_combinations: Vec<SStringCombination>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808099F7, size = 0x20)]
pub struct SStringPart {
    pub unk0: u64,
    pub data: Pointer<()>,
    pub variable_hash: FnvHash,
    pub byte_length: u16,
    pub string_length: u16,
    pub cipher_shift: u16,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808099F5)]
pub struct SStringCombination {
    pub data: Pointer<()>,
    pub part_count: i64,
}

#[derive(Debug, Default)]
pub struct LocalizedStrings {
    stringmap: FxHashMap<Language, FxHashMap<u32, String>>,
}

impl LocalizedStrings {
    pub fn load(tag: impl Into<TagHash>) -> anyhow::Result<Self> {
        let mut stringmap = FxHashMap::default();
        let textset_header: SLocalizedStrings =
            tiger_pkg::package_manager().read_tag_struct(tag)?;

        for (i, data_tag) in textset_header.languages.iter().enumerate() {
            let Some(lang) = Language::from_repr(i as u16) else {
                return Err(anyhow::anyhow!("Invalid language found"));
            };

            let mut language_strings = FxHashMap::default();

            let data = tiger_pkg::package_manager().read_tag(*data_tag).unwrap();
            let mut cur = Cursor::new(&data);
            let text_data: SStringData = TigerReadable::read_ds(&mut cur)?;

            for (combination, hash) in text_data
                .string_combinations
                .iter()
                .zip(textset_header.string_hashes.iter())
            {
                let mut final_string = String::new();

                for ip in 0..combination.part_count {
                    cur.seek(SeekFrom::Start(combination.data.offset() as u64))?;
                    cur.seek(SeekFrom::Current(ip * 0x20))?;
                    let part: SStringPart = TigerReadable::read_ds(&mut cur)?;
                    if part.variable_hash != FNV1_PRIME {
                        final_string += &format!("<{:08X}>", part.variable_hash);
                    } else {
                        cur.seek(SeekFrom::Start(part.data.offset() as u64))?;
                        let mut data = vec![0u8; part.byte_length as usize];
                        cur.read_exact(&mut data)?;
                        final_string += &String::from_utf8_lossy(&data);
                    }
                }

                language_strings.insert(*hash, final_string);
            }

            stringmap.insert(lang, language_strings);
        }

        Ok(Self { stringmap })
    }

    pub fn get(&self, lang: &Language, hash: u32) -> Option<&String> {
        if let Some(localized) = self.stringmap.get(lang) {
            localized.get(&hash)
        } else {
            None
        }
    }

    pub fn strings(&self, lang: &Language) -> Option<FxHashMap<u32, String>> {
        self.stringmap.get(lang).cloned()
    }

    pub fn stringmap(&self) -> &FxHashMap<Language, FxHashMap<u32, String>> {
        &self.stringmap
    }
}
