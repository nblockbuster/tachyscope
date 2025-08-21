use tiger_parse::{Pointer, PointerOptional, tiger_tag};
use tiger_tag::{WideHash, WideTag};

use crate::{
    data::{
        image::{InvestmentIcon, SInvestmentIcon},
        text::IndexedString,
    },
    global_instance::investment_manager,
};

#[derive(Clone)]
pub struct InventoryItem {
    pub data: SInventoryItem,
    pub display: SInventoryItemDisplay,
}
impl InventoryItem {
    pub fn new(data: SInventoryItem, display: SInventoryItemDisplay) -> Self {
        Self { data, display }
    }

    pub fn icon(&self) -> Option<InvestmentIcon> {
        investment_manager()
            .icons()
            .get(self.display.icon_index as usize)
            .map(InvestmentIcon::new)
    }
}

#[derive(Clone)]
#[tiger_tag(id = 0x80807997, size = 0x28)]
pub struct SItemList {
    pub file_size: u64,
    pub items: Vec<S8080799B>,
    pub unk: Vec<()>, // weird i16 vec
}

#[derive(Clone)]
#[tiger_tag(id = 0x8080799B, size = 0x20)]
pub struct S8080799B {
    pub hash: u32,
    #[tag(offset = 0x10)]
    pub item: WideTag<SInventoryItem>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805499, size = 0x18)]
pub struct SInventoryItemDisplayList {
    pub file_size: u64,
    pub stringmap: Vec<S8080549D>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x8080549D, size = 0x20)]
pub struct S8080549D {
    pub hash: u32,
    #[tag(offset = 0x10)]
    pub string_tag: WideTag<SInventoryItemDisplay>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x8080549F, size = 0x138)]
pub struct SInventoryItemDisplay {
    pub file_size: u64,
    pub unk8: Pointer<()>,
    pub unk10: Pointer<()>,
    pub unk18: Pointer<()>,
    pub unk20: Pointer<()>,
    pub unk28: Pointer<()>,
    #[tag(offset = 0x48)]
    pub unk48: Pointer<()>,
    #[tag(offset = 0x60)]
    pub unk60: Pointer<()>,
    pub unk68: Pointer<()>,
    #[tag(offset = 0x78)]
    pub icon_index: i16,
    pub foundry_index: i16,
    pub unk7c_index: i16,
    pub emblem_icon_index: i16,
    pub name: IndexedString,
    pub unk88: u32,
    pub item_type: IndexedString,
    pub display_source: IndexedString,
    pub toast: IndexedString,
    pub flavor: IndexedString,
    pub unkac: u32,
    pub unkb0: Vec<()>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x8080799D, size = 0x120)]
pub struct SInventoryItem {
    pub file_size: u64,
    pub unk8: PointerOptional<()>,
    #[tag(offset = 0x18)]
    pub unk18: PointerOptional<()>,
    #[tag(offset = 0x48)]
    pub unk48: PointerOptional<()>,
    #[tag(offset = 0x60)]
    pub sockets: PointerOptional<S808077C0>,
    pub stats_perks: PointerOptional<S80807381>,
    pub unk70: PointerOptional<()>,
    pub unk78: PointerOptional<()>,
    #[tag(offset = 0x88)]
    pub hash: u32,
    #[tag(offset = 0x98)]
    pub unk98: IndexedString,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808077C0, size = 0x20)]
pub struct S808077C0 {
    pub data: Vec<S808077C3>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808077C3, size = 0x58)]
pub struct S808077C3 {
    pub socket_index: i16,
    #[tag(offset = 0x6)]
    pub single_initial_item_index: i16,
    #[tag(offset = 0x10)]
    pub reusable_plugset_index: i16,
    #[tag(offset = 0x28)]
    pub reusable_plugset_index_2: i16,
    #[tag(offset = 0x48)]
    pub plug_items: Vec<S808077D5>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x808077D5, size = 0x40)]
pub struct S808077D5 {
    #[tag(offset = 0x20)]
    pub plug_index: i16,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80807381, size = 0x30)]
pub struct S80807381 {
    pub stats: Vec<S80807386>,
    pub perks: Vec<S80807387>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80807386, size = 0x30)]
pub struct S80807386 {
    pub stat_type: i32,
    pub stat_value: i32,
    #[tag(offset = 0x28)]
    pub unk28: u64,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80807387, size = 0x18)]
pub struct S80807387 {
    pub sandbox_perk_index: i16,
}
