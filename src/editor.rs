// エディタの起動(設定値 + メモのパス)。終了を待つかはエディタの種類で切り替える(config.md)。

use std::path::Path;
use std::process::Command;

use crate::error::{CnError, Result};

// 終了を待たずに起動するGUIエディタ。これ以外は待つ。
const DETACHED_EDITORS: [&str; 2] = ["code", "zed"];

#[derive(Debug, PartialEq)]
pub enum LaunchMode {
    // 起動して、エディタが終了するまで待つ
    Wait,
    // 起動して、待たずに戻る
    Detach,
}

// 設定値のファイル名部分(拡張子を除く、大小無視)で決める。未知のエディタは Wait。
pub fn launch_mode(editor: &str) -> LaunchMode {
    let name = Path::new(editor)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_lowercase());
    match name {
        Some(name) if DETACHED_EDITORS.contains(&name.as_str()) => LaunchMode::Detach,
        _ => LaunchMode::Wait,
    }
}

// 起動できなければOSのエラーを返す。待つ場合も、終了コードはエラーにしない。
pub fn open_in_editor(editor: &str, path: &Path) -> Result<()> {
    let mut command = Command::new(editor);
    command.arg(path);
    let started = match launch_mode(editor) {
        LaunchMode::Wait => command.status().map(|_| ()),
        LaunchMode::Detach => command.spawn().map(|_| ()),
    };
    started.map_err(|source| CnError::Io {
        context: format!("failed to start editor '{editor}'"),
        source,
    })
}
