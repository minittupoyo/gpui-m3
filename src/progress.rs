//! Progress indicators（Expressive: 波線 Wavy 対応の Linear / Circular）

use std::{f32::consts::TAU, rc::Rc, time::Duration};

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Bounds, ElementId, Hsla, IntoElement,
    PathBuilder, Pixels, Point, RenderOnce, SharedString, SpringAnimation, Window, canvas, div,
    fill, point, prelude::*, px, size,
};

use super::{ActiveTheme, f};

// ---------------------------------------------------------------------------
// 描画ヘルパー
// ---------------------------------------------------------------------------

pub(crate) fn paint_dot(window: &mut Window, center: Point<Pixels>, d: f32, color: Hsla) {
    let r = px(d / 2.);
    window.paint_quad(
        fill(Bounds::new(center - point(r, r), size(px(d), px(d))), color).corner_radii(r),
    );
}

/// 丸キャップ付きのポリライン
pub(crate) fn paint_polyline(window: &mut Window, pts: &[Point<Pixels>], width: f32, color: Hsla) {
    if pts.len() < 2 {
        if let Some(p) = pts.first() {
            paint_dot(window, *p, width, color);
        }
        return;
    }
    let mut b = PathBuilder::stroke(px(width));
    b.move_to(pts[0]);
    for p in &pts[1..] {
        b.line_to(*p);
    }
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
    paint_dot(window, pts[0], width, color);
    paint_dot(window, pts[pts.len() - 1], width, color);
}

/// value があれば spring で滑らかに追従させ、animate なら毎フレーム t (0..1) を渡して描画する
pub(crate) fn animated_canvas(
    id: &SharedString,
    value: Option<f32>,
    animate: bool,
    period: Duration,
    spring: gpui::SpringConfig,
    paint: impl Fn(Bounds<Pixels>, Option<f32>, f32, &mut Window) + 'static,
) -> AnyElement {
    let paint = Rc::new(paint);
    let anim_id = ElementId::Name(format!("{id}::anim").into());
    let make = move |v: Option<f32>| -> AnyElement {
        let paint = paint.clone();
        if animate {
            div()
                .size_full()
                .with_animation(anim_id, Animation::new(period).repeat(), move |el, t| {
                    let p = paint.clone();
                    el.child(canvas(|_, _, _| {}, move |b, _, w, _| p(b, v, t, w)).size_full())
                })
                .into_any_element()
        } else {
            canvas(|_, _, _| {}, move |b, _, w, _| paint(b, v, 0., w))
                .size_full()
                .into_any_element()
        }
    };
    match value {
        Some(v) => div()
            .size_full()
            .with_spring(
                ElementId::Name(format!("{id}::value").into()),
                SpringAnimation::new(spring).to(v.clamp(0., 1.)),
                move |el, v| el.child(make(Some(v.clamp(0., 1.)))),
            )
            .into_any_element(),
        None => make(None),
    }
}

/// 不確定アニメーション用の 2 本のセグメント（0..1 の区間）
fn indeterminate_segments(t: f32) -> [(f32, f32); 2] {
    let ease = |x: f32| {
        let x = x.clamp(0., 1.);
        x * x * (3. - 2. * x)
    };
    let seg = |t: f32| {
        let head = ease(t / 0.75);
        let tail = ease((t - 0.25) / 0.75);
        (tail, head)
    };
    [seg(t), seg((t + 0.5) % 1.)]
}

// ---------------------------------------------------------------------------
// Linear
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct LinearProgress {
    id: SharedString,
    value: Option<f32>,
    wavy: bool,
    thickness: f32,
}

impl LinearProgress {
    /// value: Some(0..1) で確定、None で不確定
    pub fn new(id: impl Into<SharedString>, value: Option<f32>) -> Self {
        Self {
            id: id.into(),
            value,
            wavy: false,
            thickness: 4.,
        }
    }
    pub fn wavy(mut self, wavy: bool) -> Self {
        self.wavy = wavy;
        self
    }
    /// 線の太さ（既定 4、Expressive では 8 の太いバリエーションもある）
    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = t;
        self
    }
}

impl RenderOnce for LinearProgress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let active = theme.colors.primary;
        let track = theme.colors.secondary_container;
        let stroke = self.thickness;
        let wavy = self.wavy;
        let amplitude = 3.;
        let wavelength = 40.;
        let gap = 4.;
        let height = if wavy {
            stroke + amplitude * 2. + 2.
        } else {
            stroke
        };
        let animate = wavy || self.value.is_none();
        let period = if self.value.is_none() {
            Duration::from_millis(1800)
        } else {
            Duration::from_millis(1000)
        };

        let paint = move |b: Bounds<Pixels>, v: Option<f32>, t: f32, window: &mut Window| {
            let left = f(b.origin.x) + stroke / 2.;
            let w = f(b.size.width) - stroke;
            let cy = f(b.origin.y) + f(b.size.height) / 2.;
            let x_at = |frac: f32| left + w * frac;

            // 波線の y（determinate では進行に応じて波が流れる）
            let phase = t * TAU;
            let amp_for = |len: f32| {
                if len < 0.05 {
                    amplitude * len / 0.05
                } else {
                    amplitude
                }
            };
            let active_segment = |window: &mut Window, from: f32, to: f32, amp: f32| {
                if to - from <= 0.0001 {
                    return;
                }
                let x0 = x_at(from);
                let x1 = x_at(to);
                let mut pts = vec![];
                if wavy {
                    let steps = ((x1 - x0) / 2.).ceil().max(1.) as usize;
                    for i in 0..=steps {
                        let x = x0 + (x1 - x0) * i as f32 / steps as f32;
                        let y = cy + amp * ((x - left) / wavelength * TAU - phase).sin();
                        pts.push(point(px(x), px(y)));
                    }
                } else {
                    pts.push(point(px(x0), px(cy)));
                    pts.push(point(px(x1), px(cy)));
                }
                paint_polyline(window, &pts, stroke, active);
            };
            let track_segment = |window: &mut Window, from: f32, to: f32| {
                let x0 = x_at(from);
                let x1 = x_at(to);
                if x1 - x0 <= 0. {
                    return;
                }
                paint_polyline(
                    window,
                    &[point(px(x0), px(cy)), point(px(x1), px(cy))],
                    stroke,
                    track,
                );
            };
            let gap_frac = (gap + stroke) / w;

            match v {
                Some(v) => {
                    let track_start = if v > 0. { v + gap_frac } else { 0. };
                    track_segment(window, track_start.min(1.), 1.);
                    // ストップインジケーター
                    paint_dot(window, point(px(x_at(1.)), px(cy)), stroke, active);
                    active_segment(window, 0., v, amp_for(v));
                }
                None => {
                    let segs = indeterminate_segments(t);
                    // 区間の外側をトラックで埋める
                    let mut covered: Vec<(f32, f32)> = segs
                        .iter()
                        .filter(|(a, b)| b - a > 0.001)
                        .copied()
                        .collect();
                    covered.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                    let mut cursor = 0.;
                    for (a, b) in &covered {
                        track_segment(window, cursor, (a - gap_frac).max(cursor));
                        cursor = (b + gap_frac).min(1.);
                    }
                    track_segment(window, cursor, 1.);
                    for (a, b) in covered {
                        active_segment(window, a, b, amplitude);
                    }
                }
            }
        };

        div().w_full().h(px(height)).child(animated_canvas(
            &self.id,
            self.value,
            animate,
            period,
            theme.motion.default_spatial,
            paint,
        ))
    }
}

// ---------------------------------------------------------------------------
// Circular
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct CircularProgress {
    id: SharedString,
    value: Option<f32>,
    wavy: bool,
    diameter: f32,
}

impl CircularProgress {
    pub fn new(id: impl Into<SharedString>, value: Option<f32>) -> Self {
        Self {
            id: id.into(),
            value,
            wavy: false,
            diameter: 40.,
        }
    }
    pub fn wavy(mut self, wavy: bool) -> Self {
        self.wavy = wavy;
        if wavy && self.diameter == 40. {
            self.diameter = 48.;
        }
        self
    }
    pub fn diameter(mut self, d: f32) -> Self {
        self.diameter = d;
        self
    }
}

impl RenderOnce for CircularProgress {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let active = theme.colors.primary;
        let track = theme.colors.secondary_container;
        let stroke = 4.;
        let wavy = self.wavy;
        let amplitude = if wavy { 1.6 } else { 0. };
        let diameter = self.diameter;
        let animate = wavy || self.value.is_none();
        let period = if self.value.is_none() {
            Duration::from_millis(1600)
        } else {
            Duration::from_millis(1200)
        };

        let paint = move |b: Bounds<Pixels>, v: Option<f32>, t: f32, window: &mut Window| {
            let c = b.center();
            let (cx0, cy0) = (f(c.x), f(c.y));
            let r = diameter / 2. - stroke / 2. - amplitude;
            let waves = (TAU * r / 14.).round();
            let arc =
                |window: &mut Window, from: f32, to: f32, color: Hsla, wave: bool, rot: f32| {
                    // from/to は 0..1（12 時起点・時計回り）
                    if to - from <= 0.0005 {
                        return;
                    }
                    let steps = (((to - from) * 160.).ceil() as usize).max(2);
                    let pts: Vec<_> = (0..=steps)
                        .map(|i| {
                            let u = from + (to - from) * i as f32 / steps as f32;
                            let a = u * TAU - TAU / 4. + rot;
                            let rr = if wave {
                                r + amplitude * (a * waves - t * TAU).sin()
                            } else {
                                r
                            };
                            point(px(cx0 + rr * a.cos()), px(cy0 + rr * a.sin()))
                        })
                        .collect();
                    paint_polyline(window, &pts, stroke, color);
                };
            let gap = (4. + stroke) / (TAU * r);

            match v {
                Some(v) => {
                    if v < 1. {
                        let start = if v > 0. { v + gap } else { 0. };
                        let end = if v > 0. { 1. - gap } else { 1. };
                        arc(window, start, end, track, false, 0.);
                    }
                    arc(window, 0., v, active, wavy, 0.);
                }
                None => {
                    // 伸び縮みしながら回転する弧。
                    // 一周（t: 0→1）で「弧の長さ」も「中心角」も元に戻る（中心は整数周）ので、
                    // ループの継ぎ目で飛ばない。
                    const MIN_LEN: f32 = 0.04;
                    const MAX_LEN: f32 = 0.70;
                    const TURNS: f32 = 2.;
                    let len = MIN_LEN + (MAX_LEN - MIN_LEN) * (0.5 - 0.5 * (TAU * t).cos());
                    let mid = TURNS * t;
                    let (from, to) = (mid - len / 2., mid + len / 2.);
                    arc(window, to + gap, from + 1. - gap, track, false, 0.);
                    arc(window, from, to, active, wavy, 0.);
                }
            }
        };

        div().size(px(diameter)).flex_none().child(animated_canvas(
            &self.id,
            self.value,
            animate,
            period,
            theme.motion.default_spatial,
            paint,
        ))
    }
}
