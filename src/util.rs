use std::{
    f32::consts::PI,
    ops::{Add, Mul},
};

/// Interpolates between `a` and `b` in a smooth curve.
pub fn animate_twist_angle(a: f32, b: f32, t: f32) -> f32 {
    lerp(a, b, (1.0 - (t * PI).cos()) / 2.0)
}

/// Linearly interpolates (unclamped) between two numbers.
pub fn lerp<A>(a: A, b: A, t: f32) -> A
where
    A: Add<Output = A> + Mul<f32, Output = A>,
{
    a * (1.0 - t) + b * t
}
