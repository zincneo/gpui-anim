use std::time::Duration;

use gpui::{
    App, Application, Bounds, Context, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::Linear;

struct ClickExample;

impl Render for ClickExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size(px(400.0))
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("click-box")
                    .size_32()
                    .bg(rgb(0x3b82f6))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0xffffff))
                    .child("Click me")
                    .with_transition("click-box")
                    .transition_on_click(Duration::from_millis(2000), Linear, |_, state| {
                        state.bg(rgb(0xfbbf24)).size_48()
                    }),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.0), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| ClickExample),
        )
        .unwrap();
    });
}
