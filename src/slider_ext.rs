//! 範囲スライダー（つまみ 2 つ）と縦スライダー
//!
//! 見た目は [`Slider`](super::Slider) と同じ（縦長のつまみ + 両脇のギャップ）。

use std::rc::Rc;

use gpui::{
    App, BorderStyle, Bounds, Corners, DispatchPhase, Edges, ElementId, IntoElement, MouseButton,
    MouseMoveEvent, MouseUpEvent, Pixels, RenderOnce, SharedString, Window, canvas, div, fill,
    point, prelude::*, px, quad, relative, size,
};

use super::{ActiveTheme, StyledExt, TypeScale, f, use_ripple};

#[derive(Default)]
struct LinearState {
    bounds: Bounds<Pixels>,
    dragging: Option<usize>,
}

type Change = Rc<dyn Fn(&[f32], &mut Window, &mut App)>;

struct Linear {
    id: SharedString,
    values: Vec<f32>,
    min: f32,
    max: f32,
    step: Option<f32>,
    vertical: bool,
    track: f32,
    length: Pixels,
    disabled: bool,
    on_change: Option<Change>,
}

fn quantize(v: f32, min: f32, max: f32, step: Option<f32>) -> f32 {
    let v = match step {
        Some(s) if s > 0. => min + ((v - min) / s).round() * s,
        _ => v,
    };
    v.clamp(min, max)
}

/// 軸方向の区間 [a0, a1]（最小値側が 0）を矩形にする
fn segment_bounds(
    b: Bounds<Pixels>,
    vertical: bool,
    a0: f32,
    a1: f32,
    track: f32,
) -> Bounds<Pixels> {
    if vertical {
        let cx = f(b.origin.x) + f(b.size.width) / 2.;
        let bottom = f(b.origin.y) + f(b.size.height);
        Bounds::new(
            point(px(cx - track / 2.), px(bottom - a1)),
            size(px(track), px(a1 - a0)),
        )
    } else {
        let cy = f(b.origin.y) + f(b.size.height) / 2.;
        Bounds::new(
            point(px(f(b.origin.x) + a0), px(cy - track / 2.)),
            size(px(a1 - a0), px(track)),
        )
    }
}

fn render_linear(l: Linear, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let theme = cx.m3().clone();
    let c = theme.colors.clone();
    let state = window.use_keyed_state(
        SharedString::from(format!("{}::linear", l.id)),
        cx,
        |_, _| LinearState::default(),
    );
    let ripple = use_ripple(&l.id, window, cx);
    let ring_focus = ripple.focus().clone();
    let dragging = state.read(cx).dragging;

    let Linear {
        id,
        values,
        min,
        max,
        step,
        vertical,
        track,
        length,
        disabled,
        on_change,
    } = l;
    let range = values.len() == 2;
    let fracs: Vec<f32> = values
        .iter()
        .map(|v| ((v - min) / (max - min)).clamp(0., 1.))
        .collect();
    let handle_len = (track + 28.).max(44.);
    let handle_w = move |i: usize| if dragging == Some(i) { 2. } else { 4. };
    let gap = 6.;
    let (active, inactive, handle) = if disabled {
        (
            theme.disabled_content(),
            c.on_surface.opacity(0.12),
            theme.disabled_content(),
        )
    } else {
        (c.primary, c.secondary_container, c.primary)
    };

    let to_frac = move |pos: Pixels, b: Bounds<Pixels>| -> f32 {
        if vertical {
            let h = f(b.size.height).max(1.);
            ((f(b.origin.y) + h - f(pos)) / h).clamp(0., 1.)
        } else {
            let w = f(b.size.width).max(1.);
            ((f(pos) - f(b.origin.x)) / w).clamp(0., 1.)
        }
    };
    let axis = move |p: gpui::Point<Pixels>| if vertical { p.y } else { p.x };

    let measure = state.clone();
    let paint_state = state.clone();
    let values_paint = values.clone();
    let on_change_paint = on_change.clone();
    let fracs_paint = fracs.clone();
    let drawing = canvas(
        move |b, _, cx| {
            measure.update(cx, |s, _| s.bounds = b);
        },
        move |b, _, window, _| {
            let len = if vertical {
                f(b.size.height)
            } else {
                f(b.size.width)
            };
            let outer = track / 2.;
            let inner = 2.;
            let pos: Vec<f32> = fracs_paint.iter().map(|t| t * len).collect();

            // (開始, 終了, active)
            let mut segs: Vec<(f32, f32, bool)> = Vec::new();
            if range {
                segs.push((0., pos[0] - gap - handle_w(0) / 2., false));
                segs.push((
                    pos[0] + gap + handle_w(0) / 2.,
                    pos[1] - gap - handle_w(1) / 2.,
                    true,
                ));
                segs.push((pos[1] + gap + handle_w(1) / 2., len, false));
            } else {
                segs.push((0., pos[0] - gap - handle_w(0) / 2., true));
                segs.push((pos[0] + gap + handle_w(0) / 2., len, false));
            }
            for (a0, a1, is_active) in segs {
                if a1 - a0 <= 0.5 {
                    continue;
                }
                let r_start = if a0 <= 0.01 {
                    outer.min((a1 - a0) / 2.)
                } else {
                    inner
                };
                let r_end = if a1 >= len - 0.01 {
                    outer.min((a1 - a0) / 2.)
                } else {
                    inner
                };
                let corners = if vertical {
                    // 最小値側が下
                    Corners {
                        bottom_left: px(r_start),
                        bottom_right: px(r_start),
                        top_left: px(r_end),
                        top_right: px(r_end),
                    }
                } else {
                    Corners {
                        top_left: px(r_start),
                        bottom_left: px(r_start),
                        top_right: px(r_end),
                        bottom_right: px(r_end),
                    }
                };
                window.paint_quad(
                    fill(
                        segment_bounds(b, vertical, a0, a1, track),
                        if is_active { active } else { inactive },
                    )
                    .corner_radii(corners),
                );
            }
            for (i, p) in pos.iter().enumerate() {
                let hw = handle_w(i);
                let hb = if vertical {
                    let cx = f(b.origin.x) + f(b.size.width) / 2.;
                    let y = f(b.origin.y) + f(b.size.height) - p;
                    Bounds::new(
                        point(px(cx - handle_len / 2.), px(y - hw / 2.)),
                        size(px(handle_len), px(hw)),
                    )
                } else {
                    let cy = f(b.origin.y) + f(b.size.height) / 2.;
                    Bounds::new(
                        point(px(f(b.origin.x) + p - hw / 2.), px(cy - handle_len / 2.)),
                        size(px(hw), px(handle_len)),
                    )
                };
                window.paint_quad(fill(hb, handle).corner_radii(px(hw / 2.)));
                if !disabled && ring_focus.is_focused(window) && window.last_input_was_keyboard() {
                    window.paint_quad(quad(
                        Bounds::new(
                            hb.origin - point(px(4.), px(4.)),
                            size(hb.size.width + px(8.), hb.size.height + px(8.)),
                        ),
                        px(9.),
                        gpui::transparent_black(),
                        Edges::all(px(2.)),
                        handle,
                        BorderStyle::Solid,
                    ));
                }
            }

            if disabled {
                return;
            }
            // ドラッグ中はウィンドウ全体でマウスを追う
            let st = paint_state.clone();
            let ch = on_change_paint.clone();
            let vals = values_paint.clone();
            window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                let (Some(i), bounds) = ({
                    let s = st.read(cx);
                    (s.dragging, s.bounds)
                }) else {
                    return;
                };
                let t = to_frac(axis(e.position), bounds);
                let mut next = vals.clone();
                let v = quantize(min + t * (max - min), min, max, step);
                next[i] = v;
                if next.len() == 2 {
                    if i == 0 {
                        next[0] = next[0].min(next[1]);
                    } else {
                        next[1] = next[1].max(next[0]);
                    }
                }
                if let Some(h) = ch.as_ref() {
                    h(&next, window, cx);
                }
            });
            let st = paint_state.clone();
            window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                if phase != DispatchPhase::Bubble {
                    return;
                }
                st.update(cx, |s, cx| {
                    if s.dragging.take().is_some() {
                        cx.notify();
                    }
                });
            });
        },
    )
    .absolute()
    .size_full();

    let mut el = div()
        .id(ElementId::Name(id.clone()))
        .relative()
        .child(drawing);
    el = if vertical {
        el.w(px(handle_len)).h(length)
    } else {
        el.w_full().h(px(handle_len))
    };

    if !disabled {
        let st = state.clone();
        let vals = values.clone();
        let ch = on_change.clone();
        let fr = fracs.clone();
        el = el
            .track_focus(ripple.focus())
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |e, window, cx| {
                let bounds = st.read(cx).bounds;
                let t = to_frac(axis(e.position), bounds);
                // いちばん近いつまみを掴む
                let i = fr
                    .iter()
                    .enumerate()
                    .min_by(|a, b| (a.1 - t).abs().partial_cmp(&(b.1 - t).abs()).unwrap())
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                st.update(cx, |s, cx| {
                    s.dragging = Some(i);
                    cx.notify();
                });
                let mut next = vals.clone();
                next[i] = quantize(min + t * (max - min), min, max, step);
                if next.len() == 2 {
                    if i == 0 {
                        next[0] = next[0].min(next[1]);
                    } else {
                        next[1] = next[1].max(next[0]);
                    }
                }
                if let Some(h) = ch.as_ref() {
                    h(&next, window, cx);
                }
            });
        // キーボード: 矢印・Home / End（範囲スライダーでは近い方でなく、上側のつまみ）
        let vals = values.clone();
        let ch = on_change.clone();
        el = el.on_key_down(move |e, window, cx| {
            let unit = step.unwrap_or((max - min) / 20.);
            let i = vals.len() - 1;
            let cur = vals[i];
            let next = match e.keystroke.key.as_str() {
                "left" | "down" => cur - unit,
                "right" | "up" => cur + unit,
                "home" => min,
                "end" => max,
                _ => return,
            };
            cx.stop_propagation();
            let mut out = vals.clone();
            out[i] = quantize(next, min, max, step);
            if out.len() == 2 {
                out[1] = out[1].max(out[0]);
            }
            if let Some(h) = ch.as_ref() {
                h(&out, window, cx);
            }
        });
    }

    // ドラッグ中の値ラベル
    if let Some(i) = dragging {
        let label = if step.is_some_and(|s| s >= 1.) || (max - min) >= 10. {
            format!("{:.0}", values[i])
        } else {
            format!("{:.2}", values[i])
        };
        let bubble = div()
            .w(px(48.))
            .h(px(40.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .bg(c.inverse_surface)
            .text_color(c.inverse_on_surface)
            .m3_type(TypeScale::LabelLarge)
            .child(label);
        let holder = if vertical {
            div()
                .absolute()
                .bottom(relative(fracs[i]))
                .left(px(handle_len + 4.))
                .mb(px(-20.))
                .child(bubble)
        } else {
            div()
                .absolute()
                .left(relative(fracs[i]))
                .top(px(-44.))
                .ml(px(-24.))
                .child(bubble)
        };
        el = el.child(holder);
    }
    el
}

/// 範囲スライダー（つまみ 2 つ）
#[derive(IntoElement)]
pub struct RangeSlider(Linear);

impl RangeSlider {
    pub fn new(id: impl Into<SharedString>, values: (f32, f32)) -> Self {
        Self(Linear {
            id: id.into(),
            values: vec![values.0, values.1],
            min: 0.,
            max: 1.,
            step: None,
            vertical: false,
            track: 16.,
            length: px(0.),
            disabled: false,
            on_change: None,
        })
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.0.min = min;
        self.0.max = max;
        self
    }
    pub fn step(mut self, step: f32) -> Self {
        self.0.step = Some(step);
        self
    }
    pub fn track_height(mut self, h: f32) -> Self {
        self.0.track = h;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.0.disabled = v;
        self
    }
    pub fn on_change(mut self, f: impl Fn((f32, f32), &mut Window, &mut App) + 'static) -> Self {
        self.0.on_change = Some(Rc::new(move |v, w, cx| f((v[0], v[1]), w, cx)));
        self
    }
}

impl RenderOnce for RangeSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        render_linear(self.0, window, cx)
    }
}

/// 縦スライダー（下が最小値）
#[derive(IntoElement)]
pub struct VerticalSlider(Linear);

impl VerticalSlider {
    pub fn new(id: impl Into<SharedString>, value: f32) -> Self {
        Self(Linear {
            id: id.into(),
            values: vec![value],
            min: 0.,
            max: 1.,
            step: None,
            vertical: true,
            track: 40.,
            length: px(200.),
            disabled: false,
            on_change: None,
        })
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.0.min = min;
        self.0.max = max;
        self
    }
    pub fn step(mut self, step: f32) -> Self {
        self.0.step = Some(step);
        self
    }
    /// トラックの太さ（既定 40）
    pub fn track_width(mut self, w: f32) -> Self {
        self.0.track = w;
        self
    }
    /// スライダーの長さ（高さ）
    pub fn height(mut self, h: Pixels) -> Self {
        self.0.length = h;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.0.disabled = v;
        self
    }
    pub fn on_change(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.0.on_change = Some(Rc::new(move |v, w, cx| f(v[0], w, cx)));
        self
    }
}

impl RenderOnce for VerticalSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        render_linear(self.0, window, cx)
    }
}
