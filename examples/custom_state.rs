use std::time::Duration;

use gpui::{
    App, Bounds, Context, InteractiveElement, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::Linear;
use gpui_platform::application;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Idle,
    Loading,
    Success,
    Error,
}

struct CustomStateExample {
    state: AppState,
}

impl Render for CustomStateExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .size(px(600.0))
            .items_center()
            .justify_center()
            .bg(rgb(0x1a1a1a))
            .child(
                div()
                    .text_color(rgb(0xcccccc))
                    .text_lg()
                    .child("State-Driven Animation Example"),
            )
            .child(
                div()
                    .text_color(rgb(0x888888))
                    .text_sm()
                    .child("Click buttons to change state and watch the animated box"),
            )
            .child(
                div()
                    .id("state-box")
                    .size_32()
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0xffffff))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(self.state_text())
                    .with_transition("state-box")
                    .transition_when(
                        self.state == AppState::Idle,
                        Duration::from_millis(2000),
                        Linear,
                        |state| state.bg(rgb(0x6b7280)).size_32(),
                    )
                    .transition_when(
                        self.state == AppState::Loading,
                        Duration::from_millis(2000),
                        Linear,
                        |state| state.bg(rgb(0x3b82f6)).size_40(),
                    )
                    .transition_when(
                        self.state == AppState::Success,
                        Duration::from_millis(2000),
                        Linear,
                        |state| state.bg(rgb(0x10b981)).size_48(),
                    )
                    .transition_when(
                        self.state == AppState::Error,
                        Duration::from_millis(2000),
                        Linear,
                        |state| state.bg(rgb(0xef4444)).size_40(),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(self.state_button("Idle", AppState::Idle, cx))
                    .child(self.state_button("Loading", AppState::Loading, cx))
                    .child(self.state_button("Success", AppState::Success, cx))
                    .child(self.state_button("Error", AppState::Error, cx)),
            )
            .child(
                div()
                    .mt_4()
                    .text_color(rgb(0x666666))
                    .text_xs()
                    .child(format!("Current state: {:?}", self.state)),
            )
    }
}

impl CustomStateExample {
    fn state_text(&self) -> &'static str {
        match self.state {
            AppState::Idle => "○",
            AppState::Loading => "⟳",
            AppState::Success => "✓",
            AppState::Error => "✗",
        }
    }

    fn state_button(
        &self,
        label: &'static str,
        target_state: AppState,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_active = self.state == target_state;

        div()
            .id(format!("btn-{:?}", target_state))
            .px_4()
            .py_2()
            .rounded_md()
            .text_color(if is_active {
                rgb(0xffffff)
            } else {
                rgb(0xcccccc)
            })
            .bg(if is_active {
                rgb(0x3b82f6)
            } else {
                rgb(0x374151)
            })
            .child(label)
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.state = target_state;
                // Trigger re-render to apply declarative transitions
                cx.notify();
            }))
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(700.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| CustomStateExample {
                    state: AppState::Idle,
                })
            },
        )
        .unwrap();
    });
}
