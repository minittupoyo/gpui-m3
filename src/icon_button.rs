//! Icon buttons（Expressive: 5 サイズ × 3 幅、丸/角、トグル、押下モーフ）

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    App, ClickEvent, ElementId, IntoElement, Pixels, RenderOnce, SharedString, SpringAnimation,
    Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, ButtonShape, ClickHandler, ComponentSize, Icon, IconName, Theme, button_metrics,
    resolve_round, ripple_overlay, state, state_layer, track_press, track_ripple, transparent,
    use_pressed, use_ripple,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonVariant {
    #[default]
    Standard,
    Filled,
    Tonal,
    Outlined,
}

impl IconButtonVariant {
    pub const ALL: [IconButtonVariant; 4] =
        [Self::Standard, Self::Filled, Self::Tonal, Self::Outlined];

    pub fn name(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::Filled => "Filled",
            Self::Tonal => "Tonal",
            Self::Outlined => "Outlined",
        }
    }
}

/// アイコンボタンの幅
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonWidth {
    Narrow,
    #[default]
    Default,
    Wide,
}

pub(crate) fn icon_button_size(size: ComponentSize, width: IconButtonWidth) -> (Pixels, Pixels) {
    use ComponentSize::*;
    use IconButtonWidth::*;
    let (h, icon) = match size {
        XSmall => (32., 20.),
        Small => (40., 24.),
        Medium => (56., 24.),
        Large => (96., 32.),
        XLarge => (136., 40.),
    };
    let w = match (size, width) {
        (_, Default) => h,
        (XSmall, Narrow) => 28.,
        (XSmall, Wide) => 40.,
        (Small, Narrow) => 32.,
        (Small, Wide) => 52.,
        (Medium, Narrow) => 48.,
        (Medium, Wide) => 72.,
        (Large, Narrow) => 64.,
        (Large, Wide) => 128.,
        (XLarge, Narrow) => 104.,
        (XLarge, Wide) => 184.,
    };
    (px(w), px(icon))
}

pub(crate) fn icon_button_colors(
    theme: &Theme,
    variant: IconButtonVariant,
    selected: Option<bool>,
    disabled: bool,
) -> (gpui::Hsla, gpui::Hsla, Option<gpui::Hsla>) {
    let c = &theme.colors;
    let (mut bg, mut fg, mut border) = match (variant, selected) {
        (IconButtonVariant::Standard, Some(true)) => (transparent(), c.primary, None),
        (IconButtonVariant::Standard, _) => (transparent(), c.on_surface_variant, None),
        (IconButtonVariant::Filled, Some(false)) => {
            (c.surface_container, c.on_surface_variant, None)
        }
        (IconButtonVariant::Filled, _) => (c.primary, c.on_primary, None),
        (IconButtonVariant::Tonal, Some(true)) => (c.secondary, c.on_secondary, None),
        (IconButtonVariant::Tonal, _) => (c.secondary_container, c.on_secondary_container, None),
        (IconButtonVariant::Outlined, Some(true)) => {
            (c.inverse_surface, c.inverse_on_surface, None)
        }
        (IconButtonVariant::Outlined, _) => {
            (transparent(), c.on_surface_variant, Some(c.outline_variant))
        }
    };
    if disabled {
        if bg.a > 0. {
            bg = theme.disabled_container();
        }
        fg = theme.disabled_content();
        border = border.map(|_| theme.disabled_container());
    }
    (bg, fg, border)
}

#[derive(IntoElement)]
pub struct IconButton {
    id: SharedString,
    icon: IconName,
    selected_icon: Option<IconName>,
    variant: IconButtonVariant,
    size: ComponentSize,
    width: IconButtonWidth,
    shape: ButtonShape,
    selected: Option<bool>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    pub fn new(id: impl Into<SharedString>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            selected_icon: None,
            variant: IconButtonVariant::Standard,
            size: ComponentSize::Small,
            width: IconButtonWidth::Default,
            shape: ButtonShape::Round,
            selected: None,
            disabled: false,
            on_click: None,
        }
    }
    pub fn variant(mut self, v: IconButtonVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn filled(self) -> Self {
        self.variant(IconButtonVariant::Filled)
    }
    pub fn tonal(self) -> Self {
        self.variant(IconButtonVariant::Tonal)
    }
    pub fn outlined(self) -> Self {
        self.variant(IconButtonVariant::Outlined)
    }
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }
    pub fn width(mut self, width: IconButtonWidth) -> Self {
        self.width = width;
        self
    }
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }
    /// 選択時に表示するアイコン（例: Favorite / FavoriteBorder）
    pub fn selected_icon(mut self, icon: IconName) -> Self {
        self.selected_icon = Some(icon);
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let m = button_metrics(self.size);
        let (w, icon_size) = icon_button_size(self.size, self.width);
        let (bg, fg, border) =
            icon_button_colors(&theme, self.variant, self.selected, self.disabled);

        let pressed_state = use_pressed(&self.id, window, cx);
        let pressed = *pressed_state.read(cx) && !self.disabled;
        let round = resolve_round(self.shape, self.selected);
        let radius = if pressed {
            m.pressed_radius
        } else if round {
            m.height.min(w) / 2.
        } else {
            m.square_radius
        };

        let icon = match (self.selected, self.selected_icon) {
            (Some(true), Some(sel)) => sel,
            _ => self.icon,
        };
        let filled = self.selected == Some(true) && self.selected_icon.is_none();

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .w(w)
            .h(m.height)
            .bg(bg);

        if let Some(b) = border {
            el = match m.outline {
                1 => el.border_1(),
                2 => el.border_2(),
                _ => el.border_3(),
            }
            .border_color(b);
        }

        if !self.disabled {
            el = el
                .cursor_pointer()
                .hover(move |s| s.bg(state_layer(bg, fg, state::HOVER)));
            let ripple = use_ripple(&self.id, window, cx);
            el = el.child(ripple_overlay(&ripple, fg, gpui::Corners::all(radius)));
            el = track_ripple(el, &ripple);
            el = track_press(el, &pressed_state);
            if let Some(handler) = self.on_click.clone() {
                el = el.on_click(move |e, w, cx| handler(e, w, cx));
            }
        }
        el = el.child(Icon::new(icon).filled(filled).size(icon_size).color(fg));

        el.with_spring(
            ElementId::Name(format!("{}::radius", self.id).into()),
            SpringAnimation::new(theme.motion.fast_spatial).to(radius),
            |el, r| el.rounded(r),
        )
    }
}
