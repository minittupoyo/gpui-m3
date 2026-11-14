//! Material Design 3 / M3 Expressive コンポーネント for GPUI
//!
//! ```ignore
//! application().with_assets(m3::Assets).run(|cx| {
//!     m3::init(cx, m3::Brightness::Dark);
//!     // ...
//! });
//! ```

mod app_bar;
mod button;
mod button_group;
mod card;
mod chip;
mod fab;
mod fab_menu;
mod feedback;
mod icon_button;
mod icons;
mod loading;
mod menu;
mod navigation;
mod progress;
mod selection;
mod sheet;
mod slider;
mod split_button;
mod tabs;
mod text_field;
mod theme;
mod toolbar;

pub use app_bar::*;
pub use button::*;
pub use button_group::*;
pub use card::*;
pub use chip::*;
pub use fab::*;
pub use fab_menu::*;
pub use feedback::*;
pub use icon_button::*;
pub use icons::*;
pub use loading::*;
pub use menu::*;
pub use navigation::*;
pub use progress::*;
pub use selection::*;
pub use sheet::*;
pub use slider::*;
pub use split_button::*;
pub use tabs::*;
pub use text_field::*;
pub use theme::*;
pub use toolbar::*;

use std::rc::Rc;

use gpui::{
    App, ClickEvent, Div, Entity, InteractiveElement, MouseButton, SharedString, Stateful, Window,
};

/// テーマを登録する
pub fn init(cx: &mut App, brightness: Brightness) {
    cx.set_global(Theme::new(brightness));
    bind_text_field_keys(cx);
}

/// テーマを切り替える（全ウィンドウを再描画）
pub fn set_brightness(cx: &mut App, brightness: Brightness) {
    let dynamic = cx.m3().dynamic;
    rebuild_theme(cx, brightness, dynamic);
}

/// Dynamic color を設定する（None でベースラインの固定スキームに戻す）
pub fn set_dynamic_color(cx: &mut App, config: Option<DynamicColorConfig>) {
    let brightness = cx.m3().brightness;
    rebuild_theme(cx, brightness, config);
}

fn rebuild_theme(cx: &mut App, brightness: Brightness, dynamic: Option<DynamicColorConfig>) {
    let old = cx.m3().clone();
    let mut theme = Theme::with_dynamic(brightness, dynamic);
    theme.motion = old.motion;
    theme.font_family = old.font_family;
    cx.set_global(theme);
    cx.refresh_windows();
}

pub type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type ChangeHandler<T> = Rc<dyn Fn(T, &mut Window, &mut App) + 'static>;

/// Expressive の 5 段階サイズ
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ComponentSize {
    XSmall,
    #[default]
    Small,
    Medium,
    Large,
    XLarge,
}

impl ComponentSize {
    pub const ALL: [ComponentSize; 5] = [
        Self::XSmall,
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::XLarge,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::XSmall => "XS",
            Self::Small => "S",
            Self::Medium => "M",
            Self::Large => "L",
            Self::XLarge => "XL",
        }
    }
}

/// ボタンの形（Expressive）
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    #[default]
    Round,
    Square,
}

// ---------------------------------------------------------------------------
// 押下状態（シェイプモーフ用）
// ---------------------------------------------------------------------------

/// 要素ごとの「押されているか」状態を取得する
pub(crate) fn use_pressed(id: &SharedString, window: &mut Window, cx: &mut App) -> Entity<bool> {
    window.use_keyed_state(
        SharedString::from(format!("{id}::pressed")),
        cx,
        |_, _| false,
    )
}

fn set_pressed(state: &Entity<bool>, value: bool, cx: &mut App) {
    state.update(cx, |p, cx| {
        if *p != value {
            *p = value;
            cx.notify();
        }
    });
}

/// 押下/解放で状態を更新するハンドラを付ける
pub(crate) fn track_press(el: Stateful<Div>, state: &Entity<bool>) -> Stateful<Div> {
    let down = state.clone();
    let up = state.clone();
    let out = state.clone();
    el.on_mouse_down(MouseButton::Left, move |_, _, cx| set_pressed(&down, true, cx))
        .on_mouse_up(MouseButton::Left, move |_, _, cx| set_pressed(&up, false, cx))
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            set_pressed(&out, false, cx)
        })
}
