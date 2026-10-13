// 現在選択中の引き出しを .current で管理する。保存場所は呼び出し側が決める。

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::{CnError, Result};
use crate::validation::validate_drawer_name;

const CURRENT_FILE_NAME: &str = ".current";

// .current が無い、または空のときに選択されているものとして扱う引き出し名。
pub const DEFAULT_DRAWER: &str = "home";

pub fn current_file_path(dir: &Path) -> PathBuf {
    dir.join(CURRENT_FILE_NAME)
}

// 無い・空なら "home"。読んだ名前は検証し、不正ならエラー。
pub fn read_current(dir: &Path) -> Result<String> {
    let path = current_file_path(dir);
    match fs::read_to_string(&path) {
        Ok(content) => {
            let name = content.trim();
            if name.is_empty() {
                Ok(DEFAULT_DRAWER.to_string())
            } else {
                validate_drawer_name(name)?;
                Ok(name.to_string())
            }
        }
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Ok(DEFAULT_DRAWER.to_string())
            } else {
                Err(CnError::Io {
                    context: format!("failed to read {}", path.display()),
                    source: e,
                })
            }
        }
    }
}

// 名前を検証してから書く。dir が無ければ作成せずエラー。
pub fn write_current(dir: &Path, name: &str) -> Result<()> {
    validate_drawer_name(name)?;
    let path = current_file_path(dir);
    fs::write(&path, name).map_err(|source| CnError::Io {
        context: format!("failed to write {}", path.display()),
        source,
    })
}
