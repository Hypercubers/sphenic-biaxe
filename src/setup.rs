use std::f32::consts::{PI, TAU};

use egui::*;
use serde::{Deserialize, Serialize};

const CONSERVATIVENESS: u32 = 1;
const POLYGON_RESOLUTION: u32 = 100;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct PuzzleSetup {
    pub a: u32,
    pub b: u32,
}
impl Default for PuzzleSetup {
    fn default() -> Self {
        Self { a: 5, b: 3 }
    }
}
impl PuzzleSetup {
    pub fn a_radius(self) -> f32 {
        polygon_circumradius(self.a + CONSERVATIVENESS)
    }
    pub fn b_radius(self) -> f32 {
        polygon_circumradius(self.b + CONSERVATIVENESS)
    }

    pub fn height(self) -> f32 {
        f32::max(self.a_radius(), self.b_radius()) * 2.0
    }
    pub fn width(self) -> f32 {
        self.a_radius()
            + self.b_radius()
            + polygon_apothem(self.a + CONSERVATIVENESS)
            + polygon_apothem(self.b + CONSERVATIVENESS)
    }
    pub fn midpoint_x(self) -> f32 {
        self.a_radius() + polygon_apothem(self.a + CONSERVATIVENESS)
    }
    pub fn size(self) -> Vec2 {
        vec2(self.width(), self.height())
    }

    pub fn a_center(self) -> Vec2 {
        vec2(self.a_radius(), self.height() * 0.5)
    }
    pub fn b_center(self) -> Vec2 {
        vec2(self.width() - self.b_radius(), self.height() * 0.5)
    }

    pub fn hovers_a(self, p: Vec2) -> bool {
        (p - self.a_center()).length_sq() < self.a_radius() * self.a_radius()
            && p.x < self.midpoint_x()
    }
    pub fn hovers_b(self, p: Vec2) -> bool {
        (p - self.b_center()).length_sq() < self.b_radius() * self.b_radius()
            && p.x > self.midpoint_x()
    }

    pub fn sphene_points(self) -> Vec<Vec2> {
        let mut points = vec![];

        let r2 = self.a_radius() * self.a_radius();
        let resolution = POLYGON_RESOLUTION / self.a;
        for i in 0..resolution {
            let y = i as f32 / resolution as f32 - 0.5;
            let x = self.a_center().x + (r2 - y * y).sqrt();
            points.push(vec2(x, y + self.height() * 0.5));
        }

        let r2 = self.b_radius() * self.b_radius();
        let resolution = POLYGON_RESOLUTION / self.b;
        for i in 0..resolution {
            let y = -(i as f32 / resolution as f32 - 0.5);
            let x = self.b_center().x - (r2 - y * y).sqrt();
            points.push(vec2(x, y + self.height() * 0.5));
        }

        points
    }

    pub fn a_sector_points(self) -> impl Iterator<Item = Vec2> {
        sector_points(TAU / self.a as f32).map(move |p| self.a_center() + p * self.a_radius())
    }
    pub fn b_sector_points(self) -> impl Iterator<Item = Vec2> {
        sector_points(TAU / self.b as f32).map(move |p| self.b_center() - p * self.b_radius())
    }

    pub fn sticker_color(self, i: u32, lightness: f32) -> Color32 {
        if i < self.a {
            sample_rainbow(i, self.a, lightness * 0.5)
        } else {
            sample_rainbow(i - self.a + 1, self.b, lightness * 0.125)
        }
    }
}

/// Returns the circumradius for a unit-edge-length polygon with `n` sides.
fn polygon_circumradius(n: u32) -> f32 {
    0.5 / (PI / n as f32).sin()
}
/// Returns the apothem (inradius) for a unit-edge-length polygon with `n`
/// sides.
fn polygon_apothem(n: u32) -> f32 {
    0.5 / (PI / n as f32).tan()
}

fn sector_points(angle: f32) -> impl Iterator<Item = Vec2> {
    let frac = (TAU / angle) as u32 + 1;
    let n = POLYGON_RESOLUTION / frac;
    (0..=n)
        .map(move |i| {
            let (sin, cos) = ((i as f32 / n as f32 - 0.5) * angle).sin_cos();
            vec2(cos, sin)
        })
        .chain([Vec2::ZERO])
}

/// Samples a rainbow with `n` colors at index `i`. `lightness` ranges from 0 to 1, with 0.5 being default.
fn sample_rainbow(i: u32, n: u32, lightness: f32) -> Color32 {
    let colorous::Color { r, g, b } = colorous::RAINBOW.eval_rational(i as usize, n as usize);
    let blend_color = if lightness > 0.5 {
        Color32::WHITE.gamma_multiply(lightness * 2.0 - 1.0)
    } else {
        Color32::BLACK.gamma_multiply(1.0 - lightness * 2.0)
    };
    Color32::from_rgb(r, g, b).blend(blend_color)
}
