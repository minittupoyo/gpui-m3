//! Split button（M3 Expressive）
//!
//! メインアクション（左）とメニュー（右）を 2px の隙間でつないだボタン。
//! - 外側の角は丸、内側の角は小さく、ホバー・押下で内側の角が丸くなる
//! - メニューを開くと右側が正円にモーフし、シェブロンが 180° 回転する
//!
//! ```ignore
//! SplitButton::new("save", "保存", menu)
//!     .icon(IconName::Check)
//!     .on_click(|_, _, _| { /* メインアクション */ })
//! ```

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    Anchor, App, ClickEvent, Div, ElementId, Entity, IntoElement, Pixels, RenderOnce, SharedString,
    SpringAnimation, Stateful, Transformation, Window, anchored, deferred, div, prelude::*, px,
    radians, relative, svg,
};

use super::{
    ActiveTheme, ButtonColors, ButtonVariant, ClickHandler, ComponentSize, Icon, IconName, Menu,
    RippleHandle, StyledExt, button_colors, button_metrics, dismisser, popup_state, ripple_overlay,
    state, state_layer, track_press, track_ripple, use_pressed, use_ripple,
};

/// 内側の角（通常, ホバー・押下時）とトレーリング側の幅
fn split_metrics(size: ComponentSize) -> (Pixels, Pixels, Pixels) {
    match size {
        ComponentSize::XSmall => (px(4.), px(8.), px(36.)),
        ComponentSize::Small => (px(4.), px(8.), px(44.)),
        ComponentSize::Medium => (px(4.), px(12.), px(60.)),
        ComponentSize::Large => (px(8.), px(20.), px(96.)),
        ComponentSize::XLarge => (px(12.), px(20.), px(128.)),
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[derive(IntoElement)]
pub struct SplitButton {
    id: SharedString,
    label: SharedString,
    icon: Option<IconName>,
    variant: ButtonVariant,
    size: ComponentSize,
    menu: Menu,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl SplitButton {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>, menu: Menu) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            variant: ButtonVariant::Filled,
            size: ComponentSize::Small,
            menu,
            disabled: false,
            on_click: None,
        }
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Filled / Tonal / Elevated / Outlined（Text は非対応）
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn tonal(self) -> Self {
        self.variant(ButtonVariant::Tonal)
    }
    pub fn elevated(self) -> Self {
        self.variant(ButtonVariant::Elevated)
    }
    pub fn outlined(self) -> Self {
        self.variant(ButtonVariant::Outlined)
    }
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// メインアクション
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

/// ホバー状態を keyed state に保持する
fn use_hovered(id: &SharedString, window: &mut Window, cx: &mut App) -> Entity<bool> {
    window.use_keyed_state(SharedString::from(format!("{id}::hovered")), cx, |_, _| {
        false
    })
}

fn track_hover(el: Stateful<Div>, state: &Entity<bool>) -> Stateful<Div> {
    let s = state.clone();
    el.on_hover(move |h, _, cx| {
        let h = *h;
        s.update(cx, |v, cx| {
            if *v != h {
                *v = h;
                cx.notify();
            }
        })
    })
}

impl RenderOnce for SplitButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let m = button_metrics(self.size);
        let (inner, inner_active, trailing_w) = split_metrics(self.size);
        let variant = if self.variant == ButtonVariant::Text {
            ButtonVariant::Filled
        } else {
            self.variant
        };
        let ButtonColors {
            container,
            content,
            border,
            elevation,
        } = button_colors(&theme, variant, None, self.disabled);
        let shadow = theme.colors.shadow;
        let disabled = self.disabled;
        let h = m.height;
        let outer = h / 2.;
        let spring = theme.motion.fast_spatial;
        let id = self.id.clone();

        let lead_id: SharedString = format!("{id}::lead").into();
        let trail_id: SharedString = format!("{id}::trail").into();
        let lead_pressed = use_pressed(&lead_id, window, cx);
        let trail_pressed = use_pressed(&trail_id, window, cx);
        let lead_hover = use_hovered(&lead_id, window, cx);
        let trail_hover = use_hovered(&trail_id, window, cx);
        let popup = popup_state(&id, window, cx);
        let open = popup.read(cx).open;

        let active = |p: &Entity<bool>, hv: &Entity<bool>, cx: &App| {
            !disabled && (*p.read(cx) || *hv.read(cx))
        };
        let lead_inner = if active(&lead_pressed, &lead_hover, cx) {
            inner_active
        } else {
            inner
        };
        let trail_inner = if open {
            outer
        } else if active(&trail_pressed, &trail_hover, cx) {
            inner_active
        } else {
            inner
        };

        let lead_ripple = use_ripple(&lead_id, window, cx);
        let trail_ripple = use_ripple(&trail_id, window, cx);
        let apply_common = move |el: Stateful<Div>,
                                 ripple: RippleHandle,
                                 radii: gpui::Corners<Pixels>|
              -> Stateful<Div> {
            let mut el = el
                .h(h)
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .bg(container)
                .text_color(content)
                .m3_elevation(elevation, shadow);
            if let Some(b) = border {
                el = match m.outline {
                    1 => el.border_1(),
                    2 => el.border_2(),
                    _ => el.border_3(),
                }
                .border_color(b);
            }
            if !disabled {
                el = el
                    .cursor_pointer()
                    .hover(move |s| s.bg(state_layer(container, content, state::HOVER)));
                el = el.child(ripple_overlay(&ripple, content, radii));
                el = track_ripple(el, &ripple);
            }
            el
        };

        // ----- 左: メインアクション -----
        let mut lead = apply_common(
            div().id(ElementId::Name(lead_id.clone())),
            lead_ripple,
            gpui::Corners {
                top_left: outer,
                bottom_left: outer,
                top_right: lead_inner,
                bottom_right: lead_inner,
            },
        )
        .gap(m.gap)
        .pl(m.padding)
        .pr(m.padding * 0.75)
        .m3_type(m.type_scale);
        if let Some(icon) = self.icon {
            lead = lead.child(Icon::new(icon).size(m.icon).color(content));
        }
        lead = lead.child(self.label);
        if !disabled {
            lead = track_hover(track_press(lead, &lead_pressed), &lead_hover);
            if let Some(handler) = self.on_click.clone() {
                lead = lead.on_click(move |e, w, cx| handler(e, w, cx));
            }
        }
        let lead = lead.with_spring(
            ElementId::Name(format!("{id}::lead-r").into()),
            SpringAnimation::new(spring).to(lead_inner),
            move |el, r| {
                el.rounded_tl(outer)
                    .rounded_bl(outer)
                    .rounded_tr(r.max(px(0.)))
                    .rounded_br(r.max(px(0.)))
            },
        );

        // ----- 右: メニュートリガー（開くと正円に） -----
        let icon_size = m.icon;
        let trail_id2 = trail_id.clone();
        let toggle = popup.clone();
        let trail_pressed2 = trail_pressed.clone();
        let trail_hover2 = trail_hover.clone();
        let id2 = id.clone();
        let trailing = div().flex_none().with_spring(
            ElementId::Name(format!("{id}::open").into()),
            SpringAnimation::new(spring).to(if open { 1. } else { 0. }),
            move |wrapper, t| {
                let width = lerp(f32::from(trailing_w), f32::from(h), t.clamp(0., 1.2));
                let chevron = svg()
                    .size(icon_size)
                    .flex_none()
                    .path(IconName::KeyboardArrowDown.path())
                    .text_color(content)
                    .with_transformation(Transformation::rotate(radians(t * std::f32::consts::PI)));
                let tr = trail_inner.min(outer);
                let mut btn = apply_common(
                    div().id(ElementId::Name(trail_id2.clone())),
                    trail_ripple.clone(),
                    gpui::Corners {
                        top_left: tr,
                        bottom_left: tr,
                        top_right: outer,
                        bottom_right: outer,
                    },
                )
                .w(px(width))
                .child(chevron);
                if !disabled {
                    btn = track_hover(track_press(btn, &trail_pressed2), &trail_hover2);
                    let toggle = toggle.clone();
                    btn = btn.on_click(move |_, _, cx| {
                        toggle.update(cx, |s, cx| {
                            if !s.open && !s.recently_dismissed() {
                                s.open = true;
                                cx.notify();
                            }
                        })
                    });
                }
                let btn = btn.with_spring(
                    ElementId::Name(format!("{id2}::trail-r").into()),
                    SpringAnimation::new(spring).to(trail_inner),
                    move |el, r| {
                        let r = r.max(px(0.)).min(outer);
                        el.rounded_tl(r)
                            .rounded_bl(r)
                            .rounded_tr(outer)
                            .rounded_br(outer)
                    },
                );
                wrapper.child(btn)
            },
        );

        let mut root = div()
            .id(ElementId::Name(id.clone()))
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(2.))
            .child(lead)
            .child(trailing);

        if open {
            let menu = self.menu.on_dismiss(dismisser(&popup));
            root = root.child(
                div()
                    .absolute()
                    .top(relative(1.))
                    .mt(px(4.))
                    .right_0()
                    .child(
                        deferred(
                            anchored()
                                .anchor(Anchor::TopRight)
                                .snap_to_window_with_margin(px(8.))
                                .child(menu),
                        )
                        .priority(1),
                    ),
            );
        }
        root
    }
}
