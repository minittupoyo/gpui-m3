//! Modal bottom sheet
//!
//! 画面ルートの最後の子として常に配置し、`open` で開閉する（閉じるアニメーションのため）。
//! ドラッグハンドルを下に引っ張ると閉じる。

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, App, DispatchPhase, Div, ElementId, IntoElement, MouseButton, MouseMoveEvent,
    MouseUpEvent, ParentElement, RenderOnce, SharedString, SpringAnimation, StyleRefinement,
    Styled, Window, canvas, div, prelude::*, px,
};

use super::{ActiveTheme, StyledExt, dismiss_on_escape, shape, use_popup_focus};

#[derive(Default)]
struct SheetState {
    /// ドラッグ開始時のマウス Y
    drag_start: Option<f32>,
    /// 現在のドラッグ量（下方向が正）
    drag: f32,
    height: f32,
}

#[derive(IntoElement)]
pub struct BottomSheet {
    id: SharedString,
    open: bool,
    base: Div,
    max_width: f32,
    on_dismiss: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

impl BottomSheet {
    pub fn new(id: impl Into<SharedString>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            base: div(),
            max_width: 640.,
            on_dismiss: None,
        }
    }
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }
    /// スクリムのクリック・下方向ドラッグで呼ばれる
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl Styled for BottomSheet {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for BottomSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for BottomSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let state = window.use_keyed_state(
            SharedString::from(format!("{}::sheet", self.id)),
            cx,
            |_, _| SheetState::default(),
        );
        let (dragging, drag, height) = {
            let s = state.read(cx);
            (s.drag_start.is_some(), s.drag, s.height.max(200.))
        };
        let open = self.open;
        let on_dismiss = self.on_dismiss.clone();
        let id = self.id.clone();
        let motion = theme.motion;
        let base = self.base;
        let max_w = self.max_width;

        // ドラッグハンドル
        let handle_state = state.clone();
        let handle = div()
            .id(ElementId::Name(format!("{id}::handle").into()))
            .w_full()
            .h(px(36.))
            .flex()
            .items_center()
            .justify_center()
            .cursor_grab()
            .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                let y = f32::from(e.position.y);
                handle_state.update(cx, |s, cx| {
                    s.drag_start = Some(y);
                    s.drag = 0.;
                    cx.notify();
                })
            })
            .child(
                div()
                    .w(px(32.))
                    .h(px(4.))
                    .rounded_full()
                    .bg(c.on_surface_variant.opacity(0.4)),
            );

        // 高さ計測 + ドラッグ中のグローバルマウス追跡
        let measure = state.clone();
        let track = state.clone();
        let dismiss_on_drag = on_dismiss.clone();
        let tracker = canvas(
            move |b, _, cx| {
                let h = f32::from(b.size.height);
                measure.update(cx, |s, cx| {
                    if (s.height - h).abs() > 0.5 {
                        s.height = h;
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
                        if let Some(start) = s.drag_start {
                            s.drag = (f32::from(e.position.y) - start).max(-24.);
                            cx.notify();
                        }
                    })
                });
                let st = track.clone();
                let dismiss = dismiss_on_drag.clone();
                window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    let mut close = false;
                    st.update(cx, |s, cx| {
                        if s.drag_start.take().is_some() {
                            close = s.drag > (s.height * 0.3).min(160.);
                            s.drag = 0.;
                            cx.notify();
                        }
                    });
                    if close {
                        if let Some(d) = dismiss.as_ref() {
                            d(window, cx);
                        }
                    }
                });
            },
        )
        .absolute()
        .size_full();

        let sheet = div()
            .id(ElementId::Name(format!("{id}::body").into()))
            .occlude()
            .relative()
            .w_full()
            .max_w(px(max_w))
            .flex()
            .flex_col()
            .pb(px(24.))
            .rounded_t(shape::EXTRA_LARGE)
            .bg(c.surface_container_low)
            .text_color(c.on_surface)
            .m3_elevation(1, c.shadow)
            .child(tracker)
            .child(handle)
            .child(base.w_full());

        // ドラッグ量（離すとバネで戻る）
        let drag_spring = if dragging {
            motion.fast_effects
        } else {
            motion.default_spatial
        };
        let sheet = sheet.with_spring(
            ElementId::Name(format!("{id}::drag").into()),
            SpringAnimation::new(drag_spring).to(if open { drag } else { 0. }),
            |el, d| el.mb(px(-d.max(-24.))),
        );

        let scrim_dismiss = on_dismiss.clone();
        let root = div()
            .id(ElementId::Name(format!("{id}::root").into()))
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .justify_end()
            .items_center()
            .overflow_hidden()
            .on_click(move |_, w, cx| {
                if let Some(d) = scrim_dismiss.as_ref() {
                    d(w, cx);
                }
            })
            .child(sheet);
        // 開いている間だけフォーカスを持ち、Esc で閉じる
        let root = if open {
            let focus = use_popup_focus(&id, window, cx);
            dismiss_on_escape(root, &focus, on_dismiss.clone())
        } else {
            root
        };
        root.with_spring(
                ElementId::Name(format!("{id}::open").into()),
                SpringAnimation::new(motion.default_spatial).to(if open { 1. } else { 0. }),
                move |el, t| {
                    if !open && t < 0.01 {
                        return el.hidden();
                    }
                    el.bg(c.scrim.opacity(0.32 * t.clamp(0., 1.)))
                        .pb(px(-(1. - t) * (height + 40.)))
                },
            )
    }
}
