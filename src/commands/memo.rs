// cn memo: 現在の引き出しのメモの一覧・開く・追加・削除。

use std::io::{BufRead, Write};

use crate::cli::MemoCommand;
use crate::commands::{Context, current_drawer, say};
use crate::config::resolve_editor;
use crate::editor::open_in_editor;
use crate::error::{CnError, Result};
use crate::prompt::confirm;
use crate::store::check_cabinet;
use crate::store::memo::{
    create_memo, delete_memo, find_by_number, find_by_title, list_memos, memo_file_name,
};

pub fn run(
    ctx: &Context,
    command: MemoCommand,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<()> {
    // エディタの確認は保存先に依存しないので、先に処理する。
    if let MemoCommand::Editor = command {
        return say(output, &editor(ctx)?);
    }
    check_cabinet(&ctx.cabinet)?;
    let drawer = current_drawer(ctx)?;
    match command {
        MemoCommand::List => list(ctx, &drawer, output),
        MemoCommand::OpenByTitle(title) => {
            let editor = editor(ctx)?;
            let file = find_by_title(&ctx.cabinet, &drawer, &title)?;
            open(ctx, &drawer, &file, &editor)
        }
        MemoCommand::OpenByNumber(number) => {
            let editor = editor(ctx)?;
            let file = find_by_number(&ctx.cabinet, &drawer, number)?;
            open(ctx, &drawer, &file, &editor)
        }
        MemoCommand::Editor => unreachable!("handled before the cabinet check"),
        MemoCommand::Add => add(ctx, &drawer, output),
        MemoCommand::DeleteByTitle(title) => {
            let file = find_by_title(&ctx.cabinet, &drawer, &title)?;
            delete(ctx, &drawer, &file, input, output)
        }
        MemoCommand::DeleteByNumber(number) => {
            let file = find_by_number(&ctx.cabinet, &drawer, number)?;
            delete(ctx, &drawer, &file, input, output)
        }
    }
}

fn editor(ctx: &Context) -> Result<String> {
    resolve_editor(ctx.editor_env.as_deref(), &ctx.settings)
}

fn list(ctx: &Context, drawer: &str, output: &mut impl Write) -> Result<()> {
    let memos = list_memos(&ctx.cabinet, drawer)?;
    if memos.is_empty() {
        return say(output, &format!("No memos in drawer '{drawer}'"));
    }
    for (index, memo) in memos.iter().enumerate() {
        say(output, &format!("[{}] {}", index + 1, memo.display_name()))?;
    }
    Ok(())
}

fn open(ctx: &Context, drawer: &str, file: &str, editor: &str) -> Result<()> {
    let path = ctx.cabinet.join(drawer).join(file);
    open_in_editor(editor, &path)
}

// エディタを先に決める(未設定のとき、空のファイルを残さないため)。
fn add(ctx: &Context, drawer: &str, output: &mut impl Write) -> Result<()> {
    let editor = editor(ctx)?;
    let file = memo_file_name(chrono::Local::now().naive_local());
    let path = create_memo(&ctx.cabinet, drawer, &file)?;
    say(output, &format!("Created {file}"))?;
    open_in_editor(&editor, &path)
}

// 確認のうえ削除する。対象の特定は確認の前に済ませてある。
fn delete(
    ctx: &Context,
    drawer: &str,
    file: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<()> {
    let label = label_of(ctx, drawer, file)?;
    if !confirm(input, output, &format!("Delete memo '{label}'?"))? {
        return say(output, "Canceled");
    }
    delete_memo(&ctx.cabinet, drawer, file)?;
    say(output, &format!("Deleted memo '{label}'"))
}

fn label_of(ctx: &Context, drawer: &str, file: &str) -> Result<String> {
    list_memos(&ctx.cabinet, drawer)?
        .into_iter()
        .find(|memo| memo.file_name == file)
        .map(|memo| memo.label())
        .ok_or_else(|| CnError::MemoNotFound(file.to_string()))
}
