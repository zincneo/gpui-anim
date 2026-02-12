//! Policy definitions for animation priority and interruption.

use crate::api::types::{AnimEvent, AnimPriority};

/// Policy trait for resolving animation priority conflicts.
pub trait PriorityPolicy {
    /// Return true if `incoming` should override `current`.
    fn should_override(&self, current: AnimPriority, incoming: AnimPriority) -> bool;
}

/// Default priority policy: higher or equal priority wins.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultPriorityPolicy;

impl PriorityPolicy for DefaultPriorityPolicy {
    fn should_override(&self, current: AnimPriority, incoming: AnimPriority) -> bool {
        incoming >= current
    }
}

/// Policy trait for determining whether an animation can interrupt another.
pub trait InterruptionPolicy {
    /// Return true if `incoming_event` may interrupt `current_event`.
    fn can_interrupt(&self, current_event: &AnimEvent, incoming_event: &AnimEvent) -> bool;
}

/// Default interruption policy: allow all interruptions.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultInterruptionPolicy;

impl InterruptionPolicy for DefaultInterruptionPolicy {
    fn can_interrupt(&self, _current_event: &AnimEvent, _incoming_event: &AnimEvent) -> bool {
        true
    }
}
