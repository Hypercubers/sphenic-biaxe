use egui::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Preferences {
    pub twist_duration: f32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            twist_duration: 0.2,
        }
    }
}

impl Preferences {
    pub fn show(&mut self, ui: &mut Ui) {
        let defaults = Self::default();

        ui.horizontal(|ui| {
            ui.scope(|ui| {
                if self.twist_duration == defaults.twist_duration {
                    ui.disable();
                }
                if ui
                    .add(Button::new("⟲").min_size(vec2(20.0, 20.0)))
                    .clicked()
                {
                    self.twist_duration = defaults.twist_duration;
                }
            });
            ui.add(
                DragValue::new(&mut self.twist_duration)
                    .range(0.0..=1.0)
                    .speed(0.01),
            );
            ui.label("Twist duration");
        });
    }
}
