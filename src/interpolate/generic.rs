//! Generic interpolation helpers.

use crate::interpolate::traits::{FastInterpolatable, Interpolatable};

/// Interpolation for f32.
impl Interpolatable for f32 {
    #[inline]
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        *self + (*other - *self) * t
    }
}

/// In-place interpolation for f32.
impl FastInterpolatable for f32 {
    #[inline]
    fn fast_interpolate(&self, other: &Self, t: f32, out: &mut Self) {
        *out = self.interpolate(other, t);
    }
}
