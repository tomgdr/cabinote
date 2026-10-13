mod cli;
mod commands;
mod config;
mod current;
mod editor;
mod error;
mod prompt;
mod store;
mod title;
mod validation;

use std::io::{self, ErrorKind};
use std::process::ExitCode;

use cli::{CliError, parse_args};
use commands::Context;
use error::{CnError, Result};

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        // `cn memo -l | head -1` のように、出力先が先に閉じられた場合はエラーにしない
        Err(error) if is_broken_pipe(&error) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> Result<()> {
    let args = collect_args()?;
    let command = parse_args(&args)?;
    let mut output = io::stdout().lock();
    // ヘルプとバージョンは、設定が壊れていても見られるよう、設定を読む前に表示する
    if commands::show_info(&command, &mut output)? {
        return Ok(());
    }
    let ctx = build_context()?;
    commands::run(&ctx, command, &mut io::stdin().lock(), &mut output)
}

// 変換できない文字を置き換えると意図しない名前になるため、UTF-8でない引数はエラーにする。
fn collect_args() -> std::result::Result<Vec<String>, CliError> {
    std::env::args_os()
        .skip(1)
        .map(|arg| arg.into_string().map_err(|_| CliError::NotUnicode))
        .collect()
}

// 環境変数を読む処理は、ここと config.rs の薄い関数だけに置く。
fn build_context() -> Result<Context> {
    let settings = config::load_settings()?;
    let cabinet = config::cabinet_path(&settings)?;
    let state_dir = config::current_dir(&config::home_dir()?);
    Ok(Context {
        cabinet,
        state_dir,
        settings,
        editor_env: config::editor_env(),
    })
}

fn is_broken_pipe(error: &CnError) -> bool {
    matches!(error, CnError::Io { source, .. } if source.kind() == ErrorKind::BrokenPipe)
}
