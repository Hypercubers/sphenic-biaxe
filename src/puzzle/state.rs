use serde::{Deserialize, Serialize};

use super::PuzzleConfig;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PuzzleState {
    pub a_rot: u32,
    pub b_rot: u32,
    pub a_pieces: Vec<u32>,
    pub b_pieces: Vec<u32>,
}

impl PuzzleState {
    pub fn new(config: PuzzleConfig) -> Self {
        let a_pieces = (0..).take(config.a as usize).collect();
        let mut b_pieces: Vec<_> = (config.a - 1..).take(config.b as usize).collect();
        b_pieces[0] = 0;
        Self {
            a_rot: 0,
            b_rot: 0,
            a_pieces,
            b_pieces,
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
