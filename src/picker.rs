//! Date picker / Time picker（状態はホスト側が持つ制御コンポーネント）
//!
//! ```ignore
//! DatePicker::new("dp", Date::new(2026, 11, 14)).year_month(y, m)
//!     .on_select(|d, _, cx| ...).on_month_change(|(y, m), _, cx| ...)
//! TimePicker::new("tp", hour, minute).mode(TimeMode::Hour)
//!     .on_change(|(h, m, done), _, cx| ...).on_mode(|mode, _, cx| ...)
//! ```

use std::{
    cell::{Cell, RefCell},
    f32::consts::TAU,
    rc::Rc,
};

use gpui::{
    App, Bounds, Context, ElementId, Entity, Hsla, IntoElement, MouseButton, Pixels, RenderOnce,
    ScrollHandle, SharedString, Subscription, Window, canvas, div, point, prelude::*, px,
};

use super::{
    ActiveTheme, ChangeHandler, Divider, Icon, IconButton, IconName, StyledExt, TextField,
    TextFieldEvent, TypeScale, shape, state,
};

/// 入力欄のイベントなど、`Window` を持たない場所からハンドラを呼ぶ
fn call_later(cx: &mut App, f: impl FnOnce(&mut Window, &mut App) + 'static) {
    if let Some(w) = cx.active_window() {
        cx.defer(move |cx| {
            w.update(cx, |_, window, cx| f(window, cx)).ok();
        });
    }
}

// ---------------------------------------------------------------------------
// 日付計算
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }
    /// 0 = 日曜
    pub fn weekday(self) -> u32 {
        weekday(self.year, self.month, self.day)
    }
    /// 入力欄用の「2026/11/14」
    pub fn format_input(self) -> String {
        format!("{:04}/{:02}/{:02}", self.year, self.month, self.day)
    }
    /// 「2026年11月14日（土）」
    pub fn format_long(self) -> String {
        const W: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];
        format!(
            "{}年{}月{}日（{}）",
            self.year,
            self.month,
            self.day,
            W[self.weekday() as usize]
        )
    }
    /// 「11月14日（土）」
    pub fn format_short(self) -> String {
        const W: [&str; 7] = ["日", "月", "火", "水", "木", "金", "土"];
        format!(
            "{}月{}日（{}）",
            self.month,
            self.day,
            W[self.weekday() as usize]
        )
    }
}

/// 範囲選択の状態。`pick` で「開始 → 終了」の順にクリックを反映する
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateRange {
    pub start: Option<Date>,
    pub end: Option<Date>,
}

impl DateRange {
    pub fn new(start: Option<Date>, end: Option<Date>) -> Self {
        Self { start, end }
    }
    /// 日付をクリックしたときの次の状態。
    /// 未選択・選択済み（開始と終了が揃っている）なら新しい開始日、開始のみなら終了日（前後は自動で入れ替え）
    pub fn pick(self, d: Date) -> Self {
        match (self.start, self.end) {
            (Some(s), None) => {
                if d < s {
                    Self {
                        start: Some(d),
                        end: Some(s),
                    }
                } else {
                    Self {
                        start: Some(s),
                        end: Some(d),
                    }
                }
            }
            _ => Self {
                start: Some(d),
                end: None,
            },
        }
    }
    /// 日数（両端を含む）。終了が未定なら 1
    pub fn days(self) -> Option<u32> {
        let (s, e) = (self.start?, self.end.unwrap_or(self.start?));
        Some((days_from_civil(e) - days_from_civil(s)) as u32 + 1)
    }
    /// 「11月3日 – 11月9日」（曜日なし。見出し用）
    pub fn format_compact(self) -> String {
        match (self.start, self.end) {
            (Some(s), Some(e)) => format!("{}月{}日 – {}月{}日", s.month, s.day, e.month, e.day),
            (Some(s), None) => format!("{}月{}日 –", s.month, s.day),
            _ => String::new(),
        }
    }
    /// 「11月3日（火） – 11月9日（月）」
    pub fn format_short(self) -> String {
        match (self.start, self.end) {
            (Some(s), Some(e)) => format!("{} – {}", s.format_short(), e.format_short()),
            (Some(s), None) => format!("{} –", s.format_short()),
            _ => String::new(),
        }
    }
}

/// 1970-01-01 からの通算日数（グレゴリオ暦）
fn days_from_civil(d: Date) -> i64 {
    let y = if d.month <= 2 { d.year - 1 } else { d.year } as i64;
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = d.month as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d.day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// 「2026/11/14」「2026-11-14」「2026.11.14」「2026年11月14日」を解釈する。存在しない日付は None
pub fn parse_date(text: &str) -> Option<Date> {
    let groups: Vec<&str> = text
        .split(|c: char| !c.is_ascii_digit())
        .filter(|g| !g.is_empty())
        .collect();
    if groups.len() != 3 || groups[0].len() != 4 {
        return None;
    }
    let (y, m, d) = (
        groups[0].parse::<i32>().ok()?,
        groups[1].parse::<u32>().ok()?,
        groups[2].parse::<u32>().ok()?,
    );
    if !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
        return None;
    }
    Some(Date::new(y, m, d))
}

pub fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap_year(y) {
                29
            } else {
                28
            }
        }
    }
}

/// 0 = 日曜（Sakamoto の方法）
pub fn weekday(y: i32, m: u32, d: u32) -> u32 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if m < 3 { y - 1 } else { y };
    ((y + y / 4 - y / 100 + y / 400 + T[(m - 1) as usize] + d as i32).rem_euclid(7)) as u32
}

fn shift_month(y: i32, m: u32, delta: i32) -> (i32, u32) {
    let idx = y * 12 + (m as i32 - 1) + delta;
    (idx.div_euclid(12), (idx.rem_euclid(12) + 1) as u32)
}

// ---------------------------------------------------------------------------
// Date picker
// ---------------------------------------------------------------------------

#[derive(IntoElement)]
pub struct DatePicker {
    id: SharedString,
    year: i32,
    month: u32,
    selected: Option<Date>,
    today: Option<Date>,
    modal: bool,
    min: Option<Date>,
    max: Option<Date>,
    range: Option<DateRange>,
    year_range: (i32, i32),
    on_select: Option<ChangeHandler<Date>>,
    on_month_change: Option<ChangeHandler<(i32, u32)>>,
}

impl DatePicker {
    /// `selected` の月を表示する
    pub fn new(id: impl Into<SharedString>, selected: Date) -> Self {
        Self {
            id: id.into(),
            year: selected.year,
            month: selected.month,
            selected: Some(selected),
            today: None,
            modal: false,
            min: None,
            max: None,
            range: None,
            year_range: (1900, 2100),
            on_select: None,
            on_month_change: None,
        }
    }
    /// 範囲選択モード。クリックのたびに `on_select` が呼ばれるので、`DateRange::pick` で状態を更新する
    pub fn range(mut self, range: DateRange) -> Self {
        self.range = Some(range);
        self.selected = None;
        self
    }
    /// 年選択で選べる年の範囲（既定 1900〜2100）
    pub fn year_range(mut self, min: i32, max: i32) -> Self {
        self.year_range = (min, max.max(min));
        self
    }
    /// 表示する年月（月送りで変える）
    pub fn year_month(mut self, year: i32, month: u32) -> Self {
        self.year = year;
        self.month = month.clamp(1, 12);
        self
    }
    pub fn selected(mut self, d: Option<Date>) -> Self {
        self.selected = d;
        self
    }
    pub fn today(mut self, d: Date) -> Self {
        self.today = Some(d);
        self
    }
    /// モーダル形式（見出し + 選択日の表示付き）
    pub fn modal(mut self, v: bool) -> Self {
        self.modal = v;
        self
    }
    pub fn min(mut self, d: Date) -> Self {
        self.min = Some(d);
        self
    }
    pub fn max(mut self, d: Date) -> Self {
        self.max = Some(d);
        self
    }
    pub fn on_select(mut self, f: impl Fn(Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
    pub fn on_month_change(
        mut self,
        f: impl Fn((i32, u32), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_month_change = Some(Rc::new(f));
        self
    }
}

struct DateShared {
    selected: Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
    on_select: Option<ChangeHandler<Date>>,
    on_month: Option<ChangeHandler<(i32, u32)>>,
}

/// 入力モードの状態（テキスト欄とハンドラを保持する）
struct DateInput {
    on: bool,
    field: Entity<TextField>,
    shared: Rc<RefCell<DateShared>>,
    synced: Option<Option<Date>>,
    _sub: Subscription,
}

impl DateInput {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let field = cx.new(|cx| {
            TextField::new(window, cx)
                .outlined()
                .label("日付")
                .placeholder("yyyy/mm/dd")
                .supporting_text("例: 2026/11/14")
                .notch_background(|c| c.surface_container_high)
        });
        let shared = Rc::new(RefCell::new(DateShared {
            selected: None,
            min: None,
            max: None,
            on_select: None,
            on_month: None,
        }));
        let sh = shared.clone();
        let sub = cx.subscribe(&field, move |_this, field, ev: &TextFieldEvent, cx| {
            let text = match ev {
                TextFieldEvent::Changed(t) => t.clone(),
                TextFieldEvent::Blurred => field.read(cx).text(),
                _ => return,
            };
            let blurred = matches!(ev, TextFieldEvent::Blurred);
            if text.trim().is_empty() {
                field.update(cx, |f, cx| f.set_error(None, cx));
                return;
            }
            let Some(date) = parse_date(&text) else {
                // 入力途中で警告が出ないよう、フォーカスが外れたときだけ知らせる
                if blurred {
                    field.update(cx, |f, cx| {
                        f.set_error(
                            Some("日付の形式が正しくありません（yyyy/mm/dd）".into()),
                            cx,
                        )
                    });
                }
                return;
            };
            let (out_of_range, changed, on_select, on_month) = {
                let s = sh.borrow();
                (
                    s.min.is_some_and(|m| date < m) || s.max.is_some_and(|m| date > m),
                    s.selected != Some(date),
                    s.on_select.clone(),
                    s.on_month.clone(),
                )
            };
            field.update(cx, |f, cx| {
                f.set_error(out_of_range.then(|| "選択できない日付です".into()), cx)
            });
            if out_of_range || !changed {
                return;
            }
            call_later(cx, move |w, cx| {
                if let Some(h) = &on_month {
                    h((date.year, date.month), w, cx);
                }
                if let Some(h) = &on_select {
                    h(date, w, cx);
                }
            });
        });
        Self {
            on: false,
            field,
            shared,
            synced: None,
            _sub: sub,
        }
    }
}

struct YearView {
    open: bool,
    /// 開いた直後に選択中の年へスクロールする
    pending: bool,
    scroll: ScrollHandle,
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let year_state = window.use_keyed_state(
            SharedString::from(format!("{}::yearview", self.id)),
            cx,
            |_, _| YearView {
                open: false,
                pending: false,
                scroll: ScrollHandle::new(),
            },
        );
        let (year_open, year_scroll) = {
            let s = year_state.read(cx);
            (s.open, s.scroll.clone())
        };
        // 入力モード（単一日付のモーダルのみ）
        let input_state = window.use_keyed_state(
            SharedString::from(format!("{}::dateinput", self.id)),
            cx,
            |window, cx| DateInput::new(window, cx),
        );
        let can_input = self.modal && self.range.is_none();
        let input_on = can_input && input_state.read(cx).on;
        if can_input {
            let st = input_state.read(cx);
            *st.shared.borrow_mut() = DateShared {
                selected: self.selected,
                min: self.min,
                max: self.max,
                on_select: self.on_select.clone(),
                on_month: self.on_month_change.clone(),
            };
        }
        let input_field = input_state.read(cx).field.clone();
        if input_on {
            // 選択中の日付を入力欄に反映する（入力中は書き換えない）
            let focused = input_field.read(cx).is_focused(window);
            let need = input_state.read(cx).synced != Some(self.selected);
            // 解釈できない文字列が入っている間は、警告を残すため書き換えない
            let text = input_field.read(cx).text();
            let invalid = !text.trim().is_empty() && parse_date(&text).is_none();
            if need && !focused && !invalid {
                input_state.update(cx, |s, _| s.synced = Some(self.selected));
                let txt = self.selected.map(|d| d.format_input()).unwrap_or_default();
                input_field.update(cx, |f, cx| f.set_text(txt, cx));
            }
        }
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let (y, m) = (self.year, self.month);
        let first = weekday(y, m, 1) as usize;
        let dim = days_in_month(y, m);

        let nav = |dir: i32, icon: IconName, id: String| {
            let h = self.on_month_change.clone();
            IconButton::new(id, icon).on_click(move |_, w, cx| {
                if let Some(h) = &h {
                    h(shift_month(y, m, dir), w, cx);
                }
            })
        };
        let toggle = year_state.clone();
        let title = div()
            .id(ElementId::Name(format!("{}-title", self.id).into()))
            .flex()
            .items_center()
            .gap(px(2.))
            .h(px(40.))
            .pl(px(4.))
            .pr(px(6.))
            .rounded(shape::FULL)
            .cursor_pointer()
            .hover(move |s| s.bg(c.on_surface_variant.opacity(state::HOVER)))
            .m3_type(TypeScale::LabelLarge)
            .text_color(c.on_surface_variant)
            .child(format!("{y}年{m}月"))
            .child(
                Icon::new(if year_open {
                    IconName::ArrowDropUp
                } else {
                    IconName::ArrowDropDown
                })
                .size(px(24.))
                .color(c.on_surface_variant),
            )
            .on_click(move |_, _, cx| {
                toggle.update(cx, |s, cx| {
                    s.open = !s.open;
                    s.pending = s.open;
                    cx.notify();
                })
            });
        let title_row = div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(48.))
            .pl(px(8.))
            .child(title)
            .child(div().flex().pr(px(12.)).when(!year_open, |d| {
                d.child(nav(-1, IconName::ChevronLeft, format!("{}-prev", self.id)))
                    .child(nav(1, IconName::ChevronRight, format!("{}-next", self.id)))
            }));

        let mut week = div().flex().px(px(12.));
        for w in ["日", "月", "火", "水", "木", "金", "土"] {
            week = week.child(
                div()
                    .size(px(40.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .m3_type(TypeScale::BodyLarge)
                    .text_color(c.on_surface)
                    .child(w),
            );
        }

        let mut grid = div().flex().flex_col().px(px(12.)).pb(px(12.));
        let mut day = 1i32 - first as i32;
        for row in 0..6 {
            if day > dim as i32 {
                break;
            }
            let mut r = div().flex();
            for col in 0..7 {
                let d = day;
                day += 1;
                if d < 1 || d > dim as i32 {
                    r = r.child(div().size(px(40.)));
                    continue;
                }
                let date = Date::new(y, m, d as u32);
                // 範囲選択: 両端と、その間を結ぶ帯
                let (lo, hi) = match self.range {
                    Some(DateRange {
                        start: Some(s),
                        end,
                    }) => (Some(s), Some(end.unwrap_or(s))),
                    _ => (None, None),
                };
                let is_edge = lo == Some(date) || hi == Some(date);
                let multi = lo.is_some() && lo != hi;
                let in_band = multi && lo.is_some_and(|l| date > l) && hi.is_some_and(|h| date < h);
                let band_l = multi && (lo == Some(date) || (in_band && (col == 0 || d == 1)));
                let band_r =
                    multi && (hi == Some(date) || (in_band && (col == 6 || d == dim as i32)));
                let band = multi && (in_band || is_edge);
                let is_sel = self.selected == Some(date) || (self.range.is_some() && is_edge);
                let is_today = self.today == Some(date);
                let disabled =
                    self.min.is_some_and(|mn| date < mn) || self.max.is_some_and(|mx| date > mx);
                let (bg, fg): (Option<Hsla>, Hsla) = if is_sel {
                    (Some(c.primary), c.on_primary)
                } else if in_band {
                    (None, c.on_secondary_container)
                } else if is_today {
                    (None, c.primary)
                } else {
                    (None, c.on_surface)
                };
                let h = self.on_select.clone();
                let mut cell = div()
                    .id(ElementId::Name(format!("{}-d{row}{col}", self.id).into()))
                    .size(px(40.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .m3_type(TypeScale::BodyLarge)
                    .text_color(if disabled {
                        theme.disabled_content()
                    } else {
                        fg
                    })
                    .when_some(bg, |d, b| d.bg(b))
                    .when(is_today && !is_sel, |d| {
                        d.border_1().border_color(c.outline)
                    })
                    .child(d.to_string());
                if !disabled {
                    let hover = if is_sel { c.on_primary } else { c.on_surface };
                    cell = cell
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover.opacity(state::HOVER).blend_over(bg)))
                        .on_click(move |_, w, cx| {
                            if let Some(h) = &h {
                                h(date, w, cx);
                            }
                        });
                }
                // 帯は隣のセルと隙間なくつなぐ（両端は半円）
                let band_bg = c.secondary_container;
                let wrap = div()
                    .size(px(40.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(band, |d| d.bg(band_bg))
                    .when(band && band_l && !band_r, |d| d.rounded_l(shape::FULL))
                    .when(band && band_r && !band_l, |d| d.rounded_r(shape::FULL))
                    .when(band && band_l && band_r, |d| d.rounded(shape::FULL))
                    .child(cell);
                r = r.child(wrap);
            }
            grid = grid.child(r);
        }

        // ----- 年選択 -----
        let (y_min, y_max) = self.year_range;
        let years: Vec<i32> = (y_min..=y_max).collect();
        let year_body = {
            let mut list = div()
                .id(ElementId::Name(format!("{}-years", self.id).into()))
                .track_scroll(&year_scroll)
                .h(px(40. * 7. + 12.))
                .overflow_y_scroll()
                .px(px(12.))
                .pb(px(12.));
            for (ri, chunk) in years.chunks(3).enumerate() {
                let mut row = div().flex().items_center().justify_around().h(px(52.));
                for &yr in chunk {
                    let selected = yr == y;
                    let is_now = self.today.is_some_and(|t| t.year == yr);
                    let h = self.on_month_change.clone();
                    let close = year_state.clone();
                    row = row.child(
                        div()
                            .id(ElementId::Name(format!("{}-y{yr}", self.id).into()))
                            .w(px(72.))
                            .h(px(36.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(shape::FULL)
                            .m3_type(TypeScale::BodyLarge)
                            .cursor_pointer()
                            .when(selected, |d| d.bg(c.primary).text_color(c.on_primary))
                            .when(!selected && is_now, |d| {
                                d.border_1().border_color(c.outline).text_color(c.primary)
                            })
                            .when(!selected, |d| {
                                d.hover(move |s| s.bg(c.on_surface.opacity(state::HOVER)))
                            })
                            .child(yr.to_string())
                            .on_click(move |_, w, cx| {
                                if let Some(h) = &h {
                                    h((yr, m), w, cx);
                                }
                                close.update(cx, |s, cx| {
                                    s.open = false;
                                    cx.notify();
                                });
                            }),
                    );
                }
                let _ = ri;
                list = list.child(row);
            }
            list
        };
        if year_open {
            let pending = year_state.update(cx, |s, _| std::mem::take(&mut s.pending));
            if pending {
                // 選択中の年が上から 2 行目あたりに来るようにスクロールする
                let row = ((y - y_min).max(0) / 3) as f32;
                let off = ((row - 1.).max(0.)) * 52.;
                year_scroll.set_offset(point(px(0.), px(-off)));
            }
        }

        let mut root = div()
            .w(px(if self.modal { 360. } else { 328. }))
            .flex()
            .flex_col()
            .bg(c.surface_container_high)
            .rounded(shape::EXTRA_LARGE)
            .overflow_hidden()
            .text_color(c.on_surface);
        if self.modal {
            root = root
                .child(
                    div()
                        .px(px(24.))
                        .pt(px(16.))
                        .child(
                            div()
                                .m3_type(TypeScale::LabelLarge)
                                .text_color(c.on_surface_variant)
                                .child(if self.range.is_some() {
                                    "期間を選択"
                                } else {
                                    "日付を選択"
                                }),
                        )
                        .child(
                            div()
                                .h(px(52.))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .m3_type(if self.range.is_some() {
                                            TypeScale::HeadlineSmall
                                        } else {
                                            TypeScale::HeadlineLarge
                                        })
                                        .child(match self.range {
                                            Some(r) if r.start.is_some() => r.format_compact(),
                                            Some(_) => "開始日 – 終了日".into(),
                                            None => self
                                                .selected
                                                .map(|d| d.format_short())
                                                .unwrap_or_else(|| "日付を選択".into()),
                                        }),
                                )
                                .when(can_input, |d| {
                                    let st = input_state.clone();
                                    d.child(
                                        IconButton::new(
                                            format!("{}-mode", self.id),
                                            if input_on {
                                                IconName::CalendarMonth
                                            } else {
                                                IconName::Keyboard
                                            },
                                        )
                                        .on_click(
                                            move |_, _, cx| {
                                                st.update(cx, |s, cx| {
                                                    s.on = !s.on;
                                                    s.synced = None;
                                                    let f = s.field.clone();
                                                    f.update(cx, |f, cx| f.set_text("", cx));
                                                    cx.notify();
                                                })
                                            },
                                        ),
                                    )
                                }),
                        ),
                )
                .child(Divider::horizontal());
        }
        if input_on {
            return root
                .child(div().px(px(24.)).pt(px(24.)).pb(px(24.)).child(input_field))
                .into_any_element();
        }
        root = root.child(title_row);
        if year_open {
            root.child(year_body).into_any_element()
        } else {
            root.child(week).child(grid).into_any_element()
        }
    }
}

trait BlendOver {
    fn blend_over(self, base: Option<Hsla>) -> Hsla;
}
impl BlendOver for Hsla {
    fn blend_over(self, base: Option<Hsla>) -> Hsla {
        match base {
            Some(b) => b.blend(self),
            None => self,
        }
    }
}

// ---------------------------------------------------------------------------
// Time picker
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeMode {
    Hour,
    Minute,
}

#[derive(IntoElement)]
pub struct TimePicker {
    id: SharedString,
    hour: u32,
    minute: u32,
    mode: TimeMode,
    is_24h: bool,
    on_change: Option<ChangeHandler<(u32, u32, bool)>>,
    on_mode: Option<ChangeHandler<TimeMode>>,
}

impl TimePicker {
    pub fn new(id: impl Into<SharedString>, hour: u32, minute: u32) -> Self {
        Self {
            id: id.into(),
            hour: hour % 24,
            minute: minute % 60,
            mode: TimeMode::Hour,
            is_24h: false,
            on_change: None,
            on_mode: None,
        }
    }
    pub fn mode(mut self, mode: TimeMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn is_24h(mut self, v: bool) -> Self {
        self.is_24h = v;
        self
    }
    /// `(時, 分, 操作完了)` — 完了はマウスを離したとき true
    pub fn on_change(
        mut self,
        f: impl Fn((u32, u32, bool), &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
    pub fn on_mode(mut self, f: impl Fn(TimeMode, &mut Window, &mut App) + 'static) -> Self {
        self.on_mode = Some(Rc::new(f));
        self
    }
}

struct TimeShared {
    hour: u32,
    minute: u32,
    is_24h: bool,
    on_change: Option<ChangeHandler<(u32, u32, bool)>>,
}

/// 入力モードの状態（時・分のテキスト欄とハンドラ）
struct TimeInput {
    on: bool,
    hour_f: Entity<TextField>,
    min_f: Entity<TextField>,
    shared: Rc<RefCell<TimeShared>>,
    synced_h: Option<(u32, bool)>,
    synced_m: Option<u32>,
    _subs: Vec<Subscription>,
}

impl TimeInput {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut mk = |label: &'static str| {
            cx.new(|cx| {
                TextField::new(window, cx)
                    .outlined()
                    .label(label)
                    .notch_background(|c| c.surface_container_high)
            })
        };
        let hour_f = mk("時");
        let min_f = mk("分");
        let shared = Rc::new(RefCell::new(TimeShared {
            hour: 0,
            minute: 0,
            is_24h: false,
            on_change: None,
        }));
        let mut subs = vec![];
        for (is_hour, field) in [(true, hour_f.clone()), (false, min_f.clone())] {
            let sh = shared.clone();
            subs.push(
                cx.subscribe(&field, move |_this, field, ev: &TextFieldEvent, cx| {
                    let TextFieldEvent::Changed(text) = ev else {
                        return;
                    };
                    // 数字以外は取り除く
                    let clean: String = text
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(2)
                        .collect();
                    if clean != text.as_ref() {
                        field.update(cx, |f, cx| f.set_text(clean, cx));
                        return;
                    }
                    if clean.is_empty() {
                        field.update(cx, |f, cx| f.set_error(None, cx));
                        return;
                    }
                    let n: u32 = clean.parse().unwrap_or(99);
                    let (hour, minute, is_24h, on_change) = {
                        let s = sh.borrow();
                        (s.hour, s.minute, s.is_24h, s.on_change.clone())
                    };
                    let is_pm = hour >= 12;
                    let (valid, msg, new_h, new_m) = if is_hour {
                        if is_24h {
                            (n <= 23, "0〜23", n, minute)
                        } else {
                            (
                                (1..=12).contains(&n),
                                "1〜12",
                                n % 12 + if is_pm { 12 } else { 0 },
                                minute,
                            )
                        }
                    } else {
                        (n <= 59, "0〜59", hour, n)
                    };
                    field.update(cx, |f, cx| f.set_error((!valid).then(|| msg.into()), cx));
                    if valid && (new_h, new_m) != (hour, minute) {
                        call_later(cx, move |w, cx| {
                            if let Some(h) = &on_change {
                                h((new_h, new_m, true), w, cx);
                            }
                        });
                    }
                }),
            );
        }
        Self {
            on: false,
            hour_f,
            min_f,
            shared,
            synced_h: None,
            synced_m: None,
            _subs: subs,
        }
    }
}

const DIAL: f32 = 256.;
const R_OUTER: f32 = 101.;
const R_INNER: f32 = 66.;

impl RenderOnce for TimePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let input_state = window.use_keyed_state(
            SharedString::from(format!("{}::timeinput", self.id)),
            cx,
            |window, cx| TimeInput::new(window, cx),
        );
        let input_on = input_state.read(cx).on;
        {
            let st = input_state.read(cx);
            *st.shared.borrow_mut() = TimeShared {
                hour: self.hour,
                minute: self.minute,
                is_24h: self.is_24h,
                on_change: self.on_change.clone(),
            };
        }
        let (hour_f, min_f) = {
            let st = input_state.read(cx);
            (st.hour_f.clone(), st.min_f.clone())
        };
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let is_pm = self.hour >= 12;
        let shown_h = if self.is_24h {
            self.hour
        } else if self.hour % 12 == 0 {
            12
        } else {
            self.hour % 12
        };
        let (hour, minute, is_24h, mode) = (self.hour, self.minute, self.is_24h, self.mode);

        if input_on {
            // 入力中の欄は書き換えず、外から値が変わったときだけ反映する
            let (sh, sm) = {
                let st = input_state.read(cx);
                (st.synced_h, st.synced_m)
            };
            if sh != Some((hour, is_24h)) && !hour_f.read(cx).is_focused(window) {
                input_state.update(cx, |s, _| s.synced_h = Some((hour, is_24h)));
                hour_f.update(cx, |f, cx| f.set_text(format!("{shown_h:02}"), cx));
            }
            if sm != Some(minute) && !min_f.read(cx).is_focused(window) {
                input_state.update(cx, |s, _| s.synced_m = Some(minute));
                min_f.update(cx, |f, cx| f.set_text(format!("{minute:02}"), cx));
            }
        }

        // ----- 表示部 -----
        let time_box = |label: String, active: bool, id: String, target: TimeMode| {
            let h = self.on_mode.clone();
            div()
                .id(ElementId::Name(id.into()))
                .w(px(96.))
                .h(px(80.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(shape::SMALL)
                .bg(if active {
                    c.primary_container
                } else {
                    c.surface_container_highest
                })
                .text_color(if active {
                    c.on_primary_container
                } else {
                    c.on_surface
                })
                .m3_type(TypeScale::DisplayLarge)
                .cursor_pointer()
                .child(label)
                .on_click(move |_, w, cx| {
                    if let Some(h) = &h {
                        h(target, w, cx);
                    }
                })
        };
        let set_ampm = |pm: bool, label: &'static str, id: String| {
            let h = self.on_change.clone();
            let active = pm == is_pm;
            div()
                .id(ElementId::Name(id.into()))
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .bg(if active {
                    c.tertiary_container
                } else {
                    transparent_c()
                })
                .text_color(if active {
                    c.on_tertiary_container
                } else {
                    c.on_surface_variant
                })
                .m3_type(TypeScale::TitleMedium)
                .cursor_pointer()
                .child(label)
                .on_click(move |_, w, cx| {
                    if let Some(h) = &h {
                        let nh = (hour % 12) + if pm { 12 } else { 0 };
                        h((nh, minute, true), w, cx);
                    }
                })
        };
        let display = div()
            .flex()
            .items_center()
            .gap(px(0.))
            .child(time_box(
                format!("{shown_h:02}"),
                mode == TimeMode::Hour,
                format!("{}-hbox", self.id),
                TimeMode::Hour,
            ))
            .child(
                div()
                    .w(px(24.))
                    .h(px(80.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .m3_type(TypeScale::DisplayLarge)
                    .child(":"),
            )
            .child(time_box(
                format!("{minute:02}"),
                mode == TimeMode::Minute,
                format!("{}-mbox", self.id),
                TimeMode::Minute,
            ));
        let ampm_block = (!is_24h).then(|| {
            div()
                .ml(px(12.))
                .w(px(52.))
                .h(px(80.))
                .flex()
                .flex_none()
                .flex_col()
                .rounded(shape::SMALL)
                .border_1()
                .border_color(c.outline)
                .overflow_hidden()
                .child(set_ampm(false, "AM", format!("{}-am", self.id)))
                .child(div().h(px(1.)).bg(c.outline))
                .child(set_ampm(true, "PM", format!("{}-pm", self.id)))
        });
        let input_display = div()
            .flex()
            .items_start()
            .child(div().w(px(96.)).child(hour_f))
            .child(
                div()
                    .w(px(24.))
                    .h(px(56.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .m3_type(TypeScale::HeadlineLarge)
                    .child(":"),
            )
            .child(div().w(px(96.)).child(min_f));

        // ----- ダイヤル -----
        // (値, 位置の角度 0..1, 半径)
        let items: Vec<(String, u32, f32, f32)> = match mode {
            TimeMode::Hour if is_24h => (0..12)
                .map(|i| {
                    let outer = if i == 0 { 12 } else { i };
                    let inner = if i == 0 { 0 } else { 12 + i };
                    [(outer, R_OUTER), (inner, R_INNER)]
                        .into_iter()
                        .map(move |(v, r)| (format!("{v:02}"), v, i as f32 / 12., r))
                        .collect::<Vec<_>>()
                })
                .flatten()
                .collect(),
            TimeMode::Hour => (0..12)
                .map(|i| {
                    let v = if i == 0 { 12 } else { i };
                    (v.to_string(), v, i as f32 / 12., R_OUTER)
                })
                .collect(),
            TimeMode::Minute => (0..12)
                .map(|i| (format!("{:02}", i * 5), i * 5, i as f32 / 12., R_OUTER))
                .collect(),
        };

        // 選択位置
        let (sel_frac, sel_r) = match mode {
            TimeMode::Minute => (minute as f32 / 60., R_OUTER),
            TimeMode::Hour => {
                let i = (hour % 12) as f32 / 12.;
                let r = if is_24h && (hour == 0 || hour > 12) {
                    R_INNER
                } else {
                    R_OUTER
                };
                (i, r)
            }
        };
        let half = DIAL / 2.;
        let pos = |frac: f32, r: f32| {
            let a = frac * TAU;
            (half + r * a.sin(), half - r * a.cos())
        };
        let (sx, sy) = pos(sel_frac, sel_r);
        let sel_color = c.primary;
        let dial_bg = c.surface_container_highest;
        let on_primary = c.on_primary;
        let on_surface = c.on_surface;

        let bounds_cell: Rc<Cell<Bounds<Pixels>>> = Rc::new(Cell::new(Bounds::default()));
        let bc = bounds_cell.clone();
        let paint_sel = canvas(
            move |b, _, _| bc.set(b),
            move |b, _, window, _| {
                let o = b.origin;
                let center = point(o.x + px(half), o.y + px(half));
                let target = point(o.x + px(sx), o.y + px(sy));
                super::progress::paint_polyline(window, &[center, target], 2., sel_color);
                // 選択円
                let mut path = gpui::PathBuilder::fill();
                let steps = 48;
                for s in 0..=steps {
                    let a = s as f32 / steps as f32 * TAU;
                    let p = point(target.x + px(24. * a.cos()), target.y + px(24. * a.sin()));
                    if s == 0 {
                        path.move_to(p);
                    } else {
                        path.line_to(p);
                    }
                }
                path.close();
                if let Ok(path) = path.build() {
                    window.paint_path(path, sel_color);
                }
            },
        )
        .absolute()
        .size_full();

        let on_change = self.on_change.clone();
        let on_mode = self.on_mode.clone();
        let compute = {
            let bounds_cell = bounds_cell.clone();
            move |p: gpui::Point<Pixels>| -> (u32, u32) {
                let b = bounds_cell.get();
                let cx_ = b.origin.x + b.size.width / 2.;
                let cy_ = b.origin.y + b.size.height / 2.;
                let dx = f32::from(p.x - cx_);
                let dy = f32::from(p.y - cy_);
                let mut a = dx.atan2(-dy);
                if a < 0. {
                    a += TAU;
                }
                let dist = (dx * dx + dy * dy).sqrt();
                match mode {
                    TimeMode::Minute => (hour, ((a / TAU * 60.).round() as u32) % 60),
                    TimeMode::Hour => {
                        let i = ((a / TAU * 12.).round() as u32) % 12;
                        let h = if is_24h {
                            if dist < (R_INNER + R_OUTER) / 2. {
                                if i == 0 { 0 } else { 12 + i }
                            } else if i == 0 {
                                12
                            } else {
                                i
                            }
                        } else {
                            i + if is_pm { 12 } else { 0 }
                        };
                        (h, minute)
                    }
                }
            }
        };
        let compute = Rc::new(compute);

        let mut dial = div()
            .id(ElementId::Name(format!("{}-dial", self.id).into()))
            .relative()
            .size(px(DIAL))
            .rounded_full()
            .bg(dial_bg)
            .cursor_pointer()
            .child(paint_sel);
        for (label, v, frac, r) in items {
            let (x, y) = pos(frac, r);
            let selected = match mode {
                TimeMode::Hour => {
                    v == hour % if is_24h { 24 } else { 12 }
                        || (!is_24h && v == 12 && hour % 12 == 0)
                }
                TimeMode::Minute => v == minute,
            };
            dial = dial.child(
                div()
                    .absolute()
                    .left(px(x - 24.))
                    .top(px(y - 24.))
                    .size(px(48.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .m3_type(if r < R_OUTER {
                        TypeScale::BodyLarge
                    } else {
                        TypeScale::BodyLarge
                    })
                    .text_color(if selected { on_primary } else { on_surface })
                    .child(label),
            );
        }
        // 中心の点
        dial = dial.child(
            div()
                .absolute()
                .left(px(half - 4.))
                .top(px(half - 4.))
                .size(px(8.))
                .rounded_full()
                .bg(sel_color),
        );
        {
            let (oc, cm, cc) = (on_change.clone(), compute.clone(), compute.clone());
            let (oc2, oc3) = (on_change.clone(), on_change.clone());
            let (cm2, cm3) = (cm.clone(), cm.clone());
            let _ = (cc, on_mode);
            dial = dial
                .on_mouse_down(MouseButton::Left, move |e, w, cx| {
                    if let Some(h) = &oc {
                        let (hh, mm) = cm(e.position);
                        h((hh, mm, false), w, cx);
                    }
                })
                .on_mouse_move(move |e, w, cx| {
                    if e.pressed_button == Some(MouseButton::Left) {
                        if let Some(h) = &oc2 {
                            let (hh, mm) = cm2(e.position);
                            h((hh, mm, false), w, cx);
                        }
                    }
                })
                .on_mouse_up(MouseButton::Left, move |e, w, cx| {
                    if let Some(h) = &oc3 {
                        let (hh, mm) = cm3(e.position);
                        h((hh, mm, true), w, cx);
                    }
                });
        }

        div()
            .w(px(312.))
            .flex()
            .flex_col()
            .gap(px(24.))
            .p(px(24.))
            .bg(c.surface_container_high)
            .rounded(shape::EXTRA_LARGE)
            .text_color(c.on_surface)
            .child(
                div()
                    .m3_type(TypeScale::LabelMedium)
                    .text_color(c.on_surface_variant)
                    .child("時刻を選択"),
            )
            .map(move |d| {
                if input_on {
                    d.child(input_display.children(ampm_block))
                } else {
                    d.child(display.children(ampm_block))
                        .child(div().flex().justify_center().child(dial))
                }
            })
            .child(
                div().flex().items_center().child(
                    IconButton::new(
                        format!("{}-mode", self.id),
                        if input_on {
                            IconName::Schedule
                        } else {
                            IconName::Keyboard
                        },
                    )
                    .on_click(move |_, _, cx| {
                        input_state.update(cx, |s, cx| {
                            s.on = !s.on;
                            s.synced_h = None;
                            s.synced_m = None;
                            cx.notify();
                        })
                    }),
                ),
            )
    }
}

fn transparent_c() -> Hsla {
    super::transparent()
}

#[allow(dead_code)]
fn _unused(_: Icon) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekday_known_dates() {
        assert_eq!(weekday(1970, 1, 1), 4); // 木
        assert_eq!(weekday(2000, 1, 1), 6); // 土
        assert_eq!(weekday(2026, 11, 14), 6); // 土
        assert_eq!(weekday(2024, 2, 29), 4); // 木
    }

    #[test]
    fn month_lengths_and_leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2100, 2), 28); // 100 の倍数は平年
        assert_eq!(days_in_month(2000, 2), 29); // 400 の倍数は閏年
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn shifting_months_wraps_years() {
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 6, 0), (2026, 6));
        assert_eq!(shift_month(2026, 3, -15), (2024, 12));
    }

    #[test]
    fn parse_date_accepts_common_formats() {
        let want = Some(Date::new(2026, 11, 14));
        assert_eq!(parse_date("2026/11/14"), want);
        assert_eq!(parse_date("2026-11-14"), want);
        assert_eq!(parse_date("2026.11.14"), want);
        assert_eq!(parse_date("2026年11月14日"), want);
        assert_eq!(parse_date("  2026/1/5 "), Some(Date::new(2026, 1, 5)));
    }

    #[test]
    fn parse_date_rejects_invalid_input() {
        assert_eq!(parse_date(""), None);
        assert_eq!(parse_date("2027/2/30"), None);
        assert_eq!(parse_date("2026/13/1"), None);
        assert_eq!(parse_date("2026/0/1"), None);
        assert_eq!(parse_date("2026/1/0"), None);
        assert_eq!(parse_date("26/1/1"), None); // 年は 4 桁
        assert_eq!(parse_date("2026/1"), None);
        assert_eq!(parse_date("2026/1/1/1"), None);
        assert_eq!(parse_date("2023/2/29"), None);
        assert_eq!(parse_date("2024/2/29"), Some(Date::new(2024, 2, 29)));
    }

    #[test]
    fn format_round_trips_through_parse() {
        let d = Date::new(2026, 3, 9);
        assert_eq!(d.format_input(), "2026/03/09");
        assert_eq!(parse_date(&d.format_input()), Some(d));
    }

    #[test]
    fn range_pick_flow() {
        let a = Date::new(2026, 11, 10);
        let b = Date::new(2026, 11, 17);
        let r = DateRange::default().pick(a);
        assert_eq!(r, DateRange::new(Some(a), None));
        let r = r.pick(b);
        assert_eq!(r, DateRange::new(Some(a), Some(b)));
        // 完成した範囲をクリックすると新しい開始日になる
        let c = Date::new(2026, 12, 1);
        assert_eq!(r.pick(c), DateRange::new(Some(c), None));
    }

    #[test]
    fn range_pick_swaps_reversed_endpoints() {
        let a = Date::new(2026, 11, 10);
        let early = Date::new(2026, 11, 3);
        let r = DateRange::new(Some(a), None).pick(early);
        assert_eq!(r, DateRange::new(Some(early), Some(a)));
    }

    #[test]
    fn range_days_are_inclusive() {
        let r = |a, b| DateRange::new(Some(a), Some(b)).days();
        assert_eq!(r(Date::new(2026, 11, 10), Date::new(2026, 11, 17)), Some(8));
        assert_eq!(r(Date::new(2026, 11, 10), Date::new(2026, 11, 10)), Some(1));
        assert_eq!(r(Date::new(2026, 12, 30), Date::new(2027, 1, 2)), Some(4));
        assert_eq!(r(Date::new(2024, 2, 28), Date::new(2024, 3, 1)), Some(3)); // 閏日をまたぐ
        assert_eq!(DateRange::default().days(), None);
        assert_eq!(
            DateRange::new(Some(Date::new(2026, 1, 1)), None).days(),
            Some(1)
        );
    }

    #[test]
    fn range_compact_format() {
        let r = DateRange::new(Some(Date::new(2026, 11, 3)), Some(Date::new(2026, 11, 9)));
        assert_eq!(r.format_compact(), "11月3日 – 11月9日");
        assert_eq!(DateRange::default().format_compact(), "");
    }
}
