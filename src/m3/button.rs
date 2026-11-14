//! Common buttons（Expressive: 5 サイズ・丸/角の 2 シェイプ・押下時シェイプモーフ・トグル）

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    App, ClickEvent, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, SpringAnimation, Styled, Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, ButtonShape, ClickHandler, ComponentSize, Icon, IconName, StyledExt, Theme,
    TypeScale, state, state_layer, track_press, transparent, use_pressed,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Filled,
    Tonal,
    Elevated,
    Outlined,
    Text,
}

impl ButtonVariant {
    pub const ALL: [ButtonVariant; 5] = [
        Self::Filled,
        Self::Tonal,
        Self::Elevated,
        Self::Outlined,
        Self::Text,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Filled => "Filled",
            Self::Tonal => "Tonal",
            Self::Elevated => "Elevated",
            Self::Outlined => "Outlined",
            Self::Text => "Text",
        }
    }
}

/// サイズごとの寸法
pub(crate) struct ButtonMetrics {
    pub height: Pixels,
    pub padding: Pixels,
    pub icon: Pixels,
    pub gap: Pixels,
    pub type_scale: TypeScale,
    pub square_radius: Pixels,
    pub pressed_radius: Pixels,
    pub outline: u8,
}

pub(crate) fn button_metrics(size: ComponentSize) -> ButtonMetrics {
    use ComponentSize::*;
    match size {
        XSmall => ButtonMetrics {
            height: px(32.),
            padding: px(12.),
            icon: px(20.),
            gap: px(4.),
            type_scale: TypeScale::LabelLarge,
            square_radius: px(12.),
            pressed_radius: px(8.),
            outline: 1,
        },
        Small => ButtonMetrics {
            height: px(40.),
            padding: px(16.),
            icon: px(20.),
            gap: px(8.),
            type_scale: TypeScale::LabelLarge,
            square_radius: px(12.),
            pressed_radius: px(8.),
            outline: 1,
        },
        Medium => ButtonMetrics {
            height: px(56.),
            padding: px(24.),
            icon: px(24.),
            gap: px(8.),
            type_scale: TypeScale::TitleMedium,
            square_radius: px(16.),
            pressed_radius: px(12.),
            outline: 1,
        },
        Large => ButtonMetrics {
            height: px(96.),
            padding: px(48.),
            icon: px(32.),
            gap: px(12.),
            type_scale: TypeScale::HeadlineSmall,
            square_radius: px(28.),
            pressed_radius: px(16.),
            outline: 2,
        },
        XLarge => ButtonMetrics {
            height: px(136.),
            padding: px(64.),
            icon: px(40.),
            gap: px(16.),
            type_scale: TypeScale::HeadlineLarge,
            square_radius: px(28.),
            pressed_radius: px(16.),
            outline: 3,
        },
    }
}

/// コンテナ色・コンテンツ色・枠線色・エレベーション
pub(crate) struct ButtonColors {
    pub container: Hsla,
    pub content: Hsla,
    pub border: Option<Hsla>,
    pub elevation: u8,
}

pub(crate) fn button_colors(
    theme: &Theme,
    variant: ButtonVariant,
    selected: Option<bool>,
    disabled: bool,
) -> ButtonColors {
    let c = &theme.colors;
    let mut colors = match (variant, selected) {
        (ButtonVariant::Filled, None) | (ButtonVariant::Filled, Some(true)) => ButtonColors {
            container: c.primary,
            content: c.on_primary,
            border: None,
            elevation: 0,
        },
        (ButtonVariant::Filled, Some(false)) => ButtonColors {
            container: c.surface_container,
            content: c.on_surface_variant,
            border: None,
            elevation: 0,
        },
        (ButtonVariant::Tonal, Some(true)) => ButtonColors {
            container: c.secondary,
            content: c.on_secondary,
            border: None,
            elevation: 0,
        },
        (ButtonVariant::Tonal, _) => ButtonColors {
            container: c.secondary_container,
            content: c.on_secondary_container,
            border: None,
            elevation: 0,
        },
        (ButtonVariant::Elevated, Some(true)) => ButtonColors {
            container: c.primary,
            content: c.on_primary,
            border: None,
            elevation: 1,
        },
        (ButtonVariant::Elevated, _) => ButtonColors {
            container: c.surface_container_low,
            content: c.primary,
            border: None,
            elevation: 1,
        },
        (ButtonVariant::Outlined, Some(true)) => ButtonColors {
            container: c.inverse_surface,
            content: c.inverse_on_surface,
            border: None,
            elevation: 0,
        },
        (ButtonVariant::Outlined, _) => ButtonColors {
            container: transparent(),
            content: c.on_surface_variant,
            border: Some(c.outline_variant),
            elevation: 0,
        },
        (ButtonVariant::Text, _) => ButtonColors {
            container: transparent(),
            content: c.primary,
            border: None,
            elevation: 0,
        },
    };
    if disabled {
        if colors.container.a > 0. {
            colors.container = theme.disabled_container();
        }
        colors.content = theme.disabled_content();
        colors.border = colors.border.map(|_| theme.disabled_container());
        colors.elevation = 0;
    }
    colors
}

/// M3 Expressive ボタン
#[derive(IntoElement)]
pub struct Button {
    id: SharedString,
    label: SharedString,
    icon: Option<IconName>,
    variant: ButtonVariant,
    size: ComponentSize,
    shape: ButtonShape,
    selected: Option<bool>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            variant: ButtonVariant::Filled,
            size: ComponentSize::Small,
            shape: ButtonShape::Round,
            selected: None,
            disabled: false,
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn filled(self) -> Self {
        self.variant(ButtonVariant::Filled)
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
    pub fn text(self) -> Self {
        self.variant(ButtonVariant::Text)
    }
    pub fn size(mut self, size: ComponentSize) -> Self {
        self.size = size;
        self
    }
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    /// トグルボタンにする（Some(選択状態)）
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
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

/// トグル選択時は形が反転する（丸→角、角→丸）
pub(crate) fn resolve_round(shape: ButtonShape, selected: Option<bool>) -> bool {
    let round = shape == ButtonShape::Round;
    if selected == Some(true) { !round } else { round }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let m = button_metrics(self.size);
        let colors = button_colors(&theme, self.variant, self.selected, self.disabled);
        let pressed_state = use_pressed(&self.id, window, cx);
        let pressed = *pressed_state.read(cx) && !self.disabled;

        let round = resolve_round(self.shape, self.selected);
        let radius = if pressed {
            m.pressed_radius
        } else if round {
            m.height / 2.
        } else {
            m.square_radius
        };

        let ButtonColors {
            container,
            content,
            border,
            elevation,
        } = colors;
        let shadow_color = theme.colors.shadow;

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .h(m.height)
            .px(m.padding)
            .gap(m.gap)
            .bg(container)
            .text_color(content)
            .m3_type(m.type_scale)
            .m3_elevation(elevation, shadow_color);

        if let Some(b) = border {
            el = match m.outline {
                1 => el.border_1(),
                2 => el.border_2(),
                _ => el.border_3(),
            }
            .border_color(b);
        }

        if !self.disabled {
            let hover_elev = if elevation > 0 { elevation + 1 } else { 0 };
            el = el
                .cursor_pointer()
                .hover(move |s| {
                    let s = s.bg(state_layer(container, content, state::HOVER));
                    if hover_elev > 0 {
                        s.m3_elevation(hover_elev, shadow_color)
                    } else {
                        s
                    }
                })
                .active(move |s| s.bg(state_layer(container, content, state::PRESSED)));
            el = track_press(el, &pressed_state);
            if let Some(handler) = self.on_click.clone() {
                el = el.on_click(move |e, w, cx| handler(e, w, cx));
            }
        }

        if let Some(icon) = self.icon {
            el = el.child(Icon::new(icon).size(m.icon).color(content));
        }
        el = el.child(self.label);

        el.with_spring(
            ElementId::Name(format!("{}::radius", self.id).into()),
            SpringAnimation::new(theme.motion.fast_spatial).to(radius),
            |el, r| el.rounded(r),
        )
    }
}
