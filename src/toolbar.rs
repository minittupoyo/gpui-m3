//! Toolbars（M3 Expressive）
//!
//! - [`DockedToolbar`]: 画面下端いっぱいに置くツールバー（Bottom app bar の後継）
//! - [`FloatingToolbar`]: 浮かぶピル型ツールバー（横 / 縦、FAB とのペア、スクロールで隠す）
//!
//! 項目は [`ToolbarItem::action`]（アイコンボタン）か [`ToolbarItem::element`]（任意の要素）。
//! 選択中のアクションは横長のピルに広がる。

use std::rc::Rc;

use gpui::AnimationExt as _;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, Hsla, IntoElement, RenderOnce, SharedString,
    SpringAnimation, Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, ClickHandler, Fab, FabColor, Icon, IconName, RippleHandle, StyledExt, Theme,
    ripple_overlay, state, state_layer, tooltip, track_ripple, transparent, use_ripple,
};

pub struct ToolbarAction {
    icon: IconName,
    selected_icon: Option<IconName>,
    tooltip: Option<SharedString>,
    selected: bool,
    on_click: Option<ClickHandler>,
}

pub enum ToolbarItem {
    Action(ToolbarAction),
    Element(AnyElement),
}

impl ToolbarItem {
    pub fn action(icon: IconName) -> ToolbarAction {
        ToolbarAction {
            icon,
            selected_icon: None,
            tooltip: None,
            selected: false,
            on_click: None,
        }
    }
    pub fn element(el: impl IntoElement) -> Self {
        Self::Element(el.into_any_element())
    }
}

impl ToolbarAction {
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    pub fn selected_icon(mut self, icon: IconName) -> Self {
        self.selected_icon = Some(icon);
        self
    }
    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl From<ToolbarAction> for ToolbarItem {
    fn from(a: ToolbarAction) -> Self {
        Self::Action(a)
    }
}

/// (背景, 項目の色, 選択中の背景, 選択中の色)
fn toolbar_colors(theme: &Theme, vibrant: bool) -> (Hsla, Hsla, Hsla, Hsla) {
    let c = &theme.colors;
    if vibrant {
        (
            c.primary_container,
            c.on_primary_container,
            c.surface_container,
            c.on_surface,
        )
    } else {
        (
            c.surface_container,
            c.on_surface_variant,
            c.secondary_container,
            c.on_secondary_container,
        )
    }
}

fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    if a.a <= 0.001 {
        b.opacity(t.clamp(0., 1.))
    } else {
        a.blend(b.opacity(t.clamp(0., 1.)))
    }
}

/// 1 つのアクションボタン（選択で 48 → 64px のピルに広がる）
fn render_action(
    theme: &Theme,
    ripple: RippleHandle,
    id: String,
    action: ToolbarAction,
    vibrant: bool,
    vertical: bool,
) -> AnyElement {
    let (_, fg, sel_bg, sel_fg) = toolbar_colors(theme, vibrant);
    let selected = action.selected;
    let icon = match (selected, action.selected_icon) {
        (true, Some(i)) => i,
        _ => action.icon,
    };
    let handler = action.on_click.clone();
    let tip = action.tooltip.clone();
    let eid = ElementId::Name(id.clone().into());

    let mut el = div()
        .id(eid)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded_full()
        .cursor_pointer();
    if let Some(h) = handler {
        el = el.on_click(move |e, w, cx| h(e, w, cx));
    }
    if let Some(t) = tip {
        el = el.tooltip(tooltip(t));
    }
    let el = track_ripple(el, &ripple);

    el.with_spring(
        ElementId::Name(format!("{id}::sel").into()),
        SpringAnimation::new(theme.motion.fast_spatial).to(if selected { 1. } else { 0. }),
        move |el, t| {
            let tc = t.clamp(0., 1.);
            let bg = mix(transparent(), sel_bg, tc);
            let color = fg.blend(sel_fg.opacity(tc));
            let long = 48. + 16. * t.clamp(-0.1, 1.2);
            let el = if vertical {
                el.w(px(48.)).h(px(long))
            } else {
                el.h(px(48.)).w(px(long))
            };
            let radius = px(24.);
            el.bg(bg)
                .hover(move |s| s.bg(state_layer(bg, color, state::HOVER)))
                .child(ripple_overlay(&ripple, color, gpui::Corners::all(radius)))
                .child(Icon::new(icon).size(px(24.)).color(color))
        },
    )
    .into_any_element()
}

// ---------------------------------------------------------------------------
// Docked
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct DockedToolbar {
    id: SharedString,
    items: Vec<ToolbarItem>,
    vibrant: bool,
}

impl DockedToolbar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            items: vec![],
            vibrant: false,
        }
    }
    pub fn item(mut self, item: impl Into<ToolbarItem>) -> Self {
        self.items.push(item.into());
        self
    }
    pub fn vibrant(mut self, v: bool) -> Self {
        self.vibrant = v;
        self
    }
}

impl RenderOnce for DockedToolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let (bg, fg, _, _) = toolbar_colors(&theme, self.vibrant);
        let mut bar = div()
            .w_full()
            .h(px(64.))
            .flex_none()
            .flex()
            .items_center()
            .justify_around()
            .px(px(16.))
            .bg(bg)
            .text_color(fg);
        for (i, item) in self.items.into_iter().enumerate() {
            bar = bar.child(match item {
                ToolbarItem::Action(a) => {
                    let rid = format!("{}-{i}", self.id);
                    let r = use_ripple(&rid.clone().into(), window, cx);
                    render_action(&theme, r, rid, a, self.vibrant, false)
                }
                ToolbarItem::Element(e) => e,
            });
        }
        bar
    }
}

// ---------------------------------------------------------------------------
// Floating
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct FloatingToolbar {
    id: SharedString,
    items: Vec<ToolbarItem>,
    vibrant: bool,
    vertical: bool,
    visible: bool,
    fab: Option<(IconName, Option<ClickHandler>)>,
}

impl FloatingToolbar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            items: vec![],
            vibrant: false,
            vertical: false,
            visible: true,
            fab: None,
        }
    }
    pub fn item(mut self, item: impl Into<ToolbarItem>) -> Self {
        self.items.push(item.into());
        self
    }
    pub fn vibrant(mut self, v: bool) -> Self {
        self.vibrant = v;
        self
    }
    pub fn vertical(mut self, v: bool) -> Self {
        self.vertical = v;
        self
    }
    /// false でツールバーが下（縦向きなら右）に引っ込む
    pub fn visible(mut self, v: bool) -> Self {
        self.visible = v;
        self
    }
    /// 横に FAB を並べる
    pub fn fab(
        mut self,
        icon: IconName,
        on_click: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>>,
    ) -> Self {
        self.fab = Some((icon, on_click));
        self
    }
}

impl RenderOnce for FloatingToolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let (bg, fg, _, _) = toolbar_colors(&theme, self.vibrant);
        let vertical = self.vertical;
        let shadow = theme.colors.shadow;

        let mut bar = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(4.))
            .p(px(8.))
            .rounded_full()
            .bg(bg)
            .text_color(fg)
            .m3_elevation(3, shadow);
        bar = if vertical {
            bar.flex_col().w(px(64.))
        } else {
            bar.h(px(64.))
        };
        for (i, item) in self.items.into_iter().enumerate() {
            bar = bar.child(match item {
                ToolbarItem::Action(a) => {
                    let rid = format!("{}-{i}", self.id);
                    let r = use_ripple(&rid.clone().into(), window, cx);
                    render_action(&theme, r, rid, a, self.vibrant, vertical)
                }
                ToolbarItem::Element(e) => e,
            });
        }

        let mut group = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .child(bar);
        if vertical {
            group = group.flex_col();
        }
        if let Some((icon, handler)) = self.fab {
            let mut fab = Fab::new(format!("{}::fab", self.id), icon).color(if self.vibrant {
                FabColor::Tertiary
            } else {
                FabColor::TertiaryContainer
            });
            if let Some(h) = handler {
                fab = fab.on_click(move |e, w, cx| h(e, w, cx));
            }
            group = group.child(fab);
        }

        let visible = self.visible;
        div().relative().child(group).with_spring(
            ElementId::Name(format!("{}::visible", self.id).into()),
            SpringAnimation::new(theme.motion.default_spatial).to(if visible { 1. } else { 0. }),
            move |el, t| {
                let off = px((1. - t) * 120.);
                let el = if vertical { el.left(off) } else { el.top(off) };
                el.opacity(t.clamp(0., 1.))
            },
        )
    }
}
