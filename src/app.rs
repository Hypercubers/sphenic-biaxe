use serde::{Deserialize, Serialize};

use crate::{Preferences, PuzzleView};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    pub puzzle: PuzzleView,
    pub prefs: Preferences,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(2.0);
        cc.egui_ctx
            .style_mut(|style| style.spacing.scroll = egui::style::ScrollStyle::solid());

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
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
            powered_by_egui_and_eframe(ui);
        });

        egui::SidePanel::right("left_panel")
            .exact_width(250.0)
            .resizable(false)
            .frame(egui::Frame::central_panel(&ctx.style()))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(true)
                    .show(ui, |ui| {
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
                            ui.strong("Interaction");
                            self.prefs.show_interaction_prefs(ui);
                        });

                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong("Visuals");
                            self.prefs.show_visuals_prefs(ui);
                        });

                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong("Controls");
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
                            ui.add_space(ui.spacing().item_spacing.y);
                            ui.label("Left click or scroll up to rotate counterclockwise");
                            ui.add_space(ui.spacing().item_spacing.y);
                            ui.label("Right click or scroll down to rotate clockwise");
                        });

                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.strong("Solved state");
                            ui.label("The center has the dot piece.");
                            ui.add_space(ui.spacing().item_spacing.y);
                            ui.label(
                                "The left circle has letters increasing clockwise from the dot.",
                            );
                            ui.add_space(ui.spacing().item_spacing.y);
                            ui.label(
                                "The right circle has numbers increasing clockwise from the dot.",
                            );
                        });
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if self.puzzle.was_scrambled() && self.puzzle.is_solved() {
                        ui.heading("Solved!");
                    }
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.heading("Sphenic Biaxe Puzzle");
                    });
                });
                self.puzzle.show_puzzle(ui, &self.prefs);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
