//! 最小の使用例: ボタン・スイッチ・テキスト欄を並べる。
//!
//! ```text
//! cargo run --example hello
//! ```

use gpui::{
    App, Bounds, Context, Entity, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_m3::{self as m3, ActiveTheme, Button, StyledExt, Switch, TextField, TypeScale};
use gpui_platform::application;

struct Hello {
    clicks: u32,
    notifications: bool,
    name: Entity<TextField>,
}

impl Hello {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            clicks: 0,
            notifications: true,
            name: cx.new(|cx| TextField::new(window, cx).label("名前").clearable(true)),
        }
    }
}

impl Render for Hello {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let this = cx.entity().downgrade();
        let name = self.name.read(cx).text();

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(20.))
            .p(px(32.))
            .bg(c.surface)
            .text_color(c.on_surface)
            .font_family(theme.font_family)
            .child(div().m3_type(TypeScale::HeadlineMedium).child("gpui-m3"))
            .child(self.name.clone())
            .child(div().child(if name.is_empty() {
                "こんにちは".to_string()
            } else {
                format!("こんにちは、{name}さん")
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(Switch::new("notifications", self.notifications).on_change(
                        move |on, _, cx| {
                            this.update(cx, |h, cx| {
                                h.notifications = on;
                                cx.notify();
                            })
                            .ok();
                        },
                    ))
                    .child("通知を受け取る"),
            )
            .child(
                div().flex().child(
                    Button::new("count", format!("クリック {}", self.clicks)).on_click(
                        cx.listener(|h, _, _, cx| {
                            h.clicks += 1;
                            cx.notify();
                        }),
                    ),
                ),
            )
    }
}

fn main() {
    application().with_assets(m3::Assets).run(|cx: &mut App| {
        m3::init(cx, m3::Brightness::Dark);
        let bounds = Bounds::centered(None, size(px(480.), px(420.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Hello::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
