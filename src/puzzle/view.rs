use std::{
    f32::consts::{PI, TAU},
    hash::{DefaultHasher, Hash, Hasher},
};

use egui::*;
use rand::{Rng, SeedableRng};
use web_time::{Duration, Instant};

use super::{Grip, PuzzleConfig, PuzzleState, TwistAnimation, TwistAnimationState, TwistDir};
use crate::Preferences;

const ASSUMED_FPS: f32 = 120.0;

#[derive(Debug, Default, Clone)]
pub struct PuzzleView {
    config: PuzzleConfig,
    state: Option<PuzzleState>,
    was_scrambled: bool,

    animation: TwistAnimationState,
    last_frame_time: Option<Instant>,

    drag_start: Option<Pos2>,
    drag_angle_offset: Option<(Grip, f32)>,
}

impl PuzzleView {
    pub fn reset(&mut self) {
        self.state = None;
        self.animation = TwistAnimationState::default();
        self.was_scrambled = false;
    }

    pub fn scramble(&mut self) {
        self.reset();
        let state = self.state.insert(PuzzleState::new(self.config));

        // this is awful seeding but it's fine for this puzzle and I couldn't
        // get `getrandom` to work on web
        let mut h = DefaultHasher::new();
        Instant::now().hash(&mut h);
        let bytes = h.finish().to_ne_bytes();
        let mut rng = rand::rngs::StdRng::from_seed(
            [bytes; 4]
                .as_flattened()
                .try_into()
                .expect("error casting [[u8; 8]; 4] to [u8; 32]"),
        );
        for _ in 0..500 {
            state.twist_cw(Grip::A, rng.random_range(0..state.n(Grip::A)));
            state.twist_cw(Grip::B, rng.random_range(0..state.n(Grip::B)));
        }

        self.was_scrambled = true;
    }

    pub fn was_scrambled(&self) -> bool {
        self.was_scrambled
    }
    pub fn is_solved(&self) -> bool {
        self.state
            .as_ref()
            .is_some_and(|state| state.is_solved(self.config))
    }

    pub fn show_config(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.horizontal(|ui| {
            changed |= ui
                .add(Slider::new(&mut self.config.a, 2..=16).logarithmic(true))
                .changed();
            ui.label("Left");
        });
        ui.horizontal(|ui| {
            changed |= ui
                .add(Slider::new(&mut self.config.b, 2..=16).logarithmic(true))
                .changed();
            ui.label("Right");
        });
        changed |= ui
            .checkbox(&mut self.config.a_axis_stationary, "Left axis stationary")
            .clicked();
        changed |= ui
            .checkbox(&mut self.config.b_axis_stationary, "Right axis stationary")
            .clicked();

        if changed {
            self.reset();
        }
    }

    pub fn show_puzzle(&mut self, ui: &mut Ui, prefs: &Preferences) {
        let desired_size = self.config.size();

        let available = ui.available_size_before_wrap();

        let scale = (available / desired_size).min_elem();

        let rect = Rect::from_center_size(
            ui.available_rect_before_wrap().center(),
            desired_size * scale,
        );

        // Generate puzzle if necessary.
        let cfg = self.config;
        self.state.get_or_insert_with(|| PuzzleState::new(cfg));

        // Compute hovered grip.
        let r = ui.interact(rect, Id::new("puzzle"), Sense::click_and_drag());
        let hovered_grip = self
            .drag_start
            .or(r.hover_pos())
            .map(|p| (p - rect.min) / scale)
            .and_then(|cursor_pos| self.config.hovered_grip(cursor_pos));

        ui.input(|input| {
            for ev in &input.raw.events {
                if let Event::Key {
                    key,
                    physical_key,
                    pressed: true,
                    ..
                } = ev
                {
                    match physical_key.unwrap_or(*key) {
                        Key::D => self.twist(Grip::A, TwistDir::Ccw, 1),
                        Key::F => self.twist(Grip::A, TwistDir::Cw, 1),
                        Key::J => self.twist(Grip::B, TwistDir::Ccw, 1),
                        Key::K => self.twist(Grip::B, TwistDir::Cw, 1),
                        _ => (),
                    }
                }
            }
        });

        // Handle drag twists.
        if r.drag_started() {
            self.drag_start = r.hover_pos();
        }
        if r.drag_stopped() || r.dragged() && r.hover_pos().is_none() {
            if let Some((grip, angle)) = self.drag_angle_offset {
                let amt = (angle / (TAU / cfg.n(grip) as f32)).round() as i32;
                if amt < 0 {
                    self.twist_with_initial_angle(grip, TwistDir::Ccw, -angle, (-amt) as u32);
                } else {
                    self.twist_with_initial_angle(grip, TwistDir::Cw, -angle, amt as u32);
                }
            }
        }
        if r.dragged() && r.hover_pos().is_some() {
            self.animation = TwistAnimationState::default(); // cancel animations
            if let Some(grip) = hovered_grip {
                if let Some(drag_start) = self.drag_start {
                    let drag_end = r.hover_pos().unwrap_or(drag_start);
                    let center = rect.min + cfg.center(grip) * scale;
                    let init_angle = (drag_start - center).angle();
                    let final_angle = (drag_end - center).angle();
                    let angle_delta = final_angle - init_angle;
                    self.drag_angle_offset = Some((grip, angle_delta));
                }
            }
        } else {
            self.drag_start = None;
            self.drag_angle_offset = None;
        }

        // Handle click & scroll twists.
        if let Some(grip) = hovered_grip {
            if prefs.sector_click_mode && r.clicked() {
                if let Some(click_pos) = r.hover_pos() {
                    let mut angle = (click_pos - (rect.min + cfg.center(grip) * scale)).angle();
                    if grip == Grip::B {
                        angle += PI;
                    }
                    let sector = (angle / (TAU / cfg.n(grip) as f32)).round() as i32;
                    if sector > 0 {
                        self.twist(grip, TwistDir::Ccw, sector as u32);
                    } else if sector < 0 {
                        self.twist(grip, TwistDir::Cw, (-sector) as u32);
                    }
                }
            }

            let amt = (!prefs.sector_click_mode && r.clicked()) as i32
                + -((!prefs.sector_click_mode && r.secondary_clicked()) as i32)
                + ui.input(|input| {
                    input
                        .raw
                        .events
                        .iter()
                        .filter_map(|ev| match ev {
                            Event::MouseWheel {
                                unit: MouseWheelUnit::Line | MouseWheelUnit::Page,
                                delta,
                                modifiers: _,
                            } => Some((delta.x + delta.y).signum() as i32),
                            _ => None,
                        })
                        .sum::<i32>()
                });

            if amt > 0 {
                self.twist(grip, TwistDir::Ccw, amt as u32);
            } else if amt < 0 {
                self.twist(grip, TwistDir::Cw, (-amt) as u32);
            }
        }

        // Update animation state.
        let now = Instant::now();
        let delta = match self.last_frame_time {
            Some(then) => now - then,
            None => Duration::from_secs_f32(1.0 / ASSUMED_FPS),
        };
        if self.animation.proceed(delta, prefs) {
            ui.ctx().request_repaint();
            self.last_frame_time = Some(now);
        } else {
            self.last_frame_time = None;
        }

        let moving_grip = Option::or(
            self.drag_angle_offset.map(|(g, _)| g),
            self.animation.current().map(|(anim, _)| anim.grip),
        );

        // Draw the moving circle on top of non-moving circle.
        let grip_draw_order = match moving_grip {
            Some(Grip::A) => [Grip::B, Grip::A],
            _ => [Grip::A, Grip::B],
        };
        let mut is_second = false;
        for g in grip_draw_order {
            self.draw_grip(ui, g, is_second, rect, scale, prefs);
            // Draw non-hovered grips if something is moving.
            if moving_grip.is_some() && hovered_grip != Some(g) {
                self.draw_grip_circle(ui, g, false, rect, scale);
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
        ui: &mut Ui,
        grip: Grip,
        draw_intersection: bool,
        rect: Rect,
        scale: f32,
        prefs: &Preferences,
    ) {
        let cfg = self.config;
        let Some(state) = &self.state else { return };

        let center = cfg.center(grip);
        let radius = cfg.radius(grip);

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
        if let Some((g, offset)) = self.drag_angle_offset {
            if g == grip {
                grip_offset = offset;
            }
        }
        let get_angle = |i: u32| i as f32 * TAU / cfg.n(grip) as f32;

        let visual_state = match self.animation.current() {
            Some((anim, _)) => &anim.state,
            None => state,
        };

        let show_label = |ui: &mut Ui, text, pos, angle| {
            ui.put(
                Rect::from_center_size(transform(pos, angle), vec2(50.0, 50.0)),
                Label::new(
                    WidgetText::from(format!(" {text} "))
                        .color(ui.visuals().strong_text_color())
                        .background_color(ui.visuals().panel_fill.gamma_multiply(0.8)),
                )
                .selectable(false),
            );
        };

        if cfg.axis_stationary(grip) {
            ui.painter().circle_filled(
                transform(center, 0.0),
                radius * scale,
                ui.visuals().code_bg_color,
            );
        }

        // Draw sectors.
        let make_sector = |angle| init_sector.iter().map(|&p| transform(p, angle)).collect();
        for i in 0..cfg.n(grip) {
            let mut j = i;
            let mut angle = get_angle(i);
            if !cfg.axis_stationary(grip) {
                j = (j + visual_state.rot(grip)) % cfg.n(grip);
                angle += grip_offset;
            }
            let color_index = cfg.color_index_in_grip(grip, j);
            ui.painter().add(Shape::convex_polygon(
                make_sector(angle),
                cfg.sector_color(color_index, ui.visuals().dark_mode),
                sector_stroke,
            ));
            if prefs.show_labels {
                let pos = crate::util::lerp(center, cfg.midpoint(), 1.0 / 3.0);
                show_label(ui, cfg.sector_name(grip, j), pos, angle);
            }
        }

        // Draw sphenes
        let make_sphene = |angle| init_sphene.iter().map(|&p| transform(p, angle)).collect();
        for i in 0..cfg.n(grip) {
            if i > 0 || draw_intersection {
                let angle = grip_offset + get_angle(i);
                let sticker = visual_state.pieces(grip)[i as usize];
                ui.painter().add(Shape::convex_polygon(
                    make_sphene(angle),
                    cfg.sticker_color(sticker, ui.visuals().dark_mode),
                    sticker_stroke,
                ));
                if prefs.show_labels {
                    show_label(ui, cfg.sticker_name(sticker), cfg.midpoint(), angle);
                }
            }
        }
    }

    fn draw_grip_circle(&self, ui: &mut Ui, grip: Grip, is_hovered: bool, rect: Rect, scale: f32) {
        let cfg = self.config;
        let radius = cfg.radius(grip);
        let center = cfg.center(grip);

        let stroke = if is_hovered {
            Stroke {
                width: 0.015 * scale,
                color: Color32::RED,
            }
        } else {
            Stroke {
                width: 0.01 * scale,
                color: ui.visuals().strong_text_color(),
            }
        };

        ui.painter().circle_stroke(
            rect.min + center * scale,
            radius * scale - stroke.width / 2.0,
            stroke,
        );
    }

    fn twist(&mut self, grip: Grip, direction: TwistDir, amt: u32) {
        self.twist_with_initial_angle(grip, direction, 0.0, amt);
    }

    fn twist_with_initial_angle(
        &mut self,
        grip: Grip,
        direction: TwistDir,
        initial_angle: f32,
        amt: u32,
    ) {
        if let Some(state) = &mut self.state {
            let old_state = state.clone();
            match direction {
                TwistDir::Cw => state.twist_cw(grip, amt),
                TwistDir::Ccw => state.twist_ccw(grip, amt),
            }
            let mut final_angle = match grip {
                Grip::A => TAU / self.config.a as f32 * direction.to_f32() * amt as f32,
                Grip::B => TAU / self.config.b as f32 * direction.to_f32() * amt as f32,
            };
            if initial_angle + PI < final_angle {
                final_angle -= TAU;
            }
            if initial_angle - PI > final_angle {
                final_angle += TAU;
            }
            self.animation.push(TwistAnimation {
                state: old_state,
                grip,
                initial_angle,
                final_angle,
            });
        }
    }
}

fn rotate_point(p: Vec2, center: Vec2, angle: f32) -> Vec2 {
    let Vec2 { x, y } = p - center;
    let (sin, cos) = angle.sin_cos();
    center + vec2(cos * x - sin * y, sin * x + cos * y)
}
