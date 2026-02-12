use std::time::Duration;

use gpui::{
    App, Application, Bounds, Context, InteractiveElement, Render, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_anim::api::wrapper::TransitionExt;
use gpui_anim::transition::curves::{EaseInOutCubic, EaseOutQuad, EaseOutSine, Linear};

struct MultiElements {
    selected: Option<usize>,
}

impl Render for MultiElements {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_6()
            .size(px(700.0))
            .items_center()
            .justify_center()
            .bg(rgb(0x1a1a1a))
            .child(
                div()
                    .text_color(rgb(0xcccccc))
                    .text_xl()
                    .child("Multiple Elements with Different Curves"),
            )
            .child(
                div()
                    .text_color(rgb(0x888888))
                    .text_sm()
                    .child("Click cards to select, watch different easing effects"),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(self.card(0, "Linear", rgb(0x3b82f6), Linear, cx))
                    .child(self.card(1, "EaseOut", rgb(0x10b981), EaseOutQuad, cx))
                    .child(self.card(2, "Sine", rgb(0xf59e0b), EaseOutSine, cx))
                    .child(self.card(3, "Cubic", rgb(0xef4444), EaseInOutCubic, cx)),
            )
            .child(
                div()
                    .mt_4()
                    .text_color(rgb(0x666666))
                    .text_xs()
                    .child(format!(
                        "Selected: {}",
                        self.selected
                            .map(|i| format!("Card {}", i))
                            .unwrap_or_else(|| "None".to_string())
                    )),
            )
    }
}

impl MultiElements {
    fn card<T: gpui_anim::transition::Transition + 'static>(
        &self,
        index: usize,
        label: &'static str,
        color: gpui::Rgba,
        curve: T,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_selected = self.selected == Some(index);

        div()
            .id(format!("card-{}", index))
            .w(px(120.0))
            .h_32()
            .rounded_lg()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_1()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(format!("Card {}", index)),
            )
            .child(div().text_xs().text_color(rgb(0xcccccc)).child(label))
            .with_transition(format!("card-{}", index))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.selected = Some(index);
                cx.notify();
            }))
            .transition_when(
                is_selected,
                Duration::from_millis(2000),
                curve,
                move |state| state.bg(color).h(px(144.0)),
            )
            .transition_when(!is_selected, Duration::from_millis(2000), Linear, |state| {
                state.bg(rgb(0x374151)).h_32()
            })
            .bg(color)
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| MultiElements { selected: None }),
        )
        .unwrap();
    });
}
