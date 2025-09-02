use tiger_parse::{Pointer, tiger_tag};

use crate::data::text::IndexedString;

pub mod graph;
pub mod interactables;
pub mod skulls;
pub mod types;

#[derive(Clone)]
pub struct Activity {
    pub data: SActivityData,
    pub display: SActivityDisplayData,
}

impl Activity {
    pub fn new(data: SActivityData, display: SActivityDisplayData) -> Self {
        Self { data, display }
    }
}

#[tiger_tag(id = 0x8080718D, size = 0x18)]
pub struct SActivityList {
    pub file_size: u64,
    pub activities: Vec<S808070A8>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808070A8, size = 0x10)]
pub struct S808070A8 {
    pub hash: u32,
    #[tag(offset = 0x8)]
    pub activity: Pointer<SActivityData>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808072A9, size = 0x1A8)]
pub struct SActivityData {
    pub hash: u32,
    #[tag(offset = 0x8)]
    pub unk8: Vec<i32>,
    #[tag(offset = 0x20)]
    pub matchmaking: Pointer<S808072F0>, //808072f0, 0x58
    #[tag(offset = 0x38)]
    pub unk38: Pointer<()>, //808072df, 0x58
    pub unk40: Pointer<()>,                   //808072da, 0x18
    pub insertion_points: Pointer<S808072FE>, //808072fe, 0x28, insertions: phase, unk index, unlockhash
    #[tag(offset = 0xa8)]
    pub unka8: Vec<()>,
    pub skulls: Vec<S8080B6FE>,
    #[tag(offset = 0x100)]
    pub unk100: Vec<()>,
    #[tag(offset = 0x128)]
    pub unk128: Vec<()>,
    #[tag(offset = 0x188)]
    pub trait_indices: Vec<i16>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808072F0, size = 0x58)]
pub struct S808072F0 {
    pub unk0: IndexedString,
    pub unk8: u32,
    pub min_party: u32,
    pub max_party: u32,
    pub max_players: u32,
}

// #[derive(Clone)]
// #[tiger_tag(id = 0x808072DF, size = 0x58)]
// pub struct S808072DF {
//     #[tag(offset = 0x8)]
//     pub unk8: Vec<Vec<(u32, u32)>>,
// }

// #[derive(Clone)]
// #[tiger_tag(id = 0x808072DA, size = 0x18)]
// pub struct S808072DA {
//     #[tag(offset = 0x8)]
//     pub unk8: Vec<()>,
// }

// #[derive(Clone)]
// #[tiger_tag(id = 0x808072DC, size = 0x18)]
// pub struct S808072DC {
//     pub unk0: Vec<()>,
//     pub unk10: u16,
// }

#[derive(Clone)]
#[tiger_tag(id = 0x808072FE, size = 0x28)]
pub struct S808072FE {
    #[tag(offset = 0x8)]
    pub phases: Vec<S80807300>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80807300, size = 0xC)]
pub struct S80807300 {
    pub phase_hash: u32,
    pub unlock_index: u32,
    pub unk_hash: u32,
}

#[derive(Clone)]
#[tiger_tag(id = 0x8080B6FE, size = 0x48)]
pub struct S8080B6FE {
    pub hash: u32,
    #[tag(offset = 0x28)]
    pub skull_options: Vec<Pointer<()>>,
}

#[tiger_tag(id = 0x808055e2, size = 0x18)]
pub struct SActivityDisplayList {
    pub file_size: u64,
    pub activities: Vec<S808055EC>,
}

#[tiger_tag(id = 0x808055ec, size = 0x10)]
pub struct S808055EC {
    pub hash: u32,
    #[tag(offset = 0x8)]
    pub activity: Pointer<SActivityDisplayData>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808055ef, size = 0x128)]
pub struct SActivityDisplayData {
    #[tag(offset = 0x18)]
    pub display_properties: Pointer<S808055FB>,
    pub selection_screen_display_properties: Pointer<S808055F9>,
    pub requirements: Pointer<S80805691>,
    pub rewards: Pointer<S80805622>,
    #[tag(offset = 0x48)]
    pub unk40: Pointer<()>, // 80803763
    #[tag(offset = 0x70)]
    pub modifiers: Vec<S808055F7>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808055fb, size = 0x28)]
pub struct S808055FB {
    #[tag(offset = 0x4)]
    pub name: IndexedString,
    pub description: IndexedString,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808055f9, size = 0x28)]
pub struct S808055F9 {
    pub name: IndexedString,
    pub description: IndexedString,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805691, size = 0x30)]
pub struct S80805691 {
    pub unk0: Vec<()>,
    pub unk10: Vec<S80805694>, // 80805694
    pub unk20: Vec<S80805694>, // 80805694
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805694, size = 0x28)]
pub struct S80805694 {
    pub unk0: u32,
    pub unk4: IndexedString,
    pub unk8: IndexedString,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805694, size = 0x28)]
pub struct S80805622 {
    pub rewards: Vec<S80805624>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805624, size = 0x18)]
pub struct S80805624 {
    pub unk0: u64,
    pub items: Vec<S80805626>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805626, size = 0x20)]
pub struct S80805626 {
    pub item_index: i16,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808055f7, size = 0x18)]
pub struct S808055F7 {
    pub data: Pointer<S808059EE>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808059ee, size = 0x18)]
pub struct S808059EE {
    pub icon_index: i16,
    #[tag(offset = 0x4)]
    pub name: IndexedString,
    pub description: IndexedString,
}
