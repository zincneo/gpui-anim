//! Core animation state and minimal interpolation support.

use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::{StyleRefinement, Styled};

use crate::api::types::AnimPriority;
use crate::interpolate::traits::FastInterpolatable;
use crate::transition::Transition;

/// Core animation state.
#[derive(Clone)]
pub struct AnimState<T: FastInterpolatable + Default + PartialEq> {
    pub origin: T,
    pub from: T,
    pub to: T,
    pub cur: T,
    pub progress: f32,
    pub start_at: Instant,
    pub version: usize,
    pub priority: AnimPriority,
}

impl<T: FastInterpolatable + Default + PartialEq> Default for AnimState<T> {
    fn default() -> Self {
        Self {
            origin: T::default(),
            from: T::default(),
            to: T::default(),
            cur: T::default(),
            progress: 1.0,
            start_at: Instant::now(),
            version: 0,
            priority: AnimPriority::default(),
        }
    }
}

impl<T: FastInterpolatable + Default + PartialEq> PartialEq for AnimState<T> {
    fn eq(&self, other: &Self) -> bool {
        self.to == other.to
    }
}

impl<T: FastInterpolatable + Default + PartialEq> AnimState<T> {
    /// Initialize state from an initial value.
    pub fn new(init: T) -> Self {
        Self {
            origin: init.clone(),
            from: init.clone(),
            to: init.clone(),
            cur: init,
            ..Default::default()
        }
    }

    /// Prepare for animation and return (version, effective_duration).
    pub fn pre_animated(&mut self, duration: Duration) -> (usize, Duration) {
        self.version += 1;

        let is_reversing = self.to == self.from;
        let effective = if is_reversing {
            duration.mul_f32(self.progress)
        } else {
            duration
        };

        self.from = self.cur.clone();
        self.start_at = Instant::now();
        self.progress = 0.0;

        (self.version, effective)
    }

    /// Advance animation. Returns true when finished.
    pub fn animated(
        &mut self,
        snapshot_version: usize,
        duration: Duration,
        transition: &Arc<dyn Transition>,
    ) -> bool {
        if snapshot_version != self.version {
            return true;
        }

        self.progress = transition.run(self.start_at, duration);

        if self.progress >= 1.0 {
            self.cur = self.to.clone();
            return true;
        }

        self.from
            .fast_interpolate(&self.to, self.progress, &mut self.cur);
        false
    }

    /// Reset animation target to origin.
    pub fn origin(mut self) -> Self {
        self.to = self.origin.clone();
        self
    }
}

impl Styled for AnimState<StyleRefinement> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.to
    }
}
