//! Controller inputs: core sensors followed by Eye Bins.

use crate::geometry::{Torus, Vec2, normalize_angle};

/// The world as seen through one eye: sectors spanning `heading ± fov/2`,
/// bin 0 at the clockwise edge (`heading - fov/2`).
pub(crate) struct Eye {
    pub position: Vec2,
    pub heading: f64,
    pub range: f64,
    pub field_of_view: f64,
    pub bins: usize,
}

impl Eye {
    /// Per sector, `1 - distance / range` for the nearest target in that
    /// sector, or 0 when there is none.
    pub fn sense(&self, torus: Torus, targets: impl Iterator<Item = Vec2>) -> Vec<f64> {
        let mut nearest = vec![f64::INFINITY; self.bins];
        let half = self.field_of_view / 2.0;
        for target in targets {
            let offset = torus.displacement(self.position, target);
            let distance = offset.length();
            if distance > self.range || distance == 0.0 {
                continue;
            }
            let relative = normalize_angle(offset.angle() - self.heading);
            if relative.abs() > half {
                continue;
            }
            let sector = ((relative + half) / self.field_of_view * self.bins as f64) as usize;
            let sector = sector.min(self.bins - 1);
            nearest[sector] = nearest[sector].min(distance);
        }
        nearest
            .into_iter()
            .map(|d| {
                if d.is_finite() {
                    1.0 - d / self.range
                } else {
                    0.0
                }
            })
            .collect()
    }
}
