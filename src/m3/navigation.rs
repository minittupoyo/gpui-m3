//! Navigation rail / Navigation bar（アクティブインジケーターがバネで伸びる）

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, SpringAnimation, Window,
    div, prelude::*, px,
};

use super::{
    ActiveTheme, Badge, ChangeHandler, Icon, IconName, StyledExt, Theme, TypeScale, state,
};

pub struct NavItem {
    pub icon: IconName,
    pub selected_icon: Option<IconName>,
    pub label: SharedString,
    pub badge: Option<Option<u32>>,
}

impl NavItem {
    pub fn new(icon: IconName, label: impl Into<SharedString>) -> Self {
        Self {
            icon,
            selected_icon: None,
            label: label.into(),
            badge: None,
        }
    }
    pub fn selected_icon(mut self, icon: IconName) -> Self {
        self.selected_icon = Some(icon);
        self
    }
    /// バッジ（None=小さな点, Some(n)=数字）
    pub fn badge(mut self, count: Option<u32>) -> Self {
        self.badge = Some(count);
        self
    }
}

fn nav_item(
    theme: &Theme,
    id: SharedString,
    item: NavItem,
    selected: bool,
    on_select: Option<ChangeHandler<usize>>,
    index: usize,
) -> AnyElement {
    let c = &theme.colors;
    let icon_color = if selected {
        c.on_secondary_container
    } else {
        c.on_surface_variant
    };
    let label_color = if selected {
        c.secondary
    } else {
        c.on_surface_variant
    };
    let indicator_w = px(56.);
    let group: SharedString = format!("{id}::group").into();
    let layer = c.on_surface.opacity(state::HOVER);
    let icon = match (selected, item.selected_icon) {
        (true, Some(i)) => i,
        _ => item.icon,
    };

    let pill = div()
        .absolute()
        .top_0()
        .h_full()
        .rounded_full()
        .bg(c.secondary_container)
        .with_spring(
            ElementId::Name(format!("{id}::pill").into()),
            SpringAnimation::new(theme.motion.fast_spatial).to(if selected {
                indicator_w
            } else {
                px(0.)
            }),
            move |el, w| {
                let w = w.max(px(0.));
                el.w(w).left((indicator_w - w) / 2.)
            },
        );

    let mut icon_box = div()
        .relative()
        .w(indicator_w)
        .h(px(32.))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .group_hover(group.clone(), move |s| s.bg(layer))
        .child(pill)
        .child(div().relative().child(Icon::new(icon).size(px(24.)).color(icon_color)));

    if let Some(badge) = item.badge {
        icon_box = icon_box.child(
            div()
                .absolute()
                .top(px(2.))
                .left(px(34.))
                .child(Badge::new(badge)),
        );
    }

    let mut el = div()
        .id(ElementId::Name(id))
        .group(group)
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.))
        .cursor_pointer()
        .child(icon_box)
        .child(
            div()
                .m3_type(if selected {
                    TypeScale::LabelMedium
                } else {
                    TypeScale::LabelMedium
                })
                .text_color(label_color)
                .child(item.label),
        );
    if let Some(h) = on_select {
        el = el.on_click(move |_, w, cx| h(index, w, cx));
    }
    el.into_any_element()
}

#[derive(IntoElement)]
pub struct NavigationRail {
    id: SharedString,
    items: Vec<NavItem>,
    selected: usize,
    header: Vec<AnyElement>,
    footer: Vec<AnyElement>,
    on_select: Option<ChangeHandler<usize>>,
}

impl NavigationRail {
    pub fn new(id: impl Into<SharedString>, selected: usize) -> Self {
        Self {
            id: id.into(),
            items: vec![],
            selected,
            header: vec![],
            footer: vec![],
            on_select: None,
        }
    }
    pub fn item(mut self, item: NavItem) -> Self {
        self.items.push(item);
        self
    }
    /// 上部（メニューボタンや FAB）
    pub fn header(mut self, el: impl IntoElement) -> Self {
        self.header.push(el.into_any_element());
        self
    }
    /// 下部
    pub fn footer(mut self, el: impl IntoElement) -> Self {
        self.footer.push(el.into_any_element());
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for NavigationRail {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let mut items = div().flex().flex_col().items_center().gap(px(12.));
        for (i, item) in self.items.into_iter().enumerate() {
            items = items.child(nav_item(
                &theme,
                format!("{}-{i}", self.id).into(),
                item,
                i == self.selected,
                self.on_select.clone(),
                i,
            ));
        }
        div()
            .w(px(96.))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .py(px(16.))
            .gap(px(24.))
            .bg(theme.colors.surface)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(16.))
                    .children(self.header),
            )
            .child(items)
            .child(div().flex_1())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(8.))
                    .children(self.footer),
            )
    }
}

#[derive(IntoElement)]
pub struct NavigationBar {
    id: SharedString,
    items: Vec<NavItem>,
    selected: usize,
    on_select: Option<ChangeHandler<usize>>,
}

impl NavigationBar {
    pub fn new(id: impl Into<SharedString>, selected: usize) -> Self {
        Self {
            id: id.into(),
            items: vec![],
            selected,
            on_select: None,
        }
    }
    pub fn item(mut self, item: NavItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for NavigationBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let mut bar = div()
            .w_full()
            .h(px(80.))
            .flex()
            .items_center()
            .justify_around()
            .bg(theme.colors.surface_container);
        for (i, item) in self.items.into_iter().enumerate() {
            bar = bar.child(div().flex_1().flex().justify_center().child(nav_item(
                &theme,
                format!("{}-{i}", self.id).into(),
                item,
                i == self.selected,
                self.on_select.clone(),
                i,
            )));
        }
        bar
    }
}
