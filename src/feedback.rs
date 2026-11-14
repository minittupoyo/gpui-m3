//! Badge / Divider / Tooltip / Snackbar / Dialog / ListItem

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, AnyView, App, Context, ElementId, IntoElement, Render, RenderOnce, SharedString,
    SpringAnimation, Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, ClickHandler, DismissFn, Icon, IconName, StyledExt, TypeScale, dismiss_on_escape,
    ripple_overlay, shape, state, state_layer, track_ripple, use_popup_focus, use_ripple,
};

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Badge {
    count: Option<u32>,
}

impl Badge {
    /// None で小さな点、Some(n) で数字（999+ まで）
    pub fn new(count: Option<u32>) -> Self {
        Self { count }
    }
}

impl RenderOnce for Badge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let c = &cx.m3().colors;
        match self.count {
            None => div().size(px(6.)).rounded_full().bg(c.error),
            Some(n) => div()
                .h(px(16.))
                .min_w(px(16.))
                .px(px(4.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(c.error)
                .text_color(c.on_error)
                .m3_type(TypeScale::LabelSmall)
                .child(if n > 999 {
                    "999+".to_string()
                } else {
                    n.to_string()
                }),
        }
    }
}

// ---------------------------------------------------------------------------
// Divider
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Divider {
    vertical: bool,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self { vertical: false }
    }
    pub fn vertical() -> Self {
        Self { vertical: true }
    }
}

impl RenderOnce for Divider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = cx.m3().colors.outline_variant;
        if self.vertical {
            div().w(px(1.)).h_full().bg(color)
        } else {
            div().h(px(1.)).w_full().bg(color)
        }
    }
}

// ---------------------------------------------------------------------------
// Tooltip
// ---------------------------------------------------------------------------

pub struct PlainTooltip {
    text: SharedString,
}

impl Render for PlainTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = &cx.m3().colors;
        div()
            .font_family(cx.m3().font_family)
            .px(px(8.))
            .py(px(4.))
            .rounded(shape::EXTRA_SMALL)
            .bg(c.inverse_surface)
            .text_color(c.inverse_on_surface)
            .m3_type(TypeScale::BodySmall)
            .child(self.text.clone())
    }
}

/// `.tooltip(gpui_m3::tooltip("説明"))` のように使う
pub fn tooltip(text: impl Into<SharedString>) -> impl Fn(&mut Window, &mut App) -> AnyView {
    let text = text.into();
    move |_, cx| cx.new(|_| PlainTooltip { text: text.clone() }).into()
}

// ---------------------------------------------------------------------------
// Snackbar
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Snackbar {
    message: SharedString,
    action: Option<(SharedString, ClickHandler)>,
    on_close: Option<ClickHandler>,
}

impl Snackbar {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            action: None,
            on_close: None,
        }
    }
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        f: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), std::rc::Rc::new(f)));
        self
    }
    pub fn on_close(
        mut self,
        f: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for Snackbar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let mut el = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .min_h(px(48.))
            .min_w(px(320.))
            .pl(px(16.))
            .pr(px(8.))
            .rounded(shape::EXTRA_SMALL)
            .bg(c.inverse_surface)
            .text_color(c.inverse_on_surface)
            .m3_elevation(3, c.shadow)
            .child(
                div()
                    .flex_1()
                    .py(px(14.))
                    .m3_type(TypeScale::BodyMedium)
                    .child(self.message),
            );
        if let Some((label, h)) = self.action {
            let fg = c.inverse_primary;
            el = el.child(
                div()
                    .id("snackbar-action")
                    .px(px(12.))
                    .h(px(40.))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .cursor_pointer()
                    .text_color(fg)
                    .m3_type(TypeScale::LabelLarge)
                    .hover(move |s| s.bg(fg.opacity(state::HOVER)))
                    .on_click(move |e, w, cx| h(e, w, cx))
                    .child(label),
            );
        }
        if let Some(h) = self.on_close {
            let fg = c.inverse_on_surface;
            el = el.child(
                div()
                    .id("snackbar-close")
                    .size(px(40.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .cursor_pointer()
                    .hover(move |s| s.bg(fg.opacity(state::HOVER)))
                    .on_click(move |e, w, cx| h(e, w, cx))
                    .child(Icon::new(IconName::Close).size(px(24.)).color(fg)),
            );
        }
        el
    }
}

// ---------------------------------------------------------------------------
// Dialog
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Dialog {
    icon: Option<IconName>,
    headline: SharedString,
    body: SharedString,
    actions: Vec<AnyElement>,
    on_dismiss: Option<DismissFn>,
}

impl Dialog {
    pub fn new(headline: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self {
            icon: None,
            headline: headline.into(),
            body: body.into(),
            actions: vec![],
            on_dismiss: None,
        }
    }
    /// Esc キーで閉じるときに呼ばれる
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(std::rc::Rc::new(f));
        self
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn action(mut self, el: impl IntoElement) -> Self {
        self.actions.push(el.into_any_element());
        self
    }
}

impl RenderOnce for Dialog {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let centered = self.icon.is_some();
        // 開いたらフォーカスを持ち、Esc で閉じる
        let focus = use_popup_focus(&"m3-dialog".into(), window, cx);
        let mut el = dismiss_on_escape(div().id("m3-dialog"), &focus, self.on_dismiss.clone())
            .occlude()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(24.))
            .min_w(px(280.))
            .max_w(px(560.))
            .rounded(shape::EXTRA_LARGE)
            .bg(c.surface_container_high)
            .m3_elevation(3, c.shadow);
        if let Some(icon) = self.icon {
            el = el.child(
                div()
                    .flex()
                    .justify_center()
                    .child(Icon::new(icon).size(px(24.)).color(c.secondary)),
            );
        }
        let mut headline = div()
            .m3_type(TypeScale::HeadlineSmall)
            .text_color(c.on_surface)
            .child(self.headline);
        if centered {
            headline = headline.text_center();
        }
        el.child(headline)
            .child(
                div()
                    .m3_type(TypeScale::BodyMedium)
                    .text_color(c.on_surface_variant)
                    .child(self.body),
            )
            .child(
                div()
                    .pt(px(8.))
                    .flex()
                    .justify_end()
                    .gap(px(8.))
                    .children(self.actions),
            )
    }
}

// ---------------------------------------------------------------------------
// Dialog overlay / Snackbar host（出入りのアニメーション付き）
// ---------------------------------------------------------------------------

/// ダイアログをスクリム付きで画面中央に出す。閉じるアニメーションのため
/// 画面ルートの最後の子として**常に配置**し、`open` で開閉する。
/// スクリムのクリックと Esc で閉じる。
#[derive(IntoElement)]
pub struct DialogOverlay {
    id: SharedString,
    open: bool,
    dialog: Dialog,
    on_dismiss: Option<DismissFn>,
}

impl DialogOverlay {
    pub fn new(id: impl Into<SharedString>, open: bool, dialog: Dialog) -> Self {
        Self {
            id: id.into(),
            open,
            dialog,
            on_dismiss: None,
        }
    }
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for DialogOverlay {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let scrim = theme.colors.scrim;
        let open = self.open;
        let id = self.id;
        let dismiss = self.on_dismiss.clone();
        // Esc も同じ処理で閉じる
        let mut dialog = self.dialog;
        if let Some(d) = self.on_dismiss {
            dialog = dialog.on_dismiss(move |w, cx| d(w, cx));
        }

        let mut root = div()
            .id(ElementId::Name(format!("{id}::scrim").into()))
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center();
        if let Some(d) = dismiss {
            root = root.on_click(move |_, w, cx| d(w, cx));
        }
        root.with_spring(
            ElementId::Name(format!("{id}::open").into()),
            SpringAnimation::new(theme.motion.default_spatial).to(if open { 1. } else { 0. }),
            move |el, t| {
                if !open && t < 0.01 {
                    return el.hidden();
                }
                let tc = t.clamp(0., 1.);
                el.bg(scrim.opacity(0.32 * tc)).child(
                    div()
                        .opacity(tc)
                        .relative()
                        .top(px((1. - t) * 28.))
                        .child(dialog),
                )
            },
        )
    }
}

/// スナックバーの出入り（下からせり上がってフェードイン）。
/// 位置は親で決め、`Snackbar` をこのホストで包んで常に配置する。
#[derive(IntoElement)]
pub struct SnackbarHost {
    id: SharedString,
    visible: bool,
    snackbar: Snackbar,
}

impl SnackbarHost {
    pub fn new(id: impl Into<SharedString>, visible: bool, snackbar: Snackbar) -> Self {
        Self {
            id: id.into(),
            visible,
            snackbar,
        }
    }
}

impl RenderOnce for SnackbarHost {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let visible = self.visible;
        div().relative().child(self.snackbar).with_spring(
            ElementId::Name(format!("{}::visible", self.id).into()),
            SpringAnimation::new(theme.motion.default_spatial).to(if visible { 1. } else { 0. }),
            move |el, t| {
                if !visible && t < 0.01 {
                    return el.hidden();
                }
                el.opacity(t.clamp(0., 1.)).top(px((1. - t) * 40.))
            },
        )
    }
}
// ---------------------------------------------------------------------------
// List item
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct ListItem {
    id: SharedString,
    headline: SharedString,
    supporting: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    on_click: Option<ClickHandler>,
    corners: gpui::Corners<gpui::Pixels>,
}

impl ListItem {
    pub fn new(id: impl Into<SharedString>, headline: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            headline: headline.into(),
            supporting: None,
            leading: None,
            trailing: None,
            on_click: None,
            corners: gpui::Corners::all(px(0.)),
        }
    }
    /// 親コンテナの角丸に合わせる（ホバー・波紋がはみ出さないように）。
    /// 先頭の項目は上側、末尾の項目は下側の角にコンテナの半径を指定する
    pub fn corners(mut self, corners: gpui::Corners<gpui::Pixels>) -> Self {
        self.corners = corners;
        self
    }
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    pub fn leading(mut self, el: impl IntoElement) -> Self {
        self.leading = Some(el.into_any_element());
        self
    }
    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing = Some(el.into_any_element());
        self
    }
    pub fn on_click(
        mut self,
        f: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let bg = gpui::hsla(0., 0., 0., 0.);
        let fg = c.on_surface;
        let ripple = self
            .on_click
            .is_some()
            .then(|| use_ripple(&self.id, window, cx));
        let mut el = div()
            .id(ElementId::Name(self.id))
            .flex()
            .items_center()
            .gap(px(16.))
            .px(px(16.))
            .py(px(8.))
            .rounded_tl(self.corners.top_left)
            .rounded_tr(self.corners.top_right)
            .rounded_br(self.corners.bottom_right)
            .rounded_bl(self.corners.bottom_left)
            .min_h(theme.density.shrink_by(
                px(if self.supporting.is_some() { 72. } else { 56. }),
                4.,
                40.,
            ));
        if let Some(r) = ripple.as_ref() {
            el = el.child(ripple_overlay(r, fg, self.corners));
            el = track_ripple(el, r);
        }
        if let Some(l) = self.leading {
            el = el.child(l);
        }
        let mut text = div().flex_1().flex().flex_col().child(
            div()
                .m3_type(TypeScale::BodyLarge)
                .text_color(c.on_surface)
                .child(self.headline),
        );
        if let Some(s) = self.supporting {
            text = text.child(
                div()
                    .m3_type(TypeScale::BodyMedium)
                    .text_color(c.on_surface_variant)
                    .child(s),
            );
        }
        el = el.child(text);
        if let Some(t) = self.trailing {
            el = el.child(t);
        }
        if let Some(h) = self.on_click {
            el = el
                .cursor_pointer()
                .hover(move |s| s.bg(state_layer(bg, fg, state::HOVER)))
                .on_click(move |e, w, cx| h(e, w, cx));
        }
        el
    }
}
