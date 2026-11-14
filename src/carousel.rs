//! Carousel（Multi-browse / Uncontained / Hero / Full-screen）
//!
//! 子要素を横に並べ、スクロール位置に応じて幅が連続的に変わる（大 → 中 → 小）。
//! ドラッグ・横ホイール・←→キーで動かし、離すと最寄りの項目にバネで吸着する。
//!
//! ```ignore
//! Carousel::new("c").kind(CarouselKind::MultiBrowse).height(200.)
//!     .child(div().size_full().bg(red))
//! ```

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, App, DispatchPhase, ElementId, FocusHandle, Hsla, IntoElement, MouseButton,
    MouseMoveEvent, MouseUpEvent, ParentElement, RenderOnce, ScrollDelta, SharedString,
    SpringAnimation, Window, canvas, div, prelude::*, px,
};

use super::{ActiveTheme, shape};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselKind {
    /// 大 → 中 → 小の順に並ぶ（既定）
    #[default]
    MultiBrowse,
    /// 項目は固定幅で、端で切れながら自由にスクロール
    Uncontained,
    /// 大きな項目 1 つと小さな項目
    Hero,
    /// 一度に 1 項目だけ表示
    FullScreen,
}

const SMALL: f32 = 56.;

struct CarState {
    /// 吸着先の項目位置（整数）
    target: f32,
    /// ドラッグ中の連続的な位置
    live: f32,
    drag: Option<(f32, f32)>,
    width: f32,
    wheel_acc: f32,
    focus: FocusHandle,
}

#[derive(IntoElement)]
pub struct Carousel {
    id: SharedString,
    kind: CarouselKind,
    height: f32,
    gap: f32,
    items: Vec<(Option<Hsla>, AnyElement)>,
}

impl Carousel {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            kind: CarouselKind::default(),
            height: 200.,
            gap: 8.,
            items: vec![],
        }
    }
    pub fn kind(mut self, kind: CarouselKind) -> Self {
        self.kind = kind;
        self
    }
    /// 背景色つきの項目を追加する（項目自身が角丸で塗られ、幅が縮むときも角が丸いまま）
    pub fn item(mut self, bg: Hsla, content: impl IntoElement) -> Self {
        self.items.push((Some(bg), content.into_any_element()));
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }
}

impl ParentElement for Carousel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.items.extend(elements.into_iter().map(|e| (None, e)));
    }
}

/// スロット幅（先頭から順）。ビューポート幅 `w` に収まるよう決める。
fn slots(kind: CarouselKind, w: f32, gap: f32) -> Vec<f32> {
    match kind {
        CarouselKind::FullScreen => vec![w],
        CarouselKind::Hero => vec![(w - SMALL - gap).max(SMALL), SMALL],
        CarouselKind::MultiBrowse => {
            // 大 + 中 + 小。大は全体の約 45%
            let large = (w * 0.45).max(SMALL * 2.);
            let rest = (w - large - gap * 2. - SMALL).max(SMALL);
            vec![large, rest.min(large).max(SMALL), SMALL]
        }
        CarouselKind::Uncontained => vec![(w * 0.3).clamp(120., 320.)],
    }
}

/// 項目 `i` の幅（位置 `pos` のとき）
fn item_width(kind: CarouselKind, base: &[f32], i: usize, pos: f32) -> f32 {
    if kind == CarouselKind::Uncontained {
        return base[0];
    }
    let lead = pos.floor();
    let frac = pos - lead;
    let k = i as f32 - lead;
    if k < 0. {
        return 0.;
    }
    let k = k as usize;
    let at = |j: isize| -> f32 {
        if j < 0 || j as usize >= base.len() {
            0.
        } else {
            base[j as usize]
        }
    };
    at(k as isize) * (1. - frac) + at(k as isize - 1) * frac
}

impl RenderOnce for Carousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let n = self.items.len();
        let kind = self.kind;
        let gap = self.gap;
        let id = self.id.clone();
        let state = window.use_keyed_state(
            SharedString::from(format!("{id}::carousel")),
            cx,
            |_, cx| CarState {
                target: 0.,
                live: 0.,
                drag: None,
                width: 600.,
                wheel_acc: 0.,
                focus: cx.focus_handle().tab_index(0).tab_stop(true),
            },
        );
        let (target, live, dragging, width, focus) = {
            let s = state.read(cx);
            (s.target, s.live, s.drag.is_some(), s.width, s.focus.clone())
        };
        let base = slots(kind, width, gap);
        let visible = if kind == CarouselKind::Uncontained {
            (width / (base[0] + gap)).floor().max(1.) as usize
        } else {
            base.len()
        };
        let max_pos = n.saturating_sub(visible).max(0) as f32;
        let pos_goal = if dragging { live } else { target }.clamp(-0.4, max_pos + 0.4);

        let motion = theme.motion;
        let spring = if dragging {
            motion.fast_effects
        } else {
            motion.default_spatial
        };

        // ----- 項目 -----
        let lead_w = base[0];
        let mut row = div()
            .id(ElementId::Name(format!("{id}::row").into()))
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .overflow_hidden();
        let items = self.items;
        for (i, (bg, content)) in items.into_iter().enumerate() {
            let b = base.clone();
            let item = div()
                .id(ElementId::Name(format!("{id}::item{i}").into()))
                .relative()
                .flex_none()
                .h_full()
                .overflow_hidden()
                .rounded(shape::EXTRA_LARGE)
                .when_some(bg, |d, c| d.bg(c))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w(px(lead_w))
                        .h_full()
                        .child(content),
                );
            let item = if kind == CarouselKind::Uncontained {
                // 自由スクロール: 先頭項目を左へずらす
                item.with_spring(
                    ElementId::Name(format!("{id}::w{i}").into()),
                    SpringAnimation::new(spring).to(pos_goal),
                    move |el, p| {
                        let w = item_width(kind, &b, i, p);
                        let off = if i == 0 { -p * (w + gap) } else { 0. };
                        el.w(px(w)).ml(px(off)).mr(px(gap))
                    },
                )
            } else {
                item.with_spring(
                    ElementId::Name(format!("{id}::w{i}").into()),
                    SpringAnimation::new(spring).to(pos_goal),
                    move |el, p| {
                        let w = item_width(kind, &b, i, p);
                        if w < 0.5 {
                            el.w(px(0.)).hidden()
                        } else {
                            el.w(px(w)).mr(px(gap))
                        }
                    },
                )
            };
            row = row.child(item);
        }

        // ----- 入力 -----
        let measure = state.clone();
        let track = state.clone();
        let tracker = canvas(
            move |b, _, cx| {
                let w = f32::from(b.size.width);
                measure.update(cx, |s, cx| {
                    if (s.width - w).abs() > 0.5 && w > 0. {
                        s.width = w;
                        cx.notify();
                    }
                })
            },
            move |_, _, window, _| {
                let st = track.clone();
                window.on_mouse_event(move |e: &MouseMoveEvent, phase, _, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    st.update(cx, |s, cx| {
                        if let Some((x0, p0)) = s.drag {
                            let unit = unit_px(kind, s.width, gap);
                            s.live = p0 - (f32::from(e.position.x) - x0) / unit;
                            cx.notify();
                        }
                    })
                });
                let st = track.clone();
                window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    st.update(cx, |s, cx| {
                        if let Some((_, p0)) = s.drag.take() {
                            let vis = visible_count(kind, s.width, gap, n);
                            let max = n.saturating_sub(vis) as f32;
                            let d = s.live - p0;
                            // 少し動かしただけでも次の項目へ進む
                            let steps = if d.abs() > 0.2 {
                                d.abs().round().max(1.) * d.signum()
                            } else {
                                0.
                            };
                            s.target = (p0 + steps).clamp(0., max);
                            cx.notify();
                        }
                    });
                });
            },
        )
        .absolute()
        .size_full();

        let down = state.clone();
        let wheel = state.clone();
        let keys = state.clone();
        div()
            .id(ElementId::Name(id.to_string().into()))
            .relative()
            .w_full()
            .min_w_0()
            .overflow_hidden()
            .h(px(self.height))
            .track_focus(&focus)
            .cursor_grab()
            .child(tracker)
            .child(row)
            .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                let x = f32::from(e.position.x);
                down.update(cx, |s, cx| {
                    s.drag = Some((x, s.target));
                    s.live = s.target;
                    cx.notify();
                })
            })
            .on_scroll(move |e, _, cx| {
                let d = match e.delta {
                    ScrollDelta::Pixels(p) => f32::from(p.x),
                    ScrollDelta::Lines(l) => l.x * 40.,
                };
                if d.abs() < 0.5 {
                    return;
                }
                cx.stop_propagation();
                wheel.update(cx, |s, cx| {
                    s.wheel_acc += d;
                    if s.wheel_acc.abs() > 40. {
                        let dir = if s.wheel_acc > 0. { -1. } else { 1. };
                        let vis = visible_count(kind, s.width, gap, n);
                        let max = n.saturating_sub(vis) as f32;
                        s.target = (s.target + dir).clamp(0., max);
                        s.wheel_acc = 0.;
                        cx.notify();
                    }
                })
            })
            .on_key_down(move |e, _, cx| {
                let dir = match e.keystroke.key.as_str() {
                    "left" => -1.,
                    "right" => 1.,
                    _ => return,
                };
                cx.stop_propagation();
                keys.update(cx, |s, cx| {
                    let vis = visible_count(kind, s.width, gap, n);
                    let max = n.saturating_sub(vis) as f32;
                    s.target = (s.target + dir).clamp(0., max);
                    cx.notify();
                })
            })
    }
}

fn visible_count(kind: CarouselKind, width: f32, gap: f32, _n: usize) -> usize {
    let base = slots(kind, width, gap);
    if kind == CarouselKind::Uncontained {
        (width / (base[0] + gap)).floor().max(1.) as usize
    } else {
        base.len()
    }
}

/// 1 項目分のスクロールに必要なマウス移動量
fn unit_px(kind: CarouselKind, width: f32, gap: f32) -> f32 {
    let base = slots(kind, width, gap);
    (base[0] + gap).max(40.)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAP: f32 = 8.;

    #[test]
    fn slots_fit_inside_the_viewport() {
        for kind in [
            CarouselKind::MultiBrowse,
            CarouselKind::Hero,
            CarouselKind::FullScreen,
        ] {
            for w in [400., 640., 972., 1400.] {
                let s = slots(kind, w, GAP);
                let total: f32 = s.iter().sum::<f32>() + GAP * (s.len() as f32 - 1.);
                assert!(total <= w + 0.5, "{kind:?} at {w}: {total}");
                assert!(
                    s.windows(2).all(|p| p[0] >= p[1] - 0.001),
                    "{kind:?}: not descending"
                );
            }
        }
    }

    #[test]
    fn widths_at_rest_follow_the_slots() {
        let base = slots(CarouselKind::MultiBrowse, 972., GAP);
        for (i, w) in base.iter().enumerate() {
            assert_eq!(item_width(CarouselKind::MultiBrowse, &base, i, 0.), *w);
        }
        // 範囲外の項目は幅 0
        assert_eq!(
            item_width(CarouselKind::MultiBrowse, &base, base.len(), 0.),
            0.
        );
    }

    #[test]
    fn scrolling_one_step_shifts_widths_by_one_slot() {
        let base = slots(CarouselKind::MultiBrowse, 972., GAP);
        // 位置 1 では、項目 1 が先頭スロット、項目 2 が 2 番目…になる
        assert_eq!(item_width(CarouselKind::MultiBrowse, &base, 0, 1.), 0.);
        assert_eq!(item_width(CarouselKind::MultiBrowse, &base, 1, 1.), base[0]);
        assert_eq!(item_width(CarouselKind::MultiBrowse, &base, 2, 1.), base[1]);
    }

    #[test]
    fn widths_change_continuously() {
        let base = slots(CarouselKind::MultiBrowse, 972., GAP);
        let mut prev = item_width(CarouselKind::MultiBrowse, &base, 1, 0.);
        for step in 1..=20 {
            let pos = step as f32 / 20.;
            let w = item_width(CarouselKind::MultiBrowse, &base, 1, pos);
            assert!(
                (w - prev).abs() < base[0] / 10.,
                "jump at {pos}: {prev} -> {w}"
            );
            prev = w;
        }
    }

    #[test]
    fn uncontained_items_keep_a_fixed_width() {
        let base = slots(CarouselKind::Uncontained, 900., GAP);
        for pos in [0., 0.5, 3.25] {
            assert_eq!(
                item_width(CarouselKind::Uncontained, &base, 4, pos),
                base[0]
            );
        }
    }

    #[test]
    fn visible_count_matches_the_layout() {
        assert_eq!(visible_count(CarouselKind::MultiBrowse, 972., GAP, 9), 3);
        assert_eq!(visible_count(CarouselKind::Hero, 972., GAP, 9), 2);
        assert_eq!(visible_count(CarouselKind::FullScreen, 972., GAP, 9), 1);
        assert!(visible_count(CarouselKind::Uncontained, 900., GAP, 9) >= 1);
    }
}
