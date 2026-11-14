//! SideSheet（標準 / モーダル）・FullScreenDialog・RichTooltip

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, AnyView, App, ClickEvent, Context, Div, ElementId, IntoElement, ParentElement,
    Render, RenderOnce, SharedString, SpringAnimation, Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, Button, ClickHandler, DismissFn, IconButton, IconName, StyledExt, TypeScale,
    dismiss_on_escape, shape, use_popup_focus,
};

// ---------------------------------------------------------------------------
// Side sheet
// ---------------------------------------------------------------------------

/// サイドシート。
///
/// - 標準（既定）: 本文の横に並び、開くと幅が広がって本文を押し出す
/// - モーダル（`.modal(true)`）: スクリムの上に右から重ねる。親のルート（`relative`）の最後の子として置く
///
/// 閉じるアニメーションのため、常に配置して `open` で開閉する。
#[derive(IntoElement)]
pub struct SideSheet {
    id: SharedString,
    open: bool,
    modal: bool,
    title: SharedString,
    width: f32,
    base: Div,
    actions: Vec<AnyElement>,
    on_close: Option<DismissFn>,
}

impl SideSheet {
    pub fn new(id: impl Into<SharedString>, open: bool, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            open,
            modal: false,
            title: title.into(),
            width: 320.,
            base: div().flex().flex_col().gap(px(12.)),
            actions: vec![],
            on_close: None,
        }
    }
    pub fn modal(mut self, v: bool) -> Self {
        self.modal = v;
        self
    }
    /// 幅（M3 では 256〜400）
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.clamp(256., 400.);
        self
    }
    /// 下部のボタン
    pub fn action(mut self, el: impl IntoElement) -> Self {
        self.actions.push(el.into_any_element());
        self
    }
    /// ✕ ボタン・（モーダルなら）スクリムと Esc で呼ばれる
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl ParentElement for SideSheet {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for SideSheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let id = self.id.clone();
        let width = self.width;
        let open = self.open;
        let modal = self.modal;
        let motion = theme.motion;
        let close = self.on_close.clone();

        let mut close_btn = IconButton::new(format!("{id}::close"), IconName::Close);
        if let Some(d) = close.clone() {
            close_btn = close_btn.on_click(move |_, w, cx| d(w, cx));
        }
        let header = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .pl(px(24.))
            .pr(px(12.))
            .h(px(72.))
            .flex_none()
            .child(
                div()
                    .flex_1()
                    .m3_type(TypeScale::TitleLarge)
                    .text_color(c.on_surface_variant)
                    .truncate()
                    .child(self.title),
            )
            .child(close_btn);
        let content = div()
            .id(ElementId::Name(format!("{id}::content").into()))
            .flex_1()
            .overflow_y_scroll()
            .px(px(24.))
            .py(px(8.))
            .text_color(c.on_surface)
            .child(self.base);
        let footer = (!self.actions.is_empty()).then(|| {
            div()
                .flex_none()
                .flex()
                .justify_end()
                .gap(px(8.))
                .px(px(24.))
                .py(px(16.))
                .children(self.actions)
        });
        let body = div()
            .w(px(width))
            .h_full()
            .flex()
            .flex_col()
            .child(header)
            .child(content)
            .children(footer);

        if !modal {
            // 標準: 幅をバネで動かして本文を押し出す
            let bg = c.surface_container_low;
            return div()
                .h_full()
                .flex_none()
                .overflow_hidden()
                .with_spring(
                    ElementId::Name(format!("{id}::width").into()),
                    SpringAnimation::new(motion.default_spatial)
                        .to(px(if open { width + 16. } else { 0. })),
                    move |el, w| {
                        el.w(w.max(px(0.))).child(
                            div()
                                .w(px(width))
                                .h_full()
                                .my(px(0.))
                                .mr(px(16.))
                                .rounded(shape::LARGE)
                                .overflow_hidden()
                                .bg(bg)
                                .child(body),
                        )
                    },
                )
                .into_any_element();
        }

        // モーダル
        let scrim = c.scrim;
        let panel_bg = c.surface_container_low;
        let shadow = c.shadow;
        let mut root = div()
            .id(ElementId::Name(format!("{id}::modal").into()))
            .absolute()
            .inset_0()
            .overflow_hidden();
        if let Some(d) = close.clone() {
            root = root.on_click(move |_, w, cx| d(w, cx));
        }
        if open {
            let focus = use_popup_focus(&id, window, cx);
            root = dismiss_on_escape(root, &focus, close);
        }
        let panel_id = ElementId::Name(format!("{id}::panel").into());
        root.with_spring(
            ElementId::Name(format!("{id}::open").into()),
            SpringAnimation::new(motion.default_spatial).to(if open { 1. } else { 0. }),
            move |el, t| {
                if !open && t < 0.01 {
                    return el.hidden();
                }
                let off = (1. - t) * (width + 40.);
                el.bg(scrim.opacity(0.32 * t.clamp(0., 1.))).child(
                    div()
                        .id(panel_id)
                        .occlude()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .right(px(-off))
                        .w(px(width))
                        .bg(panel_bg)
                        .rounded_tl(shape::EXTRA_LARGE)
                        .rounded_bl(shape::EXTRA_LARGE)
                        .m3_elevation(3, shadow)
                        .child(body),
                )
            },
        )
        .into_any_element()
    }
}

// ---------------------------------------------------------------------------
// Full-screen dialog
// ---------------------------------------------------------------------------

/// 全画面ダイアログ（上部に ✕・タイトル・保存などのアクション）。
/// 親のルート（`relative`）の最後の子として常に配置し、`open` で開閉する。
#[derive(IntoElement)]
pub struct FullScreenDialog {
    id: SharedString,
    open: bool,
    title: SharedString,
    base: Div,
    action: Option<(SharedString, ClickHandler)>,
    on_close: Option<DismissFn>,
}

impl FullScreenDialog {
    pub fn new(id: impl Into<SharedString>, open: bool, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            open,
            title: title.into(),
            base: div().flex().flex_col().gap(px(16.)),
            action: None,
            on_close: None,
        }
    }
    /// 右上のテキストボタン（保存など）
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Rc::new(f)));
        self
    }
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl ParentElement for FullScreenDialog {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for FullScreenDialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let id = self.id.clone();
        let open = self.open;
        let close = self.on_close.clone();

        let mut close_btn = IconButton::new(format!("{id}::close"), IconName::Close);
        if let Some(d) = close.clone() {
            close_btn = close_btn.on_click(move |_, w, cx| d(w, cx));
        }
        let mut header = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(56.))
            .px(px(8.))
            .child(close_btn)
            .child(
                div()
                    .flex_1()
                    .m3_type(TypeScale::TitleLarge)
                    .text_color(c.on_surface)
                    .truncate()
                    .child(self.title),
            );
        if let Some((label, handler)) = self.action {
            header = header.child(
                Button::new(format!("{id}::action"), label)
                    .on_click(move |e, w, cx| handler(e, w, cx)),
            );
        }
        let content = div()
            .id(ElementId::Name(format!("{id}::content").into()))
            .flex_1()
            .overflow_y_scroll()
            .px(px(24.))
            .py(px(16.))
            .text_color(c.on_surface)
            .child(self.base);

        let mut root = div()
            .id(ElementId::Name(format!("{id}::root").into()))
            .occlude()
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .bg(c.surface)
            .child(header)
            .child(content);
        if open {
            let focus = use_popup_focus(&id, window, cx);
            root = dismiss_on_escape(root, &focus, close);
        }
        root.with_spring(
            ElementId::Name(format!("{id}::open").into()),
            SpringAnimation::new(theme.motion.default_spatial).to(if open { 1. } else { 0. }),
            move |el, t| {
                if !open && t < 0.01 {
                    return el.hidden();
                }
                el.opacity(t.clamp(0., 1.)).top(px((1. - t) * 64.))
            },
        )
    }
}

// ---------------------------------------------------------------------------
// Rich tooltip
// ---------------------------------------------------------------------------

/// タイトル・本文・アクションボタンを持つツールチップ。マウスで中に入れる。
pub struct RichTooltip {
    title: Option<SharedString>,
    text: SharedString,
    action: Option<(SharedString, ClickHandler)>,
}

impl Render for RichTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let mut el = div()
            .font_family(theme.font_family)
            .flex()
            .flex_col()
            .gap(px(4.))
            .max_w(px(320.))
            .px(px(16.))
            .pt(px(12.))
            .pb(px(if self.action.is_some() { 8. } else { 12. }))
            .rounded(shape::MEDIUM)
            .bg(c.surface_container)
            .text_color(c.on_surface_variant)
            .m3_elevation(2, c.shadow);
        if let Some(t) = self.title.clone() {
            el = el.child(
                div()
                    .m3_type(TypeScale::TitleSmall)
                    .text_color(c.on_surface_variant)
                    .child(t),
            );
        }
        el = el.child(div().m3_type(TypeScale::BodyMedium).child(self.text.clone()));
        if let Some((label, handler)) = self.action.clone() {
            el = el.child(
                div().flex().justify_start().child(
                    Button::new("m3-rich-tooltip-action", label)
                        .text()
                        .on_click(move |e, w, cx| handler(e, w, cx)),
                ),
            );
        }
        el
    }
}

/// `.hoverable_tooltip(m3::rich_tooltip(..))` のように使う
pub fn rich_tooltip(
    title: Option<&'static str>,
    text: impl Into<SharedString>,
    action: Option<(&'static str, ClickHandler)>,
) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let text = text.into();
    move |_, cx| {
        let text = text.clone();
        let action = action
            .as_ref()
            .map(|(l, h)| (SharedString::from(*l), h.clone()));
        cx.new(|_| RichTooltip {
            title: title.map(SharedString::from),
            text,
            action,
        })
        .into()
    }
}
