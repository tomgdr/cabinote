// 引き出し(保存先の直下のディレクトリ)のファイル操作。

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::current::DEFAULT_DRAWER;
use crate::error::{CnError, Result};
use crate::validation::validate_drawer_name;

// 名前順の引き出し名。ディレクトリだけが対象で、リンク・"." 始まり・UTF-8でない名前は除く。
pub fn list_drawers(cabinet: &Path) -> Result<Vec<String>> {
    let read_error = |source| CnError::Io {
        context: format!("failed to read {}", cabinet.display()),
        source,
    };
    let mut names = Vec::new();
    for entry in fs::read_dir(cabinet).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        // file_type はシンボリックリンクを辿らないため、リンクは is_dir が false になる
        if !entry.file_type().map_err(read_error)?.is_dir() {
            continue;
        }
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

pub fn find_drawer_by_number(cabinet: &Path, number: usize) -> Result<String> {
    let mut names = list_drawers(cabinet)?;
    if number == 0 || number > names.len() {
        return Err(CnError::NumberOutOfRange {
            number,
            count: names.len(),
        });
    }
    Ok(names.swap_remove(number - 1))
}

// cabinet 直下の引き出しの有無。パスを組み立てる前に、必ず名前を検証する。
pub fn drawer_exists(cabinet: &Path, name: &str) -> Result<bool> {
    validate_drawer_name(name)?;
    let path = cabinet.join(name);
    // symlink_metadata はシンボリックリンクを辿らないため、リンクは is_dir が false になる
    match fs::symlink_metadata(&path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(e) => Err(CnError::Io {
            context: format!("failed to access {}", path.display()),
            source: e,
        }),
    }
}

// home は改名・削除できない。大文字小文字も同じ名前として扱う(Windows/Macでは同じディレクトリになるため)。
pub fn check_not_home(name: &str) -> Result<()> {
    if name.eq_ignore_ascii_case(DEFAULT_DRAWER) {
        return Err(CnError::ProtectedDrawer(name.to_string()));
    }
    Ok(())
}

// 引き出しを作る。既にあれば DrawerExists。cabinet が無ければ作成せずエラー。
pub fn add_drawer(cabinet: &Path, name: &str) -> Result<()> {
    if drawer_exists(cabinet, name)? {
        return Err(CnError::DrawerExists(name.to_string()));
    }
    let path = cabinet.join(name);
    fs::create_dir(&path).map_err(|source| CnError::Io {
        context: format!("failed to create {}", path.display()),
        source,
    })
}

// 改名する(中のメモは動かさない)。new が既にあれば DrawerExists。
// Unixでは rename が空のディレクトリを黙って上書きするため、必ず確認する。
pub fn rename_drawer(cabinet: &Path, old: &str, new: &str) -> Result<()> {
    validate_drawer_name(old)?;
    validate_drawer_name(new)?;
    check_not_home(old)?;
    if !drawer_exists(cabinet, old)? {
        return Err(CnError::DrawerNotFound(old.to_string()));
    }
    if drawer_exists(cabinet, new)? {
        return Err(CnError::DrawerExists(new.to_string()));
    }
    let from = cabinet.join(old);
    let to = cabinet.join(new);
    fs::rename(&from, &to).map_err(|source| CnError::Io {
        context: format!("failed to rename {} to {}", from.display(), to.display()),
        source,
    })
}

// 中のメモごと削除する。確認は呼び出し側。home は ProtectedDrawer。
pub fn delete_drawer(cabinet: &Path, name: &str) -> Result<()> {
    validate_drawer_name(name)?;
    check_not_home(name)?;
    if !drawer_exists(cabinet, name)? {
        return Err(CnError::DrawerNotFound(name.to_string()));
    }
    let path = cabinet.join(name);
    fs::remove_dir_all(&path).map_err(|source| CnError::Io {
        context: format!("failed to delete {}", path.display()),
        source,
    })
}
