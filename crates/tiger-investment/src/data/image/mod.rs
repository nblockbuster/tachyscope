use glam::Vec4;
use tiger_parse::{OptionalVariantPointer, PackageManagerExt, Pointer, VariantPointer, tiger_tag};
use tiger_pkg::{TagHash, package_manager};
use tiger_tag::{OptionalTag, Tag, WideHash, WideTag};

mod dxgi;
pub mod texture;

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
pub enum ColorblindMode {
    None,
    Deuteranopia,
    Protanopia,
    Tritanopia,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
pub enum IconContainerType {
    Background,
    Primary,
    Overlay,
}
#[tiger_tag(id = 0x80805A01, size = 0x18)]
pub struct SInvestmentIcons {
    pub file_size: u64,
    pub icons: Vec<S80805A07>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80805A07, size = 0x20)]
pub struct S80805A07 {
    pub hash: u32,
    #[tag(offset = 0x10)]
    pub icon: WideHash, // <SInvestmentIcon>,
}

#[derive(Clone)]
pub struct InvestmentIcon {
    pub data: SInvestmentIcon,
}

impl InvestmentIcon {
    pub fn new(tag: impl Into<TagHash>) -> anyhow::Result<Self> {
        Ok(Self {
            data: package_manager().read_tag_struct::<SInvestmentIcon>(tag)?,
        })
    }

    pub fn get_background_textures(&self, colorblind_mode: Option<ColorblindMode>) -> Vec<TagHash> {
        if let Some(background) = &self.data.background.0
            && let Some(texhash) = match *background.data {
                ref res => match res {
                    IconContainerData::S80803DCB(cb) => {
                        let mut d_iter = cb.data.iter();
                        match colorblind_mode {
                            Some(ColorblindMode::None) => d_iter.nth(0).map(|x| x.textures.clone()),
                            Some(ColorblindMode::Deuteranopia) => {
                                d_iter.nth(1).map(|x| x.textures.clone())
                            }
                            Some(ColorblindMode::Protanopia) => {
                                d_iter.nth(2).map(|x| x.textures.clone())
                            }
                            Some(ColorblindMode::Tritanopia) => {
                                d_iter.nth(3).map(|x| x.textures.clone())
                            }
                            None => {
                                Some(d_iter.flat_map(|x| x.textures.clone()).collect::<Vec<_>>())
                            }
                        }
                    }
                    IconContainerData::S80803ECD(cd) => {
                        let mut d_iter = cd.data.iter();
                        match colorblind_mode {
                            Some(ColorblindMode::None) => d_iter.nth(0).map(|x| x.textures.clone()),
                            Some(ColorblindMode::Deuteranopia) => {
                                d_iter.nth(1).map(|x| x.textures.clone())
                            }
                            Some(ColorblindMode::Protanopia) => {
                                d_iter.nth(2).map(|x| x.textures.clone())
                            }
                            Some(ColorblindMode::Tritanopia) => {
                                d_iter.nth(3).map(|x| x.textures.clone())
                            }
                            None => {
                                Some(d_iter.flat_map(|x| x.textures.clone()).collect::<Vec<_>>())
                            }
                        }
                    }
                },
                _ => unreachable!(),
            }
        {
            texhash
        } else {
            Vec::new()
        }
    }

    pub fn get_primary_textures(&self) -> Vec<TagHash> {
        if let Some(primary) = &self.data.primary.0
            && let Some(texhash) = match *primary.data {
                ref res => match res {
                    IconContainerData::S80803DCB(cb) => Some(
                        cb.data
                            .iter()
                            .flat_map(|f| f.textures.clone())
                            .collect::<Vec<_>>(),
                    ),
                    IconContainerData::S80803ECD(cd) => Some(
                        cd.data
                            .iter()
                            .flat_map(|f| f.textures.clone())
                            .collect::<Vec<_>>(),
                    ),
                },
                _ => unreachable!(),
            }
        {
            texhash
        } else {
            Vec::new()
        }
    }

    pub fn get_overlay_textures(&self) -> Vec<TagHash> {
        if let Some(overlay) = &self.data.overlay.0
            && let Some(texhash) = match *overlay.data {
                ref res => match res {
                    IconContainerData::S80803DCB(cb) => Some(
                        cb.data
                            .iter()
                            .flat_map(|f| f.textures.clone())
                            .collect::<Vec<_>>(),
                    ),
                    IconContainerData::S80803ECD(cd) => Some(
                        cd.data
                            .iter()
                            .flat_map(|f| f.textures.clone())
                            .collect::<Vec<_>>(),
                    ),
                },
                _ => unreachable!(),
            }
        {
            texhash
        } else {
            Vec::new()
        }
    }
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803EB8, size = 0x80)]
pub struct SInvestmentIcon {
    pub file_size: u64,
    #[tag(offset = 0x10)]
    pub hash: u32,
    pub primary: OptionalTag<S80803ECF>,
    #[tag(offset = 0x20)]
    pub background: OptionalTag<S80803ECF>,
    pub overlay: OptionalTag<S80803ECF>,
    // TODO: figure out why glam::Vec4 doesnt work
    #[tag(offset = 0x30)]
    pub background_color: [f32; 4],
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803ECF, size = 0x18)]
pub struct S80803ECF {
    pub file_size: u64,
    #[tag(offset = 0x10)]
    pub data: VariantPointer<IconContainerData>,
}

tiger_parse::tiger_variant_enum! {
    #[derive(Clone)]
    enum IconContainerData {
        S80803ECD,
        S80803DCB
    }
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803ECD, size = 0x20)]
pub struct S80803ECD {
    pub data: Vec<S80803ED2>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803ED2, size = 0x10)]
pub struct S80803ED2 {
    pub textures: Vec<TagHash>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803ECB, size = 0x20)]
pub struct S80803DCB {
    pub data: Vec<S80803ED0>,
}

#[derive(Clone)]
#[tiger_tag(id = 0x80803ED0, size = 0x10)]
pub struct S80803ED0 {
    pub textures: Vec<TagHash>,
}
