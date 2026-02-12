//! Core animation engine specialized for GPUI `StyleRefinement`.
//!
//! This module defines a minimal in-memory engine that can store animation
//! state, accept requests, and advance state on tick.

use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

use dashmap::DashMap;
use gpui::{ElementId, StyleRefinement};

use crate::api::types::{AnimEvent, AnimPriority};
use crate::core::policies::{
    DefaultInterruptionPolicy, DefaultPriorityPolicy, InterruptionPolicy, PriorityPolicy,
};
use crate::core::scheduler::AnimScheduler;
use crate::core::state::AnimState;
use crate::transition::Transition;

/// Minimal animation request used by the engine.
pub struct AnimRequest {
    pub id: ElementId,
    pub event: AnimEvent,
    pub duration: Duration,
    pub transition: Arc<dyn Transition>,
    pub priority: AnimPriority,
    pub modifier:
        Box<dyn FnOnce(AnimState<StyleRefinement>) -> AnimState<StyleRefinement> + Send + Sync>,
    pub persistent: bool,
    pub initial_style: StyleRefinement,
}

/// Internal tracking for active animations.
struct ActiveAnim {
    event: AnimEvent,
    duration: Duration,
    origin_duration: Duration,
    transition: Arc<dyn Transition>,
    version: usize,
    persistent: bool,
}

/// Saved persistent context to resume after interruptions.
struct PersistentContext {
    event: AnimEvent,
    style: StyleRefinement,
    duration: Duration,
    transition: Arc<dyn Transition>,
    priority: AnimPriority,
}

/// Minimal in-memory animation engine specialized for `StyleRefinement`.
///
/// This version uses `DashMap` for concurrent access. It is intended as the
/// first step toward a complete engine.
pub struct AnimEngine {
    states: DashMap<ElementId, AnimState<StyleRefinement>>,
    active: DashMap<ElementId, ActiveAnim>,
    saved_contexts: DashMap<ElementId, PersistentContext>,
}

impl Default for AnimEngine {
    fn default() -> Self {
        Self {
            states: DashMap::new(),
            active: DashMap::new(),
            saved_contexts: DashMap::new(),
        }
    }
}

/// Global, lazily initialized animation engine.
static ENGINE: LazyLock<AnimEngine> = LazyLock::new(AnimEngine::default);

/// Default priority policy (higher/equal wins).
static PRIORITY_POLICY: LazyLock<DefaultPriorityPolicy> =
    LazyLock::new(DefaultPriorityPolicy::default);

/// Default interruption policy (allow all).
static INTERRUPTION_POLICY: LazyLock<DefaultInterruptionPolicy> =
    LazyLock::new(DefaultInterruptionPolicy::default);

/// Get a reference to the global animation engine.
///
/// This is the primary entry point used by the API layer to submit
/// animation requests and advance ticks.
pub fn engine() -> &'static AnimEngine {
    &ENGINE
}

impl AnimEngine {
    /// Submit a new animation request.
    ///
    /// If the target state changes, the animation is registered as active.
    pub fn submit(&self, request: AnimRequest) {
        let AnimRequest {
            id,
            event,
            duration,
            transition,
            priority,
            modifier,
            persistent,
            initial_style,
        } = request;

        let mut state = self
            .states
            .entry(id.clone())
            .or_insert_with(|| AnimState::new(initial_style));

        if !PRIORITY_POLICY.should_override(state.priority, priority) {
            return;
        }

        if let Some(active) = self.active.get(&id) {
            if !INTERRUPTION_POLICY.can_interrupt(&active.event, &event) {
                return;
            }

            if active.persistent && matches!(active.event, AnimEvent::Hover) {
                self.saved_contexts.insert(
                    id.clone(),
                    PersistentContext {
                        event: active.event.clone(),
                        style: state.to.clone(),
                        duration: active.origin_duration,
                        transition: active.transition.clone(),
                        priority: state.priority,
                    },
                );
            }
        }

        state.priority = priority;

        let snapshot = state.clone();
        let next = modifier(state.clone());

        *state = next;

        if snapshot != *state {
            let (version, effective) = state.pre_animated(duration);
            self.active.insert(
                id,
                ActiveAnim {
                    event,
                    duration: effective,
                    origin_duration: duration,
                    transition,
                    version,
                    persistent,
                },
            );
            AnimScheduler::notify_tick();
        } else {
            state.priority = AnimPriority::Lowest;
        }
    }

    /// Advance all active animations by one tick.
    ///
    /// Returns `true` if any state changed.
    pub fn tick(&self) -> bool {
        let mut changed = false;

        let active_entries: Vec<(
            ElementId,
            Duration,
            Arc<dyn Transition>,
            usize,
            bool,
            Duration,
            AnimEvent,
        )> = self
            .active
            .iter()
            .map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().duration,
                    entry.value().transition.clone(),
                    entry.value().version,
                    entry.value().persistent,
                    entry.value().origin_duration,
                    entry.value().event.clone(),
                )
            })
            .collect();

        let mut finished = Vec::new();
        for (id, duration, transition, version, _persistent, _origin, _event) in active_entries {
            if let Some(mut state) = self.states.get_mut(&id) {
                changed = true;

                if state.animated(version, duration, &transition) {
                    state.priority = AnimPriority::Lowest;
                    finished.push(id);
                }
            } else {
                finished.push(id);
            }
        }

        for id in finished {
            let active = self.active.remove(&id);

            if let Some((_, ctx)) = self.saved_contexts.remove(&id) {
                if let Some(mut state) = self.states.get_mut(&id) {
                    let restored_priority = if matches!(ctx.event, AnimEvent::Hover) {
                        ctx.priority.max(AnimPriority::Medium)
                    } else {
                        ctx.priority
                    };
                    state.priority = restored_priority;
                    state.to = ctx.style;

                    let (version, effective) = state.pre_animated(ctx.duration);
                    self.active.insert(
                        id.clone(),
                        ActiveAnim {
                            event: ctx.event,
                            duration: effective,
                            origin_duration: ctx.duration,
                            transition: ctx.transition,
                            version,
                            persistent: true,
                        },
                    );
                    AnimScheduler::notify_tick();
                }

                continue;
            }

            if let Some((_, active)) = active {
                if !active.persistent {
                    if let Some(mut state) = self.states.get_mut(&id) {
                        if state.to != state.origin {
                            state.to = state.origin.clone();

                            let (version, effective) = state.pre_animated(active.origin_duration);
                            self.active.insert(
                                id.clone(),
                                ActiveAnim {
                                    event: AnimEvent::None,
                                    duration: effective,
                                    origin_duration: active.origin_duration,
                                    transition: active.transition,
                                    version,
                                    persistent: false,
                                },
                            );
                            AnimScheduler::notify_tick();
                        }
                    }
                }
            }
        }

        changed
    }

    /// Get the current state for an element, if present.
    pub fn state(
        &self,
        id: &ElementId,
    ) -> Option<dashmap::mapref::one::Ref<'_, ElementId, AnimState<StyleRefinement>>> {
        self.states.get(id)
    }

    /// Returns true when any animations are currently active.
    pub fn has_active_animations(&self) -> bool {
        !self.active.is_empty()
    }
}
