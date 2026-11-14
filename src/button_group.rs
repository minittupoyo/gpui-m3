//! Button groups（Expressive）
//!
//! - [`ConnectedButtonGroup`] — 隣接したボタンが内側の角を共有し、選択されると丸くなる（旧 Segmented button の後継）
//! - [`StandardButtonGroup`] — 押したボタンが横に膨らみ、隣が縮むインタラクション

use gpui::AnimationExt as _;
use gpui::{
    App, ElementId, IntoElement, MouseButton, Pixels, RenderOnce, SharedString, SpringAnimation,
    Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, ChangeHandler, ComponentSize, Icon, IconButtonVariant, IconName, StyledExt,
    button_metrics, icon_button_colors, icon_button_size, ripple_overlay, state, state_layer,
    track_press, track_ripple, use_pressed, use_ripple,
};

pub struct ButtonGroupItem {
    pub label: SharedString,
    pub icon: Option<IconName>,
}

impl ButtonGroupItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

fn inner_radius(size: ComponentSize) -> (Pixels, Pixels) {
    // (通常, 押下時)
    match size {
        ComponentSize::XSmall => (px(4.), px(2.)),
        ComponentSize::Small => (px(8.), px(4.)),
        ComponentSize::Medium => (px(8.), px(4.)),
        ComponentSize::Large => (px(16.), px(12.)),
        ComponentSize::XLarge => (px(20.), px(16.)),
    }
}

// ---------------------------------------------------------------------------
// Connected
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct ConnectedButtonGroup {
    id: SharedString,
    items: Vec<ButtonGroupItem>,
    selected: Vec<bool>,
    size: ComponentSize,
    on_toggle: Option<ChangeHandler<usize>>,
}

impl ConnectedButtonGroup {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            items: vec![],
            selected: vec![],
            size: ComponentSize::Small,
            on_toggle: None,
        }
    }
    pub fn item(mut self, item: ButtonGroupItem) -> Self {
        self.items.push(item);
        self
    }
    /// 各項目の選択状態
    pub fn selected(mut self, selected: Vec<bool>) -> Self {
        self.selected = selected;
        self
    }
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }
    pub fn on_toggle(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for ConnectedButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let m = button_metrics(self.size);
        let (inner, inner_pressed) = inner_radius(self.size);
        let outer = m.height / 2.;
        let n = self.items.len();
        let c = theme.colors.clone();

        let mut row = div().flex().flex_none().gap(px(2.));
        for (i, item) in self.items.into_iter().enumerate() {
            let id: SharedString = format!("{}-{i}", self.id).into();
            let selected = self.selected.get(i).copied().unwrap_or(false);
            let pressed_state = use_pressed(&id, window, cx);
            let pressed = *pressed_state.read(cx);

            let (bg, fg) = if selected {
                (c.secondary, c.on_secondary)
            } else {
                (c.secondary_container, c.on_secondary_container)
            };
            let inner_r = if pressed {
                inner_pressed
            } else if selected {
                outer
            } else {
                inner
            };
            let first = i == 0;
            let last = i + 1 == n;

            let mut el = div()
                .id(ElementId::Name(id.clone()))
                .flex()
                .items_center()
                .justify_center()
                .gap(m.gap)
                .h(m.height)
                .px(m.padding)
                .bg(bg)
                .text_color(fg)
                .m3_type(m.type_scale)
                .cursor_pointer()
                .hover(move |s| s.bg(state_layer(bg, fg, state::HOVER)));
            let ripple = use_ripple(&id, window, cx);
            let l = if first { outer } else { inner_r };
            let rr = if last { outer } else { inner_r };
            el = el.child(ripple_overlay(
                &ripple,
                fg,
                gpui::Corners {
                    top_left: l,
                    bottom_left: l,
                    top_right: rr,
                    bottom_right: rr,
                },
            ));
            el = track_ripple(el, &ripple);
            el = track_press(el, &pressed_state);
            if let Some(h) = self.on_toggle.clone() {
                el = el.on_click(move |_, w, cx| h(i, w, cx));
            }
            let icon = item.icon.or(selected.then_some(IconName::Check));
            if let Some(icon) = icon {
                el = el.child(Icon::new(icon).size(m.icon).color(fg));
            }
            el = el.child(item.label);

            row = row.child(el.with_spring(
                ElementId::Name(format!("{id}::inner").into()),
                SpringAnimation::new(theme.motion.fast_spatial).to(inner_r),
                move |el, r| {
                    let l = if first { outer } else { r };
                    let rr = if last { outer } else { r };
                    el.rounded_tl(l).rounded_bl(l).rounded_tr(rr).rounded_br(rr)
                },
            ));
        }
        row
    }
}

// ---------------------------------------------------------------------------
// Standard (幅が変わるインタラクション)
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct StandardButtonGroup {
    id: SharedString,
    icons: Vec<IconName>,
    selected: Vec<bool>,
    variant: IconButtonVariant,
    size: ComponentSize,
    on_toggle: Option<ChangeHandler<usize>>,
}

impl StandardButtonGroup {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icons: vec![],
            selected: vec![],
            variant: IconButtonVariant::Tonal,
            size: ComponentSize::Medium,
            on_toggle: None,
        }
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icons.push(icon);
        self
    }
    pub fn selected(mut self, selected: Vec<bool>) -> Self {
        self.selected = selected;
        self
    }
    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }
    pub fn on_toggle(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for StandardButtonGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let m = button_metrics(self.size);
        let (w, icon_size) = icon_button_size(self.size, super::IconButtonWidth::Default);
        let group_pressed = window.use_keyed_state(
            SharedString::from(format!("{}::group-pressed", self.id)),
            cx,
            |_, _| None::<usize>,
        );
        let pressed_idx = *group_pressed.read(cx);

        let mut row = div().flex().flex_none().items_center().gap(px(8.));
        for (i, icon) in self.icons.into_iter().enumerate() {
            let id: SharedString = format!("{}-{i}", self.id).into();
            let selected = self.selected.get(i).copied();
            let (bg, fg, border) = icon_button_colors(&theme, self.variant, selected, false);

            let width = match pressed_idx {
                Some(p) if p == i => w * 1.3,
                Some(p) if p.abs_diff(i) == 1 => w * 0.85,
                _ => w,
            };
            let pressed = pressed_idx == Some(i);
            let radius = if pressed {
                m.pressed_radius
            } else if selected == Some(true) {
                m.square_radius
            } else {
                m.height / 2.
            };

            let down = group_pressed.clone();
            let up = group_pressed.clone();
            let out = group_pressed.clone();
            let mut btn = div()
                .id(ElementId::Name(id.clone()))
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(bg)
                .cursor_pointer()
                .hover(move |s| s.bg(state_layer(bg, fg, state::HOVER)))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    down.update(cx, |p, cx| {
                        *p = Some(i);
                        cx.notify();
                    })
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    up.update(cx, |p, cx| {
                        *p = None;
                        cx.notify();
                    })
                })
                .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                    out.update(cx, |p, cx| {
                        *p = None;
                        cx.notify();
                    })
                });
            let ripple = use_ripple(&id, window, cx);
            btn = btn.child(ripple_overlay(&ripple, fg, gpui::Corners::all(radius)));
            btn = track_ripple(btn, &ripple);
            btn = btn.child(Icon::new(icon).size(icon_size).color(fg));
            if let Some(b) = border {
                btn = btn.border_1().border_color(b);
            }
            if let Some(h) = self.on_toggle.clone() {
                btn = btn.on_click(move |_, w, cx| h(i, w, cx));
            }
            let btn = btn.with_spring(
                ElementId::Name(format!("{id}::radius").into()),
                SpringAnimation::new(theme.motion.fast_spatial).to(radius),
                |el, r| el.rounded(r),
            );

            row = row.child(div().h(m.height).flex_none().child(btn).with_spring(
                ElementId::Name(format!("{id}::width").into()),
                SpringAnimation::new(theme.motion.fast_spatial).to(width),
                |el, w| el.w(w),
            ));
        }
        row
    }
}
