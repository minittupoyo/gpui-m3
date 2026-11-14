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
mod carousel;
mod chip;
mod drawer;
mod fab;
mod fab_menu;
mod feedback;
mod fonts;
mod icon_button;
mod icons;
mod loading;
mod menu;
mod navigation;
mod overlays;
mod picker;
mod progress;
mod ripple;
mod search;
mod selection;
mod sheet;
mod slider;
mod slider_ext;
mod split_button;
mod tabs;
mod text_field;
mod theme;
mod toolbar;

pub use app_bar::*;
pub use button::*;
pub use button_group::*;
pub use card::*;
pub use carousel::*;
pub use chip::*;
pub use drawer::*;
pub use fab::*;
pub use fab_menu::*;
pub use feedback::*;
pub use fonts::ROBOTO_FLEX_FAMILY;
pub use icon_button::*;
pub use icons::*;
pub use loading::*;
pub use menu::*;
pub use navigation::*;
pub use overlays::*;
pub use picker::*;
pub use progress::*;
#[allow(unused_imports)]
pub(crate) use ripple::*;
pub use search::*;
pub use selection::*;
pub use sheet::*;
pub use slider::*;
pub use slider_ext::*;
pub use split_button::*;
pub use tabs::*;
pub use text_field::*;
pub use theme::*;
pub use toolbar::*;

use std::rc::Rc;

use gpui::{
    App, ClickEvent, Div, Entity, InteractiveElement, MouseButton, SharedString, Stateful,
    StatefulInteractiveElement, Window,
    actions,
};

actions!(m3_focus, [FocusNext, FocusPrev]);

/// テーマを登録し、キー操作（Tab / Shift+Tab でフォーカス移動）を有効にする
pub fn init(cx: &mut App, brightness: Brightness) {
    fonts::register_fonts(cx);
    cx.set_global(Theme::new(brightness));
    bind_text_field_keys(cx);
    cx.bind_keys([
        gpui::KeyBinding::new("tab", FocusNext, None),
        gpui::KeyBinding::new("shift-tab", FocusPrev, None),
    ]);
    // アクションはウィンドウの更新中に呼ばれるので、更新が終わってからフォーカスを動かす
    cx.on_action(|_: &FocusNext, cx| move_focus(cx, true));
    cx.on_action(|_: &FocusPrev, cx| move_focus(cx, false));
}

fn move_focus(cx: &mut App, forward: bool) {
    let Some(window) = cx.active_window() else {
        return;
    };
    cx.defer(move |cx| {
        window
            .update(cx, |_, window, cx| {
                if forward {
                    window.focus_next(cx)
                } else {
                    window.focus_prev(cx)
                }
            })
            .ok();
    });
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

// ---------------------------------------------------------------------------
// ホバーで影の高さをなめらかに変える
// ---------------------------------------------------------------------------

struct ElevAnim {
    value: f32,
    last: Option<std::time::Instant>,
}

/// ホバー状態（`track_hover` で更新）
pub(crate) fn use_hover(id: &SharedString, window: &mut Window, cx: &mut App) -> Entity<bool> {
    window.use_keyed_state(SharedString::from(format!("{id}::hover")), cx, |_, _| false)
}

/// ホバー検知を付ける
pub(crate) fn track_hover(el: Stateful<Div>, state: &Entity<bool>) -> Stateful<Div> {
    let state = state.clone();
    el.on_hover(move |hovered, _, cx| {
        state.update(cx, |s, cx| {
            if *s != *hovered {
                *s = *hovered;
                cx.notify();
            }
        })
    })
}

/// 現在のエレベーションレベルを返す。ホバー中は `hover`、それ以外は `base` へ指数的に近づく。
/// アニメーション中は次のフレームを要求する。
pub(crate) fn animate_elevation(
    id: &SharedString,
    hovered: bool,
    base: f32,
    hover: f32,
    window: &mut Window,
    cx: &mut App,
) -> f32 {
    let anim = window.use_keyed_state(
        SharedString::from(format!("{id}::elev")),
        cx,
        move |_, _| ElevAnim { value: base, last: None },
    );
    let target = if hovered { hover } else { base };
    let reduce = cx.reduce_motion();
    let mut level = target;
    let mut animating = false;
    anim.update(cx, |a, _| {
        if reduce {
            a.value = target;
            a.last = None;
        } else {
            let now = std::time::Instant::now();
            let dt = a.last.map(|t| (now - t).as_secs_f32()).unwrap_or(0.).min(0.05);
            a.last = Some(now);
            // 時定数 70ms
            a.value += (target - a.value) * (1. - (-dt / 0.07).exp());
            if (target - a.value).abs() < 0.01 {
                a.value = target;
                a.last = None;
            } else {
                animating = true;
            }
        }
        level = a.value;
    });
    if animating {
        window.request_animation_frame();
    }
    level
}
