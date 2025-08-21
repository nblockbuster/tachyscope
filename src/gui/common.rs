use std::{
    fs::File,
    io::{Cursor, Write},
    num::NonZeroU32,
};

use image::ImageFormat;
use log::error;
use tiger_investment::data::image::IconContainerType;
use tiger_pkg::TagHash;

use crate::gui::texture::{Texture, cache::TextureCache};

lazy_static::lazy_static! {
    static ref CF_PNG: NonZeroU32 = clipboard_win::register_format("PNG").unwrap();
    static ref CF_FILENAME: NonZeroU32 = clipboard_win::register_format("FileNameW").unwrap();
}

pub trait ResponseExt {
    fn tag_context_with_texture(
        self,
        tags: Vec<(TagHash, IconContainerType)>,
        texture_cache: &TextureCache,
        api_hash: u32,
    ) -> Option<egui::InnerResponse<()>>;
}

impl ResponseExt for egui::Response {
    fn tag_context_with_texture(
        self,
        tags: Vec<(TagHash, IconContainerType)>,
        texture_cache: &TextureCache,
        api_hash: u32,
    ) -> Option<egui::InnerResponse<()>> {
        self.context_menu(|ui| {
            #[cfg(target_os = "windows")]
            if ui.selectable_label(false, "📷 Copy texture").clicked() {
                for (tag, icon_type) in &tags {
                    match Texture::load(&texture_cache.render_state, *tag) {
                        Ok(o) => {
                            let image = o.to_image(&texture_cache.render_state).unwrap();
                            let mut png_data = vec![];
                            let mut png_writer = Cursor::new(&mut png_data);
                            image.write_to(&mut png_writer, ImageFormat::Png).unwrap();

                            let _clipboard = clipboard_win::Clipboard::new();
                            if let Err(e) = clipboard_win::raw::set(CF_PNG.get(), &png_data) {
                                error!("Failed to copy texture to clipboard: {e}");
                            }

                            // Save to temp
                            let path =
                                std::env::temp_dir().join(format!("{api_hash}_{icon_type}.png"));
                            let mut file = File::create(&path).unwrap();
                            file.write_all(&png_data).unwrap();

                            let mut path_utf16 =
                                path.to_string_lossy().encode_utf16().collect::<Vec<u16>>();
                            path_utf16.push(0);

                            if let Err(e) = clipboard_win::raw::set_without_clear(
                                CF_FILENAME.get(),
                                bytemuck::cast_slice(&path_utf16),
                            ) {
                                error!("Failed to copy texture path to clipboard: {e}");
                            } else {
                                // TOASTS.lock().success("Texture copied to clipboard");
                            }
                        }
                        Err(e) => {
                            error!("Failed to load texture: {e}");
                        }
                    }
                }
                ui.close();
            }

            if ui
                .selectable_label(false, "📷 Save texture")
                .on_hover_text("Texture(s) will be saved to the textures/ directory")
                .clicked()
            {
                for (tag, icon_type) in &tags {
                    match Texture::load(&texture_cache.render_state, *tag) {
                        Ok(o) => {
                            std::fs::create_dir_all("textures/").unwrap();
                            let image = o.to_image(&texture_cache.render_state).unwrap();
                            image
                                .save(format!("textures/{api_hash}_{icon_type}.png"))
                                .unwrap();
                        }
                        Err(e) => {
                            error!("Failed to load texture: {e}");
                        }
                    }
                }
                ui.close();
            }
        })
    }
}
