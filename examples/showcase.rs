//! GPUI スタイリング・ショーケース
//!
//! `cargo run --bin showcase` で起動。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use gpui::{
    Animation, AnimationExt as _, AnyElement, App, Bounds, BoxShadow, Context, FontWeight, Hsla,
    PathBuilder, SharedString, SpringAnimation, SpringConfig, Window, WindowBackgroundAppearance,
    WindowBounds, WindowOptions, bounce, canvas, checkerboard, div, ease_in_out, hsla,
    linear_color_stop, linear_gradient, pattern_slash, point, prelude::*, px, relative, rgb, rgba,
    size,
};
use gpui_platform::application;

// ---------------------------------------------------------------------------
// カラーパレット
// ---------------------------------------------------------------------------

const BG: u32 = 0x0f1117;
const SURFACE: u32 = 0x181b24;
const SURFACE_HI: u32 = 0x222634;
const BORDER: u32 = 0x2c3142;
const TEXT: u32 = 0xe6e8ef;
const MUTED: u32 = 0x8b91a5;
const ACCENT: u32 = 0x7c5cff;
const ACCENT_HI: u32 = 0x9b82ff;

fn shadow(color: Hsla, x: f32, y: f32, blur: f32, spread: f32) -> BoxShadow {
    BoxShadow {
        color,
        offset: point(px(x), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset: false,
    }
}

// ---------------------------------------------------------------------------
// 共通パーツ
// ---------------------------------------------------------------------------

/// セクション（見出し + 説明 + 中身）
fn section(title: &'static str, note: &'static str, body: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_5()
        .rounded_xl()
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(BORDER))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(div().text_sm().text_color(rgb(MUTED)).child(note)),
        )
        .child(body)
}

/// ラベル付きのサンプルタイル
fn tile(label: &'static str, sample: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(sample)
        .child(div().text_xs().text_color(rgb(MUTED)).child(label))
}

fn swatch() -> gpui::Div {
    div().size(px(88.)).rounded_lg()
}

fn row() -> gpui::Div {
    div().flex().flex_wrap().gap_5().items_end()
}

// ---------------------------------------------------------------------------
// ビュー
// ---------------------------------------------------------------------------

struct Showcase {
    mica: bool,
    toggle_on: bool,
    clicks: u32,
}

impl Showcase {
    // --- グラデーション ----------------------------------------------------
    fn gradients(&self) -> impl IntoElement {
        let g = |angle: f32, a: u32, b: u32| {
            linear_gradient(
                angle,
                linear_color_stop(rgb(a), 0.),
                linear_color_stop(rgb(b), 1.),
            )
        };
        section(
            "グラデーション",
            "linear_gradient(角度, 開始色, 終了色) ― 角度は 0°=上、時計回り",
            row()
                .child(tile("0°", swatch().bg(g(0., 0xff6b6b, 0xfeca57))))
                .child(tile("45°", swatch().bg(g(45., 0x48dbfb, 0x7c5cff))))
                .child(tile("90°", swatch().bg(g(90., 0x1dd1a1, 0x10ac84))))
                .child(tile("135°", swatch().bg(g(135., 0xf368e0, 0xff9f43))))
                .child(tile(
                    "停止位置 30%→70%",
                    swatch().bg(linear_gradient(
                        90.,
                        linear_color_stop(rgb(0x0abde3), 0.3),
                        linear_color_stop(rgb(0xee5253), 0.7),
                    )),
                ))
                .child(tile(
                    "半透明",
                    swatch().bg(linear_gradient(
                        180.,
                        linear_color_stop(rgba(0x7c5cffff), 0.),
                        linear_color_stop(rgba(0x7c5cff00), 1.),
                    )),
                )),
        )
    }

    // --- 影 --------------------------------------------------------------
    fn shadows(&self) -> impl IntoElement {
        let card = || swatch().bg(rgb(SURFACE_HI));
        section(
            "影 (BoxShadow)",
            "offset / blur / spread / inset を指定。vec! で何枚でも重ねられる",
            row()
                .gap_8()
                .child(tile("shadow_lg()", card().shadow_lg()))
                .child(tile(
                    "オフセット",
                    card().shadow(vec![shadow(hsla(0., 0., 0., 0.6), 8., 8., 0., 0.)]),
                ))
                .child(tile(
                    "グロー",
                    card().bg(rgb(ACCENT)).shadow(vec![shadow(
                        rgb(ACCENT).into(),
                        0.,
                        0.,
                        28.,
                        2.,
                    )]),
                ))
                .child(tile(
                    "多重",
                    card().shadow(vec![
                        shadow(rgb(0xff6b6b).into(), -6., -6., 0., 0.),
                        shadow(rgb(0x48dbfb).into(), 6., 6., 0., 0.),
                        shadow(hsla(0., 0., 0., 0.5), 0., 18., 30., -4.),
                    ]),
                ))
                .child(tile(
                    "インセット",
                    card().shadow(vec![BoxShadow {
                        inset: true,
                        ..shadow(hsla(0., 0., 0., 0.8), 0., 4., 14., 0.)
                    }]),
                )),
        )
    }

    // --- 枠線・角丸・パターン -------------------------------------------
    fn borders(&self) -> impl IntoElement {
        section(
            "枠線・角丸・パターン背景",
            "辺ごとの太さ、破線、角ごとの角丸、pattern_slash / checkerboard",
            row()
                .child(tile(
                    "辺ごと",
                    swatch()
                        .rounded_none()
                        .bg(rgb(SURFACE_HI))
                        .border_l_4()
                        .border_b_2()
                        .border_color(rgb(ACCENT)),
                ))
                .child(tile(
                    "破線",
                    swatch()
                        .border_2()
                        .border_dashed()
                        .border_color(rgb(0x1dd1a1)),
                ))
                .child(tile(
                    "角ごと",
                    swatch()
                        .rounded_none()
                        .rounded_tl(px(44.))
                        .rounded_br(px(44.))
                        .bg(rgb(0xff9f43)),
                ))
                .child(tile(
                    "rounded_full()",
                    swatch().rounded_full().bg(rgb(0x48dbfb)),
                ))
                .child(tile(
                    "pattern_slash",
                    swatch()
                        .bg(pattern_slash(rgb(ACCENT), 3., 6.))
                        .border_1()
                        .border_color(rgb(BORDER)),
                ))
                .child(tile(
                    "checkerboard",
                    swatch().bg(checkerboard(rgb(SURFACE_HI), 11.)),
                )),
        )
    }

    // --- テキスト --------------------------------------------------------
    fn typography(&self) -> impl IntoElement {
        let weights = [
            (FontWeight::THIN, "Thin"),
            (FontWeight::LIGHT, "Light"),
            (FontWeight::NORMAL, "Regular"),
            (FontWeight::SEMIBOLD, "Semibold"),
            (FontWeight::BOLD, "Bold"),
            (FontWeight::BLACK, "Black"),
        ];
        section(
            "テキスト",
            "ウェイト・装飾・省略・行数制限",
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div().flex().flex_wrap().gap_4().text_xl().children(
                        weights
                            .into_iter()
                            .map(|(w, name)| div().font_weight(w).child(name)),
                    ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_4()
                        .child(div().italic().child("イタリック Italic"))
                        .child(div().underline().child("下線"))
                        .child(
                            div()
                                .underline()
                                .text_decoration_wavy()
                                .text_decoration_color(rgb(0xff6b6b))
                                .child("波線（スペルミス風）"),
                        )
                        .child(div().line_through().text_color(rgb(MUTED)).child("取り消し線"))
                        .child(
                            div()
                                .px_1()
                                .text_bg(rgba(0xfeca5755))
                                .child("マーカー"),
                        ),
                )
                .child(
                    div()
                        .w(px(360.))
                        .p_2()
                        .rounded_md()
                        .bg(rgb(SURFACE_HI))
                        .truncate()
                        .child("truncate(): とても長い一行のテキストは幅に収まらないと末尾が…で省略されます"),
                )
                .child(
                    div()
                        .w(px(360.))
                        .p_2()
                        .rounded_md()
                        .bg(rgb(SURFACE_HI))
                        .line_clamp(2)
                        .child("line_clamp(2): 複数行のテキストを指定した行数で打ち切ります。ここには3行以上になる長い文章を入れていますが、2行目で打ち切られて3行目以降は表示されません。"),
                ),
        )
    }

    // --- インタラクション ------------------------------------------------
    fn interaction(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let toggle_on = self.toggle_on;
        let knob_x = if toggle_on { px(26.) } else { px(2.) };

        section(
            "インタラクション",
            "hover / active / group_hover、spring アニメーション付きトグル",
            row()
                .items_center()
                .gap_6()
                // ホバー & 押下
                .child(
                    div()
                        .id("primary")
                        .px_5()
                        .py_2()
                        .rounded_lg()
                        .bg(rgb(ACCENT))
                        .text_color(rgb(0xffffff))
                        .font_weight(FontWeight::SEMIBOLD)
                        .cursor_pointer()
                        .shadow(vec![shadow(rgba(0x7c5cff66).into(), 0., 4., 14., 0.)])
                        .hover(|s| {
                            s.bg(rgb(ACCENT_HI)).shadow(vec![shadow(
                                rgba(0x7c5cffaa).into(),
                                0.,
                                6.,
                                22.,
                                2.,
                            )])
                        })
                        .active(|s| s.opacity(0.75))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.clicks += 1;
                            cx.notify();
                        }))
                        .child(format!("クリック ({})", self.clicks)),
                )
                // アウトラインボタン
                .child(
                    div()
                        .id("ghost")
                        .px_5()
                        .py_2()
                        .rounded_lg()
                        .border_1()
                        .border_color(rgb(BORDER))
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(SURFACE_HI)).border_color(rgb(ACCENT)))
                        .child("アウトライン"),
                )
                // グループホバー: カードに乗ると中の矢印と文字が変わる
                .child(
                    div()
                        .id("group-card")
                        .group("card")
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_4()
                        .py_3()
                        .rounded_lg()
                        .bg(rgb(SURFACE_HI))
                        .border_1()
                        .border_color(rgb(BORDER))
                        .cursor_pointer()
                        .hover(|s| s.border_color(rgb(0x1dd1a1)))
                        .child(
                            div()
                                .text_color(rgb(MUTED))
                                .group_hover("card", |s| s.text_color(rgb(TEXT)))
                                .child("グループホバー"),
                        )
                        .child(
                            div()
                                .text_color(rgb(MUTED))
                                .group_hover("card", |s| s.text_color(rgb(0x1dd1a1)).ml_2())
                                .child("→"),
                        ),
                )
                // トグルスイッチ（spring）
                .child(
                    div()
                        .id("toggle")
                        .relative()
                        .w(px(52.))
                        .h(px(28.))
                        .rounded_full()
                        .cursor_pointer()
                        .bg(if toggle_on {
                            rgb(0x1dd1a1)
                        } else {
                            rgb(BORDER)
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_on = !this.toggle_on;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .absolute()
                                .top(px(2.))
                                .size(px(24.))
                                .rounded_full()
                                .bg(rgb(0xffffff))
                                .shadow_md()
                                .with_spring(
                                    "toggle-knob",
                                    SpringAnimation::new(SpringConfig::new(400., 18., 1.))
                                        .to(knob_x)
                                        .with_epsilon(0.2),
                                    |this, x| this.left(x),
                                ),
                        ),
                ),
        )
    }

    // --- アニメーション --------------------------------------------------
    fn animations(&self) -> impl IntoElement {
        section(
            "アニメーション",
            "with_animation(id, Animation, |要素, 進捗0..1| …) で毎フレーム値を補間",
            div()
                .flex()
                .flex_col()
                .gap_5()
                .child(
                    row()
                        .items_center()
                        .gap_8()
                        // 脈打つドット
                        .child(tile(
                            "パルス",
                            div()
                                .size(px(40.))
                                .flex()
                                .justify_center()
                                .items_center()
                                .child(
                                    div()
                                        .size(px(20.))
                                        .rounded_full()
                                        .bg(rgb(0xff6b6b))
                                        .with_animation(
                                            "pulse",
                                            Animation::new(Duration::from_millis(1200))
                                                .repeat()
                                                .with_easing(bounce(ease_in_out)),
                                            |el, t| {
                                                let s = px(20. + 20. * t);
                                                el.size(s).opacity(1. - 0.6 * t).shadow(vec![
                                                    shadow(
                                                        rgba(0xff6b6b99).into(),
                                                        0.,
                                                        0.,
                                                        24. * t,
                                                        0.,
                                                    ),
                                                ])
                                            },
                                        ),
                                ),
                        ))
                        // 色相が回るブロック
                        .child(tile(
                            "色相",
                            swatch().size(px(40.)).with_animation(
                                "hue",
                                Animation::new(Duration::from_secs(4)).repeat(),
                                |el, t| el.bg(hsla(t, 0.75, 0.6, 1.)),
                            ),
                        ))
                        // 回るグラデーション
                        .child(tile(
                            "グラデ回転",
                            swatch().size(px(40.)).with_animation(
                                "spin-gradient",
                                Animation::new(Duration::from_secs(3)).repeat(),
                                |el, t| {
                                    el.bg(linear_gradient(
                                        360. * t,
                                        linear_color_stop(rgb(0x48dbfb), 0.),
                                        linear_color_stop(rgb(ACCENT), 1.),
                                    ))
                                },
                            ),
                        ))
                        // 角丸モーフ
                        .child(tile(
                            "モーフ",
                            swatch().size(px(40.)).bg(rgb(0xfeca57)).with_animation(
                                "morph",
                                Animation::new(Duration::from_millis(1600))
                                    .repeat()
                                    .with_easing(bounce(ease_in_out)),
                                |el, t| el.rounded(px(20. * t)),
                            ),
                        )),
                )
                // インジケータ風プログレスバー
                .child(
                    div()
                        .relative()
                        .w_full()
                        .h(px(6.))
                        .rounded_full()
                        .bg(rgb(SURFACE_HI))
                        .overflow_hidden()
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .h_full()
                                .w(relative(0.3))
                                .rounded_full()
                                .bg(linear_gradient(
                                    90.,
                                    linear_color_stop(rgb(0x48dbfb), 0.),
                                    linear_color_stop(rgb(ACCENT), 1.),
                                ))
                                .with_animation(
                                    "progress",
                                    Animation::new(Duration::from_millis(1800))
                                        .repeat()
                                        .with_easing(ease_in_out),
                                    |el, t| el.left(relative(-0.3 + 1.3 * t)),
                                ),
                        ),
                ),
        )
    }

    // --- Canvas ----------------------------------------------------------
    fn painting(&self) -> impl IntoElement {
        section(
            "Canvas / PathBuilder",
            "canvas() でベクターパスを直接描画（塗り・線・ベジェ曲線）",
            div()
                .h(px(140.))
                .w_full()
                .rounded_lg()
                .bg(rgb(SURFACE_HI))
                .overflow_hidden()
                .child(
                    canvas(
                        |_, _, _| {},
                        |bounds, _, window, _| {
                            let o = bounds.origin;
                            let w = f32::from(bounds.size.width);
                            let h = f32::from(bounds.size.height);
                            let mid = h / 2.;
                            let wave = |phase: f32, amp: f32, x: f32| {
                                mid + amp * ((x / w) * std::f32::consts::TAU * 2. + phase).sin()
                            };

                            // 塗りつぶし波形（グラデーション）
                            let mut fill = PathBuilder::fill();
                            fill.move_to(o + point(px(0.), px(h)));
                            let mut x = 0.;
                            while x <= w {
                                fill.line_to(o + point(px(x), px(wave(0., 26., x))));
                                x += 4.;
                            }
                            fill.line_to(o + point(px(w), px(h)));
                            fill.close();
                            if let Ok(path) = fill.build() {
                                window.paint_path(
                                    path,
                                    linear_gradient(
                                        180.,
                                        linear_color_stop(rgba(0x7c5cffaa), 0.),
                                        linear_color_stop(rgba(0x7c5cff10), 1.),
                                    ),
                                );
                            }

                            // 線の波形
                            let mut stroke = PathBuilder::stroke(px(2.5));
                            stroke.move_to(o + point(px(0.), px(wave(1.6, 34., 0.))));
                            let mut x = 0.;
                            while x <= w {
                                stroke.line_to(o + point(px(x), px(wave(1.6, 34., x))));
                                x += 4.;
                            }
                            if let Ok(path) = stroke.build() {
                                window.paint_path(path, rgb(0x48dbfb));
                            }

                            // ベジェ曲線
                            let mut curve = PathBuilder::stroke(px(2.));
                            curve.move_to(o + point(px(0.), px(h - 10.)));
                            curve.curve_to(
                                o + point(px(w), px(10.)),
                                o + point(px(w * 0.5), px(h - 10.)),
                            );
                            if let Ok(path) = curve.build() {
                                window.paint_path(path, rgb(0xfeca57));
                            }
                        },
                    )
                    .size_full(),
                ),
        )
    }

    // --- ヘッダー --------------------------------------------------------
    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mica = self.mica;
        div()
            .flex()
            .items_center()
            .justify_between()
            .p_6()
            .rounded_xl()
            .bg(linear_gradient(
                120.,
                linear_color_stop(rgb(0x2a1f5c), 0.),
                linear_color_stop(rgb(0x0e3a4a), 1.),
            ))
            .border_1()
            .border_color(rgba(0xffffff1a))
            .shadow(vec![shadow(hsla(0., 0., 0., 0.4), 0., 12., 32., -8.)])
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_3xl()
                            .font_weight(FontWeight::BOLD)
                            .child("GPUI Styling Showcase"),
                    )
                    .child(div().text_color(rgba(0xffffffaa)).child(
                        "Zed の GPU アクセラレーテッド UI フレームワークでできる表現の見本帳",
                    )),
            )
            .child(
                div()
                    .id("mica")
                    .px_4()
                    .py_2()
                    .rounded_full()
                    .bg(rgba(0xffffff14))
                    .border_1()
                    .border_color(rgba(0xffffff33))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba(0xffffff26)))
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.mica = !this.mica;
                        window.set_background_appearance(if this.mica {
                            WindowBackgroundAppearance::MicaBackdrop
                        } else {
                            WindowBackgroundAppearance::Opaque
                        });
                        cx.notify();
                    }))
                    .child(if mica {
                        "背景: Mica ✓"
                    } else {
                        "背景: 不透明"
                    }),
            )
    }
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Mica 有効時は背景を半透明にして、デスクトップの色を透かす
        let bg = if self.mica {
            rgba(0x0f111766)
        } else {
            rgb(BG).into()
        };

        let sections: Vec<AnyElement> = vec![
            self.gradients().into_any_element(),
            self.shadows().into_any_element(),
            self.borders().into_any_element(),
            self.typography().into_any_element(),
            self.interaction(cx).into_any_element(),
            self.animations().into_any_element(),
            self.painting().into_any_element(),
        ];

        div()
            .font_family("Segoe UI")
            .size_full()
            .bg(bg)
            .text_color(rgb(TEXT))
            .child(
                div().id("scroll").size_full().overflow_y_scroll().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_5()
                        .p_6()
                        .child(self.header(cx))
                        .children(sections)
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(MUTED))
                                .text_center()
                                .pb_4()
                                .child(SharedString::from("Built with GPUI")),
                        ),
                ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(920.), px(860.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| Showcase {
                    mica: false,
                    toggle_on: false,
                    clicks: 0,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
