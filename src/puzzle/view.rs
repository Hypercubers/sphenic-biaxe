use std::f32::consts::TAU;

use egui::*;
use serde::{Deserialize, Serialize};
use web_time::Instant;

use super::{Grip, PuzzleConfig, PuzzleState, TwistAnimation, TwistAnimationState, TwistDir};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PuzzleView {
    config: PuzzleConfig,
    state: Option<PuzzleState>,

    state_queue: Vec<PuzzleState>,
    #[serde(skip)]
    animation: TwistAnimationState,
    #[serde(skip)]
    last_repaint: Option<Instant>,

    a: f32,
}

impl PuzzleView {
    pub fn draw(&mut self, ui: &mut Ui) {
        let mut changed = false;
        changed |= ui.add(Slider::new(&mut self.config.a, 2..=16)).changed();
        changed |= ui.add(Slider::new(&mut self.config.b, 2..=16)).changed();
        if changed {
            self.state = None;
            self.animation = TwistAnimationState::default();
        }

        let desired_size = self.config.size();

        let available = ui.available_size_before_wrap();

        let scale = (available / desired_size).min_elem();

        let rect = Rect::from_center_size(
            ui.available_rect_before_wrap().center(),
            desired_size * scale,
        );

        // Generate puzzle if necessary.
        let cfg = self.config;
        let state = self.state.get_or_insert_with(|| PuzzleState::new(cfg));

        // Compute hovered grip.
        let r = ui.interact(rect, egui::Id::new("puzzle"), egui::Sense::click());
        let hovered_grip = r
            .hover_pos()
            .map(|p| (p - rect.min) / scale)
            .and_then(|cursor_pos| self.config.hovered_grip(cursor_pos));

        // Handle click twists.
        if let Some(grip) = hovered_grip {
            if r.clicked() {
                self.twist(grip, TwistDir::Ccw);
            } else if r.secondary_clicked() {
                self.twist(grip, TwistDir::Cw);
            }
        }

        // Update animation state.
        let now = Instant::now();
        let last_repaint = self.last_repaint.unwrap_or(now);
        let time_delta = now - last_repaint;
        self.last_repaint = Some(now);
        if self.animation.proceed(time_delta) {
            ui.ctx().request_repaint();
        }

        let moving_grip = self.animation.current().map(|(anim, _)| anim.grip);

        // Draw the moving circle on top of non-moving circle.
        let grip_draw_order = match moving_grip {
            Some(Grip::A) => [Grip::B, Grip::A],
            _ => [Grip::A, Grip::B],
        };
        let mut is_second = false;
        for g in grip_draw_order {
            self.draw_grip(ui, g, is_second, rect, scale);
            // Draw non-hovered grips if something is moving.
            if moving_grip.is_some() {
                if hovered_grip != Some(g) {
                    self.draw_grip_circle(ui, g, false, rect, scale);
                }
            }
            is_second = true;
        }

        // Draw non-hovered grips if neither circle is moving.
        for g in [Grip::A, Grip::B] {
            if hovered_grip != Some(g) && moving_grip.is_none() {
                self.draw_grip_circle(ui, g, false, rect, scale);
            }
        }

        // Draw hovered grip.
        if let Some(g) = hovered_grip {
            self.draw_grip_circle(ui, g, true, rect, scale);
        }
    }

    fn draw_grip(
        &mut self,
        ui: &mut egui::Ui,
        grip: Grip,
        draw_intersection: bool,
        rect: Rect,
        scale: f32,
    ) {
        let cfg = self.config;
        let Some(state) = &self.state else { return };

        let p = ui.painter();

        let center = cfg.center(grip);

        let transform = |p: Vec2, angle: f32| rect.min + rotate_point(p, center, angle) * scale;

        // Define strokes.
        let sector_stroke = Stroke::NONE;
        let sticker_stroke = Stroke {
            width: 0.005 * scale,
            color: ui.visuals().strong_text_color(),
        };

        // Compute geometry.
        let init_sphene = cfg.sphene_points();
        let init_sector: Vec<_> = cfg.sector_points(grip).collect();

        // Compute angle offset
        let mut grip_offset = 0.0;
        if let Some((anim, t)) = self.animation.current() {
            if anim.grip == grip {
                // Negate because positive angles are clockwise in egui
                grip_offset =
                    -crate::util::animate_twist_angle(anim.initial_angle, anim.final_angle, t);
            }
        }
        let get_angle = |i: u32| grip_offset + i as f32 * TAU / cfg.n(grip) as f32;

        let visual_state = match self.animation.current() {
            Some((anim, _)) => &anim.state,
            None => state,
        };

        // Draw sectors.
        let make_sector = |angle| init_sector.iter().map(|&p| transform(p, angle)).collect();
        for i in 0..cfg.n(grip) {
            let j = (visual_state.rot(grip) + i) % cfg.n(grip);
            p.add(egui::Shape::convex_polygon(
                make_sector(get_angle(i)),
                cfg.sticker_color_within_grip(grip, j, 0.8),
                sector_stroke,
            ));
        }

        // Draw sphenes
        let make_sphene = |angle| init_sphene.iter().map(|&p| transform(p, angle)).collect();
        for i in 0..cfg.n(grip) {
            if i > 0 || draw_intersection {
                p.add(egui::Shape::convex_polygon(
                    make_sphene(get_angle(i)),
                    cfg.sticker_color(visual_state.pieces(grip)[i as usize], 1.0),
                    sticker_stroke,
                ));
            }
        }
    }

    fn draw_grip_circle(&self, ui: &mut Ui, grip: Grip, is_hovered: bool, rect: Rect, scale: f32) {
        let cfg = self.config;
        let radius = cfg.radius(grip);
        let center = cfg.center(grip);

        // Define strokes.
        let grip_stroke = Stroke {
            width: 0.01 * scale,
            color: ui.visuals().strong_text_color(),
        };
        let hovered_grip_stroke = Stroke {
            width: 0.015 * scale,
            color: Color32::RED,
        };

        ui.painter().circle_stroke(
            rect.min + center * scale,
            radius * scale * 0.995,
            match is_hovered {
                true => hovered_grip_stroke,
                false => grip_stroke,
            },
        );
    }

    fn twist(&mut self, grip: Grip, direction: TwistDir) {
        if let Some(state) = &mut self.state {
            let old_state = state.clone();
            match (grip, direction) {
                (Grip::A, TwistDir::Cw) => state.twist_a_cw(),
                (Grip::A, TwistDir::Ccw) => state.twist_a_ccw(),
                (Grip::B, TwistDir::Cw) => state.twist_b_cw(),
                (Grip::B, TwistDir::Ccw) => state.twist_b_ccw(),
            }
            self.animation.push(TwistAnimation {
                state: old_state,
                grip,
                initial_angle: 0.0,
                final_angle: match grip {
                    Grip::A => TAU / self.config.a as f32 * direction.to_f32(),
                    Grip::B => TAU / self.config.b as f32 * direction.to_f32(),
                },
            });
        }
    }
}

fn rotate_point(p: Vec2, center: Vec2, angle: f32) -> Vec2 {
    let Vec2 { x, y } = p - center;
    let (sin, cos) = angle.sin_cos();
    center + vec2(cos * x - sin * y, sin * x + cos * y)
}
