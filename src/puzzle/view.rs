use std::f32::consts::TAU;

use egui::*;
use serde::{Deserialize, Serialize};

use super::{PuzzleConfig, PuzzleState};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PuzzleView {
    config: PuzzleConfig,
    state: Option<PuzzleState>,
}

impl PuzzleView {
    pub fn draw(&mut self, ui: &mut Ui) {
        let mut changed = false;
        changed |= ui.add(Slider::new(&mut self.config.a, 2..=16)).changed();
        changed |= ui.add(Slider::new(&mut self.config.b, 2..=16)).changed();
        if changed {
            self.state = None;
        }

        let desired_size = self.config.size();

        let available = ui.available_size_before_wrap();

        let scale = (available / desired_size).min_elem();

        let rect = Rect::from_center_size(
            ui.available_rect_before_wrap().center(),
            desired_size * scale,
        );

        let p = ui.painter();

        let big_stroke = Stroke {
            width: 0.01 * scale,
            color: ui.visuals().strong_text_color(),
        };
        let big_stroke_hovered = Stroke {
            width: 0.015 * scale,
            color: Color32::RED,
        };
        let stroke = Stroke {
            width: 0.005 * scale,
            color: ui.visuals().strong_text_color(),
        };

        let state = self
            .state
            .get_or_insert_with(|| PuzzleState::new(self.config));

        let sphene = self.config.sphene_points();
        let a_sector: Vec<_> = self.config.a_sector_points().collect();
        let b_sector: Vec<_> = self.config.b_sector_points().collect();

        for i in 0..self.config.b {
            let angle = i as f32 * TAU / self.config.b as f32;

            let j = (state.b_rot + i) % self.config.b;
            let color = self
                .config
                .sticker_color(if j == 0 { 0 } else { state.a() + j - 1 }, 0.8);
            p.add(egui::Shape::convex_polygon(
                b_sector
                    .iter()
                    .map(|&p| rect.min + rotate_point(p, self.config.b_center(), angle) * scale)
                    .collect(),
                color,
                Stroke::NONE,
            ));

            if i > 0 {
                let color = self.config.sticker_color(state.b_pieces[i as usize], 1.0);
                p.add(egui::Shape::convex_polygon(
                    sphene
                        .iter()
                        .map(|&p| rect.min + rotate_point(p, self.config.b_center(), angle) * scale)
                        .collect(),
                    color,
                    stroke,
                ));
            }
        }

        for i in 0..self.config.a {
            let angle = i as f32 * TAU / self.config.a as f32;

            let j = (state.a_rot + i) % self.config.a;
            let color = self.config.sticker_color(j, 0.8);
            p.add(egui::Shape::convex_polygon(
                a_sector
                    .iter()
                    .map(|&p| rect.min + rotate_point(p, self.config.a_center(), angle) * scale)
                    .collect(),
                color,
                Stroke::NONE,
            ));

            let color = self.config.sticker_color(state.a_pieces[i as usize], 1.0);
            p.add(egui::Shape::convex_polygon(
                sphene
                    .iter()
                    .map(|&p| rect.min + rotate_point(p, self.config.a_center(), angle) * scale)
                    .collect(),
                color,
                stroke,
            ));
        }

        let r = ui.interact(rect, egui::Id::new("puzzle"), egui::Sense::click());
        let hov = r
            .hover_pos()
            .map(|p| (p - rect.min) / scale)
            .unwrap_or_default();

        // TODO: draw hovered one last
        p.circle_stroke(
            rect.min + self.config.a_center() * scale,
            self.config.a_radius() * scale * 0.995,
            if self.config.hovers_a(hov) {
                big_stroke_hovered
            } else {
                big_stroke
            },
        );
        p.circle_stroke(
            rect.min + self.config.b_center() * scale,
            self.config.b_radius() * scale * 0.995,
            if self.config.hovers_b(hov) {
                big_stroke_hovered
            } else {
                big_stroke
            },
        );

        if self.config.hovers_a(hov) {
            if r.clicked() {
                state.twist_a_cw();
            } else if r.secondary_clicked() {
                state.twist_a_ccw();
            }
        }
        if self.config.hovers_b(hov) {
            if r.clicked() {
                state.twist_b_cw();
            } else if r.secondary_clicked() {
                state.twist_b_ccw();
            }
        }
    }
}

fn rotate_point(p: Vec2, center: Vec2, angle: f32) -> Vec2 {
    let Vec2 { x, y } = p - center;
    let (sin, cos) = angle.sin_cos();
    center + vec2(cos * x - sin * y, sin * x + cos * y)
}
