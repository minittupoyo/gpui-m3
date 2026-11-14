//! FAB menu（M3 Expressive）
//!
//! FAB を押すと丸い「閉じる」ボタンにモーフし、その上にピル型の項目が
//! 時間差のスプリングで現れる。親側で画面右下などに `absolute` 配置して使う。
//!
//! ```ignore
//! div().absolute().right(px(16.)).bottom(px(16.)).child(
//!     FabMenu::new("fab-menu", IconName::Add)
//!         .item(FabMenuItem::new(IconName::Mail, "メール").on_click(|_, _, _| {}))
//! )
//! ```

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    App, ClickEvent, ElementId, Hsla, IntoElement, MouseButton, RenderOnce, SharedString,
    SpringAnimation, Transformation, Window, div, prelude::*, px, radians, svg,
};

use super::{
    ActiveTheme, ClickHandler, Icon, IconName, StyledExt, Theme, TypeScale, dismiss_on_escape,
    ripple_overlay, spring_ratio, state, state_layer, track_ripple, use_popup_focus, use_ripple,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabMenuColor {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

impl FabMenuColor {
    pub const ALL: [FabMenuColor; 3] = [Self::Primary, Self::Secondary, Self::Tertiary];

    /// (閉じた FAB / 項目のコンテナ, その上の色, 閉じるボタン, その上の色)
    fn resolve(self, t: &Theme) -> (Hsla, Hsla, Hsla, Hsla) {
        let c = &t.colors;
        match self {
            Self::Primary => (
                c.primary_container,
                c.on_primary_container,
                c.primary,
                c.on_primary,
            ),
            Self::Secondary => (
                c.secondary_container,
                c.on_secondary_container,
                c.secondary,
                c.on_secondary,
            ),
            Self::Tertiary => (
                c.tertiary_container,
                c.on_tertiary_container,
                c.tertiary,
                c.on_tertiary,
            ),
        }
    }
}

pub struct FabMenuItem {
    icon: IconName,
    label: SharedString,
    on_click: Option<ClickHandler>,
}

impl FabMenuItem {
    pub fn new(icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            icon,
            label: label.into(),
            on_click: None,
        }
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

#[derive(IntoElement)]
pub struct FabMenu {
    id: SharedString,
    icon: IconName,
    items: Vec<FabMenuItem>,
    color: FabMenuColor,
}

impl FabMenu {
    pub fn new(id: impl Into<SharedString>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            items: vec![],
            color: FabMenuColor::Primary,
        }
    }
    /// 項目（M3 では 2〜6 個を推奨）
    pub fn item(mut self, item: FabMenuItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn color(mut self, color: FabMenuColor) -> Self {
        self.color = color;
        self
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    a.blend(b.opacity(t.clamp(0., 1.)))
}

impl RenderOnce for FabMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let (container, on_container, close_bg, on_close) = self.color.resolve(&theme);
        let shadow = theme.colors.shadow;
        let open_state = window.use_keyed_state(
            SharedString::from(format!("{}::open", self.id)),
            cx,
            |_, _| false,
        );
        let open = *open_state.read(cx);
        let id = self.id.clone();
        let n = self.items.len();

        let set_open = {
            let s = open_state.clone();
            move |v: bool, cx: &mut App| {
                s.update(cx, |o, cx| {
                    if *o != v {
                        *o = v;
                        cx.notify();
                    }
                })
            }
        };

        // ----- 項目（FAB に近いものから順に、少しずつ遅れて出る）-----
        let mut items = div().flex().flex_col().items_end().gap(px(4.));
        for (i, item) in self.items.into_iter().enumerate() {
            // FAB に近いほど（index が大きいほど）速いバネ → 下から順に出る時間差
            let from_fab = (n - 1 - i) as f32;
            let spring = spring_ratio((700. - from_fab * 90.).max(220.), 0.7);
            let fg = on_container;
            let bg = container;
            let close = set_open.clone();
            let handler = item.on_click.clone();
            let item_id: SharedString = format!("{id}-item-{i}").into();
            let ripple = use_ripple(&item_id, window, cx);
            let pill = div()
                .id(ElementId::Name(item_id.clone()))
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(56.))
                .pl(px(20.))
                .pr(px(24.))
                .rounded_full()
                .bg(bg)
                .text_color(fg)
                .m3_type(TypeScale::TitleMedium)
                .m3_elevation(1, shadow)
                .cursor_pointer()
                .hover(move |s| s.bg(state_layer(bg, fg, state::HOVER)))
                .on_click(move |e, w, cx| {
                    if let Some(h) = handler.as_ref() {
                        h(e, w, cx);
                    }
                    close(false, cx);
                })
                .child(ripple_overlay(&ripple, fg, gpui::Corners::all(px(28.))))
                .child(Icon::new(item.icon).size(px(24.)).color(fg))
                .child(item.label);
            let pill = track_ripple(pill, &ripple);

            let rise = 16. + from_fab * 6.;
            items = items.child(div().relative().child(pill).with_spring(
                ElementId::Name(format!("{id}-item-{i}::t").into()),
                SpringAnimation::new(spring).to(if open { 1. } else { 0. }),
                move |el, t| {
                    if !open && t < 0.02 {
                        return el.hidden();
                    }
                    let t = t.clamp(-0.2, 1.2);
                    el.opacity(t.clamp(0., 1.))
                        .top(px((1. - t) * rise))
                        .right(px((1. - t) * 8.))
                },
            ));
        }

        // ----- FAB ⇄ 閉じるボタン -----
        let toggle = set_open.clone();
        let base_icon = self.icon;
        let fab_ripple = use_ripple(&format!("{id}::fab").into(), window, cx);
        let fab_ripple_overlay = fab_ripple.clone();
        let fab = track_ripple(
            div()
                .id(ElementId::Name(format!("{id}::fab").into()))
                .relative()
                .size(px(56.))
                .flex_none()
                .cursor_pointer()
                .on_click(move |_, _, cx| toggle(!open, cx)),
            &fab_ripple,
        )
            .with_spring(
                ElementId::Name(format!("{id}::morph").into()),
                SpringAnimation::new(theme.motion.fast_spatial).to(if open { 1. } else { 0. }),
                move |el, t| {
                    let tc = t.clamp(0., 1.);
                    let bg = mix(container, close_bg, tc);
                    let fg = mix(on_container, on_close, tc);
                    let radius = lerp(16., 28., t.clamp(0., 1.1));
                    let hover_bg = state_layer(bg, fg, state::HOVER);
                    let icon = |name: IconName, opacity: f32, angle: f32| {
                        svg()
                            .absolute()
                            .top(px(16.))
                            .left(px(16.))
                            .size(px(24.))
                            .path(name.path())
                            .text_color(fg.opacity(opacity))
                            .with_transformation(Transformation::rotate(radians(angle)))
                    };
                    el.rounded(px(radius))
                        .bg(bg)
                        .hover(move |s| s.bg(hover_bg))
                        .m3_elevation(if open { 2 } else { 3 }, shadow)
                        .child(ripple_overlay(
                            &fab_ripple_overlay,
                            fg,
                            gpui::Corners::all(px(radius)),
                        ))
                        .child(icon(
                            base_icon,
                            1. - tc,
                            t * std::f32::consts::FRAC_PI_2,
                        ))
                        .child(icon(
                            IconName::Close,
                            tc,
                            (t - 1.) * std::f32::consts::FRAC_PI_2,
                        ))
                },
            );

        let close_out = set_open.clone();
        let close_esc = set_open.clone();
        let root = div()
            .id(ElementId::Name(id.clone()))
            .flex()
            .flex_col()
            .items_end()
            .gap(px(8.))
            .on_mouse_down_out(move |_, _, cx| close_out(false, cx))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(items)
            .child(fab);
        // 開いている間は Esc で閉じる
        if open {
            let focus = use_popup_focus(&id, window, cx);
            dismiss_on_escape(
                root,
                &focus,
                Some(Rc::new(move |_, cx| close_esc(false, cx))),
            )
        } else {
            root
        }
    }
}
