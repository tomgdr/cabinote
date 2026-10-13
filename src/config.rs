// 保存先とエディタの決定(環境変数 > config.yaml > デフォルト)。
// 判断は渡された値だけで決まる関数にし、環境変数やファイルを読む処理は薄い関数にとどめる。

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use yaml_rust2::{Yaml, YamlLoader};

use crate::error::{CnError, Result};

const CABINET_DIR: &str = ".cn";
const CONFIG_YAML: &str = "config.yaml";
const CONFIG_EDITOR: &str = "EDITOR";
const CONFIG_CABINET: &str = "CABINET";
const ENV_EDITOR: &str = "CN_EDITOR";
const ENV_CABINET: &str = "CN_CABINET";

// 環境変数 → config.yaml の順で、最初に見つかった空でない値を返す。
pub fn pick_setting(env: Option<&str>, file: Option<&str>) -> Option<String> {
    for value in [env, file] {
        if let Some(v) = value
            && !v.is_empty()
        {
            return Some(v.to_string());
        }
    }
    None
}

// フルパスか確認する。`~` 始まりと相対パスは NotFullPath。
pub fn check_full_path(path: &str) -> Result<PathBuf> {
    let cabinet_path = PathBuf::from(path);
    if cabinet_path.is_absolute() {
        Ok(cabinet_path)
    } else {
        Err(CnError::NotFullPath(path.to_string()))
    }
}

// 保存先のデフォルト(<home>/.cn)。
pub fn default_cabinet(home: &str) -> PathBuf {
    PathBuf::from(home).join(CABINET_DIR)
}

// .current を置くディレクトリ(<home>/.cn)。保存先とは独立して固定する。
pub fn current_dir(home: &str) -> PathBuf {
    PathBuf::from(home).join(CABINET_DIR)
}

// Linux / Mac の config.yaml のパス。
pub fn config_file_path_unix(home: &str) -> PathBuf {
    PathBuf::from(home)
        .join(".config")
        .join("cn")
        .join(CONFIG_YAML)
}

// Windows の config.yaml のパス。
pub fn config_file_path_windows(appdata: &str) -> PathBuf {
    PathBuf::from(appdata).join("cn").join(CONFIG_YAML)
}

// 環境変数が未設定または空なら MissingEnvVar(name)。
pub fn require_env(name: &'static str, value: Option<String>) -> Result<String> {
    match value {
        Some(v) if !v.is_empty() => Ok(v),
        _ => Err(CnError::MissingEnvVar(name)),
    }
}

fn read_required_env(name: &'static str) -> Result<String> {
    require_env(name, std::env::var(name).ok())
}

// Windows は USERPROFILE、それ以外は HOME。
pub fn home_dir() -> Result<String> {
    let name = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    read_required_env(name)
}

// config.yaml のパスを返す。Windows は APPDATA、それ以外は HOME から組み立てる。
pub fn config_file_path() -> Result<PathBuf> {
    if cfg!(windows) {
        let appdata = read_required_env("APPDATA")?;
        Ok(config_file_path_windows(&appdata))
    } else {
        let home = home_dir()?;
        Ok(config_file_path_unix(&home))
    }
}

// config.yaml から読み取った設定。キーが無い・値が空(null)の場合は None。
#[derive(Debug, Default, PartialEq)]
pub struct FileSettings {
    pub editor: Option<String>,
    pub cabinet: Option<String>,
}

// YAMLの値を文字列にする。キー無し・空は None、配列やマップは InvalidConfig。
fn yaml_to_string(key: &str, value: &Yaml) -> Result<Option<String>> {
    match value {
        Yaml::String(s) => Ok(Some(s.clone())),
        Yaml::Integer(i) => Ok(Some(i.to_string())),
        Yaml::Real(r) => Ok(Some(r.clone())),
        Yaml::Boolean(b) => Ok(Some(b.to_string())),
        Yaml::Null | Yaml::BadValue => Ok(None),
        _ => Err(CnError::InvalidConfig(format!("{key} must be a string"))),
    }
}

// 空・コメントのみは既定値、未知のキーは無視、構文エラーや非マップは InvalidConfig。
// 先頭のBOMは無視する(Windowsのエディタや PowerShell 5.1 が付けることがある)。
pub fn parse_config(text: &str) -> Result<FileSettings> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let docs =
        YamlLoader::load_from_str(text).map_err(|e| CnError::InvalidConfig(e.to_string()))?;
    let Some(doc) = docs.first() else {
        return Ok(FileSettings::default());
    };
    if !doc.is_hash() {
        return Err(CnError::InvalidConfig(
            "top level must be a mapping (KEY: value)".to_string(),
        ));
    }
    let editor = yaml_to_string(CONFIG_EDITOR, &doc[CONFIG_EDITOR])?;
    let cabinet = yaml_to_string(CONFIG_CABINET, &doc[CONFIG_CABINET])?;
    Ok(FileSettings { editor, cabinet })
}

// CN_EDITOR → config.yaml の順。どちらも無ければ EditorNotConfigured。
pub fn resolve_editor(env: Option<&str>, settings: &FileSettings) -> Result<String> {
    pick_setting(env, settings.editor.as_deref()).ok_or(CnError::EditorNotConfigured)
}

// CN_CABINET → config.yaml の順で、見つかった値はフルパス検査を通す。
// 無ければ None(既定値はホームが必要なため呼び出し側で決める)。
pub fn resolve_cabinet(env: Option<&str>, settings: &FileSettings) -> Result<Option<PathBuf>> {
    match pick_setting(env, settings.cabinet.as_deref()) {
        Some(path) => Ok(Some(check_full_path(&path)?)),
        None => Ok(None),
    }
}

// ファイルが無ければ既定値(設定ファイルは任意)。それ以外の読み込み失敗は Io。
pub fn load_file_settings(path: &Path) -> Result<FileSettings> {
    match fs::read_to_string(path) {
        Ok(text) => parse_config(&text),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                Ok(FileSettings::default())
            } else {
                Err(CnError::Io {
                    context: format!("failed to read {}", path.display()),
                    source: e,
                })
            }
        }
    }
}

pub fn load_settings() -> Result<FileSettings> {
    let path = config_file_path()?;
    load_file_settings(&path)
}

// CN_CABINET → config.yaml → デフォルト(<home>/.cn)の順。
pub fn cabinet_path(settings: &FileSettings) -> Result<PathBuf> {
    let env = std::env::var(ENV_CABINET).ok();
    match resolve_cabinet(env.as_deref(), settings)? {
        Some(path) => Ok(path),
        None => Ok(default_cabinet(&home_dir()?)),
    }
}

// 環境変数 CN_EDITOR の値。未設定(またはUTF-8でない)なら None。
pub fn editor_env() -> Option<String> {
    std::env::var(ENV_EDITOR).ok()
}
