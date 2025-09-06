use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use egui::{Align2, CollapsingHeader, Color32, CornerRadius, RichText, Stroke, Vec2};
use log::info;
use strum::IntoEnumIterator;
use tiger_investment::{
    InvestmentData,
    data::image::{ColorblindMode, IconContainerData, IconContainerType},
    global_instance::{
        initialize_investment_manager, investment_manager, investment_manager_checked,
    },
    manager::InvestmentManager,
};
use tiger_text::Language;

use crate::gui::{
    common::DisplayUi,
    texture::{cache::TextureCache, icon_container},
};

mod common;
mod texture;

pub struct TachyscopeApp {
    // string_containers: Vec<LocalizedStrings>,
    language: Language,
    results: Vec<InvestmentData>,
    search_send: Arc<crossbeam::channel::Sender<InvestmentData>>,
    search_recv: crossbeam::channel::Receiver<InvestmentData>,
    last_update_time: Instant,
    search_changed: bool,
    search_input: String,
    selected: Vec<InvestmentData>,
    to_deselect: Vec<usize>,
    texture_cache: TextureCache,
}

impl TachyscopeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "NotoSans".into(),
            egui::FontData::from_static(include_bytes!("../../assets/NotoSans.ttf")).into(),
        );
        fonts.font_data.insert(
            "NotoSansJP".into(),
            egui::FontData::from_static(include_bytes!("../../assets/NotoSansJP.ttf")).into(),
        );
        fonts.font_data.insert(
            "NotoSansKR".into(),
            egui::FontData::from_static(include_bytes!("../../assets/NotoSansKR.ttf")).into(),
        );
        fonts.font_data.insert(
            "NotoSansSC".into(),
            egui::FontData::from_static(include_bytes!("../../assets/NotoSansSC.ttf")).into(),
        );
        fonts.font_data.insert(
            "NotoSansTC".into(),
            egui::FontData::from_static(include_bytes!("../../assets/NotoSansTC.ttf")).into(),
        );
        fonts.font_data.insert(
            "Destiny_Keys".into(),
            egui::FontData::from_static(include_bytes!("../../assets/Destiny_Keys.otf")).into(),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "NotoSans".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(1, "NotoSansJP".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(2, "NotoSansKR".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(3, "NotoSansSC".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(4, "NotoSansTC".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(5, "Destiny_Keys".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        let (search_send, search_recv) = crossbeam::channel::bounded(500);

        std::thread::spawn(|| {
            let investment = Arc::new(InvestmentManager::new().unwrap());
            initialize_investment_manager(&investment);
        });

        TachyscopeApp {
            language: Language::English,
            results: Vec::new(),
            search_send: Arc::new(search_send),
            search_recv,
            last_update_time: Instant::now(),
            search_changed: false,
            search_input: String::new(),
            selected: Vec::new(),
            to_deselect: Vec::new(),
            texture_cache: TextureCache::new(cc.wgpu_render_state.clone().unwrap()),
        }
    }
}

// TODO: checkbox grid to filter types
impl eframe::App for TachyscopeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
                egui::widgets::global_theme_preference_buttons(ui);
                ui.add_space(16.0);

                egui::ComboBox::from_id_salt("language_select")
                    .selected_text(self.language.to_string())
                    .show_ui(ui, |ui| {
                        for lang in Language::iter() {
                            if ui
                                .selectable_value(&mut self.language, lang, lang.to_string())
                                .changed()
                            {
                                investment_manager().strings().set_lang(self.language);
                            }
                        }
                    });
            });
        });

        let mut is_loading_investment = false;
        if investment_manager_checked().is_err() {
            {
                let painter = ctx.layer_painter(egui::LayerId::background());
                painter.rect_filled(
                    egui::Rect::EVERYTHING,
                    CornerRadius::default(),
                    Color32::from_black_alpha(127),
                );
            }
            egui::Window::new("Loading Investment System")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("Loading...");
                    ctx.request_repaint();
                });

            is_loading_investment = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!is_loading_investment, |ui| {
                if Instant::now().duration_since(self.last_update_time) > Duration::from_millis(500)
                    && self.search_changed
                {
                    self.search_changed = false;
                    // Explicitly drop it, so the sender stops sending.
                    drop(self.search_send.clone());
                    let (search_send, search_recv) = crossbeam::channel::bounded(500);
                    self.search_send = Arc::new(search_send);
                    self.search_recv = search_recv;
                    self.results.clear();
                    if let Ok(hash) = self.search_input.parse::<u32>() {
                        self.results = investment_manager().search_by_hash(hash);
                    } else {
                        let input = self.search_input.clone();
                        let sender = self.search_send.clone();
                        std::thread::spawn(move || {
                            investment_manager().search_by_name(sender, input)
                        });
                    };
                }

                if !self.search_recv.is_empty()
                    && self.results.len() < 500
                    && let Ok(recv) = self.search_recv.recv()
                {
                    self.results.push(recv);
                }

                ui.horizontal(|ui| {
                    ui.label("Search:");
                    if ui.text_edit_singleline(&mut self.search_input).changed() {
                        self.search_changed = true;
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, result) in self.results.iter().enumerate() {
                        if i > 500 {
                            ui.label("Only 500 results can be shown at a time.");
                            break;
                        }
                        let frame = egui::containers::Frame::new()
                            .outer_margin(egui::Margin::same(8))
                            .inner_margin(egui::Margin::same(4))
                            .corner_radius(egui::CornerRadius::same(2))
                            .stroke(Stroke::new(2.0, Color32::PLACEHOLDER))
                            .show(ui, |ui| {
                                ui.label(result.name());
                                let enum_type: &'static str = result.into();
                                let mut typetext = String::from(enum_type);
                                if let Some(itype) = result.itype() {
                                    typetext += &format!(": {itype}");
                                }
                                ui.label(RichText::new(typetext).italics());
                                ui.label(result.hash().to_string())
                            });
                        if frame.response.contains_pointer() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if frame.response.interact(egui::Sense::click()).clicked() {
                            if self.selected.iter().any(|x| x.hash() == result.hash()) {
                                self.selected.retain(|x| x.hash() != x.hash());
                            } else {
                                self.selected.push(result.clone());
                            }
                        };
                    }
                });
            });
        });

        for (i, selected) in self.selected.iter().enumerate() {
            egui::SidePanel::right(format!("selection_panel_{i}")).show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("\u{E000}").clicked() {
                            self.to_deselect.push(i);
                        }
                        // TODO: show icon here
                        ui.label(selected.name());
                        let enum_type: &'static str = selected.into();
                        ui.label(enum_type);
                        ui.label(selected.hash().to_string());
                    });
                    selected.show(self.texture_cache.clone(), selected.hash(), ui);
                });
            });
        }
        if !self.to_deselect.is_empty() {
            for index in &self.to_deselect {
                self.selected.remove(*index);
            }
            self.to_deselect.clear();
        }
    }
}
