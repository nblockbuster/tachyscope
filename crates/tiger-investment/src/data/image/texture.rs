use anyhow::Context;
use eframe::{
    egui_wgpu::RenderState,
    wgpu::{self, TextureDimension, TextureFormat, util::DeviceExt},
};
use tiger_parse::{PackageManagerExt, tiger_tag};
use tiger_pkg::{TagHash, package_manager};

use super::dxgi::DxgiFormat;


#[derive(Clone)]
#[tiger_tag(etype = 32, size = 0x40)]
pub struct TextureHeader {
    pub data_size: u32,
    pub format: DxgiFormat,
    
    #[tag(offset = 0x22)]
    pub width: u16, // prebl: 0xe / bl: 0x22
    pub height: u16,     // prebl: 0x10 / bl: 0x24
    pub depth: u16,      // prebl: 0x12 / bl: 0x26
    pub array_size: u16, // prebl: 0x14 / bl: 0x28
    
    #[tag(offset = 0x3c)]
    pub large_buffer: TagHash, // prebl: 0x24 / bl: 0x3c
}
