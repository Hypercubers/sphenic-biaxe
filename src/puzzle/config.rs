use std::f32::consts::{PI, TAU};

use egui::*;
use serde::{Deserialize, Serialize};

use super::Grip;
use super::Grip::{A, B};

const CONSERVATIVENESS: u32 = 1;
const POLYGON_RESOLUTION: u32 = 200;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct PuzzleConfig {
    pub a: u32,
    pub b: u32,
    pub a_axis_stationary: bool,
    pub b_axis_stationary: bool,
}

impl Default for PuzzleConfig {
    fn default() -> Self {
        Self {
            a: 5,
            b: 2,
            a_axis_stationary: false,
            b_axis_stationary: true,
        }
    }
}

impl PuzzleConfig {
    pub fn n(self, grip: Grip) -> u32 {
        match grip {
            Grip::A => self.a,
            Grip::B => self.b,
        }
    }
    pub fn radius(self, grip: Grip) -> f32 {
        polygon_circumradius(self.n(grip) + CONSERVATIVENESS)
    }
    pub fn radius_sq(self, grip: Grip) -> f32 {
        let r = self.radius(grip);
        r * r
    }
    pub fn center(self, grip: Grip) -> Vec2 {
        match grip {
            A => vec2(self.radius(A), self.height() * 0.5),
            B => vec2(self.width() - self.radius(B), self.height() * 0.5),
        }
    }
    pub fn is_hovered(self, grip: Grip, cursor: Vec2) -> bool {
        (cursor - self.center(grip)).length_sq() < self.radius(grip) * self.radius(grip)
            && match grip {
                A => cursor.x < self.midpoint_x(),
                B => cursor.x >= self.midpoint_x(),
            }
    }

    pub fn axis_stationary(self, grip: Grip) -> bool {
        match grip {
            A => self.a_axis_stationary,
            B => self.b_axis_stationary,
        }
    }

    pub fn height(self) -> f32 {
        f32::max(self.radius(A), self.radius(B)) * 2.0
    }
    pub fn width(self) -> f32 {
        self.radius(A)
            + self.radius(B)
            + polygon_apothem(self.a + CONSERVATIVENESS)
            + polygon_apothem(self.b + CONSERVATIVENESS)
    }
    pub fn midpoint_x(self) -> f32 {
        self.radius(A) + polygon_apothem(self.a + CONSERVATIVENESS)
    }
    pub fn midpoint(self) -> Vec2 {
        vec2(self.midpoint_x(), self.height() * 0.5)
    }
    pub fn size(self) -> Vec2 {
        vec2(self.width(), self.height())
    }

    pub fn hovered_grip(self, cursor: Vec2) -> Option<Grip> {
        Option::or(
            self.is_hovered(A, cursor).then_some(Grip::A),
            self.is_hovered(B, cursor).then_some(Grip::B),
        )
    }

    pub fn sphene_points(self) -> Vec<Vec2> {
        let mut points = vec![];

        let resolution = POLYGON_RESOLUTION / self.a;
        for i in 0..resolution {
            let y = i as f32 / resolution as f32 - 0.5;
            let x = self.center(A).x + (self.radius_sq(A) - y * y).sqrt();
            points.push(vec2(x, y + self.height() * 0.5));
        }

        let resolution = POLYGON_RESOLUTION / self.b;
        for i in 0..resolution {
            let y = -(i as f32 / resolution as f32 - 0.5);
            let x = self.center(B).x - (self.radius_sq(B) - y * y).sqrt();
            points.push(vec2(x, y + self.height() * 0.5));
        }

        points
    }

    pub fn sector_points(self, grip: Grip) -> impl Iterator<Item = Vec2> {
        let sign = match grip {
            A => 1.0,
            B => -1.0,
        };

        let radius = crate::util::lerp(
            self.radius(grip),
            polygon_apothem(self.a + CONSERVATIVENESS) + polygon_apothem(self.b + CONSERVATIVENESS)
                - self.radius(grip.other()),
            if self.axis_stationary(grip) { 0.5 } else { 0.0 },
        );

        sector_points(TAU / self.n(grip) as f32).map(move |p| self.center(grip) + p * radius * sign)
    }

    fn shared_color(self, brightness: f32, dark_mode: bool) -> Color32 {
        if self.b == 2 {
            sample_rainbow(0, 1, brightness * 0.5)
        } else {
            sample_rainbow(0, 1, brightness * if dark_mode { 0.45 } else { 0.65 })
        }
    }
    fn a_color(self, i: u32, brightness: f32, dark_mode: bool) -> Color32 {
        if i == 0 {
            self.shared_color(brightness, dark_mode)
        } else {
            sample_rainbow(self.a - i, self.a, brightness * 0.5)
        }
    }
    fn b_color(self, i: u32, brightness: f32, dark_mode: bool) -> Color32 {
        if i == 0 {
            self.shared_color(brightness, dark_mode)
        } else if self.b == 2 {
            Color32::DARK_GRAY
        } else {
            sample_rainbow(i, self.b, brightness * if dark_mode { 0.25 } else { 0.75 })
        }
    }

    pub fn color(self, i: u32, brightness: f32, dark_mode: bool) -> Color32 {
        if i < self.a {
            self.a_color(i, brightness, dark_mode)
        } else {
            self.b_color(i - self.a + 1, brightness, dark_mode)
        }
    }
    pub fn sticker_color(self, i: u32, dark_mode: bool) -> Color32 {
        self.color(i, if dark_mode { 1.0 } else { 0.85 }, dark_mode)
    }
    pub fn sector_color(self, i: u32, dark_mode: bool) -> Color32 {
        self.color(i, 0.9, dark_mode)
    }

    pub fn color_index_in_grip(self, grip: Grip, i: u32) -> u32 {
        if grip == B && i > 0 {
            i + self.a - 1
        } else {
            i
        }
    }

    pub fn sector_name(self, grip: Grip, i: u32) -> String {
        if i == 0 {
            return "•".to_string();
        }
        match grip {
            A => ((b'A' + i as u8 - 1) as char).to_string(),
            B => (self.b - i).to_string(),
        }
    }
    pub fn sticker_name(self, i: u32) -> String {
        if i == 0 {
            return "•".to_string();
        }
        if i < self.n(A) {
            ((b'A' + i as u8 - 1) as char).to_string()
        } else {
            (self.a + self.b - 1 - i).to_string()
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

/// Samples a rainbow with `n` colors at index `i`. `lightness` ranges from 0 to
/// 1, with 0.5 being default.
fn sample_rainbow(i: u32, n: u32, lightness: f32) -> Color32 {
    let colorous::Color { r, g, b } = colorous::RAINBOW.eval_rational(i as usize, n as usize);
    let blend_color = if lightness > 0.5 {
        Color32::WHITE.gamma_multiply(lightness * 2.0 - 1.0)
    } else {
        Color32::BLACK.gamma_multiply(1.0 - lightness * 2.0)
    };
    Color32::from_rgb(r, g, b).blend(blend_color)
}
