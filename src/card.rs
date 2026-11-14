//! Cards: Elevated / Filled / Outlined

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, ImageSource, IntoElement, ObjectFit,
    ParentElement, Pixels, RenderOnce, SharedString, StyleRefinement, Styled, Window, div, img,
    prelude::*,
};

use super::{
    ActiveTheme, ClickHandler, animate_elevation, elevation_f, ripple_overlay, shape, state,
    state_layer, track_hover, track_ripple, use_hover, use_ripple,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    #[default]
    Elevated,
    Filled,
    Outlined,
}

/// カードの角丸
const CARD_RADIUS: Pixels = shape::MEDIUM;

/// カード上端のメディア（画像やグラデーション）。
///
/// GPUI の `overflow_hidden` は長方形でしか切り取れず、角丸には従わない。
/// そのため子要素をカードの形に切り取る代わりに、メディア自身の上端の角を
/// カードの角丸に合わせて丸める。`Card::media` に渡すと、枠線の太さまで含めて
/// 角丸が自動で合う。
///
/// ```ignore
/// Card::new("c").media(CardMedia::image("photo.jpg").h(px(160.)))
/// Card::new("c").media(CardMedia::new().h(px(120.)).bg(linear_gradient(..)))
/// ```
///
/// 注意: `CardMedia` の中に入れた任意の子要素は、これまでどおり切り取られない
/// （画像は `CardMedia::image` を使うこと）。
#[derive(IntoElement)]
pub struct CardMedia {
    base: Div,
    image: Option<ImageSource>,
    radius: Pixels,
}

impl CardMedia {
    pub fn new() -> Self {
        Self {
            base: div().w_full().flex_none(),
            image: None,
            radius: CARD_RADIUS,
        }
    }
    /// 画像（枠いっぱいに Cover で敷き、上端の角を丸める）
    pub fn image(source: impl Into<ImageSource>) -> Self {
        let mut m = Self::new();
        m.image = Some(source.into());
        m
    }
    fn radius(mut self, r: Pixels) -> Self {
        self.radius = r;
        self
    }
}

impl Default for CardMedia {
    fn default() -> Self {
        Self::new()
    }
}

impl Styled for CardMedia {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for CardMedia {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let r = self.radius;
        let mut base = self.base.rounded_tl(r).rounded_tr(r);
        if let Some(src) = self.image {
            base = base.child(
                img(src)
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .rounded_tl(r)
                    .rounded_tr(r),
            );
        }
        base
    }
}

/// カード。`Styled` / `ParentElement` を実装しているので div と同じように中身を組める
#[derive(IntoElement)]
pub struct Card {
    id: SharedString,
    variant: CardVariant,
    media: Option<CardMedia>,
    base: Div,
    on_click: Option<ClickHandler>,
}

impl Card {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            variant: CardVariant::Elevated,
            media: None,
            base: div(),
            on_click: None,
        }
    }
    /// 上端のメディア。カードの角丸に合わせて上端の角が丸くなる
    pub fn media(mut self, media: CardMedia) -> Self {
        self.media = Some(media);
        self
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let ripple = self
            .on_click
            .is_some()
            .then(|| use_ripple(&self.id, window, cx));
        let (bg, elev, border) = match self.variant {
            CardVariant::Elevated => (c.surface_container_low, 1, None),
            CardVariant::Filled => (c.surface_container_highest, 0, None),
            CardVariant::Outlined => (c.surface, 0, Some(c.outline_variant)),
        };
        let fg = c.on_surface;
        let shadow = c.shadow;

        let hover_state = use_hover(&self.id, window, cx);
        let hover_elev = if self.on_click.is_none() {
            elev as f32
        } else if self.variant == CardVariant::Elevated {
            2.
        } else {
            1.
        };
        let level = animate_elevation(
            &self.id,
            *hover_state.read(cx),
            elev as f32,
            hover_elev,
            window,
            cx,
        );

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .flex()
            .flex_col()
            .rounded(CARD_RADIUS)
            .overflow_hidden()
            .bg(bg)
            .text_color(fg)
            .shadow(elevation_f(level, shadow));
        el = track_hover(el, &hover_state);
        if let Some(b) = border {
            el = el.border_1().border_color(b);
        }
        if let Some(h) = self.on_click {
            el = el
                .cursor_pointer()
                .hover(move |s| s.bg(state_layer(bg, fg, state::HOVER)))
                .on_click(move |e, w, cx| h(e, w, cx));
        }
        if let Some(r) = ripple.as_ref() {
            el = track_ripple(el, r);
        }
        // 枠線の内側に収まるので、角丸も枠線の分だけ小さくする
        if let Some(media) = self.media {
            let inner = if border.is_some() {
                CARD_RADIUS - gpui::px(1.)
            } else {
                CARD_RADIUS
            };
            el = el.child(media.radius(inner));
        }
        el = el.child(self.base);
        // 波紋はメディアの上にも広がるよう、最後の子にする
        if let Some(r) = ripple.as_ref() {
            el = el.child(ripple_overlay(r, fg, gpui::Corners::all(CARD_RADIUS)));
        }
        el
    }
}
