use serde::{Deserialize, Serialize};

use super::{Grip, PuzzleConfig};

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

    pub fn rot(&self, grip: Grip) -> u32 {
        match grip {
            Grip::A => self.a_rot,
            Grip::B => self.b_rot,
        }
    }
    pub fn pieces(&self, grip: Grip) -> &[u32] {
        match grip {
            Grip::A => &self.a_pieces,
            Grip::B => &self.b_pieces,
        }
    }

    fn rot_mut(&mut self, grip: Grip) -> &mut u32 {
        match grip {
            Grip::A => &mut self.a_rot,
            Grip::B => &mut self.b_rot,
        }
    }
    fn pieces_mut(&mut self, grip: Grip) -> &mut [u32] {
        match grip {
            Grip::A => &mut self.a_pieces,
            Grip::B => &mut self.b_pieces,
        }
    }

    pub fn n(&self, grip: Grip) -> u32 {
        self.pieces(grip).len() as u32
    }

    pub fn twist_ccw(&mut self, grip: Grip, amt: u32) {
        let n = self.n(grip);
        let amt = amt.rem_euclid(n);
        *self.rot_mut(grip) = (self.rot(grip) + amt) % n;
        self.pieces_mut(grip).rotate_left(amt as usize);
        self.set_shared_piece(self.pieces(grip)[0]);
    }
    pub fn twist_cw(&mut self, grip: Grip, amt: u32) {
        let n = self.n(grip);
        let amt = amt.rem_euclid(n);
        *self.rot_mut(grip) = (self.rot(grip) + n - amt) % n;
        self.pieces_mut(grip).rotate_right(amt as usize);
        self.set_shared_piece(self.pieces(grip)[0]);
    }

    fn set_shared_piece(&mut self, p: u32) {
        self.a_pieces[0] = p;
        self.b_pieces[0] = p;
    }
}
