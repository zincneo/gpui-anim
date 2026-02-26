use std::time::Duration;

use gpui::{
    App, Bounds, Context, Render, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb,
    size,
};
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::Linear;
use gpui_platform::application;

struct HoverExample;

impl Render for HoverExample {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .size(px(400.0))
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("hover-box")
                    .size_32()
                    .bg(rgb(0x2e2e2e))
                    .with_transition("hover-box")
                    .transition_on_hover(Duration::from_millis(2000), Linear, |hovered, state| {
                        if *hovered {
                            state.bg(rgb(0xff0000)).size_64()
                        } else {
                            state.bg(rgb(0x2e2e2e)).size_32()
                        }
                    }),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.0), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HoverExample),
        )
        .unwrap();
    });
}
