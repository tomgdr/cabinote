// 保存先の中の引き出しとメモのファイル操作。シンボリックリンクは辿らず、保存先は自動作成しない。

pub mod drawer;
pub mod memo;

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::error::{CnError, Result};

// 保存先がディレクトリとして存在するか確認する。
pub fn check_cabinet(cabinet: &Path) -> Result<()> {
    match fs::metadata(cabinet) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(CnError::CabinetNotFound(cabinet.to_path_buf())),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Err(CnError::CabinetNotFound(cabinet.to_path_buf()))
        }
        Err(e) => Err(CnError::Io {
            context: format!("failed to access {}", cabinet.display()),
            source: e,
        }),
    }
}
