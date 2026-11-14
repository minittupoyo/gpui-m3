# Changelog

このファイルは [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) の形式に従います。

## [Unreleased]

### Added
- 初期の部品一式: ボタン類、選択・入力、ナビゲーション、カード、ダイアログ、シート、メニュー、
  ツールバー、進捗表示、チップ、スライダー
- リップルとキーボード操作、ダイアログ・スナックバーの出入りのアニメーション
- `TextField` の複数行入力、`SearchBar` / `SearchView` / `SearchAnchor`
- `DatePicker`（年選択・範囲選択・入力モード）、`TimePicker`（ダイヤル・入力モード）
- `NavigationDrawer`、`Carousel`、`BottomSheet` の途中の高さ（peek）と標準（非モーダル）、メニューのサブメニュー
- ホバー時の影のアニメーション（`Button`・`Card`・`Fab`）
- Roboto Flex の同梱（既定フォント）
- `Icon::filled` と `IconName::ALL`。選択中のナビゲーション・ツールバー・アイコンボタンは塗りつぶしアイコンになる
- 密度の設定（`Density`、`set_density`、`Theme::density`）
- `tools/gen_symbols.py`（Material Symbols のパスデータを取得して `src/symbols.rs` を生成する）
- 動的カラー（シード色から生成）

### Changed
- アイコンを従来の Material Icons から Material Symbols（Outlined）に変更。`Cancel`・`Error`・`Favorite` は常に塗りつぶし
- クレート名を `gpui-m3`（ライブラリ名 `gpui_m3`）にし、`src/m3/*` をクレート直下へ移動
- 実行ファイルを `examples/`（`gallery`・`hello`・`counter`・`showcase`）へ移動
