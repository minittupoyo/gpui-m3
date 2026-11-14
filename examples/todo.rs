//! gpui-m3 で作った ToDo リスト。
//!
//! ```text
//! cargo run --example todo
//! ```
//!
//! - 入力欄に書いて Enter（または「追加」）でタスクを追加
//! - チェックボックスで完了、ゴミ箱で削除（スナックバーの「元に戻す」で復活）
//! - チップで「すべて / 未完了 / 完了」を絞り込み、「完了を削除」でまとめて削除
//!
//! 状態はメモリ上だけで、アプリを閉じると消えます。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use gpui::{
    App, Bounds, Context, Entity, Subscription, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, size,
};
use gpui_m3::{
    self as m3, ActiveTheme, Button, Checkbox, Chip, Icon, IconButton, IconName, Snackbar,
    SnackbarHost, StyledExt, TextField, TextFieldEvent, TopAppBar, TypeScale, shape,
};
use gpui_platform::application;

#[derive(Clone)]
struct Task {
    id: u64,
    title: String,
    done: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Filter {
    All,
    Active,
    Done,
}

impl Filter {
    const ALL: [Filter; 3] = [Filter::All, Filter::Active, Filter::Done];

    fn label(self) -> &'static str {
        match self {
            Filter::All => "すべて",
            Filter::Active => "未完了",
            Filter::Done => "完了",
        }
    }

    fn matches(self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !task.done,
            Filter::Done => task.done,
        }
    }
}

struct TodoApp {
    tasks: Vec<Task>,
    next_id: u64,
    filter: Filter,
    input: Entity<TextField>,
    /// 直前に削除したタスク（元の位置つき）。スナックバーの「元に戻す」で復活する
    deleted: Option<(usize, Task)>,
    snackbar: bool,
    snack_seq: u32,
    _subscription: Subscription,
}

impl TodoApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            TextField::new(window, cx)
                .outlined()
                .label("新しいタスク")
                .placeholder("やることを入力して Enter")
                .clearable(true)
        });
        // Enter で追加する
        let subscription = cx.subscribe(&input, |this, field, ev: &TextFieldEvent, cx| {
            if let TextFieldEvent::Submitted(text) = ev {
                // 空白だけの入力も含めて、送信したら欄を空にする
                this.add(text, cx);
                field.update(cx, |f, cx| f.set_text("", cx));
            }
        });

        let sample = [
            "牛乳を買う",
            "README を読み返す",
            "ギャラリーを動かしてみる",
        ];
        let tasks = sample
            .iter()
            .enumerate()
            .map(|(i, t)| Task {
                id: i as u64,
                title: t.to_string(),
                done: i == 1,
            })
            .collect();
        Self {
            tasks,
            next_id: sample.len() as u64,
            filter: Filter::All,
            input,
            deleted: None,
            snackbar: false,
            snack_seq: 0,
            _subscription: subscription,
        }
    }

    /// 空白だけの入力は追加しない。追加したら true
    fn add(&mut self, title: &str, cx: &mut Context<Self>) -> bool {
        let title = title.trim();
        if title.is_empty() {
            return false;
        }
        self.tasks.push(Task {
            id: self.next_id,
            title: title.to_string(),
            done: false,
        });
        self.next_id += 1;
        cx.notify();
        true
    }

    fn set_done(&mut self, id: u64, done: bool, cx: &mut Context<Self>) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.done = done;
            cx.notify();
        }
    }

    fn remove(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            let task = self.tasks.remove(pos);
            self.deleted = Some((pos, task));
            self.show_snackbar(cx);
        }
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        if let Some((pos, task)) = self.deleted.take() {
            let pos = pos.min(self.tasks.len());
            self.tasks.insert(pos, task);
        }
        self.snackbar = false;
        cx.notify();
    }

    fn clear_done(&mut self, cx: &mut Context<Self>) {
        self.tasks.retain(|t| !t.done);
        self.deleted = None;
        self.snackbar = false;
        cx.notify();
    }

    /// スナックバーを出し、4 秒後に自動で閉じる（後から出した方を優先する）
    fn show_snackbar(&mut self, cx: &mut Context<Self>) {
        self.snackbar = true;
        self.snack_seq += 1;
        let seq = self.snack_seq;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(4)).await;
            this.update(cx, |t, cx| {
                if t.snack_seq == seq {
                    t.snackbar = false;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    fn task_row(&self, task: &Task, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;
        let id = task.id;
        let this = cx.entity().downgrade();
        div()
            .flex()
            .items_center()
            .gap(px(4.))
            .pl(px(8.))
            .pr(px(4.))
            .min_h(px(56.))
            .child(
                Checkbox::new(format!("done-{id}"), task.done).on_change(move |done, _, cx| {
                    this.update(cx, |t, cx| t.set_done(id, done, cx)).ok();
                }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .m3_type(TypeScale::BodyLarge)
                    .text_color(if task.done {
                        c.on_surface_variant
                    } else {
                        c.on_surface
                    })
                    .when(task.done, |d| d.line_through())
                    .child(task.title.clone()),
            )
            .child(
                IconButton::new(format!("del-{id}"), IconName::Delete)
                    .on_click(cx.listener(move |t, _, _, cx| t.remove(id, cx))),
            )
    }
}

impl Render for TodoApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = &theme.colors;

        let active = self.tasks.iter().filter(|t| !t.done).count();
        let done = self.tasks.len() - active;

        // 絞り込みチップ
        let mut chips = div().flex().gap(px(8.));
        for f in Filter::ALL {
            let count = self.tasks.iter().filter(|t| f.matches(t)).count();
            chips = chips.child(
                Chip::filter(
                    format!("filter-{}", f.label()),
                    format!("{} {count}", f.label()),
                )
                .selected(self.filter == f)
                .on_click(cx.listener(move |t, _, _, cx| {
                    t.filter = f;
                    cx.notify();
                })),
            );
        }

        // タスク一覧
        let visible: Vec<Task> = self
            .tasks
            .iter()
            .filter(|t| self.filter.matches(t))
            .cloned()
            .collect();
        let mut list = div()
            .id("task-list")
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll();
        if visible.is_empty() {
            let message = match self.filter {
                Filter::Done => "完了したタスクはありません",
                _ if self.tasks.is_empty() => "タスクがありません。上の欄から追加しましょう",
                _ => "未完了のタスクはありません。おつかれさまです",
            };
            list = list.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(8.))
                    .py(px(48.))
                    .text_color(c.on_surface_variant)
                    .child(
                        Icon::new(IconName::Check)
                            .size(px(40.))
                            .color(c.on_surface_variant),
                    )
                    .child(div().m3_type(TypeScale::BodyMedium).child(message)),
            );
        }
        for task in &visible {
            list = list.child(self.task_row(task, cx));
        }

        // 追加欄
        let add = cx.listener(|t, _, _, cx| {
            let text = t.input.read(cx).text();
            t.add(&text, cx);
            t.input.update(cx, |f, cx| f.set_text("", cx));
        });

        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(c.surface)
            .text_color(c.on_surface)
            .font_family(theme.font_family)
            .child(
                TopAppBar::new("ToDo")
                    .navigation(
                        div()
                            .size(px(48.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Icon::new(IconName::Check).color(c.primary)),
                    )
                    .subtitle(format!("未完了 {active} 件 / 完了 {done} 件"))
                    .action(
                        IconButton::new("clear-done", IconName::Delete)
                            .disabled(done == 0)
                            .on_click(cx.listener(|t, _, _, cx| t.clear_done(cx))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .gap(px(16.))
                    .px(px(16.))
                    .pb(px(16.))
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap(px(12.))
                            .child(div().flex_1().child(self.input.clone()))
                            .child(div().pt(px(4.)).child(
                                Button::new("add", "追加").icon(IconName::Add).on_click(add),
                            )),
                    )
                    .child(chips)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .rounded(shape::LARGE)
                            .bg(c.surface_container_low)
                            .overflow_hidden()
                            .child(list),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(24.))
                    .left_0()
                    .right_0()
                    .flex()
                    .justify_center()
                    .child(SnackbarHost::new(
                        "snack",
                        self.snackbar,
                        Snackbar::new(match &self.deleted {
                            Some((_, t)) => format!("「{}」を削除しました", t.title),
                            None => "削除しました".to_string(),
                        })
                        .action("元に戻す", cx.listener(|t, _, _, cx| t.undo(cx)))
                        .on_close(cx.listener(|t, _, _, cx| {
                            t.snackbar = false;
                            cx.notify();
                        })),
                    )),
            )
    }
}

fn main() {
    application().with_assets(m3::Assets).run(|cx: &mut App| {
        m3::init(cx, m3::Brightness::Dark);
        let bounds = Bounds::centered(None, size(px(520.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| TodoApp::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
