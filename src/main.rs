#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{
    App, Bounds, Context, FocusHandle, KeyBinding, SharedString, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;

actions!(counter, [Increment, Decrement, Reset, Quit]);

struct Counter {
    count: i64,
    focus_handle: FocusHandle,
}

impl Counter {
    fn increment(&mut self, _: &Increment, _: &mut Window, cx: &mut Context<Self>) {
        self.count += 1;
        cx.notify();
    }

    fn decrement(&mut self, _: &Decrement, _: &mut Window, cx: &mut Context<Self>) {
        self.count -= 1;
        cx.notify();
    }

    fn reset(&mut self, _: &Reset, _: &mut Window, cx: &mut Context<Self>) {
        self.count = 0;
        cx.notify();
    }
}

/// クリック可能なボタン要素を作る
fn button(
    id: &'static str,
    label: &'static str,
    bg: u32,
    hover_bg: u32,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .justify_center()
        .items_center()
        .w(px(96.))
        .h(px(44.))
        .rounded_md()
        .bg(rgb(bg))
        .text_color(rgb(0xffffff))
        .text_lg()
        .cursor_pointer()
        .hover(move |s| s.bg(rgb(hover_bg)))
        .active(|s| s.opacity(0.8))
        .child(label)
        .on_click(on_click)
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count_color = match self.count {
            n if n > 0 => rgb(0x4ade80),
            n if n < 0 => rgb(0xf87171),
            _ => rgb(0xe5e7eb),
        };
        let count_text: SharedString = self.count.to_string().into();

        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::increment))
            .on_action(cx.listener(Self::decrement))
            .on_action(cx.listener(Self::reset))
            .font_family("Segoe UI")
            .size_full()
            .flex()
            .flex_col()
            .justify_center()
            .items_center()
            .gap_6()
            .bg(rgb(0x1f2330))
            .text_color(rgb(0xe5e7eb))
            .child(div().text_xl().child("GPUI Counter"))
            .child(
                div()
                    .text_size(px(96.))
                    .text_color(count_color)
                    .child(count_text),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(button(
                        "dec",
                        "−",
                        0xdc2626,
                        0xef4444,
                        cx.listener(|this, _, _, cx| {
                            this.count -= 1;
                            cx.notify();
                        }),
                    ))
                    .child(button(
                        "reset",
                        "Reset",
                        0x4b5563,
                        0x6b7280,
                        cx.listener(|this, _, _, cx| {
                            this.count = 0;
                            cx.notify();
                        }),
                    ))
                    .child(button(
                        "inc",
                        "+",
                        0x16a34a,
                        0x22c55e,
                        cx.listener(|this, _, _, cx| {
                            this.count += 1;
                            cx.notify();
                        }),
                    )),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x9ca3af))
                    .child("↑ / ↓ で増減、R でリセット、Ctrl+Q で終了"),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("up", Increment, None),
            KeyBinding::new("down", Decrement, None),
            KeyBinding::new("r", Reset, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(480.), px(420.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(|cx| {
                        let focus_handle = cx.focus_handle();
                        window.focus(&focus_handle, cx);
                        Counter {
                            count: 0,
                            focus_handle,
                        }
                    })
                },
            )
            .unwrap();
        let _ = window;
        cx.activate(true);
    });
}
