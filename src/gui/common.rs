use std::{
    fs::File,
    io::{Cursor, Write},
    num::NonZeroU32,
};

use egui::{Color32, RichText};
use image::ImageFormat;
use log::error;
use tiger_investment::{
    InvestmentData,
    data::{
        activity::Activity,
        image::{ColorblindMode, IconContainerType},
        item::InventoryItem,
    },
};
use tiger_pkg::TagHash;

use crate::gui::texture::{Texture, cache::TextureCache, icon_container};

lazy_static::lazy_static! {
    static ref CF_PNG: NonZeroU32 = clipboard_win::register_format("PNG").unwrap();
    static ref CF_FILENAME: NonZeroU32 = clipboard_win::register_format("FileNameW").unwrap();
}

pub trait DisplayUi {
    fn show(&self, texture_cache: TextureCache, hash: u32, ui: &mut egui::Ui);
}

impl DisplayUi for InvestmentData {
    fn show(&self, texture_cache: TextureCache, hash: u32, ui: &mut egui::Ui) {
        match self {
            Self::Activity(act) => act.show(texture_cache, hash, ui),
            Self::InventoryItem(i) => i.show(texture_cache, hash, ui),
            _ => todo!(),
        }
    }
}

impl DisplayUi for Activity {
    fn show(&self, texture_cache: TextureCache, hash: u32, ui: &mut egui::Ui) {
        ui.collapsing(RichText::new("Display Properties").size(15.0), |ui| {
            ui.label(format!(
                "name: \"{}\"",
                self.display
                    .display_properties
                    .name
                    .get()
                    .unwrap_or_default()
            ));
            ui.label(format!(
                "description: \"{}\"",
                self.display
                    .display_properties
                    .description
                    .get()
                    .unwrap_or_default()
            ));
        });
        ui.collapsing(
            RichText::new("Selection Display Properties").size(15.0),
            |ui| {
                ui.label(format!(
                    "name: \"{}\"",
                    self.display
                        .selection_screen_display_properties
                        .name
                        .get()
                        .unwrap_or_default()
                ));
                ui.label(format!(
                    "description: \"{}\"",
                    self.display
                        .selection_screen_display_properties
                        .description
                        .get()
                        .unwrap_or_default()
                ));
            },
        );
        ui.collapsing(RichText::new("Modifiers").size(15.0), |ui| {
            for (i, modifier) in self.display.modifiers.iter().enumerate() {
                ui.collapsing(format!("Modifier {i}"), |ui| {
                    ui.label(format!(
                        "name: \"{}\"",
                        modifier.data.name.get().unwrap_or_default()
                    ));
                    ui.label(format!(
                        "description: \"{}\"",
                        modifier.data.description.get().unwrap_or_default()
                    ));
                });
            }
        });

        ui.collapsing(RichText::new("Insertion Points").size(15.0), |ui| {
            for (i, phase) in self.data.insertion_points.0.phases.iter().enumerate() {
                ui.collapsing(format!("Insertion Point {i}"), |ui| {
                    ui.label(format!("phase: {}", phase.phase_hash));
                    ui.label(format!("unlock_index: {}", phase.unlock_index));
                });
            }
        });
    }
}

impl DisplayUi for InventoryItem {
    fn show(&self, texture_cache: TextureCache, hash: u32, ui: &mut egui::Ui) {
        ui.collapsing(RichText::new("Display Properties").size(15.0), |ui| {
            let Some(icon) = self.icon() else {
                return;
            };

            let mut textures = Vec::new();
            textures.append(
                &mut icon
                    .get_background_textures(Some(ColorblindMode::None))
                    .iter()
                    .map(|x| (*x, IconContainerType::Background))
                    .collect::<Vec<_>>(),
            );
            textures.append(
                &mut icon
                    .get_primary_textures()
                    .iter()
                    .map(|x| (*x, IconContainerType::Primary))
                    .collect::<Vec<_>>(),
            );
            icon_container(
                ui,
                Color32::from_rgba_unmultiplied(
                    (icon.data.background_color[0] * 255.0) as u8,
                    (icon.data.background_color[1] * 255.0) as u8,
                    (icon.data.background_color[2] * 255.0) as u8,
                    (icon.data.background_color[3] * 255.0) as u8,
                ),
                textures.clone(),
                texture_cache.clone(),
                hash,
            );
            ui.collapsing("Icon Layers", |ui| {
                textures.append(
                    &mut icon
                        .get_overlay_textures()
                        .iter()
                        .map(|x| (*x, IconContainerType::Overlay))
                        .collect::<Vec<_>>(),
                );
                for t in &textures {
                    icon_container(ui, Color32::BLACK, vec![*t], texture_cache.clone(), hash);
                }
            });
            ui.label(format!(
                "name: \"{}\"",
                self.display.name.get().unwrap_or_default()
            ));
            ui.label(format!(
                "display_source: \"{}\"",
                self.display.display_source.get().unwrap_or_default()
            ));
            ui.label(format!(
                "type: \"{}\"",
                self.display.item_type.get().unwrap_or_default()
            ));
            ui.label(format!(
                "toast: \"{}\"",
                self.display.toast.get().unwrap_or_default()
            ));
            ui.label(format!(
                "flavor: \"{}\"",
                self.display.flavor.get().unwrap_or_default()
            ));
        });
        if let Some(sockets) = &self.data.sockets.0 {
            ui.collapsing(RichText::new("Sockets").size(15.0), |ui| {
                for (i, socket) in sockets.data.iter().enumerate() {
                    ui.collapsing(format!("Socket {i}"), |ui| {
                        ui.label(format!("socket_index: {}", socket.socket_index));
                        ui.label(format!(
                            "single_initial_item_index: {}",
                            socket.single_initial_item_index
                        ));
                        ui.label(format!(
                            "reusable_plugset_index: {}",
                            socket.reusable_plugset_index
                        ));
                        ui.label(format!(
                            "reusable_plugset_index_2: {}",
                            socket.reusable_plugset_index_2
                        ));
                        egui::CollapsingHeader::new("plug_items")
                            .id_salt(socket.socket_index + socket.reusable_plugset_index)
                            .show(ui, |ui| {
                                for plug in &socket.plug_items {
                                    ui.label(format!("plug_index: {}", plug.plug_index));
                                }
                            });
                    });
                }
            });
        }
        if let Some(stats_perks) = &self.data.stats_perks.0 {
            ui.collapsing(RichText::new("Stats").size(15.0), |ui| {
                for stat in &stats_perks.stats {
                    ui.collapsing(format!("Stat Type {}", stat.stat_type), |ui| {
                        ui.label(format!("type: {}", stat.stat_type));
                        ui.label(format!("value: {}", stat.stat_value));
                    });
                }
            });

            ui.collapsing(RichText::new("Perks").size(15.0), |ui| {
                for perk in &stats_perks.perks {
                    ui.label(format!("sandbox_perk_index: {}", perk.sandbox_perk_index));
                }
            });
        }
    }
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
