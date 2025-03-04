use serde::{Deserialize, Serialize};

use crate::{Preferences, PuzzleView};

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
enum Tab {
    #[default]
    Puzzle,
    Configuration,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    pub puzzle: PuzzleView,
    pub prefs: Preferences,
    #[serde(skip)]
    tab: Tab,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(1.25);
        cc.egui_ctx
            .style_mut(|style| style.spacing.scroll = egui::style::ScrollStyle::solid());

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn show_configuration(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");

        ui.add_space(ui.spacing().item_spacing.y);

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.strong("Puzzle");
            self.puzzle.show_config(ui);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Scramble").clicked() {
                    self.puzzle.scramble();
                }
                if ui.button("Reset").clicked() {
                    self.puzzle.reset();
                }
            })
        });

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.strong("Visuals");
            self.prefs.show_visuals_prefs(ui);
        });

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.strong("Interaction");
            self.prefs.show_interaction_prefs(ui);

            // Click controls
            ui.add_space(ui.spacing().item_spacing.y);
            ui.label(if self.prefs.sector_click_mode {
                "• Left click or tap — rotate to center"
            } else {
                "• Left click or tap — rotate counterclockwise"
            });
            ui.label(if self.prefs.sector_click_mode {
                "• Right click or long tap — rotate from center"
            } else {
                "• Right click or long tap — rotate clockwise"
            });
            ui.add_space(ui.spacing().item_spacing.y);
            ui.label("• Scroll up — rotate counterclockwise");
            ui.label("• Scroll down — rotate clockwise");

            // Keyboard controls
            ui.horizontal(|ui| {
                ui.label("Keyboard controls:");
                for key in ["D", "F", "J", "K"] {
                    ui.add(
                        egui::Button::new(key)
                            .sense(egui::Sense::empty())
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(ui.visuals().noninteractive().fg_stroke),
                    );
                }
            });
        });

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.strong("Solved state");
            ui.label("The center has the dot piece");
            ui.add_space(ui.spacing().item_spacing.y);
            ui.label("The left circle has letters increasing clockwise from the dot");
            ui.add_space(ui.spacing().item_spacing.y);
            ui.label("The right circle has numbers increasing clockwise from the dot");
        });
    }

    fn show_puzzle(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if self.puzzle.was_scrambled() && self.puzzle.is_solved() {
                ui.heading("Solved!");
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.heading("Sphenic Biaxe Puzzle");
            });
        });

        self.puzzle.show_puzzle(ui, &self.prefs);
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per
    /// second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_web = cfg!(target_arch = "wasm32");
        let is_landscape = ctx.available_rect().aspect_ratio() > 1.0;

        if !is_web {
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    // no File->Quit on web pages
                    if !is_web {
                        ui.menu_button("File", |ui| {
                            if ui.button("Quit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    }

                    egui::warn_if_debug_build(ui);
                });
            });
        }

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            let sp = std::mem::take(&mut ui.spacing_mut().item_spacing);
            if is_landscape {
                ui.horizontal(|ui| {
                    egui::ScrollArea::horizontal()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            show_credits(ui);
                            ui.add_space(sp.x);
                            ui.separator();
                            ui.add_space(sp.x);
                            show_powered_by_egui(ui);
                            ui.add_space(sp.x);
                            ui.separator();
                            ui.add_space(sp.x);
                            show_source_code_link(ui);
                        })
                });
            } else {
                ui.horizontal_wrapped(|ui| show_source_code_link(ui));
                ui.separator();
                ui.horizontal_wrapped(|ui| show_powered_by_egui(ui));
                ui.separator();
                ui.horizontal_wrapped(|ui| show_credits(ui));
            }
        });

        if is_landscape {
            egui::SidePanel::right("config_panel")
                .exact_width(f32::min(400.0, ctx.available_rect().width() / 3.0))
                .resizable(false)
                .frame(egui::Frame::central_panel(&ctx.style()))
                .show(ctx, |ui| {
                    egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                        self.show_configuration(ui);
                    });
                });
        } else {
            egui::TopBottomPanel::bottom("config_panel")
                .exact_height(ctx.available_rect().height() * 0.5)
                .resizable(false)
                .frame(egui::Frame::central_panel(&ctx.style()))
                .show(ctx, |ui| {
                    egui::ScrollArea::both().auto_shrink(true).show(ui, |ui| {
                        self.show_configuration(ui);
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| self.show_puzzle(ui));
    }
}

fn show_credits(ui: &mut egui::Ui) {
    ui.label(format!("Sphenic Biaxe v{} by ", env!("CARGO_PKG_VERSION")));
    ui.hyperlink_to("Andrew Farkas", "https://ajfarkas.dev/");
}

fn show_powered_by_egui(ui: &mut egui::Ui) {
    ui.label("Powered by ");
    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
    ui.label(" and ");
    ui.hyperlink_to(
        "eframe",
        "https://github.com/emilk/egui/tree/master/crates/eframe",
    );
}

fn show_source_code_link(ui: &mut egui::Ui) {
    ui.hyperlink_to(
        egui::RichText::new(" source code").small(),
        env!("CARGO_PKG_REPOSITORY"),
    );
}
