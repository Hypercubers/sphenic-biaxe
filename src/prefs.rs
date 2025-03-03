use egui::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preferences {
    pub twist_duration: f32,
    pub show_labels: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            twist_duration: 0.2,
            show_labels: false,
        }
    }
}

impl Preferences {
    pub fn show_interaction_prefs(&mut self, ui: &mut Ui) {
        let defaults = Self::default();

        show_with_reset_button(
            ui,
            &mut self.twist_duration,
            defaults.twist_duration,
            |ui, current| {
                ui.add(DragValue::new(current).range(0.0..=1.0).speed(0.01));
                ui.label("Twist duration");
            },
        );
    }

    pub fn show_visuals_prefs(&mut self, ui: &mut Ui) {
        ui.checkbox(&mut self.show_labels, "Show labels");

        egui::widgets::global_theme_preference_buttons(ui);
    }
}

fn show_with_reset_button<T: PartialEq>(
    ui: &mut Ui,
    current: &mut T,
    default: T,
    show_current: impl FnOnce(&mut Ui, &mut T),
) {
    ui.horizontal(|ui| {
        ui.scope(|ui| {
            if *current == default {
                ui.disable();
            }
            let reset_button = Button::new("⟲").min_size(vec2(20.0, 20.0));
            if ui.add(reset_button).clicked() {
                *current = default;
            }
        });
        show_current(ui, current);
    });
}
