//! Public API types for gpui-anim.

/// Animation event categories used to tag transitions and resolve conflicts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimEvent {
    None,
    Hover,
    Click,
    /// User-defined event tags.
    Custom(String),
}

/// Priority for resolving competing animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimPriority {
    Lowest = 0,
    Low = 25,
    Medium = 50,
    High = 75,
    Realtime = 100,
}

impl Default for AnimPriority {
    fn default() -> Self {
        AnimPriority::Lowest
    }
}
