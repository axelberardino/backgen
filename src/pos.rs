use crate::prelude::*;
use rand::{rngs::StdRng, Rng};
use std::cmp::{Eq, PartialEq};
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Pos(pub f64, pub f64);

impl Pos {
    pub fn into_tuple(self) -> (f64, f64) {
        (self.0, self.1)
    }

    // Nearest 100'th of unit
    pub fn round(self) -> (i32, i32) {
        (
            (self.0 * 100.).round() as i32,
            (self.1 * 100.).round() as i32,
        )
    }

    pub fn norm(self) -> f64 {
        self.dot_self().sqrt()
    }

    pub fn unit(self) -> Self {
        self * (1. / self.norm())
    }

    pub fn dot_self(self) -> f64 {
        self.0.powi(2) + self.1.powi(2)
    }

    pub fn dot(self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1
    }

    pub fn random(f: &Frame, rng: &mut StdRng) -> Self {
        let errx = f.w as f64 / 10.;
        let erry = f.h as f64 / 10.;
        let x = f.x as f64 - errx + rng.gen::<f64>() * f.w as f64 * 1.2;
        let y = f.y as f64 - erry + rng.gen::<f64>() * f.h as f64 * 1.2;
        Self(x, y)
    }

    pub fn dist(self, other: Self) -> f64 {
        (self - other).norm()
    }

    pub fn project(self, other: Self) -> Self {
        let other = other.unit();
        other * self.dot(other)
    }

    pub fn polar(a: isize, r: f64) -> Self {
        let theta = radians(a);
        Pos(r * theta.cos(), r * theta.sin())
    }

    pub fn intersect((pos1, rot1): (Self, isize), (pos2, rot2): (Self, isize)) -> Self {
        let pos1b = pos1 + Pos::polar(rot1, 1.);
        let pos2b = pos2 + Pos::polar(rot2, 1.);

        let dx = Pos(pos1.0 - pos1b.0, pos2.0 - pos2b.0);
        let dy = Pos(pos1.1 - pos1b.1, pos2.1 - pos2b.1);

        let det = |a: Pos, b: Pos| a.0 * b.1 - a.1 * b.0;

        let inv = {
            let div = det(dx, dy);
            if div.abs() < 0.01 {
                panic!("Malformed intersection");
            }
            1. / div
        };

        let d = Pos(det(pos1, pos1b), det(pos2, pos2b));
        let x = det(d, dx) * inv;
        let y = det(d, dy) * inv;
        Pos(x, y)
    }

    pub fn zero() -> Self {
        Self(0., 0.)
    }
}

pub fn crossprod_sign(a: Pos, b: Pos, c: Pos) -> bool {
    (a.0 - c.0) * (b.1 - c.1) - (b.0 - c.0) * (a.1 - c.1) > 0.
}

pub fn radians(a: isize) -> f64 {
    (a as f64) * PI / 180.
}

impl Add<(f64, f64)> for Pos {
    type Output = Self;
    fn add(self, (x, y): (f64, f64)) -> Self::Output {
        Pos(self.0 + x, self.1 + y)
    }
}

impl Add<Pos> for Pos {
    type Output = Self;
    fn add(self, Pos(x, y): Pos) -> Self::Output {
        Pos(self.0 + x, self.1 + y)
    }
}

impl Sub<Pos> for Pos {
    type Output = Self;
    fn sub(self, Pos(x, y): Pos) -> Self::Output {
        Pos(self.0 - x, self.1 - y)
    }
}

impl Mul<isize> for Pos {
    type Output = Self;
    fn mul(self, x: isize) -> Self::Output {
        Pos(self.0 * x as f64, self.1 * x as f64)
    }
}

impl Mul<f64> for Pos {
    type Output = Self;
    fn mul(self, x: f64) -> Self::Output {
        Pos(self.0 * x, self.1 * x)
    }
}

impl Neg for Pos {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0, -self.1)
    }
}

impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.round() == other.round()
    }
}

impl Eq for Pos {}

impl Hash for Pos {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.round().hash(state)
    }
}
