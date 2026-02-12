//! Transition curve implementations.

use crate::transition::Transition;

/// Linear transition curve (no easing).
#[derive(Debug, Clone, Copy, Default)]
pub struct Linear;

impl Transition for Linear {
    #[inline]
    fn calculate(&self, t: f32) -> f32 {
        t
    }
}

/// Ease-in quadratic curve.
#[derive(Debug, Clone, Copy, Default)]
pub struct EaseInQuad;

impl Transition for EaseInQuad {
    #[inline]
    fn calculate(&self, t: f32) -> f32 {
        t * t
    }
}

/// Ease-out quadratic curve.
#[derive(Debug, Clone, Copy, Default)]
pub struct EaseOutQuad;

impl Transition for EaseOutQuad {
    #[inline]
    fn calculate(&self, t: f32) -> f32 {
        1.0 - (1.0 - t) * (1.0 - t)
    }
}

/// Ease-in-out quadratic curve.
#[derive(Debug, Clone, Copy, Default)]
pub struct EaseInOutQuad;

impl Transition for EaseInOutQuad {
    #[inline]
    fn calculate(&self, t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
        }
    }
}

/// Ease-in-out cubic curve.
#[derive(Debug, Clone, Copy, Default)]
pub struct EaseInOutCubic;

impl Transition for EaseInOutCubic {
    #[inline]
    fn calculate(&self, t: f32) -> f32 {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
        }
    }
}

/// Ease-out sine curve.
#[derive(Debug, Clone, Copy, Default)]
pub struct EaseOutSine;

impl Transition for EaseOutSine {
    #[inline]
    fn calculate(&self, t: f32) -> f32 {
        (t * std::f32::consts::PI / 2.0).sin()
    }
}

/// Ease-in exponential curve.
#[derive(Debug, Clone, Copy, Default)]
pub struct EaseInExpo;

impl Transition for EaseInExpo {
    #[inline]
    fn calculate(&self, t: f32) -> f32 {
        if t == 0.0 {
            0.0
        } else {
            (2.0f32).powf(10.0 * t - 10.0)
        }
    }
}
