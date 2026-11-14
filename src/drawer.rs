//! Navigation drawer（標準 / モーダル）
//!
//! - 標準: 本文の横に並び、開くと幅が広がって本文を押し出す
//! - モーダル（`.modal(true)`）: スクリムの上に左から重ねる。親のルート（`relative`）の最後の子として置く
//!
//! 閉じるアニメーションのため、常に配置して `open` で開閉する。

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, SpringAnimation, Window,
    div, prelude::*, px,
};

use super::{
    ActiveTheme, ChangeHandler, DismissFn, Divider, Icon, NavItem, StyledExt, TypeScale,
    dismiss_on_escape, shape, state, use_popup_focus,
};

enum Entry {
    Item(NavItem),
    Section(SharedString),
    Divider,
}

#[derive(IntoElement)]
pub struct NavigationDrawer {
    id: SharedString,
    open: bool,
    modal: bool,
    width: f32,
    headline: Option<SharedString>,
    entries: Vec<Entry>,
    selected: usize,
    on_select: Option<ChangeHandler<usize>>,
    on_close: Option<DismissFn>,
}

impl NavigationDrawer {
    pub fn new(id: impl Into<SharedString>, open: bool) -> Self {
        Self {
            id: id.into(),
            open,
            modal: false,
            width: 360.,
            headline: None,
            entries: vec![],
            selected: 0,
            on_select: None,
            on_close: None,
        }
    }
    pub fn modal(mut self, v: bool) -> Self {
        self.modal = v;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    pub fn headline(mut self, text: impl Into<SharedString>) -> Self {
        self.headline = Some(text.into());
        self
    }
    /// 項目（選択インデックスは項目だけを数える）
    pub fn item(mut self, item: NavItem) -> Self {
        self.entries.push(Entry::Item(item));
        self
    }
    /// セクション見出し
    pub fn section(mut self, title: impl Into<SharedString>) -> Self {
        self.entries.push(Entry::Section(title.into()));
        self
    }
    pub fn divider(mut self) -> Self {
        self.entries.push(Entry::Divider);
        self
    }
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for NavigationDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let id = self.id.clone();
        let (open, modal, width) = (self.open, self.modal, self.width);
        let motion = theme.motion;
        let close = self.on_close.clone();

        let mut list = div()
            .id(ElementId::Name(format!("{id}::list").into()))
            .flex_1()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .px(px(12.))
            .pb(px(12.))
            .text_color(c.on_surface_variant);
        if let Some(h) = self.headline {
            list = list.child(
                div()
                    .h(px(56.))
                    .flex_none()
                    .flex()
                    .items_center()
                    .px(px(16.))
                    .m3_type(TypeScale::TitleSmall)
                    .text_color(c.on_surface_variant)
                    .child(h),
            );
        }
        let mut index = 0usize;
        for (n, entry) in self.entries.into_iter().enumerate() {
            match entry {
                Entry::Section(t) => {
                    list = list.child(
                        div()
                            .h(px(56.))
                            .flex_none()
                            .flex()
                            .items_center()
                            .px(px(16.))
                            .m3_type(TypeScale::TitleSmall)
                            .child(t),
                    )
                }
                Entry::Divider => {
                    list = list.child(div().py(px(8.)).px(px(4.)).child(Divider::horizontal()))
                }
                Entry::Item(item) => {
                    let i = index;
                    index += 1;
                    let selected = i == self.selected;
                    let fg = if selected {
                        c.on_secondary_container
                    } else {
                        c.on_surface_variant
                    };
                    let icon = if selected {
                        item.selected_icon.unwrap_or(item.icon)
                    } else {
                        item.icon
                    };
                    let on_select = self.on_select.clone();
                    let close = close.clone();
                    let hover = fg.opacity(state::HOVER);
                    let mut row = div()
                        .id(ElementId::Name(format!("{id}::item{n}").into()))
                        .flex_none()
                        .h(px(56.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .pl(px(16.))
                        .pr(px(24.))
                        .rounded(shape::FULL)
                        .text_color(fg)
                        .m3_type(TypeScale::LabelLarge)
                        .cursor_pointer()
                        .when(selected, |d| d.bg(c.secondary_container))
                        .hover(move |s| s.bg(hover))
                        .child(Icon::new(icon).size(px(24.)).color(fg))
                        .child(div().flex_1().truncate().child(item.label))
                        .on_click(move |_, w, cx| {
                            if let Some(h) = &on_select {
                                h(i, w, cx);
                            }
                            if modal {
                                if let Some(d) = &close {
                                    d(w, cx);
                                }
                            }
                        });
                    if let Some(count) = item.badge {
                        row = row.child(
                            div()
                                .m3_type(TypeScale::LabelLarge)
                                .child(count.map(|n| n.to_string()).unwrap_or_default()),
                        );
                    }
                    list = list.child(row);
                }
            }
        }

        let body = div()
            .w(px(width))
            .h_full()
            .flex()
            .flex_col()
            .pt(px(12.))
            .child(list);

        if !modal {
            let bg = c.surface_container_low;
            return div()
                .h_full()
                .flex_none()
                .overflow_hidden()
                .with_spring(
                    ElementId::Name(format!("{id}::width").into()),
                    SpringAnimation::new(motion.default_spatial).to(px(if open {
                        width
                    } else {
                        0.
                    })),
                    move |el, w| {
                        el.w(w.max(px(0.))).child(
                            div()
                                .w(px(width))
                                .h_full()
                                .rounded_tr(shape::LARGE)
                                .rounded_br(shape::LARGE)
                                .overflow_hidden()
                                .bg(bg)
                                .child(body),
                        )
                    },
                )
                .into_any_element();
        }

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
                        .left(px(-off))
                        .w(px(width))
                        .bg(panel_bg)
                        .rounded_tr(shape::EXTRA_LARGE)
                        .rounded_br(shape::EXTRA_LARGE)
                        .m3_elevation(3, shadow)
                        .child(body),
                )
            },
        )
        .into_any_element()
    }
}

#[allow(dead_code)]
fn _unused(_: AnyElement) {}
