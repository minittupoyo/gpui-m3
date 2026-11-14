//! Material Design 3 のデザイントークン（色・タイポグラフィ・シェイプ・モーション・エレベーション）

use gpui::{
    App, BoxShadow, FontWeight, Global, Hsla, Pixels, SpringConfig, Styled, hsla, point, px, rgb,
};

fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

/// M3 のカラーロール
#[derive(Clone, Debug)]
pub struct ColorScheme {
    pub primary: Hsla,
    pub on_primary: Hsla,
    pub primary_container: Hsla,
    pub on_primary_container: Hsla,
    pub secondary: Hsla,
    pub on_secondary: Hsla,
    pub secondary_container: Hsla,
    pub on_secondary_container: Hsla,
    pub tertiary: Hsla,
    pub on_tertiary: Hsla,
    pub tertiary_container: Hsla,
    pub on_tertiary_container: Hsla,
    pub error: Hsla,
    pub on_error: Hsla,
    pub error_container: Hsla,
    pub on_error_container: Hsla,
    pub surface: Hsla,
    pub on_surface: Hsla,
    pub on_surface_variant: Hsla,
    pub surface_container_lowest: Hsla,
    pub surface_container_low: Hsla,
    pub surface_container: Hsla,
    pub surface_container_high: Hsla,
    pub surface_container_highest: Hsla,
    pub outline: Hsla,
    pub outline_variant: Hsla,
    pub inverse_surface: Hsla,
    pub inverse_on_surface: Hsla,
    pub inverse_primary: Hsla,
    pub scrim: Hsla,
    pub shadow: Hsla,
}

impl ColorScheme {
    /// ベースライン（#6750A4）ライトスキーム
    pub fn light() -> Self {
        Self {
            primary: c(0x6750A4),
            on_primary: c(0xFFFFFF),
            primary_container: c(0xEADDFF),
            on_primary_container: c(0x4F378B),
            secondary: c(0x625B71),
            on_secondary: c(0xFFFFFF),
            secondary_container: c(0xE8DEF8),
            on_secondary_container: c(0x4A4458),
            tertiary: c(0x7D5260),
            on_tertiary: c(0xFFFFFF),
            tertiary_container: c(0xFFD8E4),
            on_tertiary_container: c(0x633B48),
            error: c(0xB3261E),
            on_error: c(0xFFFFFF),
            error_container: c(0xF9DEDC),
            on_error_container: c(0x8C1D18),
            surface: c(0xFEF7FF),
            on_surface: c(0x1D1B20),
            on_surface_variant: c(0x49454F),
            surface_container_lowest: c(0xFFFFFF),
            surface_container_low: c(0xF7F2FA),
            surface_container: c(0xF3EDF7),
            surface_container_high: c(0xECE6F0),
            surface_container_highest: c(0xE6E0E9),
            outline: c(0x79747E),
            outline_variant: c(0xCAC4D0),
            inverse_surface: c(0x322F35),
            inverse_on_surface: c(0xF5EFF7),
            inverse_primary: c(0xD0BCFF),
            scrim: c(0x000000),
            shadow: c(0x000000),
        }
    }

    /// ベースライン（#6750A4）ダークスキーム
    pub fn dark() -> Self {
        Self {
            primary: c(0xD0BCFF),
            on_primary: c(0x381E72),
            primary_container: c(0x4F378B),
            on_primary_container: c(0xEADDFF),
            secondary: c(0xCCC2DC),
            on_secondary: c(0x332D41),
            secondary_container: c(0x4A4458),
            on_secondary_container: c(0xE8DEF8),
            tertiary: c(0xEFB8C8),
            on_tertiary: c(0x492532),
            tertiary_container: c(0x633B48),
            on_tertiary_container: c(0xFFD8E4),
            error: c(0xF2B8B5),
            on_error: c(0x601410),
            error_container: c(0x8C1D18),
            on_error_container: c(0xF9DEDC),
            surface: c(0x141218),
            on_surface: c(0xE6E0E9),
            on_surface_variant: c(0xCAC4D0),
            surface_container_lowest: c(0x0F0D13),
            surface_container_low: c(0x1D1B20),
            surface_container: c(0x211F26),
            surface_container_high: c(0x2B2930),
            surface_container_highest: c(0x36343B),
            outline: c(0x938F99),
            outline_variant: c(0x49454F),
            inverse_surface: c(0xE6E0E9),
            inverse_on_surface: c(0x322F35),
            inverse_primary: c(0x6750A4),
            scrim: c(0x000000),
            shadow: c(0x000000),
        }
    }
}

// ---------------------------------------------------------------------------
// State layers
// ---------------------------------------------------------------------------

/// ステートレイヤーの不透明度
pub mod state {
    pub const HOVER: f32 = 0.08;
    pub const FOCUS: f32 = 0.10;
    pub const PRESSED: f32 = 0.10;
    pub const DRAGGED: f32 = 0.16;
    pub const DISABLED_CONTAINER: f32 = 0.12;
    pub const DISABLED_CONTENT: f32 = 0.38;
}

/// `base` の上に `content` 色のステートレイヤーを重ねた色。
/// `base` が透明なら content 色そのものを半透明にして返す。
pub fn state_layer(base: Hsla, content: Hsla, opacity: f32) -> Hsla {
    let layer = content.opacity(opacity);
    if base.a <= 0.001 {
        layer
    } else {
        base.blend(layer)
    }
}

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

/// M3 タイプスケール
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeScale {
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
    BodyLarge,
    BodyMedium,
    BodySmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
}

impl TypeScale {
    pub const ALL: [TypeScale; 15] = [
        Self::DisplayLarge,
        Self::DisplayMedium,
        Self::DisplaySmall,
        Self::HeadlineLarge,
        Self::HeadlineMedium,
        Self::HeadlineSmall,
        Self::TitleLarge,
        Self::TitleMedium,
        Self::TitleSmall,
        Self::BodyLarge,
        Self::BodyMedium,
        Self::BodySmall,
        Self::LabelLarge,
        Self::LabelMedium,
        Self::LabelSmall,
    ];

    /// (サイズ, 行高, ウェイト)
    pub fn spec(self) -> (f32, f32, FontWeight) {
        use TypeScale::*;
        match self {
            DisplayLarge => (57., 64., FontWeight::NORMAL),
            DisplayMedium => (45., 52., FontWeight::NORMAL),
            DisplaySmall => (36., 44., FontWeight::NORMAL),
            HeadlineLarge => (32., 40., FontWeight::NORMAL),
            HeadlineMedium => (28., 36., FontWeight::NORMAL),
            HeadlineSmall => (24., 32., FontWeight::NORMAL),
            TitleLarge => (22., 28., FontWeight::NORMAL),
            TitleMedium => (16., 24., FontWeight::MEDIUM),
            TitleSmall => (14., 20., FontWeight::MEDIUM),
            BodyLarge => (16., 24., FontWeight::NORMAL),
            BodyMedium => (14., 20., FontWeight::NORMAL),
            BodySmall => (12., 16., FontWeight::NORMAL),
            LabelLarge => (14., 20., FontWeight::MEDIUM),
            LabelMedium => (12., 16., FontWeight::MEDIUM),
            LabelSmall => (11., 16., FontWeight::MEDIUM),
        }
    }

    pub fn name(self) -> &'static str {
        use TypeScale::*;
        match self {
            DisplayLarge => "Display Large",
            DisplayMedium => "Display Medium",
            DisplaySmall => "Display Small",
            HeadlineLarge => "Headline Large",
            HeadlineMedium => "Headline Medium",
            HeadlineSmall => "Headline Small",
            TitleLarge => "Title Large",
            TitleMedium => "Title Medium",
            TitleSmall => "Title Small",
            BodyLarge => "Body Large",
            BodyMedium => "Body Medium",
            BodySmall => "Body Small",
            LabelLarge => "Label Large",
            LabelMedium => "Label Medium",
            LabelSmall => "Label Small",
        }
    }
}

/// `.m3_type(TypeScale::LabelLarge)` のように書くための拡張
pub trait StyledExt: Styled + Sized {
    fn m3_type(self, scale: TypeScale) -> Self {
        let (size, line, weight) = scale.spec();
        self.text_size(px(size))
            .line_height(px(line))
            .font_weight(weight)
    }

    /// Expressive の強調（Emphasized）タイポグラフィ。ウェイトを一段上げる
    fn m3_type_emphasized(self, scale: TypeScale) -> Self {
        let (size, line, weight) = scale.spec();
        let weight = if weight.0 >= FontWeight::MEDIUM.0 {
            FontWeight::BOLD
        } else {
            FontWeight::MEDIUM
        };
        self.text_size(px(size))
            .line_height(px(line))
            .font_weight(weight)
    }

    /// エレベーション（0〜5）の影
    fn m3_elevation(self, level: u8, shadow_color: Hsla) -> Self {
        self.shadow(elevation(level, shadow_color))
    }
}

impl<T: Styled> StyledExt for T {}

// ---------------------------------------------------------------------------
// Shape (Expressive の拡張シェイプスケール)
// ---------------------------------------------------------------------------

pub mod shape {
    use gpui::{Pixels, px};
    pub const NONE: Pixels = px(0.);
    pub const EXTRA_SMALL: Pixels = px(4.);
    pub const SMALL: Pixels = px(8.);
    pub const MEDIUM: Pixels = px(12.);
    pub const LARGE: Pixels = px(16.);
    pub const LARGE_INCREASED: Pixels = px(20.);
    pub const EXTRA_LARGE: Pixels = px(28.);
    pub const EXTRA_LARGE_INCREASED: Pixels = px(32.);
    pub const EXTRA_EXTRA_LARGE: Pixels = px(48.);
    pub const FULL: Pixels = px(9999.);
}

// ---------------------------------------------------------------------------
// Motion (Expressive スプリング)
// ---------------------------------------------------------------------------

/// 剛性と減衰比（ζ）から SpringConfig を作る（質量 1、c = ζ·2√k）
pub fn spring_ratio(stiffness: f32, damping_ratio: f32) -> SpringConfig {
    SpringConfig::new(stiffness, damping_ratio * 2. * stiffness.sqrt(), 1.)
}

/// モーションスキーム
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionScheme {
    /// 弾むような Expressive（推奨）
    Expressive,
    /// 落ち着いた Standard
    Standard,
}

/// モーショントークン
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub fast_spatial: SpringConfig,
    pub default_spatial: SpringConfig,
    pub slow_spatial: SpringConfig,
    pub fast_effects: SpringConfig,
    pub default_effects: SpringConfig,
    pub slow_effects: SpringConfig,
}

impl Motion {
    pub fn new(scheme: MotionScheme) -> Self {
        match scheme {
            MotionScheme::Expressive => Self {
                fast_spatial: spring_ratio(800., 0.6),
                default_spatial: spring_ratio(380., 0.8),
                slow_spatial: spring_ratio(200., 0.8),
                fast_effects: spring_ratio(3800., 1.),
                default_effects: spring_ratio(1600., 1.),
                slow_effects: spring_ratio(800., 1.),
            },
            MotionScheme::Standard => Self {
                fast_spatial: spring_ratio(1400., 0.9),
                default_spatial: spring_ratio(700., 0.9),
                slow_spatial: spring_ratio(300., 0.9),
                fast_effects: spring_ratio(3800., 1.),
                default_effects: spring_ratio(1600., 1.),
                slow_effects: spring_ratio(800., 1.),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Elevation
// ---------------------------------------------------------------------------

fn bs(color: Hsla, y: f32, blur: f32, spread: f32, alpha: f32) -> BoxShadow {
    BoxShadow {
        color: color.opacity(alpha),
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    }
}

/// M3 のエレベーションレベル 0〜5 に対応する影
pub fn elevation(level: u8, color: Hsla) -> Vec<BoxShadow> {
    match level {
        0 => vec![],
        1 => vec![bs(color, 1., 2., 0., 0.3), bs(color, 1., 3., 1., 0.15)],
        2 => vec![bs(color, 1., 2., 0., 0.3), bs(color, 2., 6., 2., 0.15)],
        3 => vec![bs(color, 1., 3., 0., 0.3), bs(color, 4., 8., 3., 0.15)],
        4 => vec![bs(color, 2., 3., 0., 0.3), bs(color, 6., 10., 4., 0.15)],
        _ => vec![bs(color, 4., 4., 0., 0.3), bs(color, 8., 12., 6., 0.15)],
    }
}

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brightness {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub brightness: Brightness,
    pub colors: ColorScheme,
    pub motion: Motion,
    pub font_family: &'static str,
}

impl Theme {
    pub fn new(brightness: Brightness) -> Self {
        Self {
            brightness,
            colors: match brightness {
                Brightness::Light => ColorScheme::light(),
                Brightness::Dark => ColorScheme::dark(),
            },
            motion: Motion::new(MotionScheme::Expressive),
            font_family: "Segoe UI",
        }
    }

    pub fn is_dark(&self) -> bool {
        self.brightness == Brightness::Dark
    }

    pub fn disabled_container(&self) -> Hsla {
        self.colors.on_surface.opacity(state::DISABLED_CONTAINER)
    }

    pub fn disabled_content(&self) -> Hsla {
        self.colors.on_surface.opacity(state::DISABLED_CONTENT)
    }
}

impl Global for Theme {}

/// `cx.m3()` でテーマにアクセスする
pub trait ActiveTheme {
    fn m3(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn m3(&self) -> &Theme {
        self.global::<Theme>()
    }
}

/// 透明色
pub fn transparent() -> Hsla {
    hsla(0., 0., 0., 0.)
}

/// Pixels を f32 に
pub fn f(p: Pixels) -> f32 {
    f32::from(p)
}
