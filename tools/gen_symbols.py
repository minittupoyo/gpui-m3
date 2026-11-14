#!/usr/bin/env python3
"""Material Symbols のパスデータを取得して src/symbols.rs を生成する。

    python tools/gen_symbols.py            # src/symbols.rs を書き換える

出典: https://github.com/google/material-design-icons （Apache License 2.0）
スタイルは Outlined、wght 400、GRAD 0、opsz 24。各アイコンについて塗りなし（FILL 0）と
塗りつぶし（FILL 1）の 2 種類を埋め込む。

アイコンを増やすときは NAMES に Material Symbols の名前を足し、IconName（src/icons.rs）にも
同じ名前の項目を追加してからこのスクリプトを実行する。
"""

import re
import sys
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# IconName::file() が返す名前（旧 Material Icons の名前と違うものは SYMBOL_NAME で対応づける）
NAMES = """
add arrow_back check check_box close dark_mode delete edit favorite favorite_border home label
light_mode mail menu more_vert person refresh remove search send share star tune progress bookmark
image settings cancel error lock visibility visibility_off arrow_drop_down arrow_drop_up
chevron_right content_copy content_cut content_paste sort logout help keyboard_arrow_down palette
menu_open mic history calendar_month schedule keyboard chevron_left
""".split()

# Material Symbols 側の名前が異なるもの
SYMBOL_NAME = {"favorite_border": "favorite", "progress": "progress_activity"}

BASE = (
    "https://raw.githubusercontent.com/google/material-design-icons/master/"
    "symbols/web/{n}/materialsymbolsoutlined/{n}{f}_24px.svg"
)
SVG_RE = re.compile(
    r'^<svg [^>]*viewBox="0 -960 960 960"[^>]*>\s*<path d="([^"]+)"\s*/>\s*</svg>\s*$'
)
# 取得したデータはコードに埋め込むので、パスデータとして妥当な文字だけを許す
PATH_RE = re.compile(r"^[MmLlHhVvCcSsQqTtAaZz0-9eE.,\s\-+]+$")


def fetch(name: str, filled: bool) -> str:
    symbol = SYMBOL_NAME.get(name, name)
    url = BASE.format(n=symbol, f="_fill1" if filled else "")
    try:
        text = urllib.request.urlopen(url, timeout=30).read().decode("utf-8")
    except urllib.error.HTTPError as e:
        sys.exit(f"{url}: HTTP {e.code}")
    m = SVG_RE.match(text.strip())
    if not m or not PATH_RE.match(m.group(1)):
        sys.exit(f"{url}: unexpected SVG content")
    return m.group(1)


def main() -> None:
    jobs = [(n, f) for n in NAMES for f in (False, True)]
    with ThreadPoolExecutor(8) as ex:
        paths = list(ex.map(lambda j: fetch(*j), jobs))

    lines = [
        "//! Material Symbols（Outlined / wght 400 / opsz 24）のパスデータ。",
        "//!",
        "//! `tools/gen_symbols.py` で生成したファイル。手で編集しないこと。",
        "//! 出典: <https://github.com/google/material-design-icons>（Apache License 2.0）",
        "",
        "/// `name` のパスデータ（viewBox は `0 -960 960 960`）。`filled` で塗りつぶし（FILL 1）を返す。",
        "pub(crate) fn symbol_path(name: &str, filled: bool) -> Option<&'static str> {",
        "    Some(match (name, filled) {",
    ]
    for (n, f), d in zip(jobs, paths):
        lines.append(f'        ("{n}", {"true" if f else "false"}) => "{d}",')
    lines += ["        _ => return None,", "    })", "}", ""]

    out = Path(__file__).resolve().parent.parent / "src" / "symbols.rs"
    out.write_text("\n".join(lines), encoding="utf-8", newline="\n")
    print(f"wrote {out} ({len(NAMES)} icons)")


if __name__ == "__main__":
    main()
