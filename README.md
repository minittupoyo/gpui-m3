# gpui-m3

[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui)（Zed のUIフレームワーク）向けの
**Material Design 3 / M3 Expressive** コンポーネント集です。
Material Design 3 (Expressive) components for GPUI.

> **注意** GPUI はまだ安定版が無く、Zed のリポジトリから git 依存で使います。
> そのため gpui-m3 も crates.io には公開できません（`publish = false`）。
> 動作確認した GPUI のリビジョンは [`Cargo.lock`](Cargo.lock) の `gpui` の項目を参照してください。
> このプロジェクトは Google の公式製品ではありません。

## 特長

- バネ（spring）ベースの Expressive モーション、押下時に角が詰まるシェイプモーフ、
  リップル、ホバー時の影のアニメーション
- Tab / Shift+Tab のフォーカス移動、Enter / Space での実行、Esc で閉じる、などのキーボード操作
- 日本語入力（IME）対応のテキスト欄（複数行・パスワード・Undo/Redo・クリップボード）
- 明暗テーマと、シード色から生成する動的カラー
- Roboto Flex を同梱（登録は `init` が行う）。アイコンも同梱で追加ファイル不要

## 部品

| 分類 | 部品 |
|---|---|
| アクション | `Button`（5種・5サイズ）、`IconButton`、`Fab`、`FabMenu`、`SplitButton`、`StandardButtonGroup`、`ConnectedButtonGroup` |
| 選択・入力 | `Checkbox`、`Radio`、`Switch`、`Chip`（Assist / Filter / Input / Suggestion）、`Slider`、`RangeSlider`、`VerticalSlider`、`TextField`、`DatePicker`、`TimePicker` |
| ナビゲーション | `NavigationBar`、`NavigationRail`（展開式）、`NavigationDrawer`、`Tabs`、`TopAppBar`、`DockedToolbar`、`FloatingToolbar`、`SearchBar`、`SearchView`、`SearchAnchor` |
| コンテナ・表示 | `Card`、`ListItem`、`Divider`、`Carousel`、`Badge` |
| オーバーレイ | `Dialog`、`FullScreenDialog`、`BottomSheet`、`SideSheet`、`Menu`（サブメニュー対応）、`DropdownMenu`、`ContextMenuArea`、`Select`、`Snackbar`、`tooltip`、`rich_tooltip` |
| 進捗 | `LinearProgress`、`CircularProgress`、`LoadingIndicator` |

すべての部品を操作できるギャラリーが付属しています。

```sh
cargo run --example gallery   # 全部品のギャラリー
cargo run --example hello     # 最小の使用例
```

## 導入

`Cargo.toml` に、GPUI と同じ指定で gpui-m3 を追加します。
**gpui-m3 が使う `gpui` と、アプリが使う `gpui` は、同じ git の取得元・ブランチでなければなりません**
（異なると、別のクレートとして扱われて型が合わずビルドできません）。

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", branch = "main" }
gpui_platform = { git = "https://github.com/zed-industries/zed", branch = "main" }
gpui-m3 = { git = "https://github.com/minittupoyo/gpui-m3" }
# ローカルで試すなら: gpui-m3 = { path = "../gpui-m3" }
```

GPUI のバージョンを固定したいときは、アプリ側で `cargo update -p gpui --precise <コミット>` を使います。

## 使い方

```rust
use gpui::{App, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px, size};
use gpui_m3::{self as m3, ActiveTheme, Button};
use gpui_platform::application;

struct Root;

impl Render for Root {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let c = &cx.m3().colors;
        div()
            .size_full()
            .p(px(24.))
            .bg(c.surface)
            .text_color(c.on_surface)
            .font_family(cx.m3().font_family) // 既定は同梱の Roboto Flex
            .child(Button::new("hello", "こんにちは"))
    }
}

fn main() {
    application()
        .with_assets(m3::Assets) // アイコンを配信する AssetSource
        .run(|cx: &mut App| {
            m3::init(cx, m3::Brightness::Dark); // フォント登録・テーマ・キーバインド
            let bounds = Bounds::centered(None, size(px(480.), px(320.)), cx);
            cx.open_window(
                WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
                |_, cx| cx.new(|_| Root),
            )
            .unwrap();
        });
}
```

### 押さえておくこと

- **ID を渡す**: 多くの部品は `Button::new("一意なID", ...)` のように ID を取ります。
  波紋・フォーカス・ホバーなどの状態を、この ID をキーにして保持します。同じ画面で重複させないでください。
- **制御コンポーネント**: ダイアログ・シート・メニュー・ピッカーなどは、開閉や選択の状態を呼び出し側が持ち、
  `open` や `on_change` / `on_select` で受け渡します。
- **自前のアセットがある場合**: `m3::Assets` は `m3-icons/` のパスだけを扱います。
  自前の `AssetSource` の `load` の先頭で `m3::Assets` に委譲してください。
- **オーバーレイの置き場所**: モーダルのシートやドロワーなどは、ウィンドウのルート（`relative` な要素）の
  最後の子として常に配置し、`open` で開閉します（閉じるアニメーションのため）。

### テーマ

```rust
m3::set_brightness(cx, m3::Brightness::Light);              // 明暗の切替
m3::set_dynamic_color(cx, Some(m3::DynamicColorConfig::new(0x6750A4))); // シード色から生成
let colors = &cx.m3().colors;                                // 現在の配色（ColorScheme）
```

## 既知の制約

- 動作確認は **Windows のみ**です。macOS・Linux でのビルド・表示は未確認です。
- GPUI は可変フォントの軸のうち太さ（wght）しか指定できないため、Roboto Flex の幅・光学サイズ・
  グレードの軸は使えません。
- 日本語はシステムフォントにフォールバックします（Roboto Flex に和文の字形が無いため）。
- 未実装: スクリーンリーダー対応、RTL、密度（Density）設定、範囲選択の入力モード。
- GPUI の API 変更に追従する必要があります（git 依存のため、`cargo update` で壊れることがあります）。

## 開発

```sh
cargo run --example gallery        # ギャラリーで見た目と操作を確認
cargo test                         # 単体テスト（日付計算・範囲選択・影の補間・カルーセルの幅計算など）
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps --open
```

CI（`.github/workflows/ci.yml`）は Windows で、上と同じ検証を実行します。
ギャラリーは `M3_PAGE=<番号>` を付けて起動すると、指定したページから始まります（0 = Buttons、10 = More）。

## ライセンス

[MIT](LICENSE-MIT) または [Apache-2.0](LICENSE-APACHE) のどちらかを選べます。

同梱のフォント（Roboto Flex、SIL OFL 1.1）とアイコン（Material Icons、Apache-2.0）などの
権利表示は [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) を参照してください。
