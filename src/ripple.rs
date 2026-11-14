//! Ripple（押した位置から広がる波紋）
//!
//! GPUI の `overflow_hidden` は長方形でしか切り取れず角丸に従わないため、
//! 「波紋の円」と「要素の角丸矩形」の共通部分（どちらも凸形）を
//! Sutherland–Hodgman 法で計算し、パスとして塗る。
//!
//! 使い方（コンポーネント内部）:
//! ```ignore
//! let ripple = use_ripple(&id, window, cx);
//! el = el.child(ripple_overlay(&ripple, content_color, Corners::all(radius)));  // 最初の子にする
//! el = track_ripple(el, &ripple);
//! ```

use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, BorderStyle, Bounds, Corners, Div, Edges, Entity, FocusHandle, Hsla,
    IntoElement, MouseButton, PathBuilder, Pixels, Point, SharedString, Stateful, Window, canvas,
    fill, prelude::*, px, quad,
};

/// 波紋が最大径まで広がる時間
const EXPAND: Duration = Duration::from_millis(450);
/// 離したあとのフェードアウト
const FADE: Duration = Duration::from_millis(260);
/// 立ち上がりのフェードイン
const FADE_IN: Duration = Duration::from_millis(90);
/// 波紋の最大不透明度（M3 の pressed state layer は 0.10）
const PEAK: f32 = 0.12;

struct Ripple {
    /// 押した位置（ウィンドウ座標）
    origin: Point<Pixels>,
    start: Instant,
    released: Option<Instant>,
}

pub(crate) struct RippleState {
    ripples: Vec<Ripple>,
    /// Tab キーで移動できるフォーカス（Enter / Space で on_click が呼ばれる）
    focus: FocusHandle,
}

/// 波紋とフォーカスをまとめたハンドル
#[derive(Clone)]
pub(crate) struct RippleHandle {
    state: Entity<RippleState>,
    focus: FocusHandle,
}

impl RippleHandle {
    pub(crate) fn focus(&self) -> &FocusHandle {
        &self.focus
    }
}

/// ポップアップ（メニュー・ダイアログ・シートなど）が開いている間だけ存在するフォーカス先。
/// 初めて描画された（＝開いた）瞬間にフォーカスを移すので、Esc や矢印キーが届く。
pub(crate) fn use_popup_focus(
    key: &SharedString,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    let state = window.use_keyed_state(
        SharedString::from(format!("{key}::popup-focus")),
        cx,
        |window, cx| {
            let handle = cx.focus_handle();
            window.focus(&handle, cx);
            handle
        },
    );
    state.read(cx).clone()
}

/// 閉じる動作のハンドラ
pub(crate) type DismissFn = std::rc::Rc<dyn Fn(&mut Window, &mut App)>;

/// ポップアップにフォーカスを持たせ、Esc で閉じられるようにする
pub(crate) fn dismiss_on_escape(
    el: Stateful<Div>,
    focus: &FocusHandle,
    dismiss: Option<DismissFn>,
) -> Stateful<Div> {
    let el = el.track_focus(focus);
    match dismiss {
        Some(d) => el.on_key_down(move |e, window, cx| {
            if e.keystroke.key == "escape" {
                cx.stop_propagation();
                d(window, cx);
            }
        }),
        None => el,
    }
}

pub(crate) fn use_ripple(id: &SharedString, window: &mut Window, cx: &mut App) -> RippleHandle {
    let state = window.use_keyed_state(SharedString::from(format!("{id}::ripple")), cx, |_, cx| {
        RippleState {
            ripples: Vec::new(),
            focus: cx.focus_handle().tab_index(0).tab_stop(true),
        }
    });
    let focus = state.read(cx).focus.clone();
    RippleHandle { state, focus }
}

fn release(state: &Entity<RippleState>, cx: &mut App) {
    state.update(cx, |s, cx| {
        let now = Instant::now();
        let mut changed = false;
        for r in s.ripples.iter_mut().filter(|r| r.released.is_none()) {
            r.released = Some(now);
            changed = true;
        }
        if changed {
            cx.notify();
        }
    });
}

/// 押下・解放を波紋の状態に反映し、キーボードフォーカスを受けられるようにする
pub(crate) fn track_ripple(el: Stateful<Div>, handle: &RippleHandle) -> Stateful<Div> {
    let down = handle.state.clone();
    let up = handle.state.clone();
    let out = handle.state.clone();
    el.track_focus(&handle.focus)
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let position = e.position;
            down.update(cx, |s, cx| {
                s.ripples.push(Ripple {
                    origin: position,
                    start: Instant::now(),
                    released: None,
                });
                if s.ripples.len() > 6 {
                    s.ripples.remove(0);
                }
                cx.notify();
            });
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| release(&up, cx))
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| release(&out, cx))
}

fn ease_out_cubic(t: f32) -> f32 {
    1. - (1. - t).powi(3)
}

type P = (f32, f32);

/// 角丸矩形の外周（画面座標で時計回り）
fn rounded_rect_polygon(b: Bounds<Pixels>, radii: Corners<Pixels>) -> Vec<P> {
    let (x0, y0) = (f32::from(b.origin.x), f32::from(b.origin.y));
    let (x1, y1) = (x0 + f32::from(b.size.width), y0 + f32::from(b.size.height));
    let max_r = ((x1 - x0).min(y1 - y0) / 2.).max(0.);
    let r = |p: Pixels| f32::from(p).clamp(0., max_r);
    let (tl, tr, br, bl) = (
        r(radii.top_left),
        r(radii.top_right),
        r(radii.bottom_right),
        r(radii.bottom_left),
    );

    let mut pts: Vec<P> = Vec::new();
    let mut push = |p: P| {
        if pts
            .last()
            .is_none_or(|l| (l.0 - p.0).abs() > 0.01 || (l.1 - p.1).abs() > 0.01)
        {
            pts.push(p);
        }
    };
    let mut arc = |cx: f32, cy: f32, r: f32, a0: f32, a1: f32, corner: P| {
        if r < 0.5 {
            push(corner);
            return;
        }
        const N: usize = 10;
        for i in 0..=N {
            let a = a0 + (a1 - a0) * i as f32 / N as f32;
            push((cx + r * a.cos(), cy + r * a.sin()));
        }
    };
    use std::f32::consts::{FRAC_PI_2, PI};
    arc(x0 + tl, y0 + tl, tl, PI, PI + FRAC_PI_2, (x0, y0));
    arc(x1 - tr, y0 + tr, tr, PI + FRAC_PI_2, 2. * PI, (x1, y0));
    arc(x1 - br, y1 - br, br, 0., FRAC_PI_2, (x1, y1));
    arc(x0 + bl, y1 - bl, bl, FRAC_PI_2, PI, (x0, y1));
    pts
}

fn circle_polygon(c: P, r: f32) -> Vec<P> {
    let n = ((r / 2.).ceil() as usize).clamp(32, 120);
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            (c.0 + r * a.cos(), c.1 + r * a.sin())
        })
        .collect()
}

/// Sutherland–Hodgman。`clip` は時計回りの凸多角形
fn clip_polygon(subject: Vec<P>, clip: &[P]) -> Vec<P> {
    let mut output = subject;
    for i in 0..clip.len() {
        if output.is_empty() {
            break;
        }
        let a = clip[i];
        let b = clip[(i + 1) % clip.len()];
        if (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4 {
            continue;
        }
        // 内側なら cross >= 0
        let side = |p: P| (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        let input = std::mem::take(&mut output);
        for j in 0..input.len() {
            let cur = input[j];
            let prev = input[(j + input.len() - 1) % input.len()];
            let (dc, dp) = (side(cur), side(prev));
            if dc >= 0. {
                if dp < 0. {
                    output.push(intersect(prev, cur, dp, dc));
                }
                output.push(cur);
            } else if dp >= 0. {
                output.push(intersect(prev, cur, dp, dc));
            }
        }
    }
    output
}

fn intersect(p: P, q: P, dp: f32, dq: f32) -> P {
    let t = dp / (dp - dq);
    (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t)
}

/// 波紋を描く透明なオーバーレイ。要素の**最初の子**として置く（後続の子が上に描かれる）
pub(crate) fn ripple_overlay(
    handle: &RippleHandle,
    color: Hsla,
    radii: Corners<Pixels>,
) -> AnyElement {
    let state = handle.state.clone();
    let focus = handle.focus.clone();
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            // キーボードで移動してきたときだけ、フォーカスの表示を出す（マウスでは出さない）
            if focus.is_focused(window) && window.last_input_was_keyboard() {
                window.paint_quad(fill(bounds, color.opacity(0.10)).corner_radii(radii));
                window.paint_quad(quad(
                    bounds,
                    radii,
                    gpui::transparent_black(),
                    Edges::all(px(3.)),
                    color,
                    BorderStyle::Solid,
                ));
            }
            let now = Instant::now();
            // 終わった波紋を片付ける（通知はしない）
            let active: Vec<(Point<Pixels>, Instant, Option<Instant>)> =
                state.update(cx, |s, _| {
                    s.ripples.retain(|r| match r.released {
                        Some(t) => now.duration_since(t) < FADE,
                        None => true,
                    });
                    s.ripples
                        .iter()
                        .map(|r| (r.origin, r.start, r.released))
                        .collect()
                });
            if active.is_empty() {
                return;
            }

            let clip = rounded_rect_polygon(bounds, radii);
            let corners = [
                (f32::from(bounds.left()), f32::from(bounds.top())),
                (f32::from(bounds.right()), f32::from(bounds.top())),
                (f32::from(bounds.right()), f32::from(bounds.bottom())),
                (f32::from(bounds.left()), f32::from(bounds.bottom())),
            ];
            for (origin, start, released) in active {
                let c = (f32::from(origin.x), f32::from(origin.y));
                let r_max = corners
                    .iter()
                    .map(|p| ((p.0 - c.0).powi(2) + (p.1 - c.1).powi(2)).sqrt())
                    .fold(0., f32::max);
                let elapsed = now.duration_since(start);
                let t = (elapsed.as_secs_f32() / EXPAND.as_secs_f32()).clamp(0., 1.);
                let radius = r_max * ease_out_cubic(t);
                let mut alpha =
                    PEAK * (elapsed.as_secs_f32() / FADE_IN.as_secs_f32()).clamp(0., 1.);
                if let Some(rel) = released {
                    alpha *= 1.
                        - (now.duration_since(rel).as_secs_f32() / FADE.as_secs_f32())
                            .clamp(0., 1.);
                }
                if radius < 0.5 || alpha <= 0.001 {
                    continue;
                }
                let poly = clip_polygon(circle_polygon(c, radius), &clip);
                if poly.len() < 3 {
                    continue;
                }
                let mut b = PathBuilder::fill();
                b.move_to(gpui::point(px(poly[0].0), px(poly[0].1)));
                for p in &poly[1..] {
                    b.line_to(gpui::point(px(p.0), px(p.1)));
                }
                b.close();
                if let Ok(path) = b.build() {
                    window.paint_path(path, color.opacity(alpha));
                }
            }
            window.request_animation_frame();
        },
    )
    // size_full だけだと、padding のある親では静的位置（padding 分ずれた場所）から
    // 描かれてしまうので、四辺を 0 に固定して親いっぱいに広げる
    .absolute()
    .inset_0()
    .into_any_element()
}
