//! Material Symbols（Outlined、wght 400、opsz 24）のアイコン。Apache License 2.0。
//!
//! パスデータは `tools/gen_symbols.py` で生成した `symbols.rs` に埋め込んである。
//! 既定は塗りなし（FILL 0）で、[`Icon::filled`] で塗りつぶし（FILL 1）に切り替わる。
//! ナビゲーションなどの選択中の項目は、`selected_icon` を指定しなければ自動で塗りつぶしになる。

use std::borrow::Cow;

use crate::{ActiveTheme, symbols::symbol_path};
use gpui::{
    App, AssetSource, Hsla, IntoElement, Pixels, RenderOnce, SharedString, Styled, Window, px, svg,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconName {
    Add,
    ArrowBack,
    Check,
    CheckBox,
    Close,
    DarkMode,
    Delete,
    Edit,
    Favorite,
    FavoriteBorder,
    Home,
    Label,
    LightMode,
    Mail,
    Menu,
    MoreVert,
    Person,
    Refresh,
    Remove,
    Search,
    Send,
    Share,
    Star,
    Tune,
    Progress,
    Bookmark,
    Image,
    Settings,
    Cancel,
    Error,
    Lock,
    Visibility,
    VisibilityOff,
    ArrowDropDown,
    ArrowDropUp,
    ChevronRight,
    ContentCopy,
    ContentCut,
    ContentPaste,
    Sort,
    Logout,
    Help,
    KeyboardArrowDown,
    Palette,
    MenuOpen,
    Mic,
    History,
    CalendarMonth,
    Schedule,
    Keyboard,
    ChevronLeft,
}

impl IconName {
    /// すべてのアイコン（ギャラリーやテストで列挙するために使う）
    pub const ALL: [IconName; 51] = [
        IconName::Add,
        IconName::ArrowBack,
        IconName::Check,
        IconName::CheckBox,
        IconName::Close,
        IconName::DarkMode,
        IconName::Delete,
        IconName::Edit,
        IconName::Favorite,
        IconName::FavoriteBorder,
        IconName::Home,
        IconName::Label,
        IconName::LightMode,
        IconName::Mail,
        IconName::Menu,
        IconName::MoreVert,
        IconName::Person,
        IconName::Refresh,
        IconName::Remove,
        IconName::Search,
        IconName::Send,
        IconName::Share,
        IconName::Star,
        IconName::Tune,
        IconName::Progress,
        IconName::Bookmark,
        IconName::Image,
        IconName::Settings,
        IconName::Cancel,
        IconName::Error,
        IconName::Lock,
        IconName::Visibility,
        IconName::VisibilityOff,
        IconName::ArrowDropDown,
        IconName::ArrowDropUp,
        IconName::ChevronRight,
        IconName::ContentCopy,
        IconName::ContentCut,
        IconName::ContentPaste,
        IconName::Sort,
        IconName::Logout,
        IconName::Help,
        IconName::KeyboardArrowDown,
        IconName::Palette,
        IconName::MenuOpen,
        IconName::Mic,
        IconName::History,
        IconName::CalendarMonth,
        IconName::Schedule,
        IconName::Keyboard,
        IconName::ChevronLeft,
    ];

    fn file(self) -> &'static str {
        use IconName::*;
        match self {
            Add => "add",
            ArrowBack => "arrow_back",
            Check => "check",
            CheckBox => "check_box",
            Close => "close",
            DarkMode => "dark_mode",
            Delete => "delete",
            Edit => "edit",
            Favorite => "favorite",
            FavoriteBorder => "favorite_border",
            Home => "home",
            Label => "label",
            LightMode => "light_mode",
            Mail => "mail",
            Menu => "menu",
            MoreVert => "more_vert",
            Person => "person",
            Refresh => "refresh",
            Remove => "remove",
            Search => "search",
            Send => "send",
            Share => "share",
            Star => "star",
            Tune => "tune",
            Progress => "progress",
            Bookmark => "bookmark",
            Image => "image",
            Settings => "settings",
            Cancel => "cancel",
            Error => "error",
            Lock => "lock",
            Visibility => "visibility",
            VisibilityOff => "visibility_off",
            ArrowDropDown => "arrow_drop_down",
            ArrowDropUp => "arrow_drop_up",
            ChevronRight => "chevron_right",
            ContentCopy => "content_copy",
            ContentCut => "content_cut",
            ContentPaste => "content_paste",
            Sort => "sort",
            Logout => "logout",
            Help => "help",
            KeyboardArrowDown => "keyboard_arrow_down",
            Palette => "palette",
            MenuOpen => "menu_open",
            Mic => "mic",
            History => "history",
            CalendarMonth => "calendar_month",
            Schedule => "schedule",
            Keyboard => "keyboard",
            ChevronLeft => "chevron_left",
        }
    }

    /// 塗りなし（FILL 0）のアセットパス
    pub fn path(self) -> SharedString {
        format!("m3-icons/{}.svg", self.file()).into()
    }

    /// 塗りつぶし（FILL 1）のアセットパス
    pub fn path_filled(self) -> SharedString {
        format!("m3-icons/{}{FILL_SUFFIX}.svg", self.file()).into()
    }

    /// M3 で常に塗りつぶしで使うアイコン（クリア・エラー・お気に入り）
    pub fn always_filled(self) -> bool {
        matches!(
            self,
            IconName::Cancel | IconName::Error | IconName::Favorite
        )
    }
}

/// 塗りつぶし版のアセットパスの接尾辞（`m3-icons/home-fill.svg`）
const FILL_SUFFIX: &str = "-fill";

/// アセット名に対応する SVG の中身（パスデータ）
fn svg_body(name: &str, filled: bool) -> Option<String> {
    let d = symbol_path(name, filled)?;
    Some(format!(r#"<path d="{d}"/>"#))
}

/// M3 アイコンを配信する AssetSource。
/// アプリ側で別のアセットがある場合は `load` の先頭でこれに委譲すればよい。
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        let Some(name) = path
            .strip_prefix("m3-icons/")
            .and_then(|p| p.strip_suffix(".svg"))
        else {
            return Ok(None);
        };
        let (name, filled) = match name.strip_suffix(FILL_SUFFIX) {
            Some(n) => (n, true),
            None => (name, false),
        };
        Ok(svg_body(name, filled).map(|body| {
            Cow::Owned(
                format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960" width="24" height="24">{body}</svg>"#
                )
                .into_bytes(),
            )
        }))
    }

    fn list(&self, _path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}

/// アイコン要素
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: Pixels,
    color: Option<Hsla>,
    filled: bool,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: px(24.),
            color: None,
            filled: false,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// 色を指定する。指定しないとテーマの `on_surface` になる（SVG は親の `text_color` を継承しない）
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// 塗りつぶし版（FILL 1）にする。選択中の状態などに使う
    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let path = if self.filled || self.name.always_filled() {
            self.name.path_filled()
        } else {
            self.name.path()
        };
        // SVG は親の text_color を継承しないので、色の指定が無いときはテーマの on_surface にする
        let color = self.color.unwrap_or_else(|| cx.m3().colors.on_surface);
        svg()
            .path(path)
            .size(self.size)
            .flex_none()
            .text_color(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_loads_in_both_variants() {
        for icon in IconName::ALL {
            for path in [icon.path(), icon.path_filled()] {
                let bytes = Assets
                    .load(&path)
                    .unwrap()
                    .unwrap_or_else(|| panic!("{icon:?}: {path} is missing"));
                let svg = String::from_utf8(bytes.into_owned()).unwrap();
                assert!(svg.starts_with("<svg"), "{path}");
                assert!(svg.contains(r#"viewBox="0 -960 960 960""#), "{path}");
                assert!(
                    svg.contains(r#"<path d="M"#) || svg.contains(r#"<path d="m"#),
                    "{path}"
                );
            }
        }
    }

    #[test]
    fn icon_files_are_unique() {
        let mut names: Vec<_> = IconName::ALL.iter().map(|i| i.file()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(
            before,
            names.len(),
            "two IconName variants share a file name"
        );
    }

    #[test]
    fn unknown_assets_are_not_served() {
        assert!(Assets.load("m3-icons/nope.svg").unwrap().is_none());
        assert!(Assets.load("other/home.svg").unwrap().is_none());
        assert!(Assets.load("m3-icons/home.png").unwrap().is_none());
    }

    #[test]
    fn filled_variant_differs_for_outlined_shapes() {
        let outline = svg_body("home", false).unwrap();
        let filled = svg_body("home", true).unwrap();
        assert_ne!(outline, filled);
    }

    #[test]
    fn status_icons_are_always_filled() {
        assert!(IconName::Cancel.always_filled());
        assert!(IconName::Error.always_filled());
        assert!(!IconName::Home.always_filled());
    }
}
