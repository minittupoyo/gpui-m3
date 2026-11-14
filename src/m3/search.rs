//! Search bar / Search view
//!
//! 入力部は `TextField::bare()` を埋め込む。
//!
//! ```ignore
//! let field = search_field(window, cx, "検索");
//! SearchBar::new("sb", field.clone()).trailing(IconButton::new("mic", IconName::Mic))
//! SearchView::new("sv", field).on_back(|_, _, cx| ...).child(ListItem::new(..))
//! ```

use gpui::{
    AnyElement, App, ClickEvent, Entity, Focusable as _, IntoElement, ParentElement, RenderOnce,
    SharedString, Window, div, prelude::*, px,
};

use super::{
    ActiveTheme, ClickHandler, Divider, Icon, IconButton, IconName, StyledExt, TextField,
    TypeScale, shape,
};

/// Search bar / view 用の枠なし入力欄を作る
pub fn search_field(
    window: &mut Window,
    cx: &mut App,
    placeholder: impl Into<SharedString>,
) -> Entity<TextField> {
    let placeholder = placeholder.into();
    cx.new(|cx| TextField::new(window, cx).bare().placeholder(placeholder))
}

/// 閉じた状態の検索バー（高さ 56dp の全丸ピル）
#[derive(IntoElement)]
pub struct SearchBar {
    id: SharedString,
    field: Entity<TextField>,
    leading: IconName,
    trailing: Vec<AnyElement>,
    on_leading: Option<ClickHandler>,
}

impl SearchBar {
    pub fn new(id: impl Into<SharedString>, field: Entity<TextField>) -> Self {
        Self {
            id: id.into(),
            field,
            leading: IconName::Search,
            trailing: vec![],
            on_leading: None,
        }
    }
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading = icon;
        self
    }
    /// 先頭アイコンをボタンとして扱う（メニューや戻るなど）
    pub fn on_leading_click(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_leading = Some(std::rc::Rc::new(f));
        self
    }
    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing.push(el.into_any_element());
        self
    }
}

impl RenderOnce for SearchBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let focus = self.field.focus_handle(cx);
        let focused = focus.is_focused(window);
        let leading: AnyElement = match self.on_leading {
            Some(h) => IconButton::new(format!("{}-lead", self.id), self.leading)
                .on_click(move |e, w, cx| h(e, w, cx))
                .into_any_element(),
            None => div()
                .size(px(40.))
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(self.leading).size(px(24.)).color(c.on_surface))
                .into_any_element(),
        };
        div()
            .id(gpui::ElementId::Name(self.id))
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(56.))
            .w_full()
            .px(px(8.))
            .rounded(shape::FULL)
            .bg(c.surface_container_high)
            .when(focused, |d| d.m3_elevation(3, c.shadow))
            .text_color(c.on_surface)
            .m3_type(TypeScale::BodyLarge)
            .cursor_text()
            .on_click(move |_, window, cx| window.focus(&focus, cx))
            .child(leading)
            .child(self.field)
            .children(self.trailing)
    }
}

/// 展開した検索ビュー（ヘッダー + 結果リスト）
#[derive(IntoElement)]
pub struct SearchView {
    id: SharedString,
    field: Entity<TextField>,
    docked: bool,
    on_back: Option<ClickHandler>,
    trailing: Vec<AnyElement>,
    children: Vec<AnyElement>,
    max_height: Option<f32>,
}

impl SearchView {
    pub fn new(id: impl Into<SharedString>, field: Entity<TextField>) -> Self {
        Self {
            id: id.into(),
            field,
            docked: true,
            on_back: None,
            trailing: vec![],
            children: vec![],
            max_height: None,
        }
    }
    /// ドッキング（角丸 28dp）か、全画面（角丸なし）か
    pub fn docked(mut self, v: bool) -> Self {
        self.docked = v;
        self
    }
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_height = Some(h);
        self
    }
    pub fn on_back(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(std::rc::Rc::new(f));
        self
    }
    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing.push(el.into_any_element());
        self
    }
}

impl ParentElement for SearchView {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SearchView {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let has_text = !self.field.read(cx).text().is_empty();
        let field = self.field.clone();
        let mut back = IconButton::new(format!("{}-back", self.id), IconName::ArrowBack);
        if let Some(h) = self.on_back {
            back = back.on_click(move |e, w, cx| h(e, w, cx));
        }
        let mut header = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(72.))
            .px(px(8.))
            .flex_none()
            .child(back)
            .child(self.field);
        if has_text {
            header = header.child(
                IconButton::new(format!("{}-clear", self.id), IconName::Close).on_click(
                    move |_, _, cx| {
                        field.update(cx, |f, cx| f.set_text("", cx));
                    },
                ),
            );
        }
        header = header.children(self.trailing);

        let mut results = div()
            .id(gpui::ElementId::Name(format!("{}-results", self.id).into()))
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .py(px(8.))
            .children(self.children);
        results = results.text_color(c.on_surface);

        div()
            .id(gpui::ElementId::Name(self.id))
            .flex()
            .flex_col()
            .w_full()
            .when_some(self.max_height, |d, h| d.max_h(px(h)))
            .overflow_hidden()
            .bg(c.surface_container_high)
            .map(|d| if self.docked { d.rounded(shape::EXTRA_LARGE) } else { d })
            .m3_type(TypeScale::BodyLarge)
            .child(header)
            .child(Divider::horizontal())
            .child(results)
    }
}
