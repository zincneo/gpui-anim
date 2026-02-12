//! Shared metrics for gpui-anim.
//!
//! Currently holds the global rem size used by interpolation.

use std::sync::atomic::{AtomicU32, Ordering};

use gpui::{Pixels, Window};

static REM_SIZE_BITS: AtomicU32 = AtomicU32::new(16.0f32.to_bits());

/// Update global rem size from a window.
pub fn update_rem_size_from_window(window: &Window) {
    set_rem_size(window.rem_size());
}

/// Set the global rem size.
pub fn set_rem_size(value: Pixels) {
    let raw: f32 = unsafe { std::mem::transmute(value) };
    REM_SIZE_BITS.store(raw.to_bits(), Ordering::Relaxed);
}

/// Get the current global rem size.
pub fn rem_size() -> Pixels {
    let raw = f32::from_bits(REM_SIZE_BITS.load(Ordering::Relaxed));
    Pixels::from(raw)
}
