//! 2D vectors and torus arithmetic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn from_angle(angle: f64) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f64) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

/// The world's toroidal extent: positions live in `[0, width) × [0, height)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Torus {
    pub width: f64,
    pub height: f64,
}

impl Torus {
    pub fn wrap(self, p: Vec2) -> Vec2 {
        Vec2::new(wrap_coord(p.x, self.width), wrap_coord(p.y, self.height))
    }

    /// Shortest displacement from `from` to `to`, going across edges when shorter.
    pub fn displacement(self, from: Vec2, to: Vec2) -> Vec2 {
        Vec2::new(
            shortest_delta(to.x - from.x, self.width),
            shortest_delta(to.y - from.y, self.height),
        )
    }

    pub fn distance(self, a: Vec2, b: Vec2) -> f64 {
        self.displacement(a, b).length()
    }
}

fn wrap_coord(v: f64, extent: f64) -> f64 {
    let wrapped = v.rem_euclid(extent);
    // rem_euclid can round up to exactly `extent` for tiny negative inputs.
    if wrapped >= extent { 0.0 } else { wrapped }
}

fn shortest_delta(d: f64, extent: f64) -> f64 {
    let d = d.rem_euclid(extent);
    if d > extent / 2.0 { d - extent } else { d }
}

/// Normalise an angle to `[-π, π)`.
pub(crate) fn normalize_angle(a: f64) -> f64 {
    use std::f64::consts::{PI, TAU};
    (a + PI).rem_euclid(TAU) - PI
}
