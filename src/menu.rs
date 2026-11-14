//! Menus（M3 Expressive の縦型メニュー）
//!
//! - [`Menu`]: メニュー本体（Standard / Vibrant）
//! - [`DropdownMenu`]: トリガーをクリックすると開く
//! - [`ContextMenuArea`]: 右クリックした位置に開く
//! - [`Select`]: Exposed dropdown（選択式フィールド）
//! - [`MenuItem::submenu`]: サブメニュー（ホバー・クリック・→キーで右に開く）

use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use gpui::AnimationExt as _;
use gpui::{
    Anchor, Animation, AnyElement, App, Bounds, ClickEvent, Div, ElementId, Hsla, IntoElement,
    MouseButton, ParentElement, Pixels, Point, RenderOnce, SharedString, SpringAnimation,
    StyleRefinement, Styled, Window, anchored, canvas, deferred, div, prelude::*, px, relative,
};

use super::{
    ActiveTheme, ChangeHandler, ClickHandler, Icon, IconName, StyledExt, TypeScale, ripple_overlay,
    shape, state, state_layer, track_ripple, transparent, use_popup_focus, use_ripple,
};

type DismissHandler = Rc<dyn Fn(&mut Window, &mut App)>;

// ---------------------------------------------------------------------------
// Menu
// ---------------------------------------------------------------------------

pub struct MenuItem {
    label: SharedString,
    icon: Option<IconName>,
    trailing_text: Option<SharedString>,
    supporting: Option<SharedString>,
    selected: bool,
    disabled: bool,
    on_click: Option<ClickHandler>,
    submenu: Option<Box<Menu>>,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            trailing_text: None,
            supporting: None,
            selected: false,
            disabled: false,
            on_click: None,
            submenu: None,
        }
    }
    /// 右に開くサブメニュー（右端に ▸ を表示。項目自体のクリックでは閉じない）
    pub fn submenu(mut self, menu: Menu) -> Self {
        self.submenu = Some(Box::new(menu));
        self
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    /// 右側の補助テキスト（ショートカットキーなど）
    pub fn trailing_text(mut self, text: impl Into<SharedString>) -> Self {
        self.trailing_text = Some(text.into());
        self
    }
    /// ラベル下の説明文
    pub fn supporting(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting = Some(text.into());
        self
    }
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

pub enum MenuEntry {
    Item(MenuItem),
    Divider,
    Header(SharedString),
}

#[derive(IntoElement)]
pub struct Menu {
    id: SharedString,
    entries: Vec<MenuEntry>,
    vibrant: bool,
    min_width: Pixels,
    on_dismiss: Option<DismissHandler>,
    /// サブメニューとして描画される（外側クリックの判定は親が行う）
    nested: bool,
    /// ネストの深さ（ルート = 0）と、ルートのサブメニュー状態
    depth: usize,
    root_subs: Option<gpui::Entity<SubState>>,
    on_back: Option<DismissHandler>,
}

#[derive(Default)]
struct SubState {
    open: Option<usize>,
    /// 深さごとの、開いているサブメニューの範囲（ルートの外側クリック判定に使う）
    rects: Vec<Option<Bounds<Pixels>>>,
}

impl Menu {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            entries: vec![],
            vibrant: false,
            min_width: px(112.),
            on_dismiss: None,
            nested: false,
            depth: 0,
            root_subs: None,
            on_back: None,
        }
    }
    pub fn item(mut self, item: MenuItem) -> Self {
        self.entries.push(MenuEntry::Item(item));
        self
    }
    pub fn divider(mut self) -> Self {
        self.entries.push(MenuEntry::Divider);
        self
    }
    pub fn header(mut self, text: impl Into<SharedString>) -> Self {
        self.entries.push(MenuEntry::Header(text.into()));
        self
    }
    /// 鮮やかな配色（tertiary container）
    pub fn vibrant(mut self, v: bool) -> Self {
        self.vibrant = v;
        self
    }
    pub fn min_width(mut self, w: Pixels) -> Self {
        self.min_width = w;
        self
    }
    fn nested_in(mut self, dismiss: Option<DismissHandler>) -> Self {
        self.nested = true;
        self.on_dismiss = dismiss;
        self
    }
    /// 項目クリック・外側クリックで呼ばれる
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let (bg, fg, icon_fg, sel_bg, sel_fg, muted) = if self.vibrant {
            (
                c.tertiary_container,
                c.on_tertiary_container,
                c.on_tertiary_container,
                c.tertiary,
                c.on_tertiary,
                c.on_tertiary_container.opacity(0.75),
            )
        } else {
            (
                c.surface_container_low,
                c.on_surface,
                c.on_surface_variant,
                c.tertiary_container,
                c.on_tertiary_container,
                c.on_surface_variant,
            )
        };

        let mut list = div()
            .id(ElementId::Name(self.id.clone()))
            .occlude()
            .flex()
            .flex_col()
            .gap(px(2.))
            .p(px(4.))
            .min_w(self.min_width)
            .max_w(px(320.))
            .rounded(shape::LARGE)
            .bg(bg)
            .text_color(fg)
            .m3_elevation(3, c.shadow);
        let subs = window.use_keyed_state(
            SharedString::from(format!("{}::subs", self.id)),
            cx,
            |_, _| SubState::default(),
        );
        let nested = self.nested;
        let depth = self.depth;
        let root_subs = self.root_subs.clone().unwrap_or_else(|| subs.clone());
        let on_back = self.on_back.clone();
        if let Some(d) = self.on_dismiss.clone() {
            if !nested {
                let guard = root_subs.clone();
                list = list.on_mouse_down_out({
                    let d = d.clone();
                    move |e, w, cx| {
                        // 開いているサブメニュー内のクリックは「外側」ではない
                        let s = guard.read(cx);
                        if s.rects.iter().flatten().any(|b| b.contains(&e.position)) {
                            return;
                        }
                        d(w, cx)
                    }
                });
            }
            // 開いたらフォーカスを持ち、Esc で閉じ、↑↓ で項目を移動する
            let focus = use_popup_focus(&self.id, window, cx);
            list = list.track_focus(&focus).on_key_down(move |e, window, cx| {
                match e.keystroke.key.as_str() {
                    "escape" => {
                        cx.stop_propagation();
                        d(window, cx);
                    }
                    "down" => {
                        cx.stop_propagation();
                        window.focus_next(cx);
                    }
                    "up" => {
                        cx.stop_propagation();
                        window.focus_prev(cx);
                    }
                    "left" => {
                        if let Some(b) = on_back.as_ref() {
                            cx.stop_propagation();
                            b(window, cx);
                        }
                    }
                    _ => {}
                }
            });
        }

        let dismiss_all = self.on_dismiss.clone();
        for (i, entry) in self.entries.into_iter().enumerate() {
            match entry {
                MenuEntry::Divider => {
                    list = list.child(
                        div()
                            .py(px(2.))
                            .child(div().h(px(1.)).mx(px(8.)).bg(c.outline_variant)),
                    );
                }
                MenuEntry::Header(text) => {
                    list = list.child(
                        div()
                            .px(px(12.))
                            .pt(px(8.))
                            .pb(px(4.))
                            .m3_type(TypeScale::LabelLarge)
                            .text_color(muted)
                            .child(text),
                    );
                }
                MenuEntry::Item(mut item) => {
                    let submenu = item.submenu.take();
                    let has_sub = submenu.is_some() && !item.disabled;
                    let (item_bg, item_fg, item_icon) = if item.disabled {
                        (
                            transparent(),
                            theme.disabled_content(),
                            theme.disabled_content(),
                        )
                    } else if item.selected {
                        (sel_bg, sel_fg, sel_fg)
                    } else {
                        (transparent(), fg, icon_fg)
                    };
                    let mut row = div()
                        .id(ElementId::Name(format!("{}-{i}", self.id).into()))
                        .relative()
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .min_h(px(44.))
                        .px(px(12.))
                        .py(px(8.))
                        .rounded(shape::MEDIUM)
                        .bg(item_bg)
                        .text_color(item_fg);
                    let mut row_focus = None;
                    if !item.disabled {
                        let rid: SharedString = format!("{}-{i}", self.id).into();
                        let ripple = use_ripple(&rid, window, cx);
                        row = row.child(ripple_overlay(
                            &ripple,
                            item_fg,
                            gpui::Corners::all(shape::MEDIUM),
                        ));
                        row = track_ripple(row, &ripple);
                        row_focus = Some(ripple.focus().clone());
                    }
                    if let Some(icon) = item.icon {
                        row = row.child(Icon::new(icon).size(px(20.)).color(item_icon));
                    } else if item.selected {
                        row = row.child(Icon::new(IconName::Check).size(px(20.)).color(item_icon));
                    }
                    let mut text = div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(div().m3_type(TypeScale::LabelLarge).child(item.label));
                    if let Some(s) = item.supporting {
                        text = text.child(
                            div()
                                .m3_type(TypeScale::BodySmall)
                                .text_color(if item.selected { item_fg } else { muted })
                                .child(s),
                        );
                    }
                    row = row.child(text);
                    if let Some(t) = item.trailing_text {
                        row = row.child(
                            div()
                                .pl(px(16.))
                                .m3_type(TypeScale::LabelLarge)
                                .text_color(if item.selected { item_fg } else { muted })
                                .child(t),
                        );
                    }
                    if has_sub {
                        row = row.child(
                            Icon::new(IconName::ChevronRight)
                                .size(px(20.))
                                .color(item_icon),
                        );
                    }
                    if !item.disabled {
                        row = row
                            .cursor_pointer()
                            .hover(move |s| s.bg(state_layer(item_bg, item_fg, state::HOVER)));
                        // ホバーでサブメニューを開閉する
                        let hs = subs.clone();
                        let hroot = root_subs.clone();
                        row = row.on_hover(move |hovered, _, cx| {
                            if *hovered {
                                let want = has_sub.then_some(i);
                                let mut changed = false;
                                hs.update(cx, |s, cx| {
                                    if s.open != want {
                                        s.open = want;
                                        changed = true;
                                        cx.notify();
                                    }
                                });
                                if changed {
                                    hroot.update(cx, |r, _| r.rects.truncate(depth));
                                }
                            }
                        });
                        if has_sub {
                            let ks = subs.clone();
                            row = row.on_key_down(move |e, _, cx| {
                                if e.keystroke.key == "right" {
                                    cx.stop_propagation();
                                    ks.update(cx, |s, cx| {
                                        s.open = Some(i);
                                        cx.notify();
                                    });
                                }
                            });
                        }
                        let handler = item.on_click.clone();
                        let dismiss = self.on_dismiss.clone();
                        let cs = subs.clone();
                        row = row.on_click(move |e, w, cx| {
                            if let Some(h) = handler.as_ref() {
                                h(e, w, cx);
                            }
                            if has_sub {
                                cs.update(cx, |s, cx| {
                                    s.open = Some(i);
                                    cx.notify();
                                });
                            } else if let Some(d) = dismiss.as_ref() {
                                d(w, cx);
                            }
                        });
                    }
                    if let Some(sub) = submenu {
                        if has_sub && subs.read(cx).open == Some(i) {
                            let back_subs = subs.clone();
                            let back_root = root_subs.clone();
                            let measure = root_subs.clone();
                            let mut menu = (*sub).nested_in(dismiss_all.clone());
                            menu.depth = depth + 1;
                            menu.root_subs = Some(root_subs.clone());
                            menu.on_back = Some(Rc::new(move |w, cx| {
                                back_subs.update(cx, |s, cx| {
                                    s.open = None;
                                    cx.notify();
                                });
                                back_root.update(cx, |r, _| r.rects.truncate(depth));
                                if let Some(f) = row_focus.as_ref() {
                                    w.focus(f, cx);
                                }
                            }));
                            row = row.child(
                                div().absolute().top(px(-4.)).left(relative(1.)).child(
                                    deferred(
                                        anchored()
                                            .anchor(Anchor::TopLeft)
                                            .snap_to_window_with_margin(px(8.))
                                            .child(
                                                div()
                                                    .relative()
                                                    .child(
                                                        canvas(
                                                            move |b, _, cx| {
                                                                measure.update(cx, |s, _| {
                                                                    if s.rects.len() <= depth {
                                                                        s.rects.resize(
                                                                            depth + 1,
                                                                            None,
                                                                        );
                                                                    }
                                                                    s.rects[depth] = Some(b);
                                                                })
                                                            },
                                                            |_, _, _, _| {},
                                                        )
                                                        .absolute()
                                                        .size_full(),
                                                    )
                                                    .child(menu),
                                            ),
                                    )
                                    .priority(2),
                                ),
                            );
                        }
                    }
                    list = list.child(row);
                }
            }
        }

        // 開くとき: 上から伸びながらフェードイン（Expressive のスプリングっぽい ease）
        list.with_animation(
            ElementId::Name(format!("{}::open", self.id).into()),
            Animation::new(Duration::from_millis(260)).with_easing(|t| {
                // ease-out-back（軽いオーバーシュート）
                let c1 = 1.2;
                let c3 = c1 + 1.;
                1. + c3 * (t - 1.).powi(3) + c1 * (t - 1.).powi(2)
            }),
            |el, t| {
                el.opacity(t.clamp(0., 1.))
                    .max_h(px(40. + 560. * t.clamp(0., 1.2)))
                    .overflow_hidden()
            },
        )
    }
}

// ---------------------------------------------------------------------------
// 開閉状態
// ---------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct PopupState {
    pub(crate) open: bool,
    position: Option<Point<Pixels>>,
    dismissed_at: Option<Instant>,
    pub(crate) trigger_bounds: Bounds<Pixels>,
}

impl PopupState {
    /// 外側クリックで閉じた直後のトリガークリックで、再度開かないようにする
    pub(crate) fn recently_dismissed(&self) -> bool {
        self.dismissed_at
            .is_some_and(|t| t.elapsed() < Duration::from_millis(250))
    }
}

pub(crate) fn popup_state(
    id: &SharedString,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<PopupState> {
    window.use_keyed_state(SharedString::from(format!("{id}::popup")), cx, |_, _| {
        PopupState::default()
    })
}

pub(crate) fn dismisser(
    state: &gpui::Entity<PopupState>,
) -> impl Fn(&mut Window, &mut App) + 'static {
    let state = state.clone();
    move |_, cx| {
        state.update(cx, |s, cx| {
            if s.open {
                s.open = false;
                s.dismissed_at = Some(Instant::now());
                cx.notify();
            }
        })
    }
}

// ---------------------------------------------------------------------------
// DropdownMenu
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct DropdownMenu {
    id: SharedString,
    trigger: AnyElement,
    menu: Menu,
    align_right: bool,
    full_width: bool,
    match_width: bool,
    state: Option<gpui::Entity<PopupState>>,
}

impl DropdownMenu {
    pub fn new(id: impl Into<SharedString>, trigger: impl IntoElement, menu: Menu) -> Self {
        Self {
            id: id.into(),
            trigger: trigger.into_any_element(),
            menu,
            align_right: false,
            full_width: false,
            match_width: false,
            state: None,
        }
    }
    fn with_state(mut self, state: gpui::Entity<PopupState>) -> Self {
        self.state = Some(state);
        self
    }
    /// メニューの最小幅をトリガーの幅に揃える
    pub fn match_width(mut self, v: bool) -> Self {
        self.match_width = v;
        self
    }
    /// 親の幅いっぱいに広げる
    pub fn full_width(mut self, v: bool) -> Self {
        self.full_width = v;
        self
    }
    /// トリガーの右端に揃える
    pub fn align_right(mut self, v: bool) -> Self {
        self.align_right = v;
        self
    }
}

impl RenderOnce for DropdownMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = match self.state.clone() {
            Some(s) => s,
            None => popup_state(&self.id, window, cx),
        };
        let (open, trigger) = {
            let s = state.read(cx);
            (s.open, s.trigger_bounds)
        };
        let toggle = state.clone();
        let measure = state.clone();

        // 下に入りきらなければ上向きに開く
        let estimated: f32 = 8.
            + self
                .menu
                .entries
                .iter()
                .map(|e| match e {
                    MenuEntry::Item(i) if i.supporting.is_some() => 62.,
                    MenuEntry::Item(_) => 46.,
                    MenuEntry::Header(_) => 34.,
                    MenuEntry::Divider => 7.,
                })
                .sum::<f32>();
        let viewport = window.viewport_size();
        let below = f32::from(viewport.height) - f32::from(trigger.bottom()) - 12.;
        let above = f32::from(trigger.top()) - 12.;
        let upward = estimated > below && above > below;

        let mut el = div()
            .id(ElementId::Name(self.id.clone()))
            .relative()
            .map(|d| {
                if self.full_width {
                    d.w_full()
                } else {
                    d.flex_none()
                }
            })
            .child(
                canvas(
                    move |b, _, cx| {
                        measure.update(cx, |s, cx| {
                            if s.trigger_bounds != b {
                                s.trigger_bounds = b;
                                cx.notify();
                            }
                        })
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(self.trigger)
            .on_click(move |_, _, cx| {
                toggle.update(cx, |s, cx| {
                    if !s.open && !s.recently_dismissed() {
                        s.open = true;
                        cx.notify();
                    }
                })
            });

        if open {
            let mut menu = self.menu.on_dismiss(dismisser(&state));
            if self.match_width {
                menu = menu.min_width(trigger.size.width);
            }
            let holder = if upward {
                div().absolute().top_0().mt(px(-4.))
            } else {
                div().absolute().top(relative(1.)).mt(px(4.))
            };
            let holder = if self.align_right {
                holder.right_0()
            } else {
                holder.left_0()
            };
            let anchor = match (upward, self.align_right) {
                (false, false) => Anchor::TopLeft,
                (false, true) => Anchor::TopRight,
                (true, false) => Anchor::BottomLeft,
                (true, true) => Anchor::BottomRight,
            };
            el = el.child(
                holder.child(
                    deferred(
                        anchored()
                            .anchor(anchor)
                            .snap_to_window_with_margin(px(8.))
                            .child(menu),
                    )
                    .priority(1),
                ),
            );
        }
        el
    }
}

// ---------------------------------------------------------------------------
// ContextMenuArea
// ---------------------------------------------------------------------------

/// 右クリックでメニューを開く領域。`Styled` / `ParentElement` を実装
#[derive(IntoElement)]
pub struct ContextMenuArea {
    id: SharedString,
    base: Div,
    menu: Menu,
}

impl ContextMenuArea {
    pub fn new(id: impl Into<SharedString>, menu: Menu) -> Self {
        Self {
            id: id.into(),
            base: div(),
            menu,
        }
    }
}

impl Styled for ContextMenuArea {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for ContextMenuArea {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.base.extend(elements)
    }
}

impl RenderOnce for ContextMenuArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = popup_state(&self.id, window, cx);
        let (open, pos) = {
            let s = state.read(cx);
            (s.open, s.position)
        };
        let opener = state.clone();
        let mut el = self
            .base
            .id(ElementId::Name(self.id.clone()))
            .on_mouse_down(MouseButton::Right, move |e, _, cx| {
                opener.update(cx, |s, cx| {
                    s.open = true;
                    s.position = Some(e.position);
                    cx.notify();
                })
            });
        if let (true, Some(pos)) = (open, pos) {
            let menu = self.menu.on_dismiss(dismisser(&state));
            el = el.child(
                deferred(
                    anchored()
                        .position(pos)
                        .snap_to_window_with_margin(px(8.))
                        .child(menu),
                )
                .priority(1),
            );
        }
        el
    }
}

// ---------------------------------------------------------------------------
// Select（Exposed dropdown menu）
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct Select {
    id: SharedString,
    label: SharedString,
    options: Vec<SharedString>,
    selected: Option<usize>,
    outlined: bool,
    leading_icon: Option<IconName>,
    notch_background: Option<Rc<dyn Fn(&super::ColorScheme) -> Hsla>>,
    on_change: Option<ChangeHandler<usize>>,
}

impl Select {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            options: vec![],
            selected: None,
            outlined: false,
            leading_icon: None,
            notch_background: None,
            on_change: None,
        }
    }
    pub fn options(mut self, options: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        self.options = options.into_iter().map(Into::into).collect();
        self
    }
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    pub fn outlined(mut self, v: bool) -> Self {
        self.outlined = v;
        self
    }
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }
    pub fn notch_background(mut self, f: impl Fn(&super::ColorScheme) -> Hsla + 'static) -> Self {
        self.notch_background = Some(Rc::new(f));
        self
    }
    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let state = popup_state(&self.id, window, cx);
        let open = state.read(cx).open;
        let has_value = self.selected.is_some();
        let floating = open || has_value;
        let outlined = self.outlined;
        let label_color = if open {
            c.primary
        } else {
            c.on_surface_variant
        };
        let text_left = if self.leading_icon.is_some() {
            52.
        } else {
            16.
        };
        let notch_bg = self
            .notch_background
            .as_ref()
            .map(|f| f(&c))
            .unwrap_or(c.surface);

        let value_text = self
            .selected
            .and_then(|i| self.options.get(i).cloned())
            .unwrap_or_default();

        // フィールド本体（幅は DropdownMenu が計測して state に入れる）
        let mut field = div()
            .relative()
            .h(px(56.))
            .w_full()
            .text_color(c.on_surface)
            .m3_type(TypeScale::BodyLarge)
            .child(
                div()
                    .absolute()
                    .left(px(text_left))
                    .right(px(52.))
                    .top(px(if outlined { 16. } else { 24. }))
                    .h(px(24.))
                    .truncate()
                    .child(value_text),
            )
            .child(
                div().absolute().right(px(12.)).top(px(16.)).child(
                    Icon::new(if open {
                        IconName::ArrowDropUp
                    } else {
                        IconName::ArrowDropDown
                    })
                    .size(px(24.))
                    .color(c.on_surface_variant),
                ),
            );
        if let Some(icon) = self.leading_icon {
            field = field.child(
                div()
                    .absolute()
                    .left(px(12.))
                    .top(px(16.))
                    .child(Icon::new(icon).size(px(24.)).color(c.on_surface_variant)),
            );
        }
        if outlined {
            field = field.rounded(shape::EXTRA_SMALL).child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(shape::EXTRA_SMALL)
                    .border_color(if open { c.primary } else { c.outline })
                    .map(|d| if open { d.border_2() } else { d.border_1() }),
            );
        } else {
            let bg = c.surface_container_highest;
            let on = c.on_surface;
            field = field
                .bg(bg)
                .rounded_t(shape::EXTRA_SMALL)
                .overflow_hidden()
                .hover(move |s| s.bg(bg.blend(on.opacity(state::HOVER))))
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(if open { 2. } else { 1. }))
                        .bg(if open {
                            c.primary
                        } else {
                            c.on_surface_variant
                        }),
                );
        }
        // ラベル
        let float_top = if outlined { -8. } else { 8. };
        let float_left = if outlined { 12. } else { text_left };
        let rest_left = text_left - if outlined { 4. } else { 0. };
        let mut label = div()
            .absolute()
            .whitespace_nowrap()
            .text_color(label_color)
            .child(self.label);
        if outlined {
            label = label.px(px(4.));
        }
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        field = field.child(label.with_spring(
            ElementId::Name(format!("{}::label", self.id).into()),
            SpringAnimation::new(theme.motion.fast_spatial).to(if floating { 1. } else { 0. }),
            move |el, t| {
                let el = el
                    .top(px(lerp(16., float_top, t)))
                    .left(px(lerp(rest_left, float_left, t.clamp(0., 1.))))
                    .text_size(px(lerp(16., 12., t)))
                    .line_height(px(lerp(24., 16., t)));
                if outlined {
                    el.bg(notch_bg.opacity(t.clamp(0., 1.)))
                } else {
                    el
                }
            },
        ));

        // メニュー
        let mut menu = Menu::new(format!("{}::menu", self.id));
        for (i, opt) in self.options.iter().enumerate() {
            let mut item = MenuItem::new(opt.clone()).selected(self.selected == Some(i));
            if let Some(h) = self.on_change.clone() {
                item = item.on_click(move |_, w, cx| h(i, w, cx));
            }
            menu = menu.item(item);
        }

        DropdownMenu::new(self.id.clone(), field, menu)
            .full_width(true)
            .match_width(true)
            .with_state(state)
    }
}
