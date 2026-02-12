use std::time::Duration;

use gpui::{
    App, Application, Bounds, Context, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::{EaseInExpo, Linear};

struct HoverClickCombo {
    outer_hovered: bool,
}

impl Render for HoverClickCombo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .size(px(600.0))
            .items_center()
            .justify_center()
            .bg(rgb(0x0a0a0a))
            .child(
                div()
                    .text_color(rgb(0xcccccc))
                    .text_lg()
                    .child("Hover + Click Combination"),
            )
            .child(
                div()
                    .text_color(rgb(0x888888))
                    .text_sm()
                    .child("Hover to scale, click for flash effect"),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .id("box1")
                            .size_24()
                            .bg(rgb(0x3b82f6))
                            .rounded_lg()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0xffffff))
                            .child("1")
                            .with_transition("box1")
                            .transition_on_hover(
                                Duration::from_millis(2000),
                                Linear,
                                |hovered, state| {
                                    if *hovered {
                                        state.size_32().bg(rgb(0x60a5fa))
                                    } else {
                                        state.size_24().bg(rgb(0x3b82f6))
                                    }
                                },
                            )
                            .transition_on_click(
                                Duration::from_millis(2000),
                                EaseInExpo,
                                |_, state| state.bg(rgb(0xfbbf24)).size_40(),
                            ),
                    )
                    .child(
                        div()
                            .id("box2")
                            .size_24()
                            .bg(rgb(0x10b981))
                            .rounded_lg()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0xffffff))
                            .child("2")
                            .with_transition("box2")
                            .transition_on_hover(
                                Duration::from_millis(2000),
                                Linear,
                                |hovered, state| {
                                    if *hovered {
                                        state.size_32().bg(rgb(0x34d399))
                                    } else {
                                        state.size_24().bg(rgb(0x10b981))
                                    }
                                },
                            )
                            .transition_on_click(
                                Duration::from_millis(2000),
                                EaseInExpo,
                                |_, state| state.bg(rgb(0xef4444)).size_40(),
                            ),
                    )
                    .child(
                        div()
                            .id("box3")
                            .size_24()
                            .bg(rgb(0xa855f7))
                            .rounded_lg()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0xffffff))
                            .child("3")
                            .with_transition("box3")
                            .transition_on_hover(
                                Duration::from_millis(2000),
                                Linear,
                                |hovered, state| {
                                    if *hovered {
                                        state.size_32().bg(rgb(0xc084fc))
                                    } else {
                                        state.size_24().bg(rgb(0xa855f7))
                                    }
                                },
                            )
                            .transition_on_click(
                                Duration::from_millis(2000),
                                EaseInExpo,
                                |_, state| state.bg(rgb(0x06b6d4)).size_40(),
                            ),
                    ),
            )
            .child(
                div()
                    .id("outer-box")
                    .mt_8()
                    .w(px(400.0))
                    .h_32()
                    .rounded_lg()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0xffffff))
                    .child(if self.outer_hovered {
                        "Outer hover active!"
                    } else {
                        "Hover this container"
                    })
                    .with_transition("outer-box")
                    .on_hover(cx.listener(|this, hovered, _, cx| {
                        this.outer_hovered = *hovered;
                        cx.notify();
                    }))
                    .transition_when(
                        self.outer_hovered,
                        Duration::from_millis(2000),
                        Linear,
                        |state| state.bg(rgb(0x6366f1)).w(px(500.0)),
                    )
                    .transition_when(
                        !self.outer_hovered,
                        Duration::from_millis(2000),
                        Linear,
                        |state| state.bg(rgb(0x4b5563)).w(px(400.0)),
                    )
                    .transition_on_click(Duration::from_millis(2000), EaseInExpo, |_, state| {
                        state.bg(rgb(0xf59e0b))
                    })
                    .bg(rgb(0x4b5563)),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(700.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HoverClickCombo {
                    outer_hovered: false,
                })
            },
        )
        .unwrap();
    });
}
