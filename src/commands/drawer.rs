// cn drawer: 引き出しの表示・一覧・追加・切り替え・改名・削除。

use std::io::{BufRead, Write};

use crate::cli::DrawerCommand;
use crate::commands::{Context, current_drawer, say};
use crate::current::{DEFAULT_DRAWER, write_current};
use crate::error::{CnError, Result};
use crate::prompt::confirm;
use crate::store::check_cabinet;
use crate::store::drawer::{
    add_drawer, check_not_home, delete_drawer, drawer_exists, find_drawer_by_number, list_drawers,
    rename_drawer,
};
use crate::store::memo::count_memos;

pub fn run(
    ctx: &Context,
    command: DrawerCommand,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<()> {
    check_cabinet(&ctx.cabinet)?;
    match command {
        DrawerCommand::Show => say(output, &current_drawer(ctx)?),
        DrawerCommand::List => list(ctx, output),
        DrawerCommand::Add(name) => {
            add_drawer(&ctx.cabinet, &name)?;
            say(output, &format!("Created drawer '{name}'"))
        }
        DrawerCommand::SwitchByName(name) => switch(ctx, &name, output),
        DrawerCommand::SwitchByNumber(number) => {
            let name = find_drawer_by_number(&ctx.cabinet, number)?;
            switch(ctx, &name, output)
        }
        DrawerCommand::Rename(new) => rename(ctx, &new, output),
        DrawerCommand::DeleteByName(name) => delete(ctx, &name, input, output),
        DrawerCommand::DeleteByNumber(number) => {
            let name = find_drawer_by_number(&ctx.cabinet, number)?;
            delete(ctx, &name, input, output)
        }
    }
}

fn list(ctx: &Context, output: &mut impl Write) -> Result<()> {
    let names = list_drawers(&ctx.cabinet)?;
    if names.is_empty() {
        return say(output, "No drawers");
    }
    let current = current_drawer(ctx)?;
    for (index, name) in names.iter().enumerate() {
        let mark = if *name == current { "*" } else { " " };
        say(output, &format!("{mark} [{}] {name}", index + 1))?;
    }
    Ok(())
}

// 切り替える。存在しなければ .current は変えない。
fn switch(ctx: &Context, name: &str, output: &mut impl Write) -> Result<()> {
    if !drawer_exists(&ctx.cabinet, name)? {
        return Err(CnError::DrawerNotFound(name.to_string()));
    }
    write_current(&ctx.state_dir, name)?;
    say(output, &format!("Switched to drawer '{name}'"))
}

// 改名し、.current も更新する。
fn rename(ctx: &Context, new: &str, output: &mut impl Write) -> Result<()> {
    let old = current_drawer(ctx)?;
    rename_drawer(&ctx.cabinet, &old, new)?;
    write_current(&ctx.state_dir, new)?;
    say(output, &format!("Renamed drawer '{old}' to '{new}'"))
}

// 確認のうえ削除する。現在の引き出しなら home に戻す。確認の前に、名前・home・存在を検証する。
fn delete(
    ctx: &Context,
    name: &str,
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> Result<()> {
    if !drawer_exists(&ctx.cabinet, name)? {
        return Err(CnError::DrawerNotFound(name.to_string()));
    }
    check_not_home(name)?;
    let count = count_memos(&ctx.cabinet, name)?;
    let unit = if count == 1 { "memo" } else { "memos" };
    let question = format!("Delete drawer '{name}' and its {count} {unit}?");
    if !confirm(input, output, &question)? {
        return say(output, "Canceled");
    }
    let was_current = current_drawer(ctx)? == name;
    delete_drawer(&ctx.cabinet, name)?;
    if was_current {
        write_current(&ctx.state_dir, DEFAULT_DRAWER)?;
    }
    say(output, &format!("Deleted drawer '{name}'"))
}
