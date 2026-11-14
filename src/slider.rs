//! Slider（M3 Expressive: 太いトラック・縦長ハンドル・ハンドル両脇のギャップ・値ラベル）

use std::rc::Rc;

use gpui::{
    App, BorderStyle, Bounds, Corners, DispatchPhase, Edges, ElementId, IntoElement, MouseButton,
    MouseMoveEvent, MouseUpEvent, Pixels, RenderOnce, SharedString, Window, canvas, div, fill,
    point, prelude::*, px, quad, relative, size,
};

use super::{ActiveTheme, ChangeHandler, StyledExt, TypeScale, f, paint_dot, use_ripple};

#[derive(Default)]
struct SliderState {
    bounds: Bounds<Pixels>,
    dragging: bool,
}

#[derive(IntoElement)]
pub struct Slider {
    id: SharedString,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    track_height: f32,
    disabled: bool,
    on_change: Option<ChangeHandler<f32>>,
}

impl Slider {
    pub fn new(id: impl Into<SharedString>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min: 0.,
            max: 1.,
            step: None,
            track_height: 16.,
            disabled: false,
            on_change: None,
        }
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// 離散スライダー（目盛り付き）
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    /// トラックの太さ（Expressive: XS 16 / S 24 / M 40 / L 56 / XL 96）
    pub fn track_height(mut self, h: f32) -> Self {
        self.track_height = h;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_change(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let state = window.use_keyed_state(
            SharedString::from(format!("{}::slider", self.id)),
            cx,
            |_, _| SliderState::default(),
        );
        let dragging = state.read(cx).dragging;

        let (min, max, step) = (self.min, self.max, self.step);
        let frac = ((self.value - min) / (max - min)).clamp(0., 1.);
        let track_h = self.track_height;
        let handle_h = (track_h + 28.).max(44.);
        let handle_w = if dragging { 2. } else { 4. };
        let gap = 6.;

        let (active, inactive, handle, stop_on_active, stop_on_inactive) = if self.disabled {
            let d = theme.disabled_content();
            (d, c.on_surface.opacity(0.12), d, c.inverse_on_surface, d)
        } else {
            (
                c.primary,
                c.secondary_container,
                c.primary,
                c.on_primary,
                c.on_secondary_container,
            )
        };

        // 位置 → 値
        let to_value = move |x: Pixels, b: Bounds<Pixels>| -> f32 {
            let w = f(b.size.width).max(1.);
            let t = ((f(x) - f(b.origin.x)) / w).clamp(0., 1.);
            let mut v = min + t * (max - min);
            if let Some(s) = step {
                v = min + ((v - min) / s).round() * s;
            }
            v.clamp(min, max)
        };

        let on_change = self.on_change.clone();
        let disabled = self.disabled;
        let paint_state = state.clone();
        let prepaint_state = state.clone();
        let ripple = use_ripple(&self.id, window, cx);
        let ring_focus = ripple.focus().clone();

        let drawing = canvas(
            move |b, _, cx| {
                prepaint_state.update(cx, |s, _| s.bounds = b);
            },
            move |b, _, window, _| {
                let left = f(b.origin.x);
                let w = f(b.size.width);
                let cy = f(b.origin.y) + f(b.size.height) / 2.;
                let x = left + w * frac;
                let top = cy - track_h / 2.;
                let outer = track_h / 2.;
                let inner = 2.;

                // アクティブトラック
                let a_end = x - handle_w / 2. - gap;
                if a_end - left > 0. {
                    window.paint_quad(
                        fill(
                            Bounds::new(
                                point(px(left), px(top)),
                                size(px(a_end - left), px(track_h)),
                            ),
                            active,
                        )
                        .corner_radii(Corners {
                            top_left: px(outer.min((a_end - left) / 2.)),
                            bottom_left: px(outer.min((a_end - left) / 2.)),
                            top_right: px(inner),
                            bottom_right: px(inner),
                        }),
                    );
                }
                // 非アクティブトラック
                let i_start = x + handle_w / 2. + gap;
                let right = left + w;
                if right - i_start > 0. {
                    window.paint_quad(
                        fill(
                            Bounds::new(
                                point(px(i_start), px(top)),
                                size(px(right - i_start), px(track_h)),
                            ),
                            inactive,
                        )
                        .corner_radii(Corners {
                            top_left: px(inner),
                            bottom_left: px(inner),
                            top_right: px(outer.min((right - i_start) / 2.)),
                            bottom_right: px(outer.min((right - i_start) / 2.)),
                        }),
                    );
                }
                // 目盛り / ストップインジケーター
                let dot = 4.;
                let inset = (track_h / 2.).min(8.);
                if let Some(s) = step {
                    let n = ((max - min) / s).round() as usize;
                    for k in 0..=n {
                        let t = k as f32 / n as f32;
                        let px_ = left + inset + (w - inset * 2.) * t;
                        if (px_ - x).abs() < handle_w / 2. + gap + dot {
                            continue;
                        }
                        let color = if px_ < x { stop_on_active } else { stop_on_inactive };
                        paint_dot(window, point(px(px_), px(cy)), dot, color);
                    }
                } else if right - inset - x > handle_w / 2. + gap + dot {
                    paint_dot(window, point(px(right - inset), px(cy)), dot, stop_on_inactive);
                }
                // ハンドル
                window.paint_quad(
                    fill(
                        Bounds::new(
                            point(px(x - handle_w / 2.), px(cy - handle_h / 2.)),
                            size(px(handle_w), px(handle_h)),
                        ),
                        handle,
                    )
                    .corner_radii(px(handle_w / 2.)),
                );
                // キーボードでフォーカスしているときは、ハンドルの周りに枠を出す
                if !disabled && ring_focus.is_focused(window) && window.last_input_was_keyboard() {
                    window.paint_quad(quad(
                        Bounds::new(
                            point(px(x - 9.), px(cy - handle_h / 2. - 4.)),
                            size(px(18.), px(handle_h + 8.)),
                        ),
                        px(9.),
                        gpui::transparent_black(),
                        Edges::all(px(2.)),
                        handle,
                        BorderStyle::Solid,
                    ));
                }

                if disabled {
                    return;
                }
                // ドラッグ中はウィンドウ全体でマウスを追う
                let st = paint_state.clone();
                let on_change = on_change.clone();
                window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    let s = st.read(cx);
                    if !s.dragging {
                        return;
                    }
                    let v = to_value(e.position.x, s.bounds);
                    if let Some(h) = on_change.as_ref() {
                        h(v, window, cx);
                    }
                });
                let st = paint_state.clone();
                window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    st.update(cx, |s, cx| {
                        if s.dragging {
                            s.dragging = false;
                            cx.notify();
                        }
                    });
                });
            },
        )
        .absolute()
        .size_full();

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .relative()
            .w_full()
            .h(px(handle_h))
            .child(drawing);

        if !self.disabled {
            // 矢印キー・Home/End・PageUp/Down で値を動かす
            let key_change = self.on_change.clone();
            let value = self.value;
            el = el
                .track_focus(ripple.focus())
                .on_key_down(move |e, window, cx| {
                    let unit = step.unwrap_or((max - min) / 20.);
                    let next = match e.keystroke.key.as_str() {
                        "left" | "down" => value - unit,
                        "right" | "up" => value + unit,
                        "pagedown" => value - unit * 5.,
                        "pageup" => value + unit * 5.,
                        "home" => min,
                        "end" => max,
                        _ => return,
                    };
                    cx.stop_propagation();
                    if let Some(h) = key_change.as_ref() {
                        h(next.clamp(min, max), window, cx);
                    }
                });
            let st = state.clone();
            let on_change = self.on_change.clone();
            el = el
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |e, window, cx| {
                    let b = st.read(cx).bounds;
                    st.update(cx, |s, cx| {
                        s.dragging = true;
                        cx.notify();
                    });
                    let v = to_value(e.position.x, b);
                    if let Some(h) = on_change.as_ref() {
                        h(v, window, cx);
                    }
                });
        }

        // 値ラベル（ドラッグ中のみ）
        if dragging {
            let label = if step.is_some_and(|s| s >= 1.) || (max - min) >= 10. {
                format!("{:.0}", self.value)
            } else {
                format!("{:.2}", self.value)
            };
            el = el.child(
                div()
                    .absolute()
                    .left(relative(frac))
                    .top(px(-44.))
                    .child(
                        div()
                            .relative()
                            .left(px(-24.))
                            .w(px(48.))
                            .h(px(44.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .bg(c.inverse_surface)
                            .text_color(c.inverse_on_surface)
                            .m3_type(TypeScale::LabelLarge)
                            .child(label),
                    ),
            );
        }
        el
    }
}
