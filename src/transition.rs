//! Transition traits and curve exports.

pub mod curves;

use std::sync::Arc;
use std::time::{Duration, Instant};

/// Animation timing curve interface.
///
/// Implementations should map a linear `t` in [0, 1] to an eased value.
pub trait Transition: Send + Sync + 'static {
    /// Run the transition based on `start` and total `duration`.
    fn run(&self, start: Instant, duration: Duration) -> f32 {
        let t = (start.elapsed().as_secs_f32() / duration.as_secs_f32()).min(1.0);
        self.calculate(t)
    }

    /// Map linear time `t` in [0, 1] to eased progress.
    fn calculate(&self, t: f32) -> f32;
}

/// Helper trait to convert a transition into `Arc<dyn Transition>`.
pub trait IntoArcTransition<T: Transition + 'static> {
    fn into_arc(self) -> Arc<T>;
}

impl<T: Transition + 'static> IntoArcTransition<T> for T {
    fn into_arc(self) -> Arc<T> {
        Arc::new(self)
    }
}

impl<T: Transition + 'static> IntoArcTransition<T> for Arc<T> {
    fn into_arc(self) -> Arc<T> {
        self
    }
}
