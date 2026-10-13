use crate::cli::CliError;
use crate::validation::NameError;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum CnError {
    InvalidDrawerName(NameError),
    Usage(CliError),
    DrawerExists(String),
    DrawerNotFound(String),
    // home は改名・削除できない(中身は指定された名前)
    ProtectedDrawer(String),
    // 同じ名前のメモのファイルが、保存先のどこかに既にある(中身はファイル名)
    MemoFileExists(String),
    MemoNotFound(String),
    // 同じタイトルのメモが複数ある。matches は (一覧の番号, 一覧に表示する文字列)
    AmbiguousTitle {
        title: String,
        matches: Vec<(usize, String)>,
    },
    // 番号が範囲外(count は存在する件数)
    NumberOutOfRange {
        number: usize,
        count: usize,
    },
    CabinetNotFound(PathBuf),
    NotFullPath(String),
    // config.yaml の内容が不正(構文エラー、値の型が不正など)。理由を文字列で保持する
    InvalidConfig(String),
    EditorNotConfigured,
    MissingEnvVar(&'static str),
    // ファイル操作に失敗した。OSのエラーは握りつぶさず原因を保持する
    Io {
        context: String,
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, CnError>;

impl fmt::Display for CnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CnError::InvalidDrawerName(reason) => {
                write!(f, "invalid drawer name: {reason}")
            }
            CnError::Usage(reason) => {
                write!(
                    f,
                    "{reason}. Run 'cn --help' to see the available commands and options."
                )
            }
            CnError::DrawerExists(name) => write!(f, "drawer already exists: {name}"),
            CnError::DrawerNotFound(name) => write!(f, "drawer not found: {name}"),
            CnError::ProtectedDrawer(name) => {
                write!(f, "drawer '{name}' cannot be renamed or deleted")
            }
            CnError::MemoFileExists(name) => {
                write!(
                    f,
                    "memo file already exists: {name}. Wait a second and try again"
                )
            }
            CnError::MemoNotFound(title) => write!(f, "memo not found: {title}"),
            CnError::AmbiguousTitle { title, matches } => {
                write!(
                    f,
                    "multiple memos have the title '{title}'; specify one by number (-on to open, -dn to delete):"
                )?;
                for (number, label) in matches {
                    write!(f, "\n  [{number}] {label}")?;
                }
                Ok(())
            }
            CnError::NumberOutOfRange { number, count } => {
                let unit = if *count == 1 { "item" } else { "items" };
                write!(f, "number {number} is out of range ({count} {unit})")
            }
            CnError::CabinetNotFound(path) => {
                write!(
                    f,
                    "cabinet directory not found: {}. Create it first (see the README)",
                    path.display()
                )
            }
            CnError::NotFullPath(path) => {
                write!(f, "cabinet path must be a full path: {path}")
            }
            CnError::InvalidConfig(reason) => {
                write!(f, "invalid config.yaml: {reason}")
            }
            CnError::EditorNotConfigured => {
                write!(
                    f,
                    "no editor configured: set CN_EDITOR or EDITOR in config.yaml"
                )
            }
            CnError::MissingEnvVar(name) => {
                write!(f, "environment variable {name} is not set")
            }
            CnError::Io { context, source } => {
                write!(f, "{context}: {source}")
            }
        }
    }
}

impl std::error::Error for CnError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CnError::Io { source, .. } => Some(source),
            CnError::InvalidDrawerName(_)
            | CnError::Usage(_)
            | CnError::DrawerExists(_)
            | CnError::DrawerNotFound(_)
            | CnError::ProtectedDrawer(_)
            | CnError::MemoFileExists(_)
            | CnError::MemoNotFound(_)
            | CnError::AmbiguousTitle { .. }
            | CnError::NumberOutOfRange { .. }
            | CnError::CabinetNotFound(_)
            | CnError::NotFullPath(_)
            | CnError::InvalidConfig(_)
            | CnError::EditorNotConfigured
            | CnError::MissingEnvVar(_) => None,
        }
    }
}

impl From<CliError> for CnError {
    fn from(value: CliError) -> Self {
        CnError::Usage(value)
    }
}

impl From<NameError> for CnError {
    fn from(value: NameError) -> Self {
        CnError::InvalidDrawerName(value)
    }
}
