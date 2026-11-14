//! Top app bar（Small / Center-aligned / Medium flexible / Large flexible）
//!
//! `ScrollHandle` を渡すと、スクロール量に応じて
//! - 背景色が surface → surface-container に変わる
//! - Medium / Large は大きなタイトルが縮んで上段に収まる

use gpui::{
    AnyElement, App, IntoElement, RenderOnce, ScrollHandle, SharedString, Window, div, prelude::*,
    px,
};

use super::{ActiveTheme, StyledExt, TypeScale};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarKind {
    #[default]
    Small,
    CenterAligned,
    Medium,
    Large,
}

impl AppBarKind {
    pub const ALL: [AppBarKind; 4] = [Self::Small, Self::CenterAligned, Self::Medium, Self::Large];
}

#[derive(IntoElement)]
pub struct TopAppBar {
    kind: AppBarKind,
    title: SharedString,
    subtitle: Option<SharedString>,
    navigation: Option<AnyElement>,
    actions: Vec<AnyElement>,
    scroll: Option<ScrollHandle>,
}

impl TopAppBar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            kind: AppBarKind::Small,
            title: title.into(),
            subtitle: None,
            navigation: None,
            actions: vec![],
            scroll: None,
        }
    }
    pub fn kind(mut self, kind: AppBarKind) -> Self {
        self.kind = kind;
        self
    }
    pub fn subtitle(mut self, s: impl Into<SharedString>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    /// 左端のナビゲーションアイコン（戻る・メニュー）
    pub fn navigation(mut self, el: impl IntoElement) -> Self {
        self.navigation = Some(el.into_any_element());
        self
    }
    pub fn action(mut self, el: impl IntoElement) -> Self {
        self.actions.push(el.into_any_element());
        self
    }
    /// 下のコンテンツの ScrollHandle（スクロール連動）
    pub fn scroll_handle(mut self, handle: &ScrollHandle) -> Self {
        self.scroll = Some(handle.clone());
        self
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl RenderOnce for TopAppBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let c = cx.m3().colors.clone();
        let scrolled: f32 = self
            .scroll
            .as_ref()
            .map(|h| -f32::from(h.offset().y))
            .unwrap_or(0.)
            .max(0.);

        let has_sub = self.subtitle.is_some();
        let collapsed_h: f32 = 64.;
        let expanded_h = match (self.kind, has_sub) {
            (AppBarKind::Medium, false) => 112.,
            (AppBarKind::Medium, true) => 136.,
            (AppBarKind::Large, false) => 120.,
            (AppBarKind::Large, true) => 152.,
            _ => collapsed_h,
        };
        let range = (expanded_h - collapsed_h).max(1.);
        let t = (scrolled / range).clamp(0., 1.);
        let flexible = matches!(self.kind, AppBarKind::Medium | AppBarKind::Large);
        let height = if flexible {
            lerp(expanded_h, collapsed_h, t)
        } else {
            collapsed_h
        };
        // スクロールで下に潜り込んだら色を変える
        let under = (scrolled / 8.).clamp(0., 1.);
        let bg = c.surface.blend(c.surface_container.opacity(under));

        let small_title_opacity = if flexible {
            ((t - 0.6) / 0.4).clamp(0., 1.)
        } else {
            1.
        };
        let small_title = |title: SharedString, subtitle: Option<SharedString>| {
            let mut d = div()
                .flex()
                .flex_col()
                .min_w_0()
                .opacity(small_title_opacity)
                .child(
                    div()
                        .m3_type(TypeScale::TitleLarge)
                        .text_color(c.on_surface)
                        .truncate()
                        .child(title),
                );
            if let Some(s) = subtitle {
                d = d.child(
                    div()
                        .m3_type(TypeScale::LabelMedium)
                        .text_color(c.on_surface_variant)
                        .truncate()
                        .child(s),
                );
            }
            d
        };

        let centered = self.kind == AppBarKind::CenterAligned;
        let top_sub = if flexible {
            None
        } else {
            self.subtitle.clone()
        };
        let mut top = div()
            .relative()
            .h(px(collapsed_h))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(4.))
            .px(px(4.));
        top = top.child(
            div()
                .w(px(48.))
                .flex()
                .justify_center()
                .children(self.navigation),
        );
        if centered {
            top = top.child(div().flex_1()).child(
                div()
                    .absolute()
                    .left(px(64.))
                    .right(px(64.))
                    .top_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(small_title(self.title.clone(), top_sub)),
            );
        } else {
            top = top.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(small_title(self.title.clone(), top_sub)),
            );
        }
        top = top.child(div().flex().items_center().children(self.actions));

        let mut bar = div()
            .w_full()
            .flex_none()
            .flex()
            .flex_col()
            .overflow_hidden()
            .h(px(height))
            .bg(bg)
            .child(top);

        if flexible {
            let (title_scale, sub_scale, pad_b) = match self.kind {
                AppBarKind::Large => (TypeScale::DisplaySmall, TypeScale::TitleMedium, 28.),
                _ => (TypeScale::HeadlineMedium, TypeScale::LabelLarge, 20.),
            };
            let big_opacity = (1. - t / 0.6).clamp(0., 1.);
            let mut big = div()
                .flex_1()
                .flex()
                .flex_col()
                .justify_end()
                .px(px(16.))
                .pb(px(pad_b * (1. - t)))
                .opacity(big_opacity)
                .child(
                    div()
                        .m3_type_emphasized(title_scale)
                        .text_color(c.on_surface)
                        .truncate()
                        .child(self.title),
                );
            if let Some(s) = self.subtitle {
                big = big.child(
                    div()
                        .m3_type(sub_scale)
                        .text_color(c.on_surface_variant)
                        .truncate()
                        .child(s),
                );
            }
            bar = bar.child(big);
        }
        bar
    }
}
