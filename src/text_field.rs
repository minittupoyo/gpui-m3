//! Text field（Filled / Outlined）
//!
//! IME・選択・クリップボード・Undo/Redo・単語移動・パスワード表示に対応した 1 行入力。
//! GPUI の `examples/input.rs` をベースに M3 の見た目とアニメーションを載せている。
//!
//! ```ignore
//! let name = cx.new(|cx| TextField::new(window, cx).label("名前").supporting_text("フルネーム"));
//! cx.subscribe(&name, |this, _, ev: &TextFieldEvent, cx| { ... }).detach();
//! ```

use std::{ops::Range, time::Duration};

use gpui::AnimationExt as _;
use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable, GlobalElementId, Hsla,
    KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad,
    Pixels, Point, ShapedLine, SharedString, SpringAnimation, Style, Subscription, TextRun,
    UTF16Selection, UnderlineStyle, Window, actions, div, fill, point, prelude::*, px, relative,
    size,
};
use unicode_segmentation::UnicodeSegmentation as _;

use super::{ActiveTheme, Icon, IconName, StyledExt, TypeScale, shape, state, transparent};

type NotchBackground = std::rc::Rc<dyn Fn(&super::ColorScheme) -> Hsla>;

actions!(
    m3_text_field,
    [
        Backspace,
        Delete,
        DeleteWordLeft,
        DeleteWordRight,
        Left,
        Right,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
        SelectAll,
        Home,
        End,
        SelectToHome,
        SelectToEnd,
        Paste,
        Cut,
        Copy,
        Undo,
        Redo,
        Submit,
        SubmitForce,
        Up,
        Down,
        SelectUp,
        SelectDown,
        DocStart,
        DocEnd,
    ]
);

const CONTEXT: &str = "M3TextField";
const BULLET: char = '•';

/// macOS は Cmd、それ以外は Ctrl
macro_rules! sec {
    ($k:literal) => {
        if cfg!(target_os = "macos") {
            concat!("cmd-", $k)
        } else {
            concat!("ctrl-", $k)
        }
    };
}

/// キーバインドを登録する（`gpui_m3::init` から呼ばれる）
pub(crate) fn bind_text_field_keys(cx: &mut App) {
    let c = Some(CONTEXT);

    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, c),
        KeyBinding::new("delete", Delete, c),
        KeyBinding::new("ctrl-backspace", DeleteWordLeft, c),
        KeyBinding::new("ctrl-delete", DeleteWordRight, c),
        KeyBinding::new("left", Left, c),
        KeyBinding::new("right", Right, c),
        KeyBinding::new("ctrl-left", WordLeft, c),
        KeyBinding::new("ctrl-right", WordRight, c),
        KeyBinding::new("shift-left", SelectLeft, c),
        KeyBinding::new("shift-right", SelectRight, c),
        KeyBinding::new("ctrl-shift-left", SelectWordLeft, c),
        KeyBinding::new("ctrl-shift-right", SelectWordRight, c),
        KeyBinding::new("home", Home, c),
        KeyBinding::new("end", End, c),
        KeyBinding::new("shift-home", SelectToHome, c),
        KeyBinding::new("shift-end", SelectToEnd, c),
        KeyBinding::new("enter", Submit, c),
        KeyBinding::new("shift-enter", Submit, c),
        KeyBinding::new("ctrl-enter", SubmitForce, c),
        KeyBinding::new("up", Up, c),
        KeyBinding::new("down", Down, c),
        KeyBinding::new("shift-up", SelectUp, c),
        KeyBinding::new("shift-down", SelectDown, c),
        KeyBinding::new("ctrl-home", DocStart, c),
        KeyBinding::new("ctrl-end", DocEnd, c),
        KeyBinding::new(sec!("a"), SelectAll, c),
        KeyBinding::new(sec!("v"), Paste, c),
        KeyBinding::new(sec!("c"), Copy, c),
        KeyBinding::new(sec!("x"), Cut, c),
        KeyBinding::new(sec!("z"), Undo, c),
        KeyBinding::new(sec!("y"), Redo, c),
        KeyBinding::new(sec!("shift-z"), Redo, c),
    ]);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldVariant {
    #[default]
    Filled,
    Outlined,
}

#[derive(Clone, Debug)]
pub enum TextFieldEvent {
    Changed(SharedString),
    Submitted(SharedString),
    Focused,
    Blurred,
}

pub struct TextField {
    focus_handle: FocusHandle,
    content: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    /// 複数行モードの直近レイアウト（論理行ごと）
    area_lines: Vec<gpui::WrappedLine>,
    area_scroll_y: Pixels,
    area_visual_rows: usize,
    preferred_x: Option<Pixels>,
    last_bounds: Option<Bounds<Pixels>>,
    scroll_x: Pixels,
    is_selecting: bool,
    undo_stack: Vec<(SharedString, Range<usize>)>,
    redo_stack: Vec<(SharedString, Range<usize>)>,
    cursor_visible: bool,
    blink_epoch: usize,
    reveal_password: bool,
    _subscriptions: Vec<Subscription>,

    // 設定
    variant: TextFieldVariant,
    label: Option<SharedString>,
    placeholder: Option<SharedString>,
    supporting_text: Option<SharedString>,
    error: Option<SharedString>,
    leading_icon: Option<IconName>,
    trailing_icon: Option<IconName>,
    clearable: bool,
    password: bool,
    max_length: Option<usize>,
    multiline: Option<(usize, Option<usize>)>,
    bare: bool,
    disabled: bool,
    notch_background: Option<NotchBackground>,
}

impl TextField {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle().tab_index(0).tab_stop(true);
        let subs = vec![
            cx.on_focus(&focus_handle, window, |this, _, cx| {
                this.start_blink(cx);
                cx.emit(TextFieldEvent::Focused);
                cx.notify();
            }),
            cx.on_blur(&focus_handle, window, |this, _, cx| {
                this.blink_epoch += 1;
                this.select_none();
                cx.emit(TextFieldEvent::Blurred);
                cx.notify();
            }),
        ];
        Self {
            focus_handle,
            content: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            area_lines: vec![],
            area_scroll_y: px(0.),
            area_visual_rows: 1,
            preferred_x: None,
            last_bounds: None,
            scroll_x: px(0.),
            is_selecting: false,
            undo_stack: vec![],
            redo_stack: vec![],
            cursor_visible: true,
            blink_epoch: 0,
            reveal_password: false,
            _subscriptions: subs,
            variant: TextFieldVariant::Filled,
            label: None,
            placeholder: None,
            supporting_text: None,
            error: None,
            leading_icon: None,
            trailing_icon: None,
            clearable: false,
            password: false,
            max_length: None,
            multiline: None,
            bare: false,
            disabled: false,
            notch_background: None,
        }
    }

    // ----- ビルダー -----

    pub fn filled(mut self) -> Self {
        self.variant = TextFieldVariant::Filled;
        self
    }
    pub fn outlined(mut self) -> Self {
        self.variant = TextFieldVariant::Outlined;
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }
    pub fn supporting_text(mut self, text: impl Into<SharedString>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }
    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }
    pub fn trailing_icon(mut self, icon: IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }
    /// 入力があるとき末尾にクリアボタンを出す
    pub fn clearable(mut self, v: bool) -> Self {
        self.clearable = v;
        self
    }
    /// パスワード入力（伏せ字 + 表示切替ボタン）
    pub fn password(mut self, v: bool) -> Self {
        self.password = v;
        self
    }
    /// 最大文字数（カウンターも表示）
    pub fn max_length(mut self, n: usize) -> Self {
        self.max_length = Some(n);
        self
    }
    /// 枠・ラベルなしで入力行だけを描画する（Search bar などへの埋め込み用）
    pub fn bare(mut self) -> Self {
        self.bare = true;
        self
    }
    /// 複数行入力（折り返しあり）。`min_lines` は最小表示行数
    pub fn multiline(mut self, min_lines: usize) -> Self {
        let max = self.multiline.and_then(|m| m.1);
        self.multiline = Some((min_lines.max(1), max));
        self
    }
    /// 複数行入力の最大表示行数（超えるとスクロール）
    pub fn max_lines(mut self, n: usize) -> Self {
        let min = self.multiline.map(|m| m.0).unwrap_or(1);
        self.multiline = Some((min, Some(n.max(min))));
        self
    }
    pub fn value(mut self, text: impl Into<SharedString>) -> Self {
        self.content = text.into();
        let end = self.content.len();
        self.selected_range = end..end;
        self
    }
    /// Outlined のラベルが枠線上に乗るときの背景色（親の背景色に合わせる）
    pub fn notch_background(
        mut self,
        color: impl Fn(&super::ColorScheme) -> Hsla + 'static,
    ) -> Self {
        self.notch_background = Some(std::rc::Rc::new(color));
        self
    }

    // ----- 外部から操作 -----

    pub fn text(&self) -> SharedString {
        self.content.clone()
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        let end = self.content.len();
        self.selected_range = end..end;
        self.marked_range = None;
        cx.emit(TextFieldEvent::Changed(self.content.clone()));
        cx.notify();
    }

    /// エラーメッセージを設定（None で解除）
    pub fn set_error(&mut self, error: Option<SharedString>, cx: &mut Context<Self>) {
        if self.error != error {
            self.error = error;
            cx.notify();
        }
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    // ----- カーソル点滅 -----

    fn start_blink(&mut self, cx: &mut Context<Self>) {
        self.blink_epoch += 1;
        self.cursor_visible = true;
        let epoch = self.blink_epoch;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(530))
                    .await;
                let keep = this
                    .update(cx, |t, cx| {
                        if t.blink_epoch != epoch {
                            return false;
                        }
                        t.cursor_visible = !t.cursor_visible;
                        cx.notify();
                        true
                    })
                    .unwrap_or(false);
                if !keep {
                    break;
                }
            }
        })
        .detach();
    }

    // ----- 編集アクション -----

    fn left(&mut self, _: &Left, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), window, cx);
        } else {
            self.move_to(self.selected_range.start, window, cx)
        }
    }
    fn right(&mut self, _: &Right, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), window, cx);
        } else {
            self.move_to(self.selected_range.end, window, cx)
        }
    }
    fn word_left(&mut self, _: &WordLeft, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.previous_word(self.cursor_offset()), window, cx);
    }
    fn word_right(&mut self, _: &WordRight, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.next_word(self.cursor_offset()), window, cx);
    }
    fn select_left(&mut self, _: &SelectLeft, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), window, cx);
    }
    fn select_right(&mut self, _: &SelectRight, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), window, cx);
    }
    fn select_word_left(
        &mut self,
        _: &SelectWordLeft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.previous_word(self.cursor_offset()), window, cx);
    }
    fn select_word_right(
        &mut self,
        _: &SelectWordRight,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.next_word(self.cursor_offset()), window, cx);
    }
    fn select_all(&mut self, _: &SelectAll, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx)
    }
    fn home(&mut self, _: &Home, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.home_target(), window, cx);
    }
    fn end(&mut self, _: &End, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.end_target(), window, cx);
    }
    fn select_to_home(&mut self, _: &SelectToHome, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.home_target(), window, cx);
    }
    fn select_to_end(&mut self, _: &SelectToEnd, window: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.end_target(), window, cx);
    }
    fn doc_start(&mut self, _: &DocStart, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
    }
    fn doc_end(&mut self, _: &DocEnd, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), window, cx);
    }
    fn up(&mut self, _: &Up, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline.is_none() {
            return self.move_to(0, window, cx);
        }
        let t = self.vertical_target(-1, window);
        self.move_to_keep_x(t, window, cx);
    }
    fn down(&mut self, _: &Down, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline.is_none() {
            return self.move_to(self.content.len(), window, cx);
        }
        let t = self.vertical_target(1, window);
        self.move_to_keep_x(t, window, cx);
    }
    fn select_up(&mut self, _: &SelectUp, window: &mut Window, cx: &mut Context<Self>) {
        let t = if self.multiline.is_some() {
            self.vertical_target(-1, window)
        } else {
            0
        };
        let keep = self.preferred_x;
        self.select_to(t, window, cx);
        self.preferred_x = keep;
    }
    fn select_down(&mut self, _: &SelectDown, window: &mut Window, cx: &mut Context<Self>) {
        let t = if self.multiline.is_some() {
            self.vertical_target(1, window)
        } else {
            self.content.len()
        };
        let keep = self.preferred_x;
        self.select_to(t, window, cx);
        self.preferred_x = keep;
    }
    fn move_to_keep_x(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        let keep = self.preferred_x;
        self.move_to(offset, window, cx);
        self.preferred_x = keep;
    }

    // ----- 複数行ヘルパー -----

    /// 現在の論理行の範囲
    fn logical_line(&self, offset: usize) -> (usize, usize) {
        let start = self.content[..offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = self.content[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(self.content.len());
        (start, end)
    }
    fn home_target(&self) -> usize {
        if self.multiline.is_some() {
            self.logical_line(self.cursor_offset()).0
        } else {
            0
        }
    }
    fn end_target(&self) -> usize {
        if self.multiline.is_some() {
            self.logical_line(self.cursor_offset()).1
        } else {
            self.content.len()
        }
    }
    /// オフセット → テキスト原点からの相対座標（スクロール前）
    fn area_point_for_index(&self, offset: usize, lh: Pixels) -> Point<Pixels> {
        let mut y = px(0.);
        let mut start = 0;
        for (i, line) in self.area_lines.iter().enumerate() {
            let end = start + line.len();
            let last = i + 1 == self.area_lines.len();
            if offset <= end || last {
                let local = offset.saturating_sub(start).min(line.len());
                let p = line.position_for_index(local, lh).unwrap_or_default();
                return point(p.x, y + p.y);
            }
            y += line.size(lh).height;
            start = end + 1;
        }
        point(px(0.), px(0.))
    }
    /// テキスト原点からの相対座標 → オフセット
    fn area_index_for_point(&self, p: Point<Pixels>, lh: Pixels) -> usize {
        let mut y = px(0.);
        let mut start = 0;
        if p.y < px(0.) {
            return 0;
        }
        for line in self.area_lines.iter() {
            let h = line.size(lh).height;
            if p.y < y + h {
                let local = line
                    .closest_index_for_position(point(p.x, p.y - y), lh)
                    .unwrap_or_else(|i| i);
                return start + local.min(line.len());
            }
            y += h;
            start += line.len() + 1;
        }
        self.content.len()
    }
    fn vertical_target(&mut self, dir: i32, window: &Window) -> usize {
        let lh = window.line_height();
        let cur = self.area_point_for_index(self.cursor_offset(), lh);
        let x = *self.preferred_x.get_or_insert(cur.x);
        let ty = cur.y + lh * dir as f32 + lh * 0.5;
        if ty < px(0.) {
            return 0;
        }
        self.area_index_for_point(point(x, ty), lh)
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            if self.cursor_offset() == prev {
                return;
            }
            self.select_to(prev, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }
    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            if self.cursor_offset() == next {
                return;
            }
            self.select_to(next, window, cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }
    fn delete_word_left(
        &mut self,
        _: &DeleteWordLeft,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_word(self.cursor_offset()), window, cx);
        }
        self.replace_text_in_range(None, "", window, cx)
    }
    fn delete_word_right(
        &mut self,
        _: &DeleteWordRight,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_word(self.cursor_offset()), window, cx);
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            let text = text.replace("\r\n", "\n").replace('\r', "\n");
            let text = if self.multiline.is_some() {
                text
            } else {
                text.replace('\n', " ")
            };
            self.replace_text_in_range(None, &text, window, cx);
        }
    }
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() && !self.is_obscured() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }
    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() && !self.is_obscured() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn undo(&mut self, _: &Undo, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((text, sel)) = self.undo_stack.pop() {
            self.redo_stack
                .push((self.content.clone(), self.selected_range.clone()));
            self.restore(text, sel, window, cx);
        }
    }
    fn redo(&mut self, _: &Redo, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((text, sel)) = self.redo_stack.pop() {
            self.undo_stack
                .push((self.content.clone(), self.selected_range.clone()));
            self.restore(text, sel, window, cx);
        }
    }
    fn restore(
        &mut self,
        text: SharedString,
        sel: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.content = text;
        self.selected_range = sel.start.min(self.content.len())..sel.end.min(self.content.len());
        self.marked_range = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.emit(TextFieldEvent::Changed(self.content.clone()));
        self.start_blink(cx);
        cx.notify();
    }

    fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        if self.multiline.is_some() {
            self.replace_text_in_range(None, "\n", window, cx);
        } else {
            cx.emit(TextFieldEvent::Submitted(self.content.clone()));
        }
    }
    fn submit_force(&mut self, _: &SubmitForce, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(TextFieldEvent::Submitted(self.content.clone()));
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, window, cx);
        self.select_to(self.content.len(), window, cx);
        self.replace_text_in_range(None, "", window, cx);
    }

    // ----- マウス -----

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        window.focus(&self.focus_handle, cx);
        self.is_selecting = true;
        let idx = self.index_for_mouse_position(event.position, window);
        self.preferred_x = None;
        match event.click_count {
            2 => {
                let (s, e) = self.word_range_at(idx);
                self.move_to(s, window, cx);
                self.select_to(e, window, cx);
            }
            n if n >= 3 => {
                self.move_to(0, window, cx);
                self.select_to(self.content.len(), window, cx);
            }
            _ if event.modifiers.shift => self.select_to(idx, window, cx),
            _ => self.move_to(idx, window, cx),
        }
    }
    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }
    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(
                self.index_for_mouse_position(event.position, window),
                window,
                cx,
            );
        }
    }

    // ----- 内部ヘルパー -----

    fn is_obscured(&self) -> bool {
        self.password && !self.reveal_password
    }

    fn select_none(&mut self) {
        let c = self.cursor_offset();
        self.selected_range = c..c;
        self.selection_reversed = false;
    }

    fn move_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        self.preferred_x = None;
        window.notify_text_selection_changed(&self.focus_handle);
        self.start_blink(cx);
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn select_to(&mut self, offset: usize, window: &Window, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        self.preferred_x = None;
        window.notify_text_selection_changed(&self.focus_handle);
        self.start_blink(cx);
        cx.notify()
    }

    /// 内容のバイト位置 → 表示文字列のバイト位置（伏せ字対応）
    fn to_display(&self, offset: usize) -> usize {
        if self.is_obscured() {
            self.content[..offset.min(self.content.len())]
                .chars()
                .count()
                * BULLET.len_utf8()
        } else {
            offset
        }
    }

    fn display_to_offset(&self, offset: usize) -> usize {
        if self.is_obscured() {
            let n = offset / BULLET.len_utf8();
            self.content
                .char_indices()
                .nth(n)
                .map(|(i, _)| i)
                .unwrap_or(self.content.len())
        } else {
            offset
        }
    }

    fn display_text(&self) -> SharedString {
        if self.is_obscured() {
            BULLET
                .to_string()
                .repeat(self.content.chars().count())
                .into()
        } else {
            self.content.clone()
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>, window: &Window) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        if self.multiline.is_some() {
            let Some(bounds) = self.last_bounds.as_ref() else {
                return 0;
            };
            let rel = point(
                position.x - bounds.left(),
                position.y - bounds.top() + self.area_scroll_y,
            );
            return self.area_index_for_point(rel, window.line_height());
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.x < bounds.left() && self.scroll_x <= px(0.) {
            return 0;
        }
        self.display_to_offset(line.closest_index_for_x(position.x - bounds.left() + self.scroll_x))
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    fn previous_word(&self, offset: usize) -> usize {
        if self.is_obscured() {
            return 0;
        }
        self.content
            .split_word_bound_indices()
            .rev()
            .find(|(idx, w)| *idx < offset && !w.trim().is_empty())
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    fn next_word(&self, offset: usize) -> usize {
        if self.is_obscured() {
            return self.content.len();
        }
        self.content
            .split_word_bound_indices()
            .find(|(idx, w)| idx + w.len() > offset && !w.trim().is_empty())
            .map(|(idx, w)| idx + w.len())
            .unwrap_or(self.content.len())
    }

    fn word_range_at(&self, offset: usize) -> (usize, usize) {
        if self.is_obscured() {
            return (0, self.content.len());
        }
        self.content
            .split_word_bound_indices()
            .find(|(idx, w)| offset >= *idx && offset <= idx + w.len())
            .map(|(idx, w)| (idx, idx + w.len()))
            .unwrap_or((offset, offset))
    }

    /// 最大文字数を超えないよう挿入テキストを切り詰める
    fn clamp_insert<'a>(&self, range: &Range<usize>, new_text: &'a str) -> &'a str {
        let Some(max) = self.max_length else {
            return new_text;
        };
        let remaining_chars =
            self.content.chars().count() - self.content[range.clone()].chars().count();
        let allowed = max.saturating_sub(remaining_chars);
        match new_text.char_indices().nth(allowed) {
            Some((i, _)) => &new_text[..i],
            None => new_text,
        }
    }
}

impl TextField {
    fn bind_actions<E: InteractiveElement>(root: E, cx: &mut Context<Self>) -> E {
        root.on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word_left))
            .on_action(cx.listener(Self::delete_word_right))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_word_left))
            .on_action(cx.listener(Self::select_word_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_to_home))
            .on_action(cx.listener(Self::select_to_end))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::undo))
            .on_action(cx.listener(Self::redo))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::submit_force))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::select_up))
            .on_action(cx.listener(Self::select_down))
            .on_action(cx.listener(Self::doc_start))
            .on_action(cx.listener(Self::doc_end))
    }
}

impl EventEmitter<TextFieldEvent> for TextField {}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn set_selected_text_range(
        &mut self,
        range_utf16: Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_range = self.range_from_utf16(&range_utf16);
        self.selection_reversed = false;
        window.notify_text_selection_changed(&self.focus_handle);
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.content.encode_utf16().count())
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        window.notify_text_input_changed(&self.focus_handle);
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());
        let new_text = self.clamp_insert(&range, new_text);

        // IME 確定時（marked あり）は変換前の状態を履歴に積まない
        if self.marked_range.is_none() {
            self.undo_stack
                .push((self.content.clone(), self.selected_range.clone()));
            if self.undo_stack.len() > 200 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.selection_reversed = false;
        self.marked_range.take();
        window.notify_text_input_changed(&self.focus_handle);
        cx.emit(TextFieldEvent::Changed(self.content.clone()));
        self.start_blink(cx);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        if self.marked_range.is_none() {
            self.undo_stack
                .push((self.content.clone(), self.selected_range.clone()));
            self.redo_stack.clear();
        }

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.marked_range =
            (!new_text.is_empty()).then(|| range.start..range.start + new_text.len());
        let insertion_start_utf16 = self.offset_to_utf16(range.start);
        let inserted_length_utf16 = new_text.encode_utf16().count();
        self.selected_range = new_selected_range_utf16
            .map(|selected| {
                self.range_from_utf16(
                    &(insertion_start_utf16 + selected.start.min(inserted_length_utf16)
                        ..insertion_start_utf16 + selected.end.min(inserted_length_utf16)),
                )
            })
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.selection_reversed = false;
        window.notify_text_input_changed(&self.focus_handle);
        self.start_blink(cx);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        if self.multiline.is_some() {
            let lh = window.line_height();
            let range = self.range_from_utf16(&range_utf16);
            let a = self.area_point_for_index(range.start, lh);
            let b = self.area_point_for_index(range.end, lh);
            let origin = point(bounds.left(), bounds.top() - self.area_scroll_y);
            return Some(Bounds::from_corners(
                point(origin.x + a.x, origin.y + a.y),
                point(origin.x + b.x, origin.y + b.y + lh),
            ));
        }
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let left = bounds.left() - self.scroll_x;
        Some(Bounds::from_corners(
            point(
                left + last_layout.x_for_index(self.to_display(range.start)),
                bounds.top(),
            ),
            point(
                left + last_layout.x_for_index(self.to_display(range.end)),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        if self.multiline.is_some() {
            let idx = self.index_for_mouse_position(point, window);
            return Some(self.offset_to_utf16(idx));
        }
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        let idx =
            self.display_to_offset(last_layout.closest_index_for_x(line_point.x + self.scroll_x));
        Some(self.offset_to_utf16(idx))
    }
}

// ---------------------------------------------------------------------------
// テキスト描画エレメント
// ---------------------------------------------------------------------------

struct TextLine {
    input: Entity<TextField>,
    text_color: Hsla,
    placeholder_color: Hsla,
    cursor_color: Hsla,
    selection_color: Hsla,
    show_placeholder: bool,
}

struct LinePrepaint {
    line: ShapedLine,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
    scroll_x: Pixels,
}

impl Element for TextLine {
    type RequestLayoutState = ();
    type PrepaintState = LinePrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let style = window.text_style();
        let content_empty = input.content.is_empty();

        let (display_text, color) = if content_empty {
            let ph = if self.show_placeholder {
                input.placeholder.clone().unwrap_or_default()
            } else {
                SharedString::default()
            };
            (ph, self.placeholder_color)
        } else {
            (input.display_text(), self.text_color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = match input.marked_range.as_ref() {
            Some(marked) if !content_empty => {
                let (ms, me) = (input.to_display(marked.start), input.to_display(marked.end));
                vec![
                    TextRun {
                        len: ms,
                        ..run.clone()
                    },
                    TextRun {
                        len: me - ms,
                        underline: Some(UnderlineStyle {
                            color: Some(run.color),
                            thickness: px(1.),
                            wavy: false,
                        }),
                        ..run.clone()
                    },
                    TextRun {
                        len: display_text.len() - me,
                        ..run
                    },
                ]
                .into_iter()
                .filter(|r| r.len > 0)
                .collect()
            }
            _ => vec![run],
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let (sel_start, sel_end, cursor_idx) = if content_empty {
            (0, 0, 0)
        } else {
            (
                input.to_display(input.selected_range.start),
                input.to_display(input.selected_range.end),
                input.to_display(input.cursor_offset()),
            )
        };

        // 横スクロール: カーソルが常に見えるように
        let width = bounds.size.width;
        let cursor_x = if content_empty {
            px(0.)
        } else {
            line.x_for_index(cursor_idx)
        };
        let mut scroll = input.scroll_x;
        if cursor_x - scroll > width - px(2.) {
            scroll = cursor_x - width + px(2.);
        }
        if cursor_x - scroll < px(0.) {
            scroll = cursor_x;
        }
        let max_scroll = (line.width - width + px(2.)).max(px(0.));
        if content_empty {
            scroll = px(0.);
        }
        scroll = scroll.min(max_scroll).max(px(0.));

        let left = bounds.left() - scroll;
        let (selection, cursor) = if input.selected_range.is_empty() || content_empty {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(left + cursor_x, bounds.top()),
                        size(px(2.), bounds.size.height),
                    ),
                    self.cursor_color,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(left + line.x_for_index(sel_start), bounds.top()),
                        point(left + line.x_for_index(sel_end), bounds.bottom()),
                    ),
                    self.selection_color,
                )),
                None,
            )
        };
        LinePrepaint {
            line,
            cursor,
            selection,
            scroll_x: scroll,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (focus_handle, cursor_visible, disabled) = {
            let i = self.input.read(cx);
            (i.focus_handle.clone(), i.cursor_visible, i.disabled)
        };
        if !disabled {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }
        let focused = focus_handle.is_focused(window);
        let line = prepaint.line.clone();
        let scroll = prepaint.scroll_x;
        let selection = prepaint.selection.take();
        let cursor = prepaint.cursor.take();
        // 少しはみ出させて、カーソルが端で切れないようにする
        let mask = Bounds::new(
            point(bounds.left() - px(2.), bounds.top() - px(4.)),
            size(bounds.size.width + px(4.), bounds.size.height + px(8.)),
        );
        window.with_content_mask(Some(ContentMask { bounds: mask }), |window| {
            if let Some(selection) = selection {
                window.paint_quad(selection)
            }
            line.paint(
                point(bounds.left() - scroll, bounds.top()),
                window.line_height(),
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();
            if focused
                && cursor_visible
                && let Some(cursor) = cursor
            {
                window.paint_quad(cursor);
            }
        });

        let line = prepaint.line.clone();
        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
            input.scroll_x = scroll;
        });
    }
}

// ---------------------------------------------------------------------------
// 複数行テキスト描画エレメント
// ---------------------------------------------------------------------------

struct TextArea {
    input: Entity<TextField>,
    text_color: Hsla,
    placeholder_color: Hsla,
    cursor_color: Hsla,
    selection_color: Hsla,
    show_placeholder: bool,
}

struct AreaPrepaint {
    lines: Vec<gpui::WrappedLine>,
    placeholder: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selections: Vec<PaintQuad>,
    scroll_y: Pixels,
    rows: usize,
}

impl TextField {
    /// 表示行数（min..=max にクランプ）
    fn area_rows(&self) -> usize {
        let (min, max) = self.multiline.unwrap_or((1, None));
        let rows = self.area_visual_rows.max(min);
        max.map(|m| rows.min(m)).unwrap_or(rows)
    }
}

impl Element for TextArea {
    type RequestLayoutState = ();
    type PrepaintState = AreaPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let rows = self.input.read(cx).area_rows();
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (window.line_height() * rows as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let style = window.text_style();
        let lh = window.line_height();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let content_empty = input.content.is_empty();
        let text: SharedString = input.content.clone();
        let wrap = (bounds.size.width - px(2.)).max(px(10.));

        let base = TextRun {
            len: text.len(),
            font: style.font(),
            color: self.text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let runs = match input.marked_range.as_ref() {
            Some(m) if !content_empty => vec![
                TextRun {
                    len: m.start,
                    ..base.clone()
                },
                TextRun {
                    len: m.end - m.start,
                    underline: Some(UnderlineStyle {
                        color: Some(base.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..base.clone()
                },
                TextRun {
                    len: text.len() - m.end,
                    ..base.clone()
                },
            ]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect(),
            _ => vec![base.clone()],
        };
        let lines: Vec<gpui::WrappedLine> = window
            .text_system()
            .shape_text(text, font_size, &runs, Some(wrap), None)
            .map(|v| v.into_vec())
            .unwrap_or_default();

        let placeholder = if content_empty && self.show_placeholder {
            input.placeholder.clone().map(|ph| {
                let run = TextRun {
                    len: ph.len(),
                    color: self.placeholder_color,
                    ..base.clone()
                };
                window.text_system().shape_line(ph, font_size, &[run], None)
            })
        } else {
            None
        };

        let rows: usize = lines
            .iter()
            .map(|l| l.wrap_boundaries().len() + 1)
            .sum::<usize>()
            .max(1);
        let total_h = lh * rows as f32;

        let cursor_pt = lines_point_for_index(&lines, input.cursor_offset(), lh);
        let mut scroll = input.area_scroll_y;
        if cursor_pt.y < scroll {
            scroll = cursor_pt.y;
        }
        if cursor_pt.y + lh > scroll + bounds.size.height {
            scroll = cursor_pt.y + lh - bounds.size.height;
        }
        scroll = scroll
            .min((total_h - bounds.size.height).max(px(0.)))
            .max(px(0.));

        let origin = point(bounds.left(), bounds.top() - scroll);
        let mut selections = vec![];
        let mut cursor = None;
        if input.selected_range.is_empty() {
            cursor = Some(fill(
                Bounds::new(
                    point(origin.x + cursor_pt.x, origin.y + cursor_pt.y),
                    size(px(2.), lh),
                ),
                self.cursor_color,
            ));
        } else {
            let sel = &input.selected_range;
            let mut y = px(0.);
            let mut start = 0usize;
            for line in &lines {
                let len = line.len();
                let (a, b) = (sel.start.max(start), sel.end.min(start + len));
                if a <= b && sel.start <= start + len && sel.end >= start {
                    let a_l = a.saturating_sub(start).min(len);
                    let b_l = b.saturating_sub(start).min(len);
                    // 改行を含んで選択されている行は末尾に少し余白を足す
                    let extra = if sel.end > start + len {
                        px(6.)
                    } else {
                        px(0.)
                    };
                    let p0 = line.position_for_index(a_l, lh).unwrap_or_default();
                    let p1 = line.position_for_index(b_l, lh).unwrap_or_default();
                    let r0 = (p0.y / lh) as usize;
                    let r1 = (p1.y / lh) as usize;
                    for r in r0..=r1 {
                        let x0 = if r == r0 { p0.x } else { px(0.) };
                        let x1 = if r == r1 {
                            p1.x + if r == line.wrap_boundaries().len() {
                                extra
                            } else {
                                px(0.)
                            }
                        } else {
                            wrap
                        };
                        if x1 > x0 {
                            let ry = y + lh * r as f32;
                            selections.push(fill(
                                Bounds::from_corners(
                                    point(origin.x + x0, origin.y + ry),
                                    point(origin.x + x1, origin.y + ry + lh),
                                ),
                                self.selection_color,
                            ));
                        }
                    }
                }
                y += line.size(lh).height;
                start += len + 1;
            }
        }
        AreaPrepaint {
            lines,
            placeholder,
            cursor,
            selections,
            scroll_y: scroll,
            rows,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (focus_handle, cursor_visible, disabled) = {
            let i = self.input.read(cx);
            (i.focus_handle.clone(), i.cursor_visible, i.disabled)
        };
        if !disabled {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }
        let focused = focus_handle.is_focused(window);
        let lh = window.line_height();
        let scroll = prepaint.scroll_y;
        let mask = Bounds::new(
            point(bounds.left() - px(2.), bounds.top()),
            size(bounds.size.width + px(4.), bounds.size.height),
        );
        let selections = std::mem::take(&mut prepaint.selections);
        let cursor = prepaint.cursor.take();
        window.with_content_mask(Some(ContentMask { bounds: mask }), |window| {
            for q in selections {
                window.paint_quad(q);
            }
            let mut y = bounds.top() - scroll;
            for line in &prepaint.lines {
                line.paint(
                    point(bounds.left(), y),
                    lh,
                    gpui::TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .ok();
                y += line.size(lh).height;
            }
            if let Some(ph) = &prepaint.placeholder {
                ph.paint(
                    point(bounds.left(), bounds.top()),
                    lh,
                    gpui::TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .ok();
            }
            if focused
                && cursor_visible
                && let Some(c) = cursor
            {
                window.paint_quad(c);
            }
        });

        let lines = std::mem::take(&mut prepaint.lines);
        let rows = prepaint.rows;
        self.input.update(cx, |input, cx| {
            input.area_lines = lines;
            input.area_scroll_y = scroll;
            input.last_bounds = Some(bounds);
            if input.area_visual_rows != rows {
                input.area_visual_rows = rows;
                cx.notify();
            }
        });
    }
}

fn lines_point_for_index(lines: &[gpui::WrappedLine], offset: usize, lh: Pixels) -> Point<Pixels> {
    let mut y = px(0.);
    let mut start = 0;
    for (i, line) in lines.iter().enumerate() {
        let end = start + line.len();
        let last = i + 1 == lines.len();
        if offset <= end || last {
            let local = offset.saturating_sub(start).min(line.len());
            let p = line.position_for_index(local, lh).unwrap_or_default();
            return point(p.x, y + p.y);
        }
        y += line.size(lh).height;
        start = end + 1;
    }
    point(px(0.), px(0.))
}

// ---------------------------------------------------------------------------
// 見た目
// ---------------------------------------------------------------------------

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let focused = self.focus_handle.is_focused(window);
        let has_text = !self.content.is_empty();
        let has_error = self.error.is_some();
        let disabled = self.disabled;
        let outlined = self.variant == TextFieldVariant::Outlined;
        let id = self.focus_handle.clone();
        let eid = |s: &str| ElementId::Name(format!("{:?}::{s}", cx.entity_id()).into());

        // 色
        let accent = if has_error { c.error } else { c.primary };
        let (indicator, indicator_w) = match (disabled, focused, has_error) {
            (true, _, _) => (c.on_surface.opacity(state::DISABLED_CONTENT), 1.),
            (_, true, _) => (accent, 2.),
            (_, false, true) => (c.error, 1.),
            _ => (
                if outlined {
                    c.outline
                } else {
                    c.on_surface_variant
                },
                1.,
            ),
        };
        let label_color = match (disabled, focused, has_error) {
            (true, _, _) => theme.disabled_content(),
            (_, _, true) => c.error,
            (_, true, _) => c.primary,
            _ => c.on_surface_variant,
        };
        let text_color = if disabled {
            theme.disabled_content()
        } else {
            c.on_surface
        };
        let icon_color = if disabled {
            theme.disabled_content()
        } else {
            c.on_surface_variant
        };
        let container_bg = if outlined {
            transparent()
        } else if disabled {
            c.on_surface.opacity(0.04)
        } else {
            c.surface_container_highest
        };

        if self.bare {
            let mut el = div()
                .relative()
                .h(px(24.))
                .w_full()
                .m3_type(TypeScale::BodyLarge)
                .text_color(text_color)
                .child(div().absolute().inset_0().child(TextLine {
                    input: cx.entity(),
                    text_color,
                    placeholder_color: c.on_surface_variant,
                    cursor_color: accent,
                    selection_color: accent.opacity(0.32),
                    show_placeholder: true,
                }));
            if !disabled {
                el = el
                    .cursor(CursorStyle::IBeam)
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                    .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
                    .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
                    .on_mouse_move(cx.listener(Self::on_mouse_move));
            }
            return Self::bind_actions(div().key_context(CONTEXT).track_focus(&id), cx)
                .flex_1()
                .min_w_0()
                .child(el)
                .into_any_element();
        }

        // レイアウト
        let has_label = self.label.is_some();
        let text_left = if self.leading_icon.is_some() {
            52.
        } else {
            16.
        };
        let trailing_count = [
            self.trailing_icon.is_some(),
            self.clearable && has_text && !disabled,
            self.password,
            has_error && self.trailing_icon.is_none() && !self.password && !self.clearable,
        ]
        .iter()
        .filter(|b| **b)
        .count() as f32;
        let text_right = if trailing_count > 0. {
            4. + 40. * trailing_count + 4.
        } else {
            16.
        };
        let floating = focused || has_text;
        let text_top = if has_label && !outlined { 24. } else { 16. };

        // テキスト行
        let multiline = self.multiline.is_some();
        let rows = self.area_rows();
        let text_h = if multiline { 24. * rows as f32 } else { 24. };
        let line = div()
            .absolute()
            .left(px(text_left))
            .right(px(text_right))
            .top(px(text_top))
            .h(px(text_h))
            .map(|d| {
                if multiline {
                    d.child(TextArea {
                        input: cx.entity(),
                        text_color,
                        placeholder_color: c.on_surface_variant,
                        cursor_color: accent,
                        selection_color: accent.opacity(0.32),
                        show_placeholder: !has_label || focused,
                    })
                } else {
                    d.child(TextLine {
                        input: cx.entity(),
                        text_color,
                        placeholder_color: c.on_surface_variant,
                        cursor_color: accent,
                        selection_color: accent.opacity(0.32),
                        show_placeholder: !has_label || focused,
                    })
                }
            });

        let mut container = div()
            .id(eid("container"))
            .relative()
            .h(px(if multiline {
                text_top + text_h + 16.
            } else {
                56.
            }))
            .w_full()
            .bg(container_bg)
            .text_color(text_color)
            .m3_type(TypeScale::BodyLarge)
            .child(line);

        if outlined {
            let group: SharedString = format!("{:?}::tf", cx.entity_id()).into();
            let mut outline = div()
                .absolute()
                .inset_0()
                .rounded(shape::EXTRA_SMALL)
                .border_color(indicator)
                .map(|d| {
                    if indicator_w > 1. {
                        d.border_2()
                    } else {
                        d.border_1()
                    }
                });
            if !disabled && !focused {
                let hover = if has_error {
                    c.on_error_container
                } else {
                    c.on_surface
                };
                outline = outline.group_hover(group.clone(), move |s| s.border_color(hover));
            }
            container = container
                .group(group)
                .rounded(shape::EXTRA_SMALL)
                .child(outline);
        } else {
            container = container
                .rounded_t(shape::EXTRA_SMALL)
                .overflow_hidden()
                .child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .h(px(indicator_w))
                        .bg(indicator),
                );
            if !disabled {
                let hover = state::HOVER;
                let on = c.on_surface;
                container = container.hover(move |s| s.bg(container_bg.blend(on.opacity(hover))));
            }
        }

        // フローティングラベル
        if let Some(label) = self.label.clone() {
            let notch_bg = self
                .notch_background
                .as_ref()
                .map(|f| f(&c))
                .unwrap_or(c.surface);
            let float_top = if outlined { -8. } else { 8. };
            let float_left = if outlined { 12. } else { text_left };
            let rest_left = text_left - if outlined { 4. } else { 0. };
            let mut label_el = div()
                .absolute()
                .text_color(label_color)
                .whitespace_nowrap()
                .child(label);
            if outlined {
                label_el = label_el.px(px(4.));
            }
            container = container.child(label_el.with_spring(
                eid("label"),
                SpringAnimation::new(theme.motion.fast_spatial).to(if floating { 1. } else { 0. }),
                move |el, t| {
                    let t = t.clamp(-0.1, 1.1);
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
        }

        // アイコン
        if let Some(icon) = self.leading_icon {
            container = container.child(
                div()
                    .absolute()
                    .left(px(12.))
                    .top(px(16.))
                    .child(Icon::new(icon).size(px(24.)).color(icon_color)),
            );
        }
        let icon_button = |id: ElementId, icon: IconName, color: Hsla| {
            div()
                .id(id)
                .size(px(40.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(color.opacity(state::HOVER)))
                .child(Icon::new(icon).size(px(24.)).color(color))
        };
        let mut trailing = div()
            .absolute()
            .right(px(4.))
            .top(px(8.))
            .flex()
            .items_center();
        if self.clearable && has_text && !disabled {
            trailing = trailing.child(
                icon_button(eid("clear"), IconName::Cancel, icon_color)
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_click(cx.listener(|this, _, window, cx| this.clear(window, cx))),
            );
        }
        if self.password {
            let icon = if self.reveal_password {
                IconName::VisibilityOff
            } else {
                IconName::Visibility
            };
            trailing = trailing.child(
                icon_button(eid("reveal"), icon, icon_color)
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reveal_password = !this.reveal_password;
                        cx.notify();
                    })),
            );
        }
        if let Some(icon) = self.trailing_icon {
            trailing = trailing.child(
                div()
                    .size(px(40.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(icon).size(px(24.)).color(icon_color)),
            );
        } else if has_error && !self.password && !self.clearable {
            trailing = trailing.child(
                div()
                    .size(px(40.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::new(IconName::Error).size(px(24.)).color(c.error)),
            );
        }
        container = container.child(trailing);

        if !disabled {
            container = container
                .cursor(CursorStyle::IBeam)
                .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
                .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
                .on_mouse_move(cx.listener(Self::on_mouse_move));
        }

        // 補助テキスト / カウンター
        let support = self.error.clone().or(self.supporting_text.clone());
        let counter = self
            .max_length
            .map(|m| format!("{}/{}", self.content.chars().count(), m));
        let support_color = if has_error {
            c.error
        } else if disabled {
            theme.disabled_content()
        } else {
            c.on_surface_variant
        };
        let mut root = Self::bind_actions(div().key_context(CONTEXT).track_focus(&id), cx)
            .flex()
            .flex_col()
            .w_full()
            .child(container);
        if support.is_some() || counter.is_some() {
            root = root.child(
                div()
                    .flex()
                    .gap(px(16.))
                    .px(px(16.))
                    .pt(px(4.))
                    .m3_type(TypeScale::BodySmall)
                    .text_color(support_color)
                    .child(div().flex_1().children(support))
                    .children(counter),
            );
        }
        root.into_any_element()
    }
}
