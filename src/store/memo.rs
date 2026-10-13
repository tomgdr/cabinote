// メモ(引き出しの中の .md ファイル)のファイル操作。

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;

use crate::error::{CnError, Result};
use crate::store::drawer::{drawer_exists, list_drawers};
use crate::title::extract_title;

// ファイル名の形式は確認しない(手で置いた .md も開けるように)。
const MEMO_EXTENSION: &str = ".md";

// 名前順のメモのファイル名。通常の .md ファイルだけが対象で、リンク・"." 始まり・UTF-8でない名前は除く。
pub fn list_memo_files(cabinet: &Path, drawer: &str) -> Result<Vec<String>> {
    if !drawer_exists(cabinet, drawer)? {
        return Err(CnError::DrawerNotFound(drawer.to_string()));
    }
    let dir = cabinet.join(drawer);
    let read_error = |source| CnError::Io {
        context: format!("failed to read {}", dir.display()),
        source,
    };
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        // file_type はシンボリックリンクを辿らないため、リンクは is_file が false になる
        if !entry.file_type().map_err(read_error)?.is_file() {
            continue;
        }
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with('.') || !name.ends_with(MEMO_EXTENSION) {
            continue;
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

// YYYYMMDD_HHmmss.md の形式(例: 20261010_093015.md)。
pub fn memo_file_name(datetime: NaiveDateTime) -> String {
    format!("{}{}", datetime.format("%Y%m%d_%H%M%S"), MEMO_EXTENSION)
}

// 空のメモを作ってパスを返す。同名のファイルが保存先のどこかにあれば MemoFileExists(ファイル名は保存先全体で一意)。
pub fn create_memo(cabinet: &Path, drawer: &str, file_name: &str) -> Result<PathBuf> {
    if !drawer_exists(cabinet, drawer)? {
        return Err(CnError::DrawerNotFound(drawer.to_string()));
    }
    for other in list_drawers(cabinet)? {
        let candidate = cabinet.join(&other).join(file_name);
        match fs::symlink_metadata(&candidate) {
            Ok(_) => return Err(CnError::MemoFileExists(file_name.to_string())),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => {
                return Err(CnError::Io {
                    context: format!("failed to access {}", candidate.display()),
                    source: e,
                });
            }
        }
    }
    let path = cabinet.join(drawer).join(file_name);
    // create_new は、ファイルが既にあれば失敗する(上書きしない)
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(_) => Ok(path),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            Err(CnError::MemoFileExists(file_name.to_string()))
        }
        Err(e) => Err(CnError::Io {
            context: format!("failed to create {}", path.display()),
            source: e,
        }),
    }
}

pub fn count_memos(cabinet: &Path, drawer: &str) -> Result<usize> {
    Ok(list_memo_files(cabinet, drawer)?.len())
}

#[derive(Debug, PartialEq)]
pub struct MemoEntry {
    // 例: 20261010_093015.md
    pub file_name: String,
    // 最初の # 見出し。見出しが無い、またはUTF-8として読めなければ None
    pub title: Option<String>,
}

impl MemoEntry {
    // ".md" と "_" を除いた日時(例: 20261010093015)。
    fn stamp(&self) -> String {
        let stem = self
            .file_name
            .strip_suffix(MEMO_EXTENSION)
            .unwrap_or(&self.file_name);
        stem.replace('_', "")
    }

    // 一覧に表示し、タイトル指定で照合する名前。タイトルがあればタイトル、無ければ日時。
    pub fn display_name(&self) -> String {
        match &self.title {
            Some(title) => title.clone(),
            None => self.stamp(),
        }
    }

    // 確認や候補の表示用。日時 + スペース + タイトル(例: 20261010093015 買い物)。
    pub fn label(&self) -> String {
        match &self.title {
            Some(title) => format!("{} {title}", self.stamp()),
            None => self.stamp(),
        }
    }
}

// 一覧に表示される名前(タイトル、無ければ日時)が完全一致(大小区別)するメモのファイル名。
// 無ければ MemoNotFound、複数なら AmbiguousTitle。
pub fn find_by_title(cabinet: &Path, drawer: &str, title: &str) -> Result<String> {
    let mut matches = Vec::new();
    for (index, memo) in list_memos(cabinet, drawer)?.into_iter().enumerate() {
        if memo.display_name() == title {
            matches.push((index + 1, memo));
        }
    }
    match matches.len() {
        0 => Err(CnError::MemoNotFound(title.to_string())),
        1 => Ok(matches.remove(0).1.file_name),
        _ => Err(CnError::AmbiguousTitle {
            title: title.to_string(),
            matches: matches
                .into_iter()
                .map(|(number, memo)| (number, memo.label()))
                .collect(),
        }),
    }
}

pub fn find_by_number(cabinet: &Path, drawer: &str, number: usize) -> Result<String> {
    let mut files = list_memo_files(cabinet, drawer)?;
    if number == 0 || number > files.len() {
        return Err(CnError::NumberOutOfRange {
            number,
            count: files.len(),
        });
    }
    Ok(files.swap_remove(number - 1))
}

// 一覧にあるファイルだけを削除できる(別の場所を指す名前が渡されても消さないため)。確認は呼び出し側。
pub fn delete_memo(cabinet: &Path, drawer: &str, file_name: &str) -> Result<()> {
    if !list_memo_files(cabinet, drawer)?
        .iter()
        .any(|f| f == file_name)
    {
        return Err(CnError::MemoNotFound(file_name.to_string()));
    }
    let path = cabinet.join(drawer).join(file_name);
    fs::remove_file(&path).map_err(|source| CnError::Io {
        context: format!("failed to delete {}", path.display()),
        source,
    })
}

// UTF-8として読めないメモは、一覧全体が見られなくなるのを避けるため None にする。それ以外の失敗は Io。
fn read_title(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(body) => Ok(extract_title(&body)),
        Err(e) if e.kind() == ErrorKind::InvalidData => Ok(None),
        Err(e) => Err(CnError::Io {
            context: format!("failed to read {}", path.display()),
            source: e,
        }),
    }
}

pub fn list_memos(cabinet: &Path, drawer: &str) -> Result<Vec<MemoEntry>> {
    let dir = cabinet.join(drawer);
    let mut memos = Vec::new();
    for file_name in list_memo_files(cabinet, drawer)? {
        let title = read_title(&dir.join(&file_name))?;
        memos.push(MemoEntry { file_name, title });
    }
    Ok(memos)
}
