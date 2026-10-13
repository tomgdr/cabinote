// コマンドの実行。表示と確認を担当し、ファイル操作は store/ に任せる。

pub mod cabinet;
pub mod drawer;
pub mod help;
pub mod memo;

use std::io::{BufRead, Write};
use std::path::PathBuf;

use crate::cli::Command;
use crate::config::FileSettings;
use crate::current::{DEFAULT_DRAWER, read_current};
use crate::error::{CnError, Result};
use crate::store::drawer::drawer_exists;

// コマンドの実行に必要な情報。main.rs が集める。
pub struct Context {
    pub cabinet: PathBuf,
    // .current を置くディレクトリ(~/.cn/)
    pub state_dir: PathBuf,
    pub settings: FileSettings,
    // 環境変数 CN_EDITOR の値(未設定なら None)。環境変数は main.rs だけが読む
    pub editor_env: Option<String>,
}

pub fn run(
    ctx: &Context,
    command: Command,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<()> {
    match command {
        Command::Help | Command::Version => show_info(&command, output).map(|_| ()),
        Command::Cabinet => cabinet::run(ctx, output),
        Command::Drawer(command) => drawer::run(ctx, command, input, output),
        Command::Memo(command) => memo::run(ctx, command, input, output),
    }
}

// ヘルプとバージョンなら、設定を読まずに表示して true を返す(設定が壊れていても見られるように)。
pub fn show_info(command: &Command, output: &mut impl Write) -> Result<bool> {
    match command {
        Command::Help => say(output, help::HELP_TEXT)?,
        Command::Version => say(output, &format!("cn {}", env!("CARGO_PKG_VERSION")))?,
        _ => return Ok(false),
    }
    Ok(true)
}

pub fn say(output: &mut impl Write, text: &str) -> Result<()> {
    writeln!(output, "{text}").map_err(|source| CnError::Io {
        context: "failed to write output".to_string(),
        source,
    })
}

// 現在の引き出し名。.current が無い・空、または存在しない引き出しを指している場合は home。
pub fn current_drawer(ctx: &Context) -> Result<String> {
    let name = read_current(&ctx.state_dir)?;
    if drawer_exists(&ctx.cabinet, &name)? {
        Ok(name)
    } else {
        Ok(DEFAULT_DRAWER.to_string())
    }
}
