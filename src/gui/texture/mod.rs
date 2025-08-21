use anyhow::Context;
use eframe::{
    egui_wgpu::RenderState,
    wgpu::{self, TextureDimension, TextureFormat, util::DeviceExt},
};
use egui::{Color32, Pos2, RichText, Stroke, Ui, Vec2};
use image::{DynamicImage, GenericImageView};
use tiger_investment::data::image::{IconContainerType, texture::TextureHeader};
use tiger_parse::PackageManagerExt;
use tiger_pkg::{TagHash, package_manager};

use crate::gui::{
    common::ResponseExt,
    texture::{cache::TextureCache, capture::capture_texture},
};

pub mod cache;
pub mod capture;
pub mod overlay;

#[derive(Clone)]
pub struct Texture {
    pub view: wgpu::TextureView,
    pub handle: wgpu::Texture,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn load_data_d2(hash: TagHash) -> anyhow::Result<(TextureHeader, Vec<u8>)> {
        let texture_header_ref = package_manager()
            .get_entry(hash)
            .context("Texture header entry not found")?
            .reference;

        let texture: TextureHeader = package_manager()
            .read_tag_struct(hash)
            .context("Failed to read texture header")?;

        let mut texture_data = if texture.large_buffer.is_some() {
            package_manager()
                .read_tag(texture.large_buffer)
                .context("Failed to read texture data")?
        } else {
            package_manager()
                .read_tag(texture_header_ref)
                .context("Failed to read texture data")?
                .to_vec()
        };

        if texture.large_buffer.is_some() {
            let ab = package_manager()
                .read_tag(texture_header_ref)
                .context("Failed to read large texture buffer")?
                .to_vec();

            texture_data.extend(ab);
        }

        Ok((texture, texture_data))
    }

    pub fn load(rs: &RenderState, hash: TagHash) -> anyhow::Result<Texture> {
        let (texture, texture_data) = Self::load_data_d2(hash)?;
        Self::create_texture(
            rs,
            hash,
            texture.format.to_wgpu()?,
            texture.width as u32,
            texture.height as u32,
            texture_data,
        )
    }

    /// Create a wgpu texture from unswizzled texture data
    fn create_texture(
        rs: &RenderState,
        hash: TagHash,
        format: TextureFormat,
        width: u32,
        height: u32,
        mut data: Vec<u8>,
    ) -> anyhow::Result<Texture> {
        // Pre-multiply alpha where possible
        if matches!(
            format,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb
        ) {
            for c in data.chunks_exact_mut(4) {
                c[0] = (c[0] as f32 * c[3] as f32 / 255.) as u8;
                c[1] = (c[1] as f32 * c[3] as f32 / 255.) as u8;
                c[2] = (c[2] as f32 * c[3] as f32 / 255.) as u8;
            }
        }

        let image_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        {
            let block_size = format.block_copy_size(None).unwrap_or(4);
            let (block_width, block_height) = format.block_dimensions();
            let physical_size = image_size.physical_size(format);
            let width_blocks = physical_size.width / block_width;
            let height_blocks = physical_size.height / block_height;

            let bytes_per_row = width_blocks * block_size;
            let expected_data_size =
                bytes_per_row * height_blocks * image_size.depth_or_array_layers;

            anyhow::ensure!(
                data.len() >= expected_data_size as usize,
                "Not enough data for texture {hash}: expected 0x{:X}, got 0x{:X}",
                expected_data_size,
                data.len()
            );
        }

        let handle = rs.device.create_texture_with_data(
            &rs.queue,
            &wgpu::TextureDescriptor {
                label: Some(&*format!("Texture {hash}")),
                size: wgpu::Extent3d {
                    depth_or_array_layers: 1,
                    ..image_size
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[format],
            },
            wgpu::util::TextureDataOrder::default(),
            &data,
        );

        let view = handle.create_view(&wgpu::TextureViewDescriptor {
            ..Default::default()
        });

        Ok(Texture {
            view,
            handle,
            format,
            width,
            height,
        })
    }

    fn load_png(render_state: &RenderState, bytes: &[u8]) -> anyhow::Result<Texture> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();
        Self::create_texture(
            render_state,
            TagHash::NONE,
            wgpu::TextureFormat::Rgba8Unorm,
            width,
            height,
            rgba.into_raw(),
        )
    }

    pub fn to_image(&self, rs: &RenderState) -> anyhow::Result<DynamicImage> {
        let (rgba_data, padded_width, padded_height) = capture_texture(rs, self)?;
        let image = image::RgbaImage::from_raw(padded_width, padded_height, rgba_data)
            .context("Failed to create image")?;

        Ok(DynamicImage::from(image).crop(0, 0, self.width, self.height))
    }
}
/// `hashes` is a vec of texture TagHashes, the in the order to render them. (background - primary - overlay)
pub fn icon_container(
    ui: &mut Ui,
    background_color: Color32,
    hashes: Vec<(TagHash, IconContainerType)>,
    texture_cache: TextureCache,
    api_hash: u32,
) {
    let img_container = ui.allocate_response(Vec2::new(96.0, 96.0), egui::Sense::click());

    let img_rect = img_container.rect;
    let painter = ui.painter_at(img_rect);
    painter.rect_filled(img_rect, 0.0, background_color);

    if ui.is_rect_visible(img_rect) {
        for (hash, t) in &hashes {
            let (_, tid) = texture_cache.get_or_default(*hash);
            let mut mesh = egui::Mesh::with_texture(tid);
            mesh.add_rect_with_uv(
                img_rect,
                egui::Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.0)),
                egui::Color32::WHITE,
            );

            img_container
                .clone()
                .on_hover_text(RichText::new(format!("{hash}: {t}")).strong());

            painter.add(egui::Shape::mesh(mesh));
        }
        if img_container.hovered() {
            ui.painter().rect_stroke(
                img_rect,
                0.0,
                Stroke::new(1.0, Color32::WHITE),
                egui::StrokeKind::Outside,
            );
        }
        img_container.tag_context_with_texture(hashes.clone(), &texture_cache, api_hash);
    }
}
