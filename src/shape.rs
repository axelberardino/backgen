use crate::prelude::*;
use crate::svg::*;

/// A generic shape that can be placed at any position according to a given center
pub struct Movable(Vec<Pos>);

impl Movable {
    pub fn render(&self, reference: Pos) -> (Pos, Path) {
        let mut data = Data::new(reference + self.0[0]);
        for p in self.0.iter().skip(1) {
            data.line_to(reference + *p);
        }
        (reference, Path::new(data))
    }

    pub fn hexagon(size: f64, rot: isize) -> Self {
        let mut pts = Vec::new();
        for i in 0..6 {
            pts.push(Pos::polar(rot + 60 * i, size))
        }
        Movable(pts)
    }

    pub fn triangle(size: f64, rot: isize) -> Self {
        let mut pts = Vec::new();
        for i in 0..3 {
            pts.push(Pos::polar(rot + 120 * i, size))
        }
        Movable(pts)
    }

    pub fn square(size: f64, rot: isize) -> Self {
        let mut pts = Vec::new();
        for i in 0..4 {
            pts.push(Pos::polar(rot + 45 + 90 * i, size))
        }
        Movable(pts)
    }

    pub fn rhombus(ldiag: f64, sdiag: f64, rot: isize) -> Self {
        Movable(vec![
            Pos::polar(rot, ldiag),
            Pos::polar(rot + 90, sdiag),
            Pos::polar(rot + 180, ldiag),
            Pos::polar(rot + 270, sdiag),
        ])
    }

    pub fn from(v: Vec<Pos>) -> Self {
        Self(v)
    }

    pub fn vertex(&self, idx: usize) -> Pos {
        self.0[idx % self.0.len()]
    }

    pub fn side(&self, idx: usize) -> Pos {
        self.0[(idx + 1) % self.0.len()] - self.0[idx % self.0.len()]
    }
}
