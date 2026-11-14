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
// Dynamic color
// ---------------------------------------------------------------------------

/// シードカラーからスキームを作るときの方式（material-color-utilities の Variant）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SchemeVariant {
    /// 既定。落ち着いたトーンでシード色相をそのまま使う
    #[default]
    TonalSpot,
    /// 彩度の高い鮮やかな配色
    Vibrant,
    /// 色相をずらした遊びのある配色（シード色相は主役にならない）
    Expressive,
    /// シード色に忠実（ブランドカラー向け）
    Fidelity,
    /// 画像などのコンテンツ色に合わせる
    Content,
    /// ほぼ無彩色
    Neutral,
    /// 完全なグレースケール
    Monochrome,
    /// 色相を大きく回した虹色
    Rainbow,
    /// 色相を回したフルーツサラダ
    FruitSalad,
}

impl SchemeVariant {
    pub const ALL: [SchemeVariant; 9] = [
        Self::TonalSpot,
        Self::Vibrant,
        Self::Expressive,
        Self::Fidelity,
        Self::Content,
        Self::Neutral,
        Self::Monochrome,
        Self::Rainbow,
        Self::FruitSalad,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::TonalSpot => "Tonal spot",
            Self::Vibrant => "Vibrant",
            Self::Expressive => "Expressive",
            Self::Fidelity => "Fidelity",
            Self::Content => "Content",
            Self::Neutral => "Neutral",
            Self::Monochrome => "Monochrome",
            Self::Rainbow => "Rainbow",
            Self::FruitSalad => "Fruit salad",
        }
    }

    fn to_mcu(self) -> material_colors::dynamic_color::Variant {
        use material_colors::dynamic_color::Variant as V;
        match self {
            Self::TonalSpot => V::TonalSpot,
            Self::Vibrant => V::Vibrant,
            Self::Expressive => V::Expressive,
            Self::Fidelity => V::Fidelity,
            Self::Content => V::Content,
            Self::Neutral => V::Neutral,
            Self::Monochrome => V::Monochrome,
            Self::Rainbow => V::Rainbow,
            Self::FruitSalad => V::FruitSalad,
        }
    }
}

/// Dynamic color の設定
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DynamicColorConfig {
    /// シードカラー（0xRRGGBB）
    pub seed: u32,
    pub variant: SchemeVariant,
    /// コントラスト（-1.0〜1.0。0 = 標準、0.5 = 中、1.0 = 高）
    pub contrast: f64,
}

impl DynamicColorConfig {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            variant: SchemeVariant::TonalSpot,
            contrast: 0.,
        }
    }
    pub fn variant(mut self, v: SchemeVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn contrast(mut self, c: f64) -> Self {
        self.contrast = c;
        self
    }
}

/// 色相 (0〜360) から鮮やかなシードカラーを作る（HCT: chroma 48, tone 50）
pub fn seed_from_hue(hue: f32) -> u32 {
    use material_colors::{color::Rgb, hct::Hct};
    let rgb: Rgb = Hct::from(hue as f64, 48., 50.).into();
    ((rgb.red as u32) << 16) | ((rgb.green as u32) << 8) | rgb.blue as u32
}

impl ColorScheme {
    /// シードカラーから M3 のカラーロールを生成する（Dynamic color）。
    /// M3 Expressive に合わせて 2025 年版のカラースペックを使う。
    pub fn from_seed(config: DynamicColorConfig, dark: bool) -> Self {
        use material_colors::{
            color::Rgb,
            dynamic_color::{DynamicScheme, Platform, SpecVersion},
            hct::Hct,
        };
        let s = DynamicScheme::from_spec(
            Hct::new(Rgb::from_u32(config.seed)),
            config.variant.to_mcu(),
            dark,
            Some(config.contrast),
            Platform::Phone,
            SpecVersion::Spec2025,
        );
        let h = |rgb: Rgb| -> Hsla {
            c(((rgb.red as u32) << 16) | ((rgb.green as u32) << 8) | rgb.blue as u32)
        };
        Self {
            primary: h(s.primary()),
            on_primary: h(s.on_primary()),
            primary_container: h(s.primary_container()),
            on_primary_container: h(s.on_primary_container()),
            secondary: h(s.secondary()),
            on_secondary: h(s.on_secondary()),
            secondary_container: h(s.secondary_container()),
            on_secondary_container: h(s.on_secondary_container()),
            tertiary: h(s.tertiary()),
            on_tertiary: h(s.on_tertiary()),
            tertiary_container: h(s.tertiary_container()),
            on_tertiary_container: h(s.on_tertiary_container()),
            error: h(s.error()),
            on_error: h(s.on_error()),
            error_container: h(s.error_container()),
            on_error_container: h(s.on_error_container()),
            surface: h(s.surface()),
            on_surface: h(s.on_surface()),
            on_surface_variant: h(s.on_surface_variant()),
            surface_container_lowest: h(s.surface_container_lowest()),
            surface_container_low: h(s.surface_container_low()),
            surface_container: h(s.surface_container()),
            surface_container_high: h(s.surface_container_high()),
            surface_container_highest: h(s.surface_container_highest()),
            outline: h(s.outline()),
            outline_variant: h(s.outline_variant()),
            inverse_surface: h(s.inverse_surface()),
            inverse_on_surface: h(s.inverse_on_surface()),
            inverse_primary: h(s.inverse_primary()),
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

/// 小数のエレベーション（隣り合うレベルの影を線形補間する。ホバーのアニメーション用）
pub fn elevation_f(level: f32, color: Hsla) -> Vec<BoxShadow> {
    // 各レベルの (y, blur, spread, alpha) × 2 層
    const L: [[(f32, f32, f32, f32); 2]; 6] = [
        [(0., 0., 0., 0.); 2],
        [(1., 2., 0., 0.3), (1., 3., 1., 0.15)],
        [(1., 2., 0., 0.3), (2., 6., 2., 0.15)],
        [(1., 3., 0., 0.3), (4., 8., 3., 0.15)],
        [(2., 3., 0., 0.3), (6., 10., 4., 0.15)],
        [(4., 4., 0., 0.3), (8., 12., 6., 0.15)],
    ];
    let level = level.clamp(0., 5.);
    if level < 0.01 {
        return vec![];
    }
    let lo = level.floor() as usize;
    let hi = (lo + 1).min(5);
    let t = level - lo as f32;
    (0..2)
        .map(|i| {
            let (a, b) = (L[lo][i], L[hi][i]);
            let m = |x: f32, y: f32| x + (y - x) * t;
            bs(color, m(a.0, b.0), m(a.1, b.1), m(a.2, b.2), m(a.3, b.3))
        })
        .collect()
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
    /// None ならベースライン（#6750A4）の固定スキーム
    pub dynamic: Option<DynamicColorConfig>,
}

impl Theme {
    pub fn new(brightness: Brightness) -> Self {
        Self::with_dynamic(brightness, None)
    }

    /// Dynamic color（シードカラー）を指定してテーマを作る
    pub fn with_dynamic(brightness: Brightness, dynamic: Option<DynamicColorConfig>) -> Self {
        let dark = brightness == Brightness::Dark;
        let colors = match dynamic {
            Some(cfg) => ColorScheme::from_seed(cfg, dark),
            None if dark => ColorScheme::dark(),
            None => ColorScheme::light(),
        };
        Self {
            brightness,
            colors,
            motion: Motion::new(MotionScheme::Expressive),
            font_family: super::ROBOTO_FLEX_FAMILY,
            dynamic,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn shadow_color() -> Hsla {
        hsla(0., 0., 0., 1.)
    }

    #[test]
    fn fractional_elevation_matches_integer_levels() {
        for level in 1..=5u8 {
            let a = elevation(level, shadow_color());
            let b = elevation_f(level as f32, shadow_color());
            assert_eq!(a.len(), b.len(), "level {level}");
            for (x, y) in a.iter().zip(&b) {
                assert_eq!(x.blur_radius, y.blur_radius, "level {level}");
                assert_eq!(x.offset, y.offset, "level {level}");
                assert!((x.color.a - y.color.a).abs() < 1e-6, "level {level}");
            }
        }
    }

    #[test]
    fn fractional_elevation_interpolates() {
        assert!(elevation_f(0., shadow_color()).is_empty());
        let (lo, hi) = (elevation_f(1., shadow_color()), elevation_f(2., shadow_color()));
        let mid = elevation_f(1.5, shadow_color());
        let (l, m, h) = (
            f32::from(lo[1].blur_radius),
            f32::from(mid[1].blur_radius),
            f32::from(hi[1].blur_radius),
        );
        assert!(l < m && m < h, "{l} < {m} < {h}");
    }

    #[test]
    fn fractional_elevation_is_clamped() {
        assert!(elevation_f(-3., shadow_color()).is_empty());
        let top = elevation(5, shadow_color());
        let over = elevation_f(9., shadow_color());
        assert_eq!(top.len(), over.len());
        assert_eq!(top[1].blur_radius, over[1].blur_radius);
    }

    #[test]
    fn dark_and_light_schemes_differ() {
        let (d, l) = (Theme::new(Brightness::Dark), Theme::new(Brightness::Light));
        assert!(d.is_dark() && !l.is_dark());
        assert_ne!(d.colors.surface, l.colors.surface);
        assert_eq!(d.font_family, super::super::ROBOTO_FLEX_FAMILY);
    }

    #[test]
    fn dynamic_color_follows_the_seed() {
        let a = Theme::with_dynamic(Brightness::Light, Some(DynamicColorConfig::new(0x6750A4)));
        let b = Theme::with_dynamic(Brightness::Light, Some(DynamicColorConfig::new(0x006A6A)));
        assert_ne!(a.colors.primary, b.colors.primary);
        assert!(a.dynamic.is_some());
    }
}
