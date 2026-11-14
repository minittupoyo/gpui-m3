//! Material Design 3 Expressive コンポーネントギャラリー
//!
//! `cargo run --bin m3_gallery`

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use gpui::{
    AnyElement, App, Bounds, Context, Entity, IntoElement, SharedString, Window, WindowBounds,
    WindowOptions, div, linear_color_stop, linear_gradient, prelude::*, px, size,
};
use gpui_platform::application;
use gpui_test::m3::{self, *};

const PAGES: [(&str, IconName); 10] = [
    ("Buttons", IconName::Home),
    ("Selection", IconName::CheckBox),
    ("Chips & Cards", IconName::Label),
    ("Progress", IconName::Progress),
    ("Sliders", IconName::Tune),
    ("Feedback", IconName::Mail),
    ("Text fields", IconName::Edit),
    ("Menus", IconName::Sort),
    ("App bars & Tabs", IconName::Bookmark),
    ("Color", IconName::Palette),
];

/// Dynamic color のプリセット（名前, シード）
const SEEDS: [(&str, u32); 7] = [
    ("Purple", 0x6750A4),
    ("Red", 0xB33B15),
    ("Orange", 0xE8A33D),
    ("Green", 0x63A002),
    ("Teal", 0x006A6A),
    ("Blue", 0x0061A4),
    ("Pink", 0xC2185B),
];

struct Gallery {
    page: usize,
    // buttons
    toggles: [bool; 5],
    favorite: [bool; 4],
    clicks: u32,
    // groups & selection
    view_mode: usize,
    text_styles: Vec<bool>,
    media: Vec<bool>,
    switches: [bool; 3],
    checks: [bool; 3],
    radio: usize,
    // chips
    filters: Vec<bool>,
    inputs: Vec<&'static str>,
    // progress / slider
    progress: f32,
    volume: f32,
    rating: f32,
    sizes: [f32; 3],
    // feedback
    dialog: bool,
    snackbar: bool,
    snack_seq: u32,
    nav_bar: usize,
    // text fields
    fields: Vec<Entity<TextField>>,
    submitted: Option<SharedString>,
    _subs: Vec<gpui::Subscription>,
    // menus
    sort_by: usize,
    menu_log: SharedString,
    fruit: Option<usize>,
    country: Option<usize>,
    // app bars & tabs
    app_bar_kind: usize,
    app_scroll: gpui::ScrollHandle,
    primary_tab: usize,
    secondary_tab: usize,
    sheet_open: bool,
    // toolbars
    tb_sel: [usize; 4],
    tb_visible: bool,
    tb_scroll: gpui::ScrollHandle,
    // dynamic color
    /// None = ベースライン、Some(i) = SEEDS[i]、Some(usize::MAX) = 色相スライダー
    seed: Option<usize>,
    hue: f32,
    variant: usize,
    contrast: usize,
    // navigation rail
    rail_expanded: bool,
    demo_rail_expanded: bool,
    demo_rail_sel: usize,
    modal_rail_open: bool,
}

impl Gallery {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let notch = |c: &ColorScheme| c.surface_container_low;
        let fields = vec![
            cx.new(|cx| {
                TextField::new(window, cx)
                    .label("名前")
                    .supporting_text("フルネームで入力")
                    .clearable(true)
            }),
            cx.new(|cx| {
                TextField::new(window, cx)
                    .label("メールアドレス")
                    .leading_icon(IconName::Mail)
                    .placeholder("name@example.com")
                    .supporting_text("Enter で送信")
            }),
            cx.new(|cx| {
                TextField::new(window, cx)
                    .label("パスワード")
                    .leading_icon(IconName::Lock)
                    .password(true)
            }),
            cx.new(|cx| {
                TextField::new(window, cx)
                    .label("自己紹介")
                    .max_length(40)
                    .value("GPUI で M3 を作っています")
            }),
            cx.new(|cx| {
                TextField::new(window, cx)
                    .outlined()
                    .notch_background(notch)
                    .label("名前")
                    .supporting_text("Outlined")
                    .clearable(true)
            }),
            cx.new(|cx| {
                TextField::new(window, cx)
                    .outlined()
                    .notch_background(notch)
                    .label("検索")
                    .leading_icon(IconName::Search)
                    .placeholder("キーワード")
                    .clearable(true)
            }),
            cx.new(|cx| {
                TextField::new(window, cx)
                    .outlined()
                    .notch_background(notch)
                    .label("ユーザー名")
                    .value("taken_name")
            }),
            cx.new(|cx| {
                let mut f = TextField::new(window, cx)
                    .label("無効")
                    .value("編集できません");
                f.set_disabled(true, cx);
                f
            }),
        ];
        // メール: 入力のたびに検証、Enter で送信
        let email_sub = cx.subscribe(
            &fields[1],
            |g: &mut Gallery, field, ev: &TextFieldEvent, cx| match ev {
                TextFieldEvent::Changed(text) => {
                    let err = (!text.is_empty() && !(text.contains('@') && text.contains('.')))
                        .then(|| SharedString::from("メールアドレスの形式が正しくありません"));
                    field.update(cx, |f, cx| f.set_error(err, cx));
                }
                TextFieldEvent::Submitted(text) => {
                    g.submitted = Some(text.clone());
                    cx.notify();
                }
                _ => {}
            },
        );
        // ユーザー名: 使用済みチェック
        let user_sub = cx.subscribe(
            &fields[6],
            |_: &mut Gallery, field, ev: &TextFieldEvent, cx| {
                if let TextFieldEvent::Changed(text) = ev {
                    let err = (text.as_ref() == "taken_name")
                        .then(|| SharedString::from("このユーザー名は使用されています"));
                    field.update(cx, |f, cx| f.set_error(err, cx));
                }
            },
        );
        fields[6].update(cx, |f, cx| {
            f.set_error(Some("このユーザー名は使用されています".into()), cx)
        });
        let mut subs = vec![email_sub, user_sub];
        for f in &fields {
            subs.push(cx.observe(f, |_, _, cx| cx.notify()));
        }

        Self {
            fields,
            submitted: None,
            _subs: subs,
            sort_by: 0,
            menu_log: "（まだ何も選択していません）".into(),
            fruit: None,
            country: Some(0),
            app_bar_kind: 2,
            app_scroll: gpui::ScrollHandle::new(),
            primary_tab: 0,
            secondary_tab: 0,
            sheet_open: false,
            tb_sel: [0, 1, 2, 0],
            tb_visible: true,
            tb_scroll: gpui::ScrollHandle::new(),
            seed: None,
            hue: 200.,
            variant: 0,
            contrast: 0,
            rail_expanded: false,
            demo_rail_expanded: true,
            demo_rail_sel: 0,
            modal_rail_open: false,
            page: 0,
            toggles: [false, true, false, true, false],
            favorite: [true, false, true, false],
            clicks: 0,
            view_mode: 0,
            text_styles: vec![true, false, false],
            media: vec![false, true, false, false],
            switches: [true, false, true],
            checks: [true, false, false],
            radio: 0,
            filters: vec![true, false, true, false],
            inputs: vec!["東京", "大阪", "札幌"],
            progress: 0.62,
            volume: 0.4,
            rating: 3.,
            sizes: [0.3, 0.5, 0.7],
            dialog: false,
            snackbar: false,
            snack_seq: 0,
            nav_bar: 0,
        }
    }

    /// 値を受け取って状態を更新するハンドラを作る
    fn set<V: 'static>(
        cx: &mut Context<Self>,
        f: impl Fn(&mut Self, V) + 'static,
    ) -> impl Fn(V, &mut Window, &mut App) + 'static {
        let this = cx.entity().downgrade();
        move |v, _, cx| {
            this.update(cx, |g, cx| {
                f(g, v);
                cx.notify();
            })
            .ok();
        }
    }

    fn show_snackbar(&mut self, cx: &mut Context<Self>) {
        self.snackbar = true;
        self.snack_seq += 1;
        let seq = self.snack_seq;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(4)).await;
            this.update(cx, |g, cx| {
                if g.snack_seq == seq {
                    g.snackbar = false;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
        cx.notify();
    }
}

// ---------------------------------------------------------------------------
// レイアウト部品
// ---------------------------------------------------------------------------

fn section(
    cx: &App,
    title: &'static str,
    note: &'static str,
    body: impl IntoElement + 'static,
) -> gpui::Div {
    let c = &cx.m3().colors;
    div()
        .flex()
        .flex_col()
        .gap(px(16.))
        .p(px(24.))
        .rounded(shape::EXTRA_LARGE)
        .bg(c.surface_container_low)
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .m3_type(TypeScale::TitleLarge)
                        .text_color(c.on_surface)
                        .child(title),
                )
                .child(
                    div()
                        .m3_type(TypeScale::BodyMedium)
                        .text_color(c.on_surface_variant)
                        .child(note),
                ),
        )
        .child(body)
}

fn caption(cx: &App, text: impl Into<SharedString>) -> gpui::Div {
    div()
        .m3_type(TypeScale::LabelMedium)
        .text_color(cx.m3().colors.on_surface_variant)
        .child(text.into())
}

fn wrap_row() -> gpui::Div {
    div().flex().flex_wrap().items_center().gap(px(12.))
}

fn muted(cx: &App) -> gpui::Hsla {
    cx.m3().colors.on_surface_variant
}

fn labeled(muted: gpui::Hsla, label: impl Into<SharedString>, el: impl IntoElement) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.))
        .child(el)
        .child(div().m3_type(TypeScale::LabelMedium).text_color(muted).child(label.into()))
}

// ---------------------------------------------------------------------------
// ページ
// ---------------------------------------------------------------------------

impl Gallery {
    fn page_buttons(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut out = vec![];

        // バリアント
        let mut variants = wrap_row();
        for v in ButtonVariant::ALL {
            variants = variants.child(
                Button::new(format!("v-{}", v.name()), v.name())
                    .variant(v)
                    .icon(IconName::Add)
                    .on_click(cx.listener(|g, _, _, cx| {
                        g.clicks += 1;
                        cx.notify();
                    })),
            );
        }
        let mut disabled = wrap_row();
        for v in ButtonVariant::ALL {
            disabled = disabled.child(
                Button::new(format!("d-{}", v.name()), v.name())
                    .variant(v)
                    .disabled(true),
            );
        }
        out.push(
            section(
                cx,
                "Common buttons",
                "Filled / Tonal / Elevated / Outlined / Text。押すと角が詰まるシェイプモーフ（Expressive のバネ）",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(variants)
                    .child(disabled)
                    .child(caption(cx, format!("クリック回数: {}", self.clicks))),
            )
            .into_any_element(),
        );

        // サイズ
        let mut round = wrap_row().items_end();
        let mut square = wrap_row().items_end();
        for s in ComponentSize::ALL {
            round = round.child(labeled(
                muted(cx),
                s.name(),
                Button::new(format!("sz-r-{}", s.name()), "Button")
                    .size(s)
                    .icon(IconName::Send),
            ));
            square = square.child(labeled(
                muted(cx),
                s.name(),
                Button::new(format!("sz-s-{}", s.name()), "Button")
                    .size(s)
                    .shape(ButtonShape::Square)
                    .tonal()
                    .icon(IconName::Send),
            ));
        }
        out.push(
            section(
                cx,
                "Sizes & shapes",
                "XS / S / M / L / XL の 5 サイズと Round / Square の 2 シェイプ",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(20.))
                    .child(round)
                    .child(square),
            )
            .into_any_element(),
        );

        // トグル
        let mut toggles = wrap_row();
        for (i, v) in ButtonVariant::ALL.into_iter().take(4).enumerate() {
            let sel = self.toggles[i];
            toggles = toggles.child(
                Button::new(format!("t-{i}"), if sel { "Selected" } else { "Toggle" })
                    .variant(v)
                    .size(ComponentSize::Medium)
                    .icon(if sel { IconName::Check } else { IconName::Add })
                    .selected(sel)
                    .on_click(cx.listener(move |g, _, _, cx| {
                        g.toggles[i] = !g.toggles[i];
                        cx.notify();
                    })),
            );
        }
        out.push(
            section(
                cx,
                "Toggle buttons",
                "選択すると色が変わり、丸 ⇄ 角 に形がモーフする",
                toggles,
            )
            .into_any_element(),
        );

        // Split buttons
        let split_menu = |cx: &mut Context<Self>, id: &str| {
            Menu::new(format!("{id}-menu"))
                .item(MenuItem::new("下書きとして保存").icon(IconName::Edit).on_click(Self::log(cx, "下書きとして保存")))
                .item(MenuItem::new("コピーを保存").icon(IconName::ContentCopy).on_click(Self::log(cx, "コピーを保存")))
                .divider()
                .item(MenuItem::new("共有して保存").icon(IconName::Share).on_click(Self::log(cx, "共有して保存")))
        };
        let mut split_variants = wrap_row();
        for v in [
            ButtonVariant::Filled,
            ButtonVariant::Tonal,
            ButtonVariant::Elevated,
            ButtonVariant::Outlined,
        ] {
            let id = format!("split-{}", v.name());
            let menu = split_menu(cx, &id);
            split_variants = split_variants.child(
                SplitButton::new(id, v.name(), menu)
                    .variant(v)
                    .icon(IconName::Send)
                    .on_click(Self::log(cx, "メインアクション")),
            );
        }
        let mut split_sizes = wrap_row().items_end();
        for s in ComponentSize::ALL {
            let id = format!("split-size-{}", s.name());
            let menu = split_menu(cx, &id);
            split_sizes = split_sizes.child(labeled(
                muted(cx),
                s.name(),
                SplitButton::new(id, "保存", menu)
                    .tonal()
                    .size(s)
                    .on_click(Self::log(cx, "保存")),
            ));
        }
        let split_disabled = SplitButton::new("split-disabled", "Disabled", split_menu(cx, "split-disabled")).disabled(true);
        out.push(
            section(
                cx,
                "Split buttons",
                "左はメインアクション、右はメニュー。ホバー・押下で内側の角が丸くなり、開くと右側が正円にモーフしてシェブロンが回転",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(20.))
                    .child(split_variants)
                    .child(wrap_row().child(split_disabled))
                    .child(split_sizes)
                    .child(caption(cx, format!("最後に選んだ項目: {}", self.menu_log))),
            )
            .into_any_element(),
        );

        // アイコンボタン
        let mut icon_variants = wrap_row();
        for (i, v) in IconButtonVariant::ALL.into_iter().enumerate() {
            let sel = self.favorite[i];
            icon_variants = icon_variants.child(labeled(
                muted(cx),
                v.name(),
                div().flex().gap(px(8.)).child(
                    IconButton::new(format!("ib-{i}"), IconName::Settings)
                        .variant(v)
                        .size(ComponentSize::Medium),
                )
                .child(
                    IconButton::new(format!("ibt-{i}"), IconName::FavoriteBorder)
                        .selected_icon(IconName::Favorite)
                        .variant(v)
                        .size(ComponentSize::Medium)
                        .selected(sel)
                        .on_click(cx.listener(move |g, _, _, cx| {
                            g.favorite[i] = !g.favorite[i];
                            cx.notify();
                        })),
                ),
            ));
        }
        let mut icon_sizes = wrap_row().items_end();
        for s in ComponentSize::ALL {
            for w in [IconButtonWidth::Narrow, IconButtonWidth::Default, IconButtonWidth::Wide] {
                icon_sizes = icon_sizes.child(
                    IconButton::new(format!("ibs-{}-{w:?}", s.name()), IconName::Edit)
                        .tonal()
                        .size(s)
                        .width(w),
                );
            }
        }
        out.push(
            section(
                cx,
                "Icon buttons",
                "Standard / Filled / Tonal / Outlined、5 サイズ × Narrow / Default / Wide",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(20.))
                    .child(icon_variants)
                    .child(icon_sizes),
            )
            .into_any_element(),
        );

        // FAB
        let mut fabs = wrap_row().items_end();
        for s in FabSize::ALL {
            fabs = fabs.child(Fab::new(format!("fab-{s:?}"), IconName::Edit).size(s));
        }
        let mut fab_colors = wrap_row();
        for (i, col) in FabColor::ALL.into_iter().enumerate() {
            fab_colors = fab_colors.child(Fab::new(format!("fabc-{i}"), IconName::Add).color(col));
        }
        let extended = wrap_row()
            .items_end()
            .child(Fab::new("efab-s", IconName::Edit).label("作成"))
            .child(
                Fab::new("efab-m", IconName::Edit)
                    .label("作成")
                    .size(FabSize::Medium)
                    .color(FabColor::SecondaryContainer),
            )
            .child(
                Fab::new("efab-l", IconName::Edit)
                    .label("作成")
                    .size(FabSize::Large)
                    .color(FabColor::Tertiary),
            );
        out.push(
            section(
                cx,
                "FAB & Extended FAB",
                "FAB / Medium / Large、6 種の配色、ラベル付き Extended FAB",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(20.))
                    .child(fabs)
                    .child(fab_colors)
                    .child(extended),
            )
            .into_any_element(),
        );

        // FAB menu（3 配色）
        let c = cx.m3().colors.clone();
        let mut frames = wrap_row().items_start().gap(px(16.));
        for (i, (color, name)) in FabMenuColor::ALL
            .into_iter()
            .zip(["Primary", "Secondary", "Tertiary"])
            .enumerate()
        {
            frames = frames.child(labeled(
                muted(cx),
                name,
                div()
                    .relative()
                    .w(px(300.))
                    .h(px(400.))
                    .rounded(shape::EXTRA_LARGE)
                    .bg(c.surface)
                    .border_1()
                    .border_color(c.outline_variant)
                    .child(
                        div()
                            .absolute()
                            .right(px(16.))
                            .bottom(px(16.))
                            .child(
                                FabMenu::new(format!("fm-{i}"), IconName::Add)
                                    .color(color)
                                    .item(FabMenuItem::new(IconName::Mail, "メール").on_click(Self::log(cx, "メール")))
                                    .item(FabMenuItem::new(IconName::Image, "写真").on_click(Self::log(cx, "写真")))
                                    .item(FabMenuItem::new(IconName::Bookmark, "ブックマーク").on_click(Self::log(cx, "ブックマーク")))
                                    .item(FabMenuItem::new(IconName::Edit, "メモ").on_click(Self::log(cx, "メモ"))),
                            ),
                    ),
            ));
        }
        out.push(
            section(
                cx,
                "FAB menu",
                "FAB が閉じるボタンにモーフし、項目が時間差のスプリングで現れる（外側クリック・項目クリックで閉じる）",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(frames)
                    .child(caption(cx, format!("最後に選んだ項目: {}", self.menu_log))),
            )
            .into_any_element(),
        );
        out
    }

    fn page_selection(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut out = vec![];

        let view_sel = (0..3).map(|i| i == self.view_mode).collect();
        let groups = div()
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(
                ConnectedButtonGroup::new("cg-view")
                    .item(ButtonGroupItem::new("日"))
                    .item(ButtonGroupItem::new("週"))
                    .item(ButtonGroupItem::new("月"))
                    .selected(view_sel)
                    .on_toggle(Self::set(cx, |g, i| g.view_mode = i)),
            )
            .child(
                ConnectedButtonGroup::new("cg-text")
                    .size(ComponentSize::Medium)
                    .item(ButtonGroupItem::new("太字"))
                    .item(ButtonGroupItem::new("斜体"))
                    .item(ButtonGroupItem::new("下線"))
                    .selected(self.text_styles.clone())
                    .on_toggle(Self::set(cx, |g, i: usize| g.text_styles[i] = !g.text_styles[i])),
            )
            .child(
                StandardButtonGroup::new("sg-media")
                    .icon(IconName::Star)
                    .icon(IconName::Favorite)
                    .icon(IconName::Bookmark)
                    .icon(IconName::Share)
                    .selected(self.media.clone())
                    .on_toggle(Self::set(cx, |g, i: usize| g.media[i] = !g.media[i])),
            );
        out.push(
            section(
                cx,
                "Button groups",
                "Connected（単一/複数選択、選択で角が丸くなる）と Standard（押したボタンが膨らみ隣が縮む）",
                groups,
            )
            .into_any_element(),
        );

        let switches = wrap_row()
            .gap(px(24.))
            .child(labeled(
                muted(cx),
                "Default",
                Switch::new("sw-0", self.switches[0])
                    .on_change(Self::set(cx, |g, v| g.switches[0] = v)),
            ))
            .child(labeled(
                muted(cx),
                "Off",
                Switch::new("sw-1", self.switches[1])
                    .on_change(Self::set(cx, |g, v| g.switches[1] = v)),
            ))
            .child(labeled(
                muted(cx),
                "Icons",
                Switch::new("sw-2", self.switches[2])
                    .icons(true)
                    .on_change(Self::set(cx, |g, v| g.switches[2] = v)),
            ))
            .child(labeled(muted(cx), "Disabled", Switch::new("sw-d1", true).disabled(true)))
            .child(labeled(muted(cx), "Disabled", Switch::new("sw-d2", false).disabled(true)));
        out.push(
            section(cx, "Switches", "ハンドルはバネで移動・拡大（押下で 28px）", switches)
                .into_any_element(),
        );

        let all = self.checks.iter().all(|c| *c);
        let any = self.checks.iter().any(|c| *c);
        let checks = div()
            .flex()
            .flex_col()
            .child(
                Checkbox::new("cb-all", all)
                    .indeterminate(any && !all)
                    .label("すべて選択")
                    .on_change(Self::set(cx, |g, v: bool| g.checks = [v; 3])),
            )
            .child(
                div()
                    .pl(px(32.))
                    .child(
                        Checkbox::new("cb-0", self.checks[0])
                            .label("メール通知")
                            .on_change(Self::set(cx, |g, v| g.checks[0] = v)),
                    )
                    .child(
                        Checkbox::new("cb-1", self.checks[1])
                            .label("プッシュ通知")
                            .on_change(Self::set(cx, |g, v| g.checks[1] = v)),
                    )
                    .child(
                        Checkbox::new("cb-2", self.checks[2])
                            .label("SMS 通知")
                            .on_change(Self::set(cx, |g, v| g.checks[2] = v)),
                    ),
            )
            .child(
                wrap_row()
                    .child(Checkbox::new("cb-e", false).error(true).label("Error"))
                    .child(Checkbox::new("cb-e2", true).error(true).label("Error"))
                    .child(Checkbox::new("cb-d", true).disabled(true).label("Disabled"))
                    .child(Checkbox::new("cb-d2", false).disabled(true).label("Disabled")),
            );

        let mut radios = div().flex().flex_col();
        for (i, label) in ["ライト", "ダーク", "システムに合わせる"].into_iter().enumerate() {
            let set = Self::set(cx, move |g, _: ()| g.radio = i);
            radios = radios.child(
                Radio::new(format!("r-{i}"), self.radio == i)
                    .label(label)
                    .on_select(move |w, cx| set((), w, cx)),
            );
        }
        radios = radios.child(Radio::new("r-d", false).disabled(true).label("Disabled"));

        out.push(
            section(
                cx,
                "Checkbox & Radio",
                "親チェックボックスは子の状態から indeterminate を表示",
                div().flex().gap(px(48.)).child(checks).child(radios),
            )
            .into_any_element(),
        );
        out
    }

    fn page_chips(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut out = vec![];
        let c = cx.m3().colors.clone();

        let mut filters = wrap_row();
        for (i, label) in ["和食", "イタリアン", "カフェ", "ラーメン"].into_iter().enumerate() {
            filters = filters.child(
                Chip::filter(format!("fc-{i}"), label)
                    .selected(self.filters[i])
                    .on_click(cx.listener(move |g, _, _, cx| {
                        g.filters[i] = !g.filters[i];
                        cx.notify();
                    })),
            );
        }
        let mut inputs = wrap_row();
        for (i, label) in self.inputs.clone().into_iter().enumerate() {
            inputs = inputs.child(
                Chip::input(format!("ic-{label}"), label)
                    .icon(IconName::Person)
                    .on_remove(cx.listener(move |g, _, _, cx| {
                        if i < g.inputs.len() {
                            g.inputs.remove(i);
                        }
                        cx.notify();
                    })),
            );
        }
        if self.inputs.is_empty() {
            inputs = inputs.child(
                Button::new("reset-inputs", "元に戻す")
                    .text()
                    .icon(IconName::Refresh)
                    .on_click(cx.listener(|g, _, _, cx| {
                        g.inputs = vec!["東京", "大阪", "札幌"];
                        cx.notify();
                    })),
            );
        }
        let chips = div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                wrap_row()
                    .child(Chip::assist("ac-0", "カレンダーに追加").icon(IconName::Add))
                    .child(
                        Chip::assist("ac-1", "共有")
                            .icon(IconName::Share)
                            .elevated(true),
                    )
                    .child(Chip::suggestion("sc-0", "明日の予定は？"))
                    .child(Chip::suggestion("sc-1", "近くのカフェ").elevated(true))
                    .child(Chip::assist("ac-d", "Disabled").disabled(true)),
            )
            .child(filters)
            .child(inputs);
        out.push(
            section(cx, "Chips", "Assist / Suggestion / Filter / Input（✕ で削除）", chips)
                .into_any_element(),
        );

        let image = |a: u32, b: u32| {
            CardMedia::new().h(px(120.)).bg(linear_gradient(
                135.,
                linear_color_stop(gpui::rgb(a), 0.),
                linear_color_stop(gpui::rgb(b), 1.),
            ))
        };
        let card_body = |title: &'static str, sub: &'static str| {
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .p(px(16.))
                .child(div().m3_type(TypeScale::TitleMedium).child(title))
                .child(
                    div()
                        .m3_type(TypeScale::BodyMedium)
                        .text_color(c.on_surface_variant)
                        .child(sub),
                )
        };
        let cards = wrap_row()
            .items_start()
            .gap(px(16.))
            .child(
                Card::new("card-e")
                    .elevated()
                    .w(px(240.))
                    .on_click(|_, _, _| {})
                    .media(image(0x7c4dff, 0x18ffff))
                    .child(card_body("Elevated", "影付き。ホバーで持ち上がる")),
            )
            .child(
                Card::new("card-f")
                    .filled()
                    .w(px(240.))
                    .on_click(|_, _, _| {})
                    .media(image(0xff8a65, 0xffd54f))
                    .child(card_body("Filled", "surface-container-highest")),
            )
            .child(
                Card::new("card-o")
                    .outlined()
                    .w(px(240.))
                    .on_click(|_, _, _| {})
                    .media(image(0x26a69a, 0x9ccc65))
                    .child(card_body("Outlined", "outline-variant の枠線")),
            );
        out.push(section(cx, "Cards", "Elevated / Filled / Outlined", cards).into_any_element());

        let avatar = |letter: &'static str, bg: gpui::Hsla, fg: gpui::Hsla| {
            div()
                .size(px(40.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(bg)
                .text_color(fg)
                .m3_type(TypeScale::TitleMedium)
                .child(letter)
        };
        let list = div()
            .flex()
            .flex_col()
            .rounded(shape::LARGE)
            .overflow_hidden()
            .bg(c.surface)
            .child(
                ListItem::new("li-0", "佐藤 花子")
                    .supporting("明日の打ち合わせの件で…")
                    .leading(avatar("佐", c.primary_container, c.on_primary_container))
                    .trailing(Badge::new(Some(3)))
                    .on_click(|_, _, _| {}),
            )
            .child(Divider::horizontal())
            .child(
                ListItem::new("li-1", "鈴木 一郎")
                    .supporting("資料を送りました")
                    .leading(avatar("鈴", c.tertiary_container, c.on_tertiary_container))
                    .trailing(Badge::new(None))
                    .on_click(|_, _, _| {}),
            )
            .child(Divider::horizontal())
            .child(
                ListItem::new("li-2", "田中 美咲")
                    .supporting("了解です！")
                    .leading(avatar("田", c.secondary_container, c.on_secondary_container))
                    .trailing(Icon::new(IconName::MoreVert).color(c.on_surface_variant))
                    .on_click(|_, _, _| {}),
            );
        out.push(
            section(cx, "Lists, Dividers & Badges", "1〜2 行のリスト項目とバッジ", list)
                .into_any_element(),
        );
        out
    }

    fn page_progress(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut out = vec![];
        let v = self.progress;
        let linear = div()
            .flex()
            .flex_col()
            .gap(px(20.))
            .child(caption(cx, format!("確定（{:.0}%）— 下のスライダーで変更", v * 100.)))
            .child(LinearProgress::new("lp-flat", Some(v)))
            .child(LinearProgress::new("lp-wavy", Some(v)).wavy(true))
            .child(LinearProgress::new("lp-thick", Some(v)).wavy(true).thickness(8.))
            .child(
                Slider::new("progress-slider", v)
                    .on_change(Self::set(cx, |g, v| g.progress = v)),
            )
            .child(caption(cx, "不確定"))
            .child(LinearProgress::new("lp-ind", None))
            .child(LinearProgress::new("lp-ind-w", None).wavy(true));
        out.push(
            section(
                cx,
                "Linear progress",
                "フラット / 波線（Wavy）/ 太線。トラックとの間のギャップとストップインジケーター",
                linear,
            )
            .into_any_element(),
        );

        let circular = wrap_row()
            .gap(px(32.))
            .child(labeled(muted(cx), "確定", CircularProgress::new("cp-0", Some(v))))
            .child(labeled(
                muted(cx),
                "確定 Wavy",
                CircularProgress::new("cp-1", Some(v)).wavy(true),
            ))
            .child(labeled(muted(cx), "不確定", CircularProgress::new("cp-2", None)))
            .child(labeled(
                muted(cx),
                "不確定 Wavy",
                CircularProgress::new("cp-3", None).wavy(true),
            ));
        out.push(
            section(cx, "Circular progress", "円形（Wavy は 48px）", circular).into_any_element(),
        );

        let loading = wrap_row()
            .gap(px(32.))
            .child(labeled(muted(cx), "Default", LoadingIndicator::new("li-0")))
            .child(labeled(
                muted(cx),
                "Contained",
                LoadingIndicator::new("li-1").contained(true),
            ))
            .child(labeled(
                muted(cx),
                "Large",
                LoadingIndicator::new("li-2").size(96.),
            ));
        out.push(
            section(
                cx,
                "Loading indicator",
                "M3 Expressive の新コンポーネント。7 種の図形をモーフィングしながら回転",
                loading,
            )
            .into_any_element(),
        );
        out
    }

    fn page_sliders(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut out = vec![];
        let sliders = div()
            .flex()
            .flex_col()
            .gap(px(24.))
            .child(caption(cx, format!("連続値: {:.2}", self.volume)))
            .child(Slider::new("s-vol", self.volume).on_change(Self::set(cx, |g, v| g.volume = v)))
            .child(caption(cx, format!("離散値（0〜5、ステップ 1）: {:.0}", self.rating)))
            .child(
                Slider::new("s-rate", self.rating)
                    .range(0., 5.)
                    .step(1.)
                    .on_change(Self::set(cx, |g, v| g.rating = v)),
            )
            .child(caption(cx, "Disabled"))
            .child(Slider::new("s-dis", 0.5).disabled(true));
        out.push(
            section(
                cx,
                "Sliders",
                "縦長ハンドル + 両脇のギャップ。ドラッグ中は値ラベルを表示",
                sliders,
            )
            .into_any_element(),
        );

        let mut sizes = div().flex().flex_col().gap(px(24.));
        for (i, (name, h)) in [("S (24)", 24.), ("M (40)", 40.), ("L (56)", 56.)]
            .into_iter()
            .enumerate()
        {
            sizes = sizes.child(caption(cx, name)).child(
                Slider::new(format!("s-size-{i}"), self.sizes[i])
                    .track_height(h)
                    .on_change(Self::set(cx, move |g, v| g.sizes[i] = v)),
            );
        }
        out.push(
            section(cx, "Track sizes", "Expressive では太いトラックも用意されている", sizes)
                .into_any_element(),
        );
        out
    }

    fn log(cx: &mut Context<Self>, msg: &'static str) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static {
        let set = Self::set(cx, move |g, _: ()| g.menu_log = msg.into());
        move |_, w, cx| set((), w, cx)
    }

    fn page_menus(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let c = cx.m3().colors.clone();

        let edit_menu = Menu::new("edit-menu")
            .item(MenuItem::new("切り取り").icon(IconName::ContentCut).trailing_text("Ctrl+X").on_click(Self::log(cx, "切り取り")))
            .item(MenuItem::new("コピー").icon(IconName::ContentCopy).trailing_text("Ctrl+C").on_click(Self::log(cx, "コピー")))
            .item(MenuItem::new("貼り付け").icon(IconName::ContentPaste).trailing_text("Ctrl+V").on_click(Self::log(cx, "貼り付け")))
            .divider()
            .item(MenuItem::new("共有").icon(IconName::Share).on_click(Self::log(cx, "共有")))
            .item(MenuItem::new("削除").icon(IconName::Delete).disabled(true));

        let sorts = ["名前", "更新日", "サイズ", "種類"];
        let mut sort_menu = Menu::new("sort-menu").header("並べ替え");
        for (i, s) in sorts.into_iter().enumerate() {
            sort_menu = sort_menu.item(
                MenuItem::new(s)
                    .selected(self.sort_by == i)
                    .on_click(cx.listener(move |g, _, _, cx| {
                        g.sort_by = i;
                        cx.notify();
                    })),
            );
        }

        let account_menu = Menu::new("account-menu")
            .vibrant(true)
            .item(MenuItem::new("プロフィール").icon(IconName::Person).supporting("名前・写真を編集").on_click(Self::log(cx, "プロフィール")))
            .item(MenuItem::new("設定").icon(IconName::Settings).on_click(Self::log(cx, "設定")))
            .item(MenuItem::new("ヘルプ").icon(IconName::Help).on_click(Self::log(cx, "ヘルプ")))
            .divider()
            .item(MenuItem::new("ログアウト").icon(IconName::Logout).on_click(Self::log(cx, "ログアウト")));

        let dropdowns = wrap_row()
            .gap(px(16.))
            .child(DropdownMenu::new(
                "dd-edit",
                Button::new("dd-edit-btn", "編集").tonal().icon(IconName::Edit),
                edit_menu,
            ))
            .child(DropdownMenu::new(
                "dd-sort",
                Button::new("dd-sort-btn", format!("並べ替え: {}", sorts[self.sort_by]))
                    .outlined()
                    .icon(IconName::Sort),
                sort_menu,
            ))
            .child(
                DropdownMenu::new(
                    "dd-account",
                    IconButton::new("dd-account-btn", IconName::MoreVert).tonal(),
                    account_menu,
                )
                .align_right(true),
            )
            .child(caption(cx, format!("最後に選んだ項目: {}", self.menu_log)));

        let context = ContextMenuArea::new(
            "ctx-area",
            Menu::new("ctx-menu")
                .item(MenuItem::new("戻る").icon(IconName::ArrowBack).trailing_text("Alt+←").on_click(Self::log(cx, "戻る")))
                .item(MenuItem::new("再読み込み").icon(IconName::Refresh).trailing_text("F5").on_click(Self::log(cx, "再読み込み")))
                .divider()
                .item(MenuItem::new("ブックマークに追加").icon(IconName::Bookmark).on_click(Self::log(cx, "ブックマーク"))),
        )
        .h(px(120.))
        .w_full()
        .flex()
        .items_center()
        .justify_center()
        .rounded(shape::LARGE)
        .border_2()
        .border_dashed()
        .border_color(c.outline_variant)
        .text_color(c.on_surface_variant)
        .child("ここを右クリック");

        let notch = |c: &ColorScheme| c.surface_container_low;
        let fruits = ["りんご", "みかん", "ぶどう", "もも", "メロン"];
        let countries = ["日本", "アメリカ", "イギリス", "フランス"];
        let selects = div()
            .flex()
            .gap(px(24.))
            .child(
                div().flex_1().child(
                    Select::new("sel-fruit", "好きな果物")
                        .options(fruits)
                        .selected(self.fruit)
                        .on_change(Self::set(cx, |g, i| g.fruit = Some(i))),
                ),
            )
            .child(
                div().flex_1().child(
                    Select::new("sel-country", "国")
                        .outlined(true)
                        .notch_background(notch)
                        .leading_icon(IconName::Home)
                        .options(countries)
                        .selected(self.country)
                        .on_change(Self::set(cx, |g, i| g.country = Some(i))),
                ),
            );

        vec![
            section(
                cx,
                "Dropdown menus",
                "アイコン・ショートカット・区切り線・見出し・選択状態・無効項目。Vibrant 配色（右端）",
                dropdowns,
            )
            .into_any_element(),
            section(cx, "Context menu", "右クリックした位置に開く", context).into_any_element(),
            section(
                cx,
                "Exposed dropdown (Select)",
                "フィールド幅に合わせたメニューから選択",
                selects,
            )
            .into_any_element(),
        ]
    }

    fn page_app_bars(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let c = cx.m3().colors.clone();
        let kind = AppBarKind::ALL[self.app_bar_kind];
        let kinds = ["Small", "Center", "Medium", "Large"];
        let mut kind_group = ConnectedButtonGroup::new("ab-kind")
            .selected((0..4).map(|i| i == self.app_bar_kind).collect())
            .on_toggle(Self::set(cx, |g, i| {
                g.app_bar_kind = i;
                g.app_scroll.set_offset(gpui::point(px(0.), px(0.)));
            }));
        for k in kinds {
            kind_group = kind_group.item(ButtonGroupItem::new(k));
        }

        let mut list = div().flex().flex_col();
        for i in 0..24 {
            list = list.child(
                ListItem::new(format!("ab-li-{i}"), format!("メッセージ {}", i + 1))
                    .supporting("スクロールするとアプリバーが縮みます")
                    .leading(
                        div()
                            .size(px(40.))
                            .rounded_full()
                            .bg(c.secondary_container)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(Icon::new(IconName::Person).color(c.on_secondary_container)),
                    ),
            );
        }

        let phone = div()
            .w(px(400.))
            .h(px(560.))
            .flex_none()
            .flex()
            .flex_col()
            .rounded(shape::EXTRA_LARGE)
            .overflow_hidden()
            .border_1()
            .border_color(c.outline_variant)
            .bg(c.surface)
            .child(
                TopAppBar::new("受信トレイ")
                    .kind(kind)
                    .subtitle("未読 12 件")
                    .navigation(IconButton::new("ab-nav", IconName::ArrowBack))
                    .action(IconButton::new("ab-search", IconName::Search))
                    .action(
                        DropdownMenu::new(
                            "ab-more",
                            IconButton::new("ab-more-btn", IconName::MoreVert),
                            Menu::new("ab-more-menu")
                                .item(MenuItem::new("すべて既読にする").icon(IconName::Check))
                                .item(MenuItem::new("設定").icon(IconName::Settings)),
                        )
                        .align_right(true),
                    )
                    .scroll_handle(&self.app_scroll),
            )
            .child(
                div()
                    .id("ab-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.app_scroll)
                    .on_scroll(|_, _, cx| cx.stop_propagation())
                    .child(list),
            )
            .relative()
            .child(
                div().absolute().right(px(16.)).bottom(px(16.)).child(
                    FabMenu::new("ab-fab", IconName::Edit)
                        .item(FabMenuItem::new(IconName::Mail, "新規メール"))
                        .item(FabMenuItem::new(IconName::Person, "連絡先を追加"))
                        .item(FabMenuItem::new(IconName::Image, "写真を送る")),
                ),
            );

        let tab_body = |text: String| {
            div()
                .h(px(96.))
                .flex()
                .items_center()
                .justify_center()
                .text_color(c.on_surface_variant)
                .m3_type(TypeScale::BodyLarge)
                .child(text)
        };
        let primary_names = ["フライト", "ホテル", "アクティビティ"];
        let secondary_names = ["概要", "仕様", "レビュー", "Q&A"];
        let tabs = div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                div()
                    .rounded(shape::LARGE)
                    .overflow_hidden()
                    .bg(c.surface)
                    .child(
                        Tabs::new("tabs-primary", self.primary_tab)
                            .tab(Tab::new(primary_names[0]).icon(IconName::Send))
                            .tab(Tab::new(primary_names[1]).icon(IconName::Home).badge(Some(3)))
                            .tab(Tab::new(primary_names[2]).icon(IconName::Star))
                            .on_select(Self::set(cx, |g, i| g.primary_tab = i)),
                    )
                    .child(tab_body(format!("Primary: {}", primary_names[self.primary_tab]))),
            )
            .child(
                div()
                    .rounded(shape::LARGE)
                    .overflow_hidden()
                    .bg(c.surface)
                    .child(
                        Tabs::new("tabs-secondary", self.secondary_tab)
                            .secondary()
                            .tab(Tab::new(secondary_names[0]))
                            .tab(Tab::new(secondary_names[1]))
                            .tab(Tab::new(secondary_names[2]).badge(None))
                            .tab(Tab::new(secondary_names[3]))
                            .on_select(Self::set(cx, |g, i| g.secondary_tab = i)),
                    )
                    .child(tab_body(format!(
                        "Secondary: {}",
                        secondary_names[self.secondary_tab]
                    ))),
            );

        vec![
            section(
                cx,
                "Top app bar",
                "Small / Center-aligned / Medium・Large flexible。中身をスクロールすると縮んで色が変わる",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(kind_group)
                    .child(phone),
            )
            .into_any_element(),
            section(
                cx,
                "Tabs",
                "Primary（内容幅のインジケーター）と Secondary（タブ幅）。インジケーターがバネで滑る",
                tabs,
            )
            .into_any_element(),
            section(
                cx,
                "Bottom sheet",
                "スクリム付きのモーダル。ハンドルを下にドラッグ、またはスクリムをクリックで閉じる",
                div().flex().child(Button::new("open-sheet", "ボトムシートを開く")
                    .icon(IconName::ArrowDropUp)
                    .on_click(cx.listener(|g, _, _, cx| {
                        g.sheet_open = true;
                        cx.notify();
                    }))),
            )
            .into_any_element(),
            self.toolbars_section(cx),
        ]
    }

    fn toolbars_section(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let c = cx.m3().colors.clone();
        let icons = [
            (IconName::Edit, "編集"),
            (IconName::Image, "画像"),
            (IconName::Label, "ラベル"),
            (IconName::Share, "共有"),
            (IconName::MoreVert, "その他"),
        ];
        // k 番目のツールバーの項目一覧を作る
        let actions = |cx: &mut Context<Self>, k: usize, sel: usize, n: usize| {
            icons
                .iter()
                .take(n)
                .enumerate()
                .map(|(i, (icon, tip))| {
                    ToolbarItem::action(*icon)
                        .tooltip(*tip)
                        .selected(i == sel)
                        .on_click(cx.listener(move |g, _, _, cx| {
                            g.tb_sel[k] = i;
                            cx.notify();
                        }))
                })
                .collect::<Vec<_>>()
        };
        let frame = |content: gpui::Div| {
            div()
                .relative()
                .w_full()
                .h(px(140.))
                .rounded(shape::LARGE)
                .overflow_hidden()
                .bg(c.surface)
                .border_1()
                .border_color(c.outline_variant)
                .child(content)
        };

        // Docked
        let mut docked = DockedToolbar::new("tb-docked");
        for a in actions(cx, 0, self.tb_sel[0], 5) {
            docked = docked.item(a);
        }
        let mut docked_v = DockedToolbar::new("tb-docked-v").vibrant(true);
        for a in actions(cx, 1, self.tb_sel[1], 5) {
            docked_v = docked_v.item(a);
        }

        // Floating（横・FAB 付き / Vibrant）
        let mut floating = FloatingToolbar::new("tb-float").fab(IconName::Add, None);
        for a in actions(cx, 2, self.tb_sel[2], 4) {
            floating = floating.item(a);
        }
        let mut vertical = FloatingToolbar::new("tb-vert").vibrant(true).vertical(true);
        for a in actions(cx, 3, self.tb_sel[3], 4) {
            vertical = vertical.item(a);
        }

        // スクロールで隠れるデモ
        let mut list = div().flex().flex_col().gap(px(8.)).p(px(16.));
        for i in 0..20 {
            list = list.child(
                div()
                    .h(px(56.))
                    .rounded(shape::MEDIUM)
                    .bg(c.surface_container_low)
                    .flex()
                    .items_center()
                    .px(px(16.))
                    .text_color(c.on_surface_variant)
                    .child(format!("ノート {}", i + 1)),
            );
        }
        let mut hiding = FloatingToolbar::new("tb-hide")
            .vibrant(true)
            .visible(self.tb_visible)
            .fab(IconName::Edit, None);
        for (icon, tip) in icons.iter().take(3) {
            hiding = hiding.item(ToolbarItem::action(*icon).tooltip(*tip));
        }
        let scroller = div()
            .relative()
            .w(px(400.))
            .h(px(420.))
            .flex_none()
            .rounded(shape::EXTRA_LARGE)
            .overflow_hidden()
            .bg(c.surface)
            .border_1()
            .border_color(c.outline_variant)
            .child(
                div()
                    .id("tb-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.tb_scroll)
                    .on_scroll(cx.listener(|g, e: &gpui::ScrollEvent, _, cx| {
                        let dy = f32::from(e.delta.pixel_delta(px(20.)).y);
                        let visible = dy > 0.;
                        if dy != 0. && g.tb_visible != visible {
                            g.tb_visible = visible;
                            cx.notify();
                        }
                        cx.stop_propagation();
                    }))
                    .child(list),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom(px(16.))
                    .flex()
                    .justify_center()
                    .child(hiding),
            );

        section(
            cx,
            "Toolbars",
            "Docked（Standard / Vibrant）、Floating（横・FAB 付き / 縦）、スクロールで隠れる Floating。選択中の項目はピル型に広がる",
            div()
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(
                    div()
                        .flex()
                        .gap(px(16.))
                        .child(div().flex_1().child(frame(
                            div().size_full().flex().flex_col().justify_end().child(docked),
                        )))
                        .child(div().flex_1().child(frame(
                            div()
                                .size_full()
                                .flex()
                                .flex_col()
                                .justify_end()
                                .child(docked_v),
                        ))),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(32.))
                        .child(floating)
                        .child(vertical),
                )
                .child(caption(cx, "↓ 中身を下へスクロールすると隠れ、上へスクロールすると戻る"))
                .child(scroller),
        )
        .into_any_element()
    }

    /// 現在の選択から Dynamic color を適用する
    fn apply_color(&self, cx: &mut App) {
        let seed = match self.seed {
            None => None,
            Some(usize::MAX) => Some(seed_from_hue(self.hue)),
            Some(i) => Some(SEEDS[i].1),
        };
        let contrast = [0., 0.5, 1.][self.contrast];
        let config = seed.map(|s| {
            DynamicColorConfig::new(s)
                .variant(SchemeVariant::ALL[self.variant])
                .contrast(contrast)
        });
        m3::set_dynamic_color(cx, config);
    }

    /// 状態を更新してテーマを作り直すハンドラ
    fn set_color<V: 'static>(
        cx: &mut Context<Self>,
        f: impl Fn(&mut Self, V) + 'static,
    ) -> impl Fn(V, &mut Window, &mut App) + 'static {
        let this = cx.entity().downgrade();
        move |v, _, cx| {
            this.update(cx, |g, cx| {
                f(g, v);
                g.apply_color(cx);
                cx.notify();
            })
            .ok();
        }
    }

    fn page_color(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let c = cx.m3().colors.clone();

        // ----- シードカラー -----
        let swatch = |id: String, color: gpui::Hsla, selected: bool, label: &str, check: gpui::Hsla| {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .id(gpui::ElementId::Name(id.into()))
                        .size(px(48.))
                        .rounded_full()
                        .bg(color)
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .border_2()
                        .border_color(if selected { c.on_surface } else { color })
                        .when(selected, |d| d.child(Icon::new(IconName::Check).color(check))),
                )
                .child(
                    div()
                        .m3_type(TypeScale::LabelMedium)
                        .text_color(c.on_surface_variant)
                        .child(label.to_string()),
                )
        };
        fn pick(
            cx: &mut Context<Gallery>,
            seed: Option<usize>,
        ) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static {
            let set = Gallery::set_color(cx, move |g, _: ()| g.seed = seed);
            move |_, w, cx| set((), w, cx)
        }
        let mut swatches = wrap_row().gap(px(16.)).items_start().child(
            div()
                .id("seed-base-wrap")
                .on_click(pick(cx, None))
                .child(swatch(
                    "seed-base".into(),
                    gpui::rgb(0x6750A4).into(),
                    self.seed.is_none(),
                    "Baseline（固定）",
                    gpui::white(),
                )),
        );
        for (i, (name, seed)) in SEEDS.iter().enumerate() {
            swatches = swatches.child(
                div()
                    .id(gpui::ElementId::Name(format!("seed-{i}-wrap").into()))
                    .on_click(pick(cx, Some(i)))
                    .child(swatch(
                        format!("seed-{i}"),
                        gpui::rgb(*seed).into(),
                        self.seed == Some(i),
                        name,
                        gpui::white(),
                    )),
            );
        }
        let hue_color: gpui::Hsla = gpui::rgb(seed_from_hue(self.hue)).into();
        swatches = swatches.child(
            div()
                .id("seed-hue-wrap")
                .on_click(pick(cx, Some(usize::MAX)))
                .child(swatch(
                    "seed-hue".into(),
                    hue_color,
                    self.seed == Some(usize::MAX),
                    "カスタム",
                    gpui::white(),
                )),
        );
        let hue_slider = div()
            .flex()
            .items_center()
            .gap(px(16.))
            .child(div().w(px(120.)).child(caption(cx, format!("色相 {:.0}°", self.hue))))
            .child(
                div().flex_1().child(
                    Slider::new("hue-slider", self.hue)
                        .range(0., 360.)
                        .on_change(Self::set_color(cx, |g, v: f32| {
                            g.hue = v;
                            g.seed = Some(usize::MAX);
                        })),
                ),
            );

        // ----- 方式とコントラスト -----
        let mut variants = wrap_row().gap(px(8.));
        for (i, v) in SchemeVariant::ALL.into_iter().enumerate() {
            let set = Self::set_color(cx, move |g, _: ()| {
                g.variant = i;
                if g.seed.is_none() {
                    g.seed = Some(0);
                }
            });
            variants = variants.child(
                Chip::filter(format!("variant-{i}"), v.name())
                    .selected(self.variant == i)
                    .on_click(move |_, w, cx| set((), w, cx)),
            );
        }
        let contrast = ConnectedButtonGroup::new("contrast")
            .item(ButtonGroupItem::new("標準"))
            .item(ButtonGroupItem::new("中"))
            .item(ButtonGroupItem::new("高"))
            .selected((0..3).map(|i| i == self.contrast).collect())
            .on_toggle(Self::set_color(cx, |g, i: usize| {
                g.contrast = i;
                if g.seed.is_none() {
                    g.seed = Some(0);
                }
            }));

        // ----- カラーロール -----
        let hex = |h: gpui::Hsla| {
            let r = gpui::Rgba::from(h);
            format!(
                "#{:02X}{:02X}{:02X}",
                (r.r * 255.).round() as u8,
                (r.g * 255.).round() as u8,
                (r.b * 255.).round() as u8
            )
        };
        let tile = |name: &str, bg: gpui::Hsla, fg: gpui::Hsla| {
            div()
                .flex_1()
                .min_w(px(150.))
                .h(px(72.))
                .p(px(10.))
                .flex()
                .flex_col()
                .justify_between()
                .rounded(shape::MEDIUM)
                .bg(bg)
                .text_color(fg)
                .child(div().m3_type(TypeScale::LabelLarge).child(name.to_string()))
                .child(div().m3_type(TypeScale::LabelSmall).child(hex(bg)))
        };
        let row = |tiles: Vec<gpui::Div>| div().flex().gap(px(8.)).children(tiles);
        let roles = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(row(vec![
                tile("Primary", c.primary, c.on_primary),
                tile("Secondary", c.secondary, c.on_secondary),
                tile("Tertiary", c.tertiary, c.on_tertiary),
                tile("Error", c.error, c.on_error),
            ]))
            .child(row(vec![
                tile("Primary container", c.primary_container, c.on_primary_container),
                tile("Secondary container", c.secondary_container, c.on_secondary_container),
                tile("Tertiary container", c.tertiary_container, c.on_tertiary_container),
                tile("Error container", c.error_container, c.on_error_container),
            ]))
            .child(row(vec![
                tile("Surface", c.surface, c.on_surface),
                tile("Container low", c.surface_container_low, c.on_surface),
                tile("Container", c.surface_container, c.on_surface),
                tile("Container high", c.surface_container_high, c.on_surface),
                tile("Container highest", c.surface_container_highest, c.on_surface),
            ]))
            .child(row(vec![
                tile("On surface", c.on_surface, c.surface),
                tile("On surface variant", c.on_surface_variant, c.surface),
                tile("Outline", c.outline, c.surface),
                tile("Outline variant", c.outline_variant, c.on_surface),
                tile("Inverse surface", c.inverse_surface, c.inverse_on_surface),
            ]));

        // ----- プレビュー -----
        let preview = div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .child(
                wrap_row()
                    .child(Button::new("pv-filled", "Filled").icon(IconName::Send))
                    .child(Button::new("pv-tonal", "Tonal").tonal())
                    .child(Button::new("pv-outlined", "Outlined").outlined())
                    .child(Fab::new("pv-fab", IconName::Edit))
                    .child(Fab::new("pv-fab2", IconName::Add).color(FabColor::TertiaryContainer))
                    .child(Switch::new("pv-switch", true).icons(true))
                    .child(Checkbox::new("pv-check", true))
                    .child(Chip::filter("pv-chip", "選択中").selected(true))
                    .child(Badge::new(Some(8))),
            )
            .child(LinearProgress::new("pv-progress", Some(0.6)).wavy(true))
            .child(Slider::new("pv-slider", 0.4))
            .child(
                div()
                    .flex()
                    .gap(px(24.))
                    .items_center()
                    .child(CircularProgress::new("pv-cp", None).wavy(true))
                    .child(LoadingIndicator::new("pv-li").contained(true))
                    .child(
                        div().w(px(320.)).child(
                            NavigationBar::new("pv-nav", 0)
                                .item(NavItem::new(IconName::Home, "ホーム"))
                                .item(NavItem::new(IconName::Search, "検索"))
                                .item(NavItem::new(IconName::Person, "自分")),
                        ),
                    ),
            );

        let current = match self.seed {
            None => "ベースライン（#6750A4 の固定スキーム）".to_string(),
            Some(usize::MAX) => format!(
                "シード {}（色相 {:.0}°）",
                hex(hue_color),
                self.hue
            ),
            Some(i) => format!("シード {} {}", SEEDS[i].0, hex(gpui::rgb(SEEDS[i].1).into())),
        };

        vec![
            section(
                cx,
                "Seed color",
                "シード色 1 つから HCT 色空間でトーナルパレットを作り、全カラーロールを生成（material-color-utilities / 2025 スペック）",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(swatches)
                    .child(hue_slider)
                    .child(caption(cx, format!("現在: {current}"))),
            )
            .into_any_element(),
            section(
                cx,
                "Scheme variant & contrast",
                "同じシードでも方式によって配色の性格が変わる。コントラストを上げるとテキストと背景の差が大きくなる",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(variants)
                    .child(contrast),
            )
            .into_any_element(),
            section(cx, "Color roles", "生成されたカラーロール", roles).into_any_element(),
            section(cx, "Preview", "全コンポーネントが即座に新しい配色で描画される", preview)
                .into_any_element(),
        ]
    }

    fn page_text_fields(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let f = &self.fields;
        let col = |a: &Entity<TextField>, b: &Entity<TextField>| {
            div()
                .flex()
                .gap(px(24.))
                .child(div().flex_1().child(a.clone()))
                .child(div().flex_1().child(b.clone()))
        };
        let values: Vec<SharedString> = f.iter().take(4).map(|f| f.read(cx).text()).collect();
        let summary = format!(
            "入力値: 名前=「{}」 メール=「{}」 パスワード={}文字{}",
            values[0],
            values[1],
            values[2].chars().count(),
            self.submitted
                .as_ref()
                .map(|s| format!("（送信: {s}）"))
                .unwrap_or_default()
        );
        vec![
            section(
                cx,
                "Filled text fields",
                "フローティングラベル・クリアボタン・入力検証（メール）・パスワード表示切替・文字数カウンター",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(col(&f[0], &f[1]))
                    .child(col(&f[2], &f[3]))
                    .child(caption(cx, summary)),
            )
            .into_any_element(),
            section(
                cx,
                "Outlined text fields",
                "ラベルは枠線の上に浮き上がる。エラー状態・無効状態",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .child(col(&f[4], &f[5]))
                    .child(col(&f[6], &f[7])),
            )
            .into_any_element(),
            section(
                cx,
                "キーボード操作",
                "IME（日本語入力）対応",
                caption(
                    cx,
                    "←→ / Ctrl+←→ 単語移動、Shift で選択、Home・End、Ctrl+A・C・X・V、Ctrl+Z・Y、Ctrl+Backspace 単語削除、ダブルクリックで単語選択・トリプルクリックで全選択",
                ),
            )
            .into_any_element(),
        ]
    }

    fn page_feedback(&mut self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let mut out = vec![];
        let c = cx.m3().colors.clone();

        let actions = wrap_row()
            .child(
                Button::new("open-dialog", "ダイアログを開く")
                    .icon(IconName::Delete)
                    .on_click(cx.listener(|g, _, _, cx| {
                        g.dialog = true;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("open-snack", "スナックバーを表示")
                    .tonal()
                    .icon(IconName::Mail)
                    .on_click(cx.listener(|g, _, _, cx| g.show_snackbar(cx))),
            )
            .child(
                div()
                    .id("tt-1")
                    .tooltip(m3::tooltip("ツールチップ（Plain tooltip）"))
                    .child(IconButton::new("tt-btn", IconName::Search).outlined()),
            )
            .child(caption(cx, "← ホバーでツールチップ"));
        out.push(
            section(
                cx,
                "Dialog / Snackbar / Tooltip",
                "ダイアログはスクリム付きのオーバーレイ、スナックバーは 4 秒で自動的に閉じる",
                actions,
            )
            .into_any_element(),
        );

        let nav = div()
            .w(px(420.))
            .rounded(shape::LARGE)
            .overflow_hidden()
            .border_1()
            .border_color(c.outline_variant)
            .child(
                div()
                    .h(px(120.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(c.surface)
                    .text_color(c.on_surface_variant)
                    .m3_type(TypeScale::BodyLarge)
                    .child(["ホーム", "検索", "受信トレイ", "プロフィール"][self.nav_bar]),
            )
            .child(
                NavigationBar::new("nb", self.nav_bar)
                    .item(NavItem::new(IconName::Home, "ホーム"))
                    .item(NavItem::new(IconName::Search, "検索"))
                    .item(NavItem::new(IconName::Mail, "受信").badge(Some(12)))
                    .item(NavItem::new(IconName::Person, "プロフィール").badge(None))
                    .on_select(Self::set(cx, |g, i| g.nav_bar = i)),
            );
        out.push(
            section(cx, "Navigation bar", "アクティブインジケーターが中央からバネで広がる", nav)
                .into_any_element(),
        );

        // ----- Navigation rail（展開型 / モーダル展開型）-----
        let demo_rail = |id: &'static str, sel: usize| {
            NavigationRail::new(id, sel)
                .fab(IconName::Edit, "作成", None)
                .item(NavItem::new(IconName::Home, "ホーム"))
                .item(NavItem::new(IconName::Search, "検索"))
                .item(NavItem::new(IconName::Mail, "受信トレイ").badge(Some(12)))
                .section("ライブラリ")
                .item(NavItem::new(IconName::Bookmark, "ブックマーク"))
                .item(NavItem::new(IconName::Star, "お気に入り"))
        };
        let toggle_demo = Self::set(cx, |g, _: ()| g.demo_rail_expanded = !g.demo_rail_expanded);
        let inline_rail = demo_rail("demo-rail", self.demo_rail_sel)
            .expanded(self.demo_rail_expanded)
            .on_toggle(move |w, cx| toggle_demo((), w, cx))
            .on_select(Self::set(cx, |g, i: usize| g.demo_rail_sel = i));
        let page_names = ["ホーム", "検索", "受信トレイ", "ブックマーク", "お気に入り"];
        let page_label = |name: &'static str| {
            div()
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(c.on_surface_variant)
                .m3_type(TypeScale::BodyLarge)
                .child(name)
        };
        let inline_frame = div()
            .relative()
            .w_full()
            .h(px(520.))
            .flex()
            .rounded(shape::LARGE)
            .overflow_hidden()
            .border_1()
            .border_color(c.outline_variant)
            .bg(c.surface)
            .child(inline_rail)
            .child(page_label(page_names[self.demo_rail_sel]));

        let open_modal = Self::set(cx, |g, _: ()| g.modal_rail_open = true);
        let close_modal = Self::set(cx, |g, _: ()| g.modal_rail_open = false);
        let modal_frame = div()
            .relative()
            .w_full()
            .h(px(520.))
            .rounded(shape::LARGE)
            .overflow_hidden()
            .border_1()
            .border_color(c.outline_variant)
            .bg(c.surface)
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_color(c.on_surface_variant)
                            .m3_type(TypeScale::BodyLarge)
                            .child(page_names[self.demo_rail_sel]),
                    )
                    .child(
                        Button::new("open-modal-rail", "モーダルレールを開く")
                            .tonal()
                            .icon(IconName::Menu)
                            .on_click(move |_, w, cx| open_modal((), w, cx)),
                    ),
            )
            .child(
                demo_rail("demo-modal-rail", self.demo_rail_sel)
                    .modal(true)
                    .expanded(self.modal_rail_open)
                    .on_dismiss(move |w, cx| close_modal((), w, cx))
                    .on_select(Self::set(cx, |g, i: usize| g.demo_rail_sel = i)),
            );
        out.push(
            section(
                cx,
                "Navigation rail",
                "左: 展開型（メニューボタンで開閉し、本文を押し出す）。右: モーダル展開型（スクリムの上に重ねて表示）。FAB は展開すると Extended FAB に変わる",
                div()
                    .flex()
                    .gap(px(16.))
                    .child(div().flex_1().child(inline_frame))
                    .child(div().flex_1().child(modal_frame)),
            )
            .into_any_element(),
        );

        let mut type_scale = div().flex().flex_col().gap(px(4.));
        for t in TypeScale::ALL {
            type_scale = type_scale.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(16.))
                    .child(div().w(px(140.)).child(caption(cx, t.name())))
                    .child(
                        div()
                            .m3_type(t)
                            .text_color(c.on_surface)
                            .child("Material 3 Expressive あいう"),
                    ),
            );
        }
        out.push(section(cx, "Typography", "M3 タイプスケール 15 段階", type_scale).into_any_element());
        out
    }
}

impl Render for Gallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.m3().clone();
        let c = theme.colors.clone();
        let dark = theme.is_dark();

        let sections = match self.page {
            0 => self.page_buttons(cx),
            1 => self.page_selection(cx),
            2 => self.page_chips(cx),
            3 => self.page_progress(cx),
            4 => self.page_sliders(cx),
            5 => self.page_feedback(cx),
            6 => self.page_text_fields(cx),
            7 => self.page_menus(cx),
            8 => self.page_app_bars(cx),
            _ => self.page_color(cx),
        };

        let toggle_rail = Self::set(cx, |g, _: ()| g.rail_expanded = !g.rail_expanded);
        let mut rail = NavigationRail::new("rail", self.page)
            .expanded(self.rail_expanded)
            .on_toggle(move |w, cx| toggle_rail((), w, cx))
            .footer(
                div()
                    .id("theme-toggle-tt")
                    .tooltip(m3::tooltip(if dark { "ライトテーマ" } else { "ダークテーマ" }))
                    .child(
                        IconButton::new(
                            "theme-toggle",
                            if dark { IconName::LightMode } else { IconName::DarkMode },
                        )
                        .on_click(move |_, _, cx| {
                            m3::set_brightness(
                                cx,
                                if dark { Brightness::Light } else { Brightness::Dark },
                            )
                        }),
                    ),
            )
            .on_select(Self::set(cx, |g, i| g.page = i));
        for (i, (name, icon)) in PAGES.into_iter().enumerate() {
            let mut item = NavItem::new(icon, name);
            if i == 5 {
                item = item.badge(None);
            }
            rail = rail.item(item);
        }

        let content = div()
            .id(gpui::ElementId::Name(format!("content-{}", self.page).into()))
            .flex_1()
            .h_full()
            .overflow_y_scroll()
            .bg(c.surface_container)
            .rounded_tl(shape::EXTRA_LARGE)
            .rounded_bl(shape::EXTRA_LARGE)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .p(px(32.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .pb(px(8.))
                            .child(
                                div()
                                    .m3_type_emphasized(TypeScale::DisplaySmall)
                                    .text_color(c.on_surface)
                                    .child(PAGES[self.page].0),
                            )
                            .child(
                                div()
                                    .m3_type(TypeScale::BodyLarge)
                                    .text_color(c.on_surface_variant)
                                    .child("Material Design 3 Expressive × GPUI"),
                            ),
                    )
                    .children(sections),
            );

        let mut root = div()
            .relative()
            .size_full()
            .font_family(theme.font_family)
            .bg(c.surface)
            .text_color(c.on_surface)
            .child(div().size_full().flex().child(rail).child(content));

        if self.dialog {
            root = root.child(
                div()
                    .id("scrim")
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(c.scrim.opacity(0.32))
                    .on_click(cx.listener(|g, _, _, cx| {
                        g.dialog = false;
                        cx.notify();
                    }))
                    .child(
                        Dialog::new(
                            "ファイルを削除しますか？",
                            "削除したファイルは 30 日間ゴミ箱に保存され、その後完全に削除されます。",
                        )
                        .icon(IconName::Delete)
                        .action(Button::new("dlg-cancel", "キャンセル").text().on_click(
                            cx.listener(|g, _, _, cx| {
                                g.dialog = false;
                                cx.notify();
                            }),
                        ))
                        .action(Button::new("dlg-ok", "削除").text().on_click(cx.listener(
                            |g, _, _, cx| {
                                g.dialog = false;
                                g.show_snackbar(cx);
                            },
                        ))),
                    ),
            );
        }
        if self.snackbar {
            root = root.child(
                div()
                    .absolute()
                    .bottom(px(24.))
                    .left(px(96.))
                    .right_0()
                    .flex()
                    .justify_center()
                    .child(
                        Snackbar::new("ファイルを削除しました")
                            .action(
                                "元に戻す",
                                cx.listener(|g, _, _, cx| {
                                    g.snackbar = false;
                                    cx.notify();
                                }),
                            )
                            .on_close(cx.listener(|g, _, _, cx| {
                                g.snackbar = false;
                                cx.notify();
                            })),
                    ),
            );
        }

        // ボトムシート（閉じるアニメーションのため常に配置）
        let close_sheet = Self::set(cx, |g, _: ()| g.sheet_open = false);
        let sheet_item = |id: &'static str, icon: IconName, label: &'static str| {
            ListItem::new(id, label)
                .leading(Icon::new(icon).color(c.on_surface_variant))
                .on_click(|_, _, _| {})
        };
        root = root.child(
            BottomSheet::new("sheet", self.sheet_open)
                .on_dismiss(move |w, cx| close_sheet((), w, cx))
                .child(
                    div()
                        .px(px(24.))
                        .pb(px(8.))
                        .m3_type(TypeScale::TitleLarge)
                        .child("共有"),
                )
                .child(sheet_item("sh-0", IconName::Mail, "メールで送信"))
                .child(sheet_item("sh-1", IconName::ContentCopy, "リンクをコピー"))
                .child(sheet_item("sh-2", IconName::Bookmark, "ブックマークに追加"))
                .child(sheet_item("sh-3", IconName::Share, "その他のアプリ")),
        );
        root
    }
}

fn main() {
    application().with_assets(m3::Assets).run(|cx: &mut App| {
        m3::init(cx, Brightness::Dark);
        let bounds = Bounds::centered(None, size(px(1180.), px(860.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| Gallery::new(window, cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
