use serde::{Deserialize, Serialize};

mod config;
mod state;
mod twist_anim;
mod view;

pub use config::PuzzleConfig;
pub use state::PuzzleState;
use twist_anim::{TwistAnimation, TwistAnimationState};
pub use view::PuzzleView;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Grip {
    A,
    B,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TwistDir {
    /// Clockwise
    Cw,
    /// Counterclockwise
    Ccw,
}
impl TwistDir {
    pub fn to_f32(self) -> f32 {
        match self {
            TwistDir::Cw => -1.0,
            TwistDir::Ccw => 1.0,
        }
    }
}
