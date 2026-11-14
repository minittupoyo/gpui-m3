//! 同梱フォント（Roboto Flex、SIL Open Font License 1.1。ライセンスは assets/fonts/OFL.txt）

use std::borrow::Cow;

use gpui::App;

/// Roboto Flex の可変フォント（wght / wdth / opsz / GRAD などの軸を含む）
static ROBOTO_FLEX: &[u8] = include_bytes!("../../assets/fonts/RobotoFlex.ttf");

/// 同梱フォントのファミリー名
pub const ROBOTO_FLEX_FAMILY: &str = "Roboto Flex";

/// 同梱フォントをテキストシステムに登録する（`m3::init` から呼ばれる）。
/// 登録に失敗してもアプリは続行し、テーマのフォント指定はシステムのフォールバックに任せる。
pub(crate) fn register_fonts(cx: &mut App) {
    if let Err(e) = cx.text_system().add_fonts(vec![Cow::Borrowed(ROBOTO_FLEX)]) {
        eprintln!("m3: failed to register Roboto Flex: {e}");
    }
}
