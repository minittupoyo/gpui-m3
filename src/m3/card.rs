//! Cards: Elevated / Filled / Outlined

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, IntoElement, ParentElement, RenderOnce,
    SharedString, StyleRefinement, Styled, Window, div, prelude::*,
};

use super::{ActiveTheme, ClickHandler, StyledExt, shape, state, state_layer};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    #[default]
    Elevated,
    Filled,
    Outlined,
}

/// カード。`Styled` / `ParentElement` を実装しているので div と同じように中身を組める
#[derive(IntoElement)]
pub struct Card {
    id: SharedString,
    variant: CardVariant,
    base: Div,
    on_click: Option<ClickHandler>,
}

impl Card {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            variant: CardVariant::Elevated,
            base: div(),
            on_click: None,
        }
    }
    pub fn variant(mut self, v: CardVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn elevated(self) -> Self {
        self.variant(CardVariant::Elevated)
    }
    pub fn filled(self) -> Self {
        self.variant(CardVariant::Filled)
    }
    pub fn outlined(self) -> Self {
        self.variant(CardVariant::Outlined)
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl Styled for Card {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let (bg, elev, border) = match self.variant {
            CardVariant::Elevated => (c.surface_container_low, 1, None),
            CardVariant::Filled => (c.surface_container_highest, 0, None),
            CardVariant::Outlined => (c.surface, 0, Some(c.outline_variant)),
        };
        let fg = c.on_surface;
        let shadow = c.shadow;

        let mut el = div()
            .id(ElementId::Name(self.id))
            .flex()
            .flex_col()
            .rounded(shape::MEDIUM)
            .overflow_hidden()
            .bg(bg)
            .text_color(fg)
            .m3_elevation(elev, shadow);
        if let Some(b) = border {
            el = el.border_1().border_color(b);
        }
        if let Some(h) = self.on_click {
            let hover_elev = if self.variant == CardVariant::Elevated { 2 } else { 1 };
            el = el
                .cursor_pointer()
                .hover(move |s| {
                    s.bg(state_layer(bg, fg, state::HOVER))
                        .m3_elevation(hover_elev, shadow)
                })
                .active(move |s| s.bg(state_layer(bg, fg, state::PRESSED)))
                .on_click(move |e, w, cx| h(e, w, cx));
        }
        el.child(self.base)
    }
}
