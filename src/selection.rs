//! 選択コントロール: Switch / Checkbox / Radio button

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    App, ElementId, IntoElement, RenderOnce, SharedString, SpringAnimation, Window, div,
    prelude::*, px,
};

use super::{
    ActiveTheme, ChangeHandler, DismissFn, Icon, IconName, StyledExt, TypeScale, ripple_overlay,
    state, track_press, track_ripple, transparent, use_pressed, use_ripple,
};

fn eid(s: String) -> ElementId {
    ElementId::Name(s.into())
}

// ---------------------------------------------------------------------------
// Switch
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Switch {
    id: SharedString,
    on: bool,
    icons: bool,
    disabled: bool,
    on_change: Option<ChangeHandler<bool>>,
}

impl Switch {
    pub fn new(id: impl Into<SharedString>, on: bool) -> Self {
        Self {
            id: id.into(),
            on,
            icons: false,
            disabled: false,
            on_change: None,
        }
    }
    /// ハンドル内にアイコン（✓ / ✕）を表示
    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let pressed_state = use_pressed(&self.id, window, cx);
        let pressed = *pressed_state.read(cx) && !self.disabled;
        let on = self.on;

        let (track, border, handle, icon_color) = match (on, self.disabled) {
            (true, false) => (c.primary, c.primary, c.on_primary, c.on_primary_container),
            (false, false) => (
                c.surface_container_highest,
                c.outline,
                c.outline,
                c.surface_container_highest,
            ),
            (true, true) => (
                c.on_surface.opacity(0.12),
                c.on_surface.opacity(0.0),
                c.surface,
                c.on_surface.opacity(0.38),
            ),
            (false, true) => (
                c.surface_container_highest.opacity(0.12),
                c.on_surface.opacity(0.12),
                c.on_surface.opacity(0.38),
                c.surface_container_highest.opacity(0.38),
            ),
        };
        let handle_hover = if on {
            c.primary_container
        } else {
            c.on_surface_variant
        };
        let layer = if on { c.primary } else { c.on_surface };

        let handle_size = if pressed {
            px(28.)
        } else if on || self.icons {
            px(24.)
        } else {
            px(16.)
        };
        // ボーダー 2px の内側座標（48×28）での中心
        let cx_target = if on { px(34.) } else { px(14.) };
        let group: SharedString = format!("{}::group", self.id).into();

        let icon = if self.icons {
            Some(if on { IconName::Check } else { IconName::Close })
        } else {
            None
        };

        let handle_inner = {
            let mut h = div()
                .absolute()
                .rounded_full()
                .bg(handle)
                .flex()
                .items_center()
                .justify_center()
                .overflow_hidden();
            if !self.disabled {
                h = h.group_hover(group.clone(), move |s| s.bg(handle_hover));
            }
            if let Some(icon) = icon {
                h = h.child(Icon::new(icon).size(px(16.)).color(icon_color));
            }
            h.with_spring(
                eid(format!("{}::handle-size", self.id)),
                SpringAnimation::new(theme.motion.fast_spatial).to(handle_size),
                |el, s| el.size(s).left(-s / 2.).top(-s / 2.),
            )
        };

        let layer_el = {
            let mut l = div()
                .absolute()
                .size(px(40.))
                .left(px(-20.))
                .top(px(-20.))
                .rounded_full()
                .bg(transparent());
            if !self.disabled {
                l = l.group_hover(group.clone(), move |s| s.bg(layer.opacity(state::HOVER)));
            }
            l
        };

        let anchor = div()
            .absolute()
            .top(px(14.))
            .size_0()
            .child(layer_el)
            .child(handle_inner)
            .with_spring(
                eid(format!("{}::handle-x", self.id)),
                SpringAnimation::new(theme.motion.fast_spatial).to(cx_target),
                |el, x| el.left(x),
            );

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .group(group)
            .relative()
            .flex_none()
            .w(px(52.))
            .h(px(32.))
            .rounded_full()
            .bg(track)
            .border_2()
            .border_color(border);

        if !self.disabled {
            let ripple = use_ripple(&self.id, window, cx);
            el = el.child(ripple_overlay(&ripple, layer, gpui::Corners::all(px(16.))));
            el = track_ripple(el, &ripple);
            el = el.cursor_pointer();
            el = track_press(el, &pressed_state);
            if let Some(h) = self.on_change {
                el = el.on_click(move |_, w, cx| h(!on, w, cx));
            }
        }
        el.child(anchor)
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Checkbox {
    id: SharedString,
    checked: bool,
    indeterminate: bool,
    error: bool,
    disabled: bool,
    label: Option<SharedString>,
    on_change: Option<ChangeHandler<bool>>,
}

impl Checkbox {
    pub fn new(id: impl Into<SharedString>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            indeterminate: false,
            error: false,
            disabled: false,
            label: None,
            on_change: None,
        }
    }
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }
    pub fn error(mut self, v: bool) -> Self {
        self.error = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let filled = self.checked || self.indeterminate;
        let accent = if self.error { c.error } else { c.primary };
        let on_accent = if self.error { c.on_error } else { c.on_primary };

        let (box_bg, box_border, mark) = match (filled, self.disabled) {
            (_, true) if filled => (
                theme.disabled_content(),
                theme.disabled_content().opacity(0.),
                c.surface,
            ),
            (_, true) => (transparent(), theme.disabled_content(), transparent()),
            (true, false) => (accent, accent, on_accent),
            (false, false) => (
                transparent(),
                if self.error {
                    c.error
                } else {
                    c.on_surface_variant
                },
                transparent(),
            ),
        };
        let layer = if filled { accent } else { c.on_surface };
        let group: SharedString = format!("{}::group", self.id).into();
        let icon = if self.indeterminate {
            IconName::Remove
        } else {
            IconName::Check
        };

        let mark_el = div()
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .child(Icon::new(icon).size(px(18.)).color(mark))
            .with_spring(
                eid(format!("{}::mark", self.id)),
                SpringAnimation::new(theme.motion.fast_spatial).to(if filled {
                    px(18.)
                } else {
                    px(0.)
                }),
                |el, s| el.size(s.max(px(0.))),
            );

        let boxed = div()
            .size(px(18.))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(2.))
            .border_2()
            .border_color(box_border)
            .bg(box_bg)
            .child(mark_el);

        let mut target = div()
            .size(px(40.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .child(boxed);
        if !self.disabled {
            target = target.group_hover(group.clone(), move |s| s.bg(layer.opacity(state::HOVER)));
        }

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .group(group)
            .flex()
            .items_center()
            .gap(px(4.));
        let ripple = (!self.disabled).then(|| use_ripple(&self.id, window, cx));
        if let Some(r) = ripple.as_ref() {
            el = el.child(ripple_overlay(r, layer, gpui::Corners::all(px(20.))));
            el = track_ripple(el, r);
        }
        el = el.child(target);
        if let Some(label) = self.label {
            el = el.child(
                div()
                    .pr_2()
                    .m3_type(TypeScale::BodyLarge)
                    .text_color(if self.disabled {
                        theme.disabled_content()
                    } else {
                        c.on_surface
                    })
                    .child(label),
            );
        }
        if !self.disabled {
            el = el.cursor_pointer();
            if let Some(h) = self.on_change {
                let checked = self.checked;
                el = el.on_click(move |_, w, cx| h(!checked, w, cx));
            }
        }
        el
    }
}

// ---------------------------------------------------------------------------
// Radio
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Radio {
    id: SharedString,
    selected: bool,
    disabled: bool,
    label: Option<SharedString>,
    on_select: Option<DismissFn>,
}

impl Radio {
    pub fn new(id: impl Into<SharedString>, selected: bool) -> Self {
        Self {
            id: id.into(),
            selected,
            disabled: false,
            label: None,
            on_select: None,
        }
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Radio {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let color = match (self.selected, self.disabled) {
            (_, true) => theme.disabled_content(),
            (true, false) => c.primary,
            (false, false) => c.on_surface_variant,
        };
        let layer = if self.selected {
            c.primary
        } else {
            c.on_surface
        };
        let group: SharedString = format!("{}::group", self.id).into();

        let dot = div().rounded_full().bg(color).with_spring(
            eid(format!("{}::dot", self.id)),
            SpringAnimation::new(theme.motion.fast_spatial).to(if self.selected {
                px(10.)
            } else {
                px(0.)
            }),
            |el, s| el.size(s.max(px(0.))),
        );

        let ring = div()
            .size(px(20.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .border_2()
            .border_color(color)
            .child(dot);

        let mut target = div()
            .size(px(40.))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .child(ring);
        if !self.disabled {
            target = target.group_hover(group.clone(), move |s| s.bg(layer.opacity(state::HOVER)));
        }

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .group(group)
            .flex()
            .items_center()
            .gap(px(4.));
        let ripple = (!self.disabled).then(|| use_ripple(&self.id, window, cx));
        if let Some(r) = ripple.as_ref() {
            el = el.child(ripple_overlay(r, layer, gpui::Corners::all(px(20.))));
            el = track_ripple(el, r);
        }
        el = el.child(target);
        if let Some(label) = self.label {
            el = el.child(
                div()
                    .pr_2()
                    .m3_type(TypeScale::BodyLarge)
                    .text_color(if self.disabled {
                        theme.disabled_content()
                    } else {
                        c.on_surface
                    })
                    .child(label),
            );
        }
        if !self.disabled {
            el = el.cursor_pointer();
            if let Some(h) = self.on_select {
                el = el.on_click(move |_, w, cx| h(w, cx));
            }
        }
        el
    }
}
