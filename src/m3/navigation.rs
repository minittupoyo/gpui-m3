//! Navigation rail / Navigation bar
//!
//! [`NavigationRail`] は M3 Expressive の 3 つの状態を持つ。
//! - 閉じた（collapsed）: 幅 96px、アイコンの下にラベル
//! - 展開（expanded）: 幅が広がり、ラベルが横に並ぶ。本文を押し出す
//! - モーダル展開（modal）: スクリムを敷いて、左から重ねて表示する
//!
//! 開閉はバネで動く。`on_toggle` を指定すると上部にメニューボタンが付く。

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, App, ElementId, IntoElement, RenderOnce, SharedString, SpringAnimation, Window,
    div, prelude::*, px,
};

use super::{
    ActiveTheme, Badge, ChangeHandler, ClickHandler, Fab, FabColor, Icon, IconButton, IconName,
    StyledExt, Theme, TypeScale, shape, state,
};

type VoidHandler = Rc<dyn Fn(&mut Window, &mut App)>;

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

/// 閉じたレール / ナビゲーションバー用（アイコンの下にラベル）
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
                .m3_type(TypeScale::LabelMedium)
                .text_color(label_color)
                .child(item.label),
        );
    if let Some(h) = on_select {
        el = el.on_click(move |_, w, cx| h(index, w, cx));
    }
    el.into_any_element()
}

/// 展開したレール用（高さ 56px のピル。アイコンとラベルが横並び）
fn nav_item_expanded(
    theme: &Theme,
    id: String,
    item: NavItem,
    selected: bool,
    on_select: Option<ChangeHandler<usize>>,
    index: usize,
) -> AnyElement {
    let c = &theme.colors;
    let fg = if selected {
        c.on_secondary_container
    } else {
        c.on_surface_variant
    };
    let group: SharedString = format!("{id}::group").into();
    let layer = c.on_surface.opacity(state::HOVER);
    let icon = match (selected, item.selected_icon) {
        (true, Some(i)) => i,
        _ => item.icon,
    };

    // 選択インジケーター（フェードイン）
    let indicator = div()
        .absolute()
        .inset_0()
        .rounded_full()
        .bg(c.secondary_container)
        .with_spring(
            ElementId::Name(format!("{id}::pillx").into()),
            SpringAnimation::new(theme.motion.fast_effects).to(if selected { 1. } else { 0. }),
            |el, t| el.opacity(t.clamp(0., 1.)),
        );
    let hover = div()
        .absolute()
        .inset_0()
        .rounded_full()
        .group_hover(group.clone(), move |s| s.bg(layer));

    let mut row = div()
        .relative()
        .w_full()
        .h(px(56.))
        .flex()
        .items_center()
        .gap(px(12.))
        .pl(px(16.))
        .pr(px(24.))
        .rounded_full()
        .child(indicator)
        .child(hover)
        .child(div().relative().child(Icon::new(icon).size(px(24.)).color(fg)))
        .child(
            div()
                .relative()
                .flex_1()
                .whitespace_nowrap()
                .m3_type(TypeScale::LabelLarge)
                .text_color(fg)
                .child(item.label),
        );
    if let Some(badge) = item.badge {
        row = row.child(div().relative().child(Badge::new(badge)));
    }

    let mut el = div()
        .id(ElementId::Name(id.into()))
        .group(group)
        .w_full()
        .px(px(20.))
        .cursor_pointer()
        .child(row);
    if let Some(h) = on_select {
        el = el.on_click(move |_, w, cx| h(index, w, cx));
    }
    el.into_any_element()
}

// ---------------------------------------------------------------------------
// Navigation rail
// ---------------------------------------------------------------------------

enum NavEntry {
    Item(NavItem),
    Section(SharedString),
}

/// 閉じたときの幅
const COLLAPSED_W: f32 = 96.;

struct RailParts {
    id: SharedString,
    entries: Vec<NavEntry>,
    selected: usize,
    header: Vec<AnyElement>,
    footer: Vec<AnyElement>,
    on_select: Option<ChangeHandler<usize>>,
    on_toggle: Option<VoidHandler>,
    toggle_icon: IconName,
    fab: Option<(IconName, SharedString, Option<ClickHandler>)>,
    expanded_width: f32,
}

/// レール本体。`exp` が true なら展開レイアウト（ラベル横並び）
fn rail_body(theme: &Theme, p: RailParts, exp: bool) -> AnyElement {
    let RailParts {
        id,
        entries,
        selected,
        header,
        footer,
        on_select,
        on_toggle,
        toggle_icon,
        fab,
        expanded_width,
    } = p;
    let c = &theme.colors;
    let rail_w = if exp { expanded_width } else { COLLAPSED_W };
    // 左端を固定したまま幅だけ変わるように、要素は常に左から配置する
    let wrap = |el: AnyElement, pl: f32| -> AnyElement {
        div().w_full().flex().pl(px(pl)).child(el).into_any_element()
    };

    let mut col = div()
        .w(px(rail_w))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .gap(px(16.))
        .py(px(16.));

    if let Some(toggle) = on_toggle {
        col = col.child(wrap(
            IconButton::new(format!("{id}::toggle"), toggle_icon)
                .on_click(move |_, w, cx| toggle(w, cx))
                .into_any_element(),
            28.,
        ));
    }
    if let Some((icon, label, handler)) = fab {
        let mut f = Fab::new(format!("{id}::fab"), icon)
            .color(FabColor::TertiaryContainer)
            .lowered(true);
        if exp {
            f = f.label(label);
        }
        if let Some(h) = handler {
            f = f.on_click(move |e, w, cx| h(e, w, cx));
        }
        col = col.child(wrap(f.into_any_element(), 20.));
    }
    for el in header {
        col = col.child(wrap(el, 28.));
    }

    let mut list = div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(if exp { 4. } else { 12. }));
    let mut index = 0;
    for entry in entries {
        match entry {
            NavEntry::Item(item) => {
                let i = index;
                index += 1;
                let item_id = format!("{id}-{i}");
                list = list.child(if exp {
                    nav_item_expanded(theme, item_id, item, i == selected, on_select.clone(), i)
                } else {
                    div()
                        .w_full()
                        .flex()
                        .justify_center()
                        .child(nav_item(
                            theme,
                            item_id.into(),
                            item,
                            i == selected,
                            on_select.clone(),
                            i,
                        ))
                        .into_any_element()
                });
            }
            NavEntry::Section(title) => {
                list = list.child(if exp {
                    div()
                        .h(px(48.))
                        .pl(px(36.))
                        .flex()
                        .items_end()
                        .pb(px(8.))
                        .m3_type(TypeScale::TitleSmall)
                        .text_color(c.on_surface_variant)
                        .child(title)
                        .into_any_element()
                } else {
                    div().h(px(4.)).into_any_element()
                });
            }
        }
    }
    col = col.child(list).child(div().flex_1());
    for el in footer {
        col = col.child(wrap(el, 28.));
    }
    col.into_any_element()
}

#[derive(IntoElement)]
pub struct NavigationRail {
    id: SharedString,
    entries: Vec<NavEntry>,
    selected: usize,
    header: Vec<AnyElement>,
    footer: Vec<AnyElement>,
    on_select: Option<ChangeHandler<usize>>,
    expanded: bool,
    modal: bool,
    expanded_width: f32,
    on_toggle: Option<VoidHandler>,
    on_dismiss: Option<VoidHandler>,
    fab: Option<(IconName, SharedString, Option<ClickHandler>)>,
}

impl NavigationRail {
    pub fn new(id: impl Into<SharedString>, selected: usize) -> Self {
        Self {
            id: id.into(),
            entries: vec![],
            selected,
            header: vec![],
            footer: vec![],
            on_select: None,
            expanded: false,
            modal: false,
            expanded_width: 280.,
            on_toggle: None,
            on_dismiss: None,
            fab: None,
        }
    }
    pub fn item(mut self, item: NavItem) -> Self {
        self.entries.push(NavEntry::Item(item));
        self
    }
    /// セクション見出し（展開時のみ表示）
    pub fn section(mut self, title: impl Into<SharedString>) -> Self {
        self.entries.push(NavEntry::Section(title.into()));
        self
    }
    /// 上部の追加要素（幅 40px 前後を想定）
    pub fn header(mut self, el: impl IntoElement) -> Self {
        self.header.push(el.into_any_element());
        self
    }
    /// 下部の追加要素（幅 40px 前後を想定）
    pub fn footer(mut self, el: impl IntoElement) -> Self {
        self.footer.push(el.into_any_element());
        self
    }
    /// 展開しているか。モーダルでは「開いているか」
    pub fn expanded(mut self, v: bool) -> Self {
        self.expanded = v;
        self
    }
    /// モーダル展開型にする。親のルート（`relative` な要素）の最後の子として置く
    pub fn modal(mut self, v: bool) -> Self {
        self.modal = v;
        self
    }
    /// 展開時の幅（M3 では 220〜360）
    pub fn expanded_width(mut self, w: f32) -> Self {
        self.expanded_width = w.clamp(220., 360.);
        self
    }
    /// 上部にメニューボタンを出し、押したときに呼ばれる（開閉の切り替え用）
    pub fn on_toggle(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
    /// モーダルのスクリム・項目選択・メニューボタンで閉じるときに呼ばれる
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
    /// レール内の FAB。展開すると Extended FAB（ラベル付き）に変わる
    pub fn fab(
        mut self,
        icon: IconName,
        label: impl Into<SharedString>,
        on_click: Option<ClickHandler>,
    ) -> Self {
        self.fab = Some((icon, label.into(), on_click));
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for NavigationRail {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let id = self.id.clone();
        let expanded = self.expanded;
        let modal = self.modal;
        let width = self.expanded_width;

        // モーダルでは項目を選んだら閉じる。メニューボタンも閉じる動作にする
        let mut on_select = self.on_select.clone();
        let mut on_toggle = self.on_toggle.clone();
        if modal {
            if let Some(dismiss) = self.on_dismiss.clone() {
                let inner = on_select.take();
                let d = dismiss.clone();
                on_select = Some(Rc::new(move |i, w, cx| {
                    if let Some(s) = inner.as_ref() {
                        s(i, w, cx);
                    }
                    d(w, cx);
                }));
                on_toggle = on_toggle.or(Some(dismiss));
            }
        }

        let parts = RailParts {
            id: id.clone(),
            entries: self.entries,
            selected: self.selected,
            header: self.header,
            footer: self.footer,
            on_select,
            on_toggle,
            toggle_icon: if expanded {
                IconName::MenuOpen
            } else {
                IconName::Menu
            },
            fab: self.fab,
            expanded_width: width,
        };

        if !modal {
            // 幅をバネで動かす。幅が半分を超えたら展開レイアウトに切り替える
            let spring = SpringAnimation::new(theme.motion.default_spatial)
                .to(px(if expanded { width } else { COLLAPSED_W }));
            let bg = theme.colors.surface;
            return div()
                .h_full()
                .flex_none()
                .overflow_hidden()
                .bg(bg)
                .with_spring(
                    ElementId::Name(format!("{id}::width").into()),
                    spring,
                    move |el, w| {
                        let w = w.max(px(COLLAPSED_W));
                        let exp = f32::from(w) > COLLAPSED_W + (width - COLLAPSED_W) * 0.4;
                        el.w(w).child(rail_body(&theme, parts, exp))
                    },
                )
                .into_any_element();
        }

        // ----- モーダル展開型 -----
        let motion = theme.motion;
        let scrim = theme.colors.scrim;
        let panel_bg = theme.colors.surface_container;
        let shadow = theme.colors.shadow;
        let body = rail_body(&theme, parts, true);
        let dismiss = self.on_dismiss.clone();
        let open = expanded;

        let mut root = div()
            .id(ElementId::Name(format!("{id}::modal").into()))
            .absolute()
            .inset_0()
            .overflow_hidden();
        if let Some(d) = dismiss {
            root = root.on_click(move |_, w, cx| d(w, cx));
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
                        .m3_elevation(2, shadow)
                        .child(body),
                )
            },
        )
        .into_any_element()
    }
}

// ---------------------------------------------------------------------------
// Navigation bar
// ---------------------------------------------------------------------------

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
