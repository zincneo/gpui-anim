use std::time::Duration;

use gpui::{
    App, Application, Bounds, Context, InteractiveElement, Render, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_anim::api::types::AnimPriority;
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::{EaseInOutCubic, Linear};

struct NestedPriority {
    show_notification: bool,
}

impl Render for NestedPriority {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .size(px(700.0))
            .items_center()
            .justify_center()
            .bg(rgb(0x0f172a))
            .child(
                div()
                    .text_color(rgb(0xcccccc))
                    .text_xl()
                    .child("Nested Animations with Priority"),
            )
            .child(
                div()
                    .text_color(rgb(0x888888))
                    .text_sm()
                    .child("Hover parent, then click child to see priority handling"),
            )
            .child(
                // Parent container with hover (Medium priority)
                div()
                    .id("parent-box")
                    .w(px(400.0))
                    .h(px(300.0))
                    .rounded_xl()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .p_8()
                    .with_transition("parent-box")
                    .transition_on_hover(
                        Duration::from_millis(2000),
                        EaseInOutCubic,
                        |hovered, state| {
                            if *hovered {
                                state
                                    .bg(rgb(0x1e293b))
                                    .border_2()
                                    .border_color(rgb(0x3b82f6))
                            } else {
                                state
                                    .bg(rgb(0x1e3a5f))
                                    .border_1()
                                    .border_color(rgb(0x334155))
                            }
                        },
                    )
                    .bg(rgb(0x1e3a5f))
                    .border_1()
                    .border_color(rgb(0x334155))
                    .child(
                        div()
                            .text_color(rgb(0xffffff))
                            .text_lg()
                            .child("Parent (Hover Priority: Medium)"),
                    )
                    .child(
                        // Child button with click (High priority)
                        div()
                            .id("child-button")
                            .px_6()
                            .py_3()
                            .rounded_lg()
                            .text_color(rgb(0xffffff))
                            .child("Click me (High Priority)")
                            .with_transition("child-button")
                            .transition_on_hover_with_priority(
                                Duration::from_millis(2000),
                                Linear,
                                AnimPriority::Medium,
                                |hovered, state| {
                                    if *hovered {
                                        state.bg(rgb(0x2563eb))
                                    } else {
                                        state.bg(rgb(0x3b82f6))
                                    }
                                },
                            )
                            .transition_on_click_with_priority(
                                Duration::from_millis(2000),
                                Linear,
                                AnimPriority::High,
                                |_, state| state.bg(rgb(0x10b981)).size_40(),
                            )
                            .bg(rgb(0x3b82f6)),
                    )
                    .child(
                        // Another child with lower priority
                        div()
                            .id("low-priority-box")
                            .size_20()
                            .rounded_md()
                            .with_transition("low-priority-box")
                            .transition_on_hover_with_priority(
                                Duration::from_millis(2000),
                                Linear,
                                AnimPriority::Low,
                                |hovered, state| {
                                    if *hovered {
                                        state.bg(rgb(0xfbbf24)).size_24()
                                    } else {
                                        state.bg(rgb(0xf59e0b)).size_20()
                                    }
                                },
                            )
                            .bg(rgb(0xf59e0b)),
                    ),
            )
            .child(
                // Notification bar (controlled by button, High priority)
                div()
                    .id("notification")
                    .w(px(400.0))
                    .h_16()
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0xffffff))
                    .child(if self.show_notification {
                        "Notification Active!"
                    } else {
                        "Notification Hidden"
                    })
                    .with_transition("notification")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.show_notification = !this.show_notification;
                        cx.notify();
                    }))
                    .transition_when_with_priority(
                        self.show_notification,
                        Duration::from_millis(2000),
                        EaseInOutCubic,
                        AnimPriority::High,
                        |state| state.bg(rgb(0xef4444)).opacity(1.0),
                    )
                    .transition_when_with_priority(
                        !self.show_notification,
                        Duration::from_millis(2000),
                        EaseInOutCubic,
                        AnimPriority::High,
                        |state| state.bg(rgb(0x6b7280)).opacity(0.3),
                    )
                    .bg(rgb(0x6b7280))
                    .opacity(0.3),
            )
            .child(
                div()
                    .mt_4()
                    .text_color(rgb(0x666666))
                    .text_xs()
                    .child("Priority order: High > Medium > Low"),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| NestedPriority {
                    show_notification: false,
                })
            },
        )
        .unwrap();
    });
}
