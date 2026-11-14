//! Tabs（Primary / Secondary）。インジケーターがバネで滑る

use gpui::AnimationExt as _;
use gpui::{
    App, Bounds, ElementId, IntoElement, Pixels, RenderOnce, SharedString, SpringAnimation, Window,
    canvas, div, prelude::*, px,
};

use super::{
    ActiveTheme, Badge, ChangeHandler, Icon, IconName, StyledExt, TypeScale, ripple_overlay, state,
    track_ripple, use_ripple,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    #[default]
    Primary,
    Secondary,
}

pub struct Tab {
    label: SharedString,
    icon: Option<IconName>,
    badge: Option<Option<u32>>,
}

impl Tab {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            badge: None,
        }
    }
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn badge(mut self, count: Option<u32>) -> Self {
        self.badge = Some(count);
        self
    }
}

#[derive(Default)]
struct TabsLayout {
    container: Bounds<Pixels>,
    tabs: Vec<Bounds<Pixels>>,
    contents: Vec<Bounds<Pixels>>,
}

#[derive(IntoElement)]
pub struct Tabs {
    id: SharedString,
    tabs: Vec<Tab>,
    selected: usize,
    variant: TabsVariant,
    scrollable: bool,
    on_select: Option<ChangeHandler<usize>>,
}

impl Tabs {
    pub fn new(id: impl Into<SharedString>, selected: usize) -> Self {
        Self {
            id: id.into(),
            tabs: vec![],
            selected,
            variant: TabsVariant::Primary,
            scrollable: false,
            on_select: None,
        }
    }
    /// タブが多いときに横スクロールさせる（各タブは内容の幅になる）
    pub fn scrollable(mut self, v: bool) -> Self {
        self.scrollable = v;
        self
    }
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }
    pub fn variant(mut self, v: TabsVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn secondary(self) -> Self {
        self.variant(TabsVariant::Secondary)
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let n = self.tabs.len();
        let layout = window.use_keyed_state(
            SharedString::from(format!("{}::layout", self.id)),
            cx,
            |_, _| TabsLayout::default(),
        );
        layout.update(cx, |l, _| {
            l.tabs.resize(n, Bounds::default());
            l.contents.resize(n, Bounds::default());
        });

        let primary = self.variant == TabsVariant::Primary;
        let has_icons = self.tabs.iter().any(|t| t.icon.is_some());
        let height = if primary && has_icons { 64. } else { 48. };
        let layer = c.on_surface.opacity(state::HOVER);

        let scrollable = self.scrollable;
        let mut row = div()
            .id(ElementId::Name(format!("{}::row", self.id).into()))
            .relative()
            .flex()
            .w_full()
            .h(px(height));
        if scrollable {
            row = row.overflow_x_scroll();
        }
        // コンテナ位置の記録
        {
            let l = layout.clone();
            row = row.child(
                canvas(
                    move |b, _, cx| {
                        l.update(cx, |s, cx| {
                            if s.container != b {
                                s.container = b;
                                cx.notify();
                            }
                        })
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            );
        }

        for (i, tab) in self.tabs.into_iter().enumerate() {
            let active = i == self.selected;
            let fg = match (primary, active) {
                (true, true) => c.primary,
                (false, true) => c.on_surface,
                _ => c.on_surface_variant,
            };
            let tab_l = layout.clone();
            let content_l = layout.clone();

            let mut content = div()
                .relative()
                .flex()
                .items_center()
                .gap(px(if primary && has_icons { 2. } else { 8. }))
                .text_color(fg)
                .m3_type(TypeScale::TitleSmall)
                .child(
                    canvas(
                        move |b, _, cx| {
                            content_l.update(cx, |s, cx| {
                                if s.contents.get(i) != Some(&b) {
                                    if let Some(slot) = s.contents.get_mut(i) {
                                        *slot = b;
                                        cx.notify();
                                    }
                                }
                            })
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                );
            if primary && has_icons {
                content = content.flex_col();
            }
            if let Some(icon) = tab.icon {
                let mut icon_box = div()
                    .relative()
                    .child(Icon::new(icon).size(px(24.)).color(fg));
                if let Some(b) = tab.badge {
                    icon_box = icon_box.child(
                        div()
                            .absolute()
                            .top(px(-2.))
                            .left(px(16.))
                            .child(Badge::new(b)),
                    );
                }
                content = content.child(icon_box);
            }
            content = content.child(tab.label);
            if tab.icon.is_none() {
                if let Some(b) = tab.badge {
                    content = content.child(Badge::new(b));
                }
            }

            let tab_id: SharedString = format!("{}-{i}", self.id).into();
            let ripple = use_ripple(&tab_id, window, cx);
            let mut tab_el = div()
                .id(ElementId::Name(tab_id.clone()))
                .relative()
                .when(!scrollable, |d| d.flex_1())
                .when(scrollable, |d| d.flex_none().min_w(px(96.)))
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .px(px(16.))
                .cursor_pointer()
                .hover(move |s| s.bg(layer))
                .child(ripple_overlay(
                    &ripple,
                    c.on_surface,
                    gpui::Corners::all(px(0.)),
                ))
                .child(
                    canvas(
                        move |b, _, cx| {
                            tab_l.update(cx, |s, cx| {
                                if s.tabs.get(i) != Some(&b) {
                                    if let Some(slot) = s.tabs.get_mut(i) {
                                        *slot = b;
                                        cx.notify();
                                    }
                                }
                            })
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .child(content);
            tab_el = track_ripple(tab_el, &ripple);
            if let Some(h) = self.on_select.clone() {
                tab_el = tab_el.on_click(move |_, w, cx| h(i, w, cx));
            }
            row = row.child(tab_el);
        }

        // 区切り線（スクロールするときは枠の側に置いて動かさない）
        let divider = div()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(1.))
            .bg(c.surface_container_highest);
        if !scrollable {
            row = row.child(divider);
        }
        let divider_outer = (scrollable).then(|| {
            div()
                .absolute()
                .left_0()
                .right_0()
                .bottom_0()
                .h(px(1.))
                .bg(c.surface_container_highest)
        });

        // インジケーター
        let l = layout.read(cx);
        if let (Some(tab_b), Some(content_b)) =
            (l.tabs.get(self.selected), l.contents.get(self.selected))
            && tab_b.size.width > px(0.)
        {
            // スクロールするときは、内容の左端（スクロールで動く）を基準にする
            let origin = if scrollable {
                l.tabs
                    .first()
                    .map(|b| b.origin.x)
                    .unwrap_or(l.container.origin.x)
            } else {
                l.container.origin.x
            };
            let (left, width) = if primary {
                let w = content_b.size.width.max(px(24.));
                let center = content_b.center().x;
                (center - w / 2. - origin, w)
            } else {
                (tab_b.origin.x - origin, tab_b.size.width)
            };
            let thickness = if primary { 3. } else { 2. };
            let spring = theme.motion.default_spatial;
            let id = self.id.clone();
            row = row.child(
                div()
                    .absolute()
                    .bottom_0()
                    .h(px(thickness))
                    .child(
                        div()
                            .h_full()
                            .bg(c.primary)
                            .map(|d| if primary { d.rounded_t(px(3.)) } else { d })
                            .with_spring(
                                ElementId::Name(format!("{id}::ind-w").into()),
                                SpringAnimation::new(spring).to(width),
                                |el, w| el.w(w.max(px(0.))),
                            ),
                    )
                    .with_spring(
                        ElementId::Name(format!("{}::ind-x", self.id).into()),
                        SpringAnimation::new(spring).to(left),
                        |el, x| el.left(x),
                    ),
            );
        }
        match divider_outer {
            Some(d) => div()
                .relative()
                .w_full()
                .child(row)
                .child(d)
                .into_any_element(),
            None => row.into_any_element(),
        }
    }
}
