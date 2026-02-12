//! Interpolation traits (placeholder).
//!
//! These traits define the contracts for value interpolation. They are
//! intentionally minimal for the initial scaffold.

/// Trait for producing a new interpolated value.
pub trait Interpolatable: Clone {
    /// Interpolate between `self` and `other` at progress `t` in [0, 1].
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

/// Trait for in-place interpolation to reduce allocations.
pub trait FastInterpolatable: Clone {
    /// Interpolate between `self` and `other` at progress `t` into `out`.
    fn fast_interpolate(&self, other: &Self, t: f32, out: &mut Self);
}
