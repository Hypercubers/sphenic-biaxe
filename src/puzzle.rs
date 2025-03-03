use std::f32::consts::TAU;

use egui::*;
use serde::{Deserialize, Serialize};

use crate::{App, PuzzleSetup};

pub fn draw(app: &mut App, ui: &mut Ui) {
    let mut changed = false;
    changed |= ui.add(Slider::new(&mut app.puzzle.a, 2..=16)).changed();
    changed |= ui.add(Slider::new(&mut app.puzzle.b, 2..=16)).changed();
    if changed {
        app.state = None;
    }

    let desired_size = app.puzzle.size();

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

    let state = app
        .state
        .get_or_insert_with(|| PuzzleState::new(app.puzzle));

    let sphene = app.puzzle.sphene_points();
    let a_sector: Vec<_> = app.puzzle.a_sector_points().collect();
    let b_sector: Vec<_> = app.puzzle.b_sector_points().collect();

    for i in 0..app.puzzle.b {
        let angle = i as f32 * TAU / app.puzzle.b as f32;

        let j = (state.b_rot + i) % app.puzzle.b;
        let color = app
            .puzzle
            .sticker_color(if j == 0 { 0 } else { state.a() + j - 1 }, 0.8);
        p.add(egui::Shape::convex_polygon(
            b_sector
                .iter()
                .map(|&p| rect.min + rotate_point(p, app.puzzle.b_center(), angle) * scale)
                .collect(),
            color,
            Stroke::NONE,
        ));

        if i > 0 {
            let color = app.puzzle.sticker_color(state.b_pieces[i as usize], 1.0);
            p.add(egui::Shape::convex_polygon(
                sphene
                    .iter()
                    .map(|&p| rect.min + rotate_point(p, app.puzzle.b_center(), angle) * scale)
                    .collect(),
                color,
                stroke,
            ));
        }
    }

    for i in 0..app.puzzle.a {
        let angle = i as f32 * TAU / app.puzzle.a as f32;

        let j = (state.a_rot + i) % app.puzzle.a;
        let color = app.puzzle.sticker_color(j, 0.8);
        p.add(egui::Shape::convex_polygon(
            a_sector
                .iter()
                .map(|&p| rect.min + rotate_point(p, app.puzzle.a_center(), angle) * scale)
                .collect(),
            color,
            Stroke::NONE,
        ));

        let color = app.puzzle.sticker_color(state.a_pieces[i as usize], 1.0);
        p.add(egui::Shape::convex_polygon(
            sphene
                .iter()
                .map(|&p| rect.min + rotate_point(p, app.puzzle.a_center(), angle) * scale)
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
        rect.min + app.puzzle.a_center() * scale,
        app.puzzle.a_radius() * scale * 0.995,
        if app.puzzle.hovers_a(hov) {
            big_stroke_hovered
        } else {
            big_stroke
        },
    );
    p.circle_stroke(
        rect.min + app.puzzle.b_center() * scale,
        app.puzzle.b_radius() * scale * 0.995,
        if app.puzzle.hovers_b(hov) {
            big_stroke_hovered
        } else {
            big_stroke
        },
    );

    if app.puzzle.hovers_a(hov) {
        if r.clicked() {
            state.twist_a_cw();
        } else if r.secondary_clicked() {
            state.twist_a_ccw();
        }
    }
    if app.puzzle.hovers_b(hov) {
        if r.clicked() {
            state.twist_b_cw();
        } else if r.secondary_clicked() {
            state.twist_b_ccw();
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PuzzleState {
    a_rot: u32,
    b_rot: u32,
    a_pieces: Vec<u32>,
    b_pieces: Vec<u32>,
}
impl PuzzleState {
    pub fn new(setup: PuzzleSetup) -> Self {
        let a = (0..).take(setup.a as usize).collect();
        let mut b: Vec<_> = (setup.a - 1..).take(setup.b as usize).collect();
        b[0] = 0;
        Self {
            a_rot: 0,
            b_rot: 0,
            a_pieces: a,
            b_pieces: b,
        }
    }

    pub fn a(&self) -> u32 {
        self.a_pieces.len() as u32
    }
    pub fn b(&self) -> u32 {
        self.b_pieces.len() as u32
    }

    pub fn twist_a_ccw(&mut self) {
        self.a_rot = (self.a_rot + self.a() - 1) % self.a();
        self.a_pieces.rotate_right(1);
        self.b_pieces[0] = self.a_pieces[0];
    }
    pub fn twist_a_cw(&mut self) {
        self.a_rot = (self.a_rot + 1) % self.a();
        self.a_pieces.rotate_left(1);
        self.b_pieces[0] = self.a_pieces[0];
    }
    pub fn twist_b_ccw(&mut self) {
        self.b_rot = (self.b_rot + self.b() - 1) % self.b();
        self.b_pieces.rotate_right(1);
        self.a_pieces[0] = self.b_pieces[0];
    }
    pub fn twist_b_cw(&mut self) {
        self.b_rot = (self.b_rot + 1) % self.b();
        self.b_pieces.rotate_left(1);
        self.a_pieces[0] = self.b_pieces[0];
    }
}

fn rotate_point(p: Vec2, center: Vec2, angle: f32) -> Vec2 {
    let Vec2 { x, y } = p - center;
    let (sin, cos) = angle.sin_cos();
    center + vec2(cos * x - sin * y, sin * x + cos * y)
}
