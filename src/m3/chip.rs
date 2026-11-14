//! Chips: Assist / Filter / Input / Suggestion

use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window, div, prelude::*,
    px,
};

use super::{
    ActiveTheme, ClickHandler, Icon, IconName, StyledExt, TypeScale, ripple_overlay, shape, state,
    state_layer, track_ripple, transparent, use_ripple,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChipKind {
    #[default]
    Assist,
    Filter,
    Input,
    Suggestion,
}

#[derive(IntoElement)]
pub struct Chip {
    id: SharedString,
    kind: ChipKind,
    label: SharedString,
    icon: Option<IconName>,
    selected: bool,
    elevated: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
    on_remove: Option<ClickHandler>,
}

impl Chip {
    fn new(id: impl Into<SharedString>, kind: ChipKind, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind,
            label: label.into(),
            icon: None,
            selected: false,
            elevated: false,
            disabled: false,
            on_click: None,
            on_remove: None,
        }
    }
    pub fn assist(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id, ChipKind::Assist, label)
    }
    pub fn filter(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id, ChipKind::Filter, label)
    }
    pub fn input(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id, ChipKind::Input, label)
    }
    pub fn suggestion(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(id, ChipKind::Suggestion, label)
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    pub fn elevated(mut self, v: bool) -> Self {
        self.elevated = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
    /// Input chip の ✕ ボタン
    pub fn on_remove(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let selected = self.selected && matches!(self.kind, ChipKind::Filter | ChipKind::Input);

        let (mut bg, mut fg, mut border) = if selected {
            (c.secondary_container, c.on_secondary_container, None)
        } else if self.elevated {
            (c.surface_container_low, c.on_surface_variant, None)
        } else {
            (transparent(), c.on_surface_variant, Some(c.outline_variant))
        };
        if self.kind == ChipKind::Assist && !selected {
            fg = c.on_surface;
        }
        let icon_color = match self.kind {
            ChipKind::Assist if !self.disabled => c.primary,
            _ => fg,
        };
        if self.disabled {
            fg = theme.disabled_content();
            if bg.a > 0. {
                bg = theme.disabled_container();
            }
            border = border.map(|_| theme.disabled_container());
        }

        let leading = match (self.kind, selected) {
            (ChipKind::Filter, true) => Some(IconName::Check),
            _ => self.icon,
        };
        let has_trailing = self.kind == ChipKind::Input;

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .h(px(32.))
            .pl(if leading.is_some() { px(8.) } else { px(16.) })
            .pr(if has_trailing { px(8.) } else { px(16.) })
            .rounded(shape::SMALL)
            .bg(bg)
            .text_color(fg)
            .m3_type(TypeScale::LabelLarge);
        if let Some(b) = border {
            el = el.border_1().border_color(b);
        }
        if self.elevated && !self.disabled {
            el = el.m3_elevation(1, c.shadow);
        }
        if !self.disabled {
            let ripple = use_ripple(&self.id, window, cx);
            el = el.child(ripple_overlay(&ripple, fg, gpui::Corners::all(shape::SMALL)));
            el = track_ripple(el, &ripple);
        }
        if let Some(icon) = leading {
            el = el.child(Icon::new(icon).size(px(18.)).color(
                if self.disabled { fg } else if selected { fg } else { icon_color },
            ));
        }
        el = el.child(self.label);
        if has_trailing {
            let mut x = div()
                .id(ElementId::Name(format!("{}::remove", self.id).into()))
                .size(px(18.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(IconName::Close).size(px(18.)).color(fg));
            if let Some(h) = self.on_remove {
                x = x
                    .cursor_pointer()
                    .hover(move |s| s.bg(fg.opacity(0.12)))
                    .on_click(move |e, w, cx| {
                        cx.stop_propagation();
                        h(e, w, cx)
                    });
            }
            el = el.child(x);
        }
        if !self.disabled {
            el = el
                .cursor_pointer()
                .hover(move |s| s.bg(state_layer(bg, fg, state::HOVER)));
            if let Some(h) = self.on_click {
                el = el.on_click(move |e, w, cx| h(e, w, cx));
            }
        }
        el
    }
}
