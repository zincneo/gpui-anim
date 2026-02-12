//! Animation scheduler with idle waiting.
//!
//! This scheduler drives the global engine tick at a fixed cadence and parks
//! when there are no active animations.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui::{App, AsyncApp};
use smol::channel::{self, Receiver, Sender};

use crate::core::engine::engine;

const DEFAULT_FPS: f32 = 120.0;

pub struct AnimScheduler {
    started: AtomicBool,
    wake_tx: Sender<()>,
    wake_rx: Receiver<()>,
}

static SCHEDULER: LazyLock<AnimScheduler> = LazyLock::new(|| {
    let (tx, rx) = channel::unbounded();
    AnimScheduler {
        started: AtomicBool::new(false),
        wake_tx: tx,
        wake_rx: rx,
    }
});

impl AnimScheduler {
    /// Initialize the scheduler loop once.
    pub fn init(cx: &mut App) {
        if !SCHEDULER.started.swap(true, Ordering::SeqCst) {
            cx.spawn(Self::run).detach();
        }
    }

    /// Wake the scheduler loop when new animations are submitted.
    pub fn notify_tick() {
        SCHEDULER.wake_tx.try_send(()).ok();
    }

    async fn run(cx: &mut AsyncApp) {
        let frame_duration = Duration::from_secs_f32(1.0 / DEFAULT_FPS);

        loop {
            // Drive the engine once per frame.
            let did_change = engine().tick();

            if did_change {
                cx.update(|cx| cx.refresh_windows());
            }

            if engine().has_active_animations() {
                smol::Timer::after(frame_duration).await;
            } else {
                // Park until an animation is submitted.
                SCHEDULER.wake_rx.recv().await.ok();
            }
        }
    }
}
