//! FAB / Extended FAB（Expressive: FAB・Medium・Large の 3 サイズ）

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    App, ClickEvent, ElementId, Hsla, IntoElement, Pixels, RenderOnce, SharedString,
    SpringAnimation, Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, ClickHandler, Icon, IconName, StyledExt, Theme, TypeScale, state, state_layer,
    ripple_overlay, track_press, track_ripple, use_pressed, use_ripple,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabSize {
    #[default]
    Regular,
    Medium,
    Large,
}

impl FabSize {
    pub const ALL: [FabSize; 3] = [Self::Regular, Self::Medium, Self::Large];

    /// (高さ, 角丸, アイコン, 横パディング, タイプスケール)
    fn spec(self) -> (Pixels, Pixels, Pixels, Pixels, TypeScale) {
        match self {
            Self::Regular => (px(56.), px(16.), px(24.), px(16.), TypeScale::TitleMedium),
            Self::Medium => (px(80.), px(20.), px(28.), px(26.), TypeScale::TitleLarge),
            Self::Large => (px(96.), px(28.), px(36.), px(28.), TypeScale::HeadlineSmall),
        }
    }
}

/// FAB の配色
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FabColor {
    #[default]
    PrimaryContainer,
    SecondaryContainer,
    TertiaryContainer,
    Primary,
    Secondary,
    Tertiary,
}

impl FabColor {
    pub const ALL: [FabColor; 6] = [
        Self::PrimaryContainer,
        Self::SecondaryContainer,
        Self::TertiaryContainer,
        Self::Primary,
        Self::Secondary,
        Self::Tertiary,
    ];

    fn resolve(self, t: &Theme) -> (Hsla, Hsla) {
        let c = &t.colors;
        match self {
            Self::PrimaryContainer => (c.primary_container, c.on_primary_container),
            Self::SecondaryContainer => (c.secondary_container, c.on_secondary_container),
            Self::TertiaryContainer => (c.tertiary_container, c.on_tertiary_container),
            Self::Primary => (c.primary, c.on_primary),
            Self::Secondary => (c.secondary, c.on_secondary),
            Self::Tertiary => (c.tertiary, c.on_tertiary),
        }
    }
}

/// FAB。`label` を付けると Extended FAB になる
#[derive(IntoElement)]
pub struct Fab {
    id: SharedString,
    icon: IconName,
    label: Option<SharedString>,
    size: FabSize,
    color: FabColor,
    lowered: bool,
    on_click: Option<ClickHandler>,
}

impl Fab {
    pub fn new(id: impl Into<SharedString>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            label: None,
            size: FabSize::Regular,
            color: FabColor::PrimaryContainer,
            lowered: false,
            on_click: None,
        }
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }
    pub fn color(mut self, color: FabColor) -> Self {
        self.color = color;
        self
    }
    /// 影を弱くする（Lowered）
    pub fn lowered(mut self, lowered: bool) -> Self {
        self.lowered = lowered;
        self
    }
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Fab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let (h, radius, icon, pad, scale) = self.size.spec();
        let (bg, fg) = self.color.resolve(&theme);
        let shadow = theme.colors.shadow;
        let (elev, hover_elev) = if self.lowered { (1, 2) } else { (3, 4) };

        let pressed_state = use_pressed(&self.id, window, cx);
        let pressed = *pressed_state.read(cx);
        // Expressive: 押下でわずかに角を詰める
        let r = if pressed { radius * 0.7 } else { radius };

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(h)
            .min_w(h)
            .bg(bg)
            .text_color(fg)
            .m3_type(scale)
            .m3_elevation(elev, shadow)
            .cursor_pointer()
            .hover(move |s| {
                s.bg(state_layer(bg, fg, state::HOVER))
                    .m3_elevation(hover_elev, shadow)
            })
            ;
        let ripple = use_ripple(&self.id, window, cx);
        el = el.child(ripple_overlay(&ripple, fg, gpui::Corners::all(r)));
        el = el.child(Icon::new(self.icon).size(icon).color(fg));

        if let Some(label) = self.label {
            el = el.px(pad).gap(pad * 0.5).child(label);
        }
        el = track_ripple(el, &ripple);
        el = track_press(el, &pressed_state);
        if let Some(handler) = self.on_click {
            el = el.on_click(move |e, w, cx| handler(e, w, cx));
        }

        el.with_spring(
            ElementId::Name(format!("{}::radius", self.id).into()),
            SpringAnimation::new(theme.motion.fast_spatial).to(r),
            |el, r| el.rounded(r),
        )
    }
}
