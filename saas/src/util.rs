use std::mem;
pub use std::num::NonZeroUsize;

extern crate serde;
use serde::{Deserialize, Serialize};

use rand::prelude::*;

// ++++++++++++++++
// + NonZeroUsize +
// ++++++++++++++++

pub trait NonZeroUsizeMath {
    fn one() -> NonZeroUsize;
    fn add(a: NonZeroUsize, b: NonZeroUsize) -> NonZeroUsize;
    fn decr(&self, a: usize) -> Option<NonZeroUsize>;
}

impl NonZeroUsizeMath for NonZeroUsize {
    fn one() -> Self {
        NonZeroUsize::new(1).unwrap()
    }

    fn add(a: NonZeroUsize, b: NonZeroUsize) -> NonZeroUsize {
        NonZeroUsize::new(a.get() + b.get()).unwrap()
    }

    fn decr(&self, a: usize) -> Option<Self> {
        NonZeroUsize::new(self.get() - a)
    }
}

// +++++++++
// + Timer +
// +++++++++

pub struct Timer(usize, usize);

impl Timer {
    pub fn new(limit: usize) -> Self {
        Timer(0, limit)
    }

    pub fn tick(&mut self) {
        mem::replace(self, Timer(self.0 + 1, self.1));
    }

    pub fn is_done(&self) -> bool {
        self.1 <= self.0
    }

    pub fn reset(&mut self) {
        mem::replace(self, Timer(0, self.1));
    }
}

// +++++++++++++
// + Direction +
// +++++++++++++

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn rand() -> Self {
        let dirs: [Direction; 4] = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];

        let mut rng = rand::thread_rng();
        *dirs.iter().choose(&mut rng).unwrap()
    }
}

// +++++++++++
// + Index2D +
// +++++++++++

#[derive(Debug, Clone, Copy)]
pub struct Index2D(i32, i32);

fn wrap(x: i32, lo: i32, hi: i32) -> i32 {
    if x < lo {
        hi - 1
    } else if hi <= x {
        lo
    } else {
        x
    }
}

impl Index2D {
    pub fn new(i: usize, j: usize) -> Self {
        Index2D(i as i32, j as i32)
    }

    pub fn rand_in_range(
        i_lo_hi: (usize, usize),
        j_lo_hi: (usize, usize),
    ) -> Self {
        let mut rng = rand::thread_rng();
        let (i_lo, i_hi) = i_lo_hi;
        let (j_lo, j_hi) = j_lo_hi;
        let i = (rng.next_u32() % i_hi as u32) + i_lo as u32;
        let j = (rng.next_u32() % j_hi as u32) + j_lo as u32;
        Index2D::new(i as usize, j as usize)
    }

    pub fn get(&self) -> (usize, usize) {
        (self.0 as usize, self.1 as usize)
    }

    pub fn get_u32(&self) -> (u32, u32) {
        (self.0 as u32, self.1 as u32)
    }

    pub fn neighbor(&self, dir: &Direction) -> Self {
        let (i, j) = (self.0, self.1);
        match dir {
            Direction::Up => Index2D(i - 1, j),
            Direction::Down => Index2D(i + 1, j),
            Direction::Left => Index2D(i, j - 1),
            Direction::Right => Index2D(i, j + 1),
        }
    }

    pub fn wrap_fst(mut self, lo: usize, hi: usize) -> Self {
        let (i, j) = (self.0, self.1);
        self = Index2D(wrap(i, lo as i32, hi as i32), j);
        self
    }

    pub fn wrap_snd(mut self, lo: usize, hi: usize) -> Self {
        let (i, j) = (self.0, self.1);
        self = Index2D(i, wrap(j, lo as i32, hi as i32));
        self
    }

    pub fn wrap(
        self,
        i_lo_hi: (usize, usize),
        j_lo_hi: (usize, usize),
    ) -> Self {
        let (i_lo, i_hi) = i_lo_hi;
        let (j_lo, j_hi) = j_lo_hi;

        self.wrap_fst(i_lo, i_hi).wrap_snd(j_lo, j_hi)
    }
}
