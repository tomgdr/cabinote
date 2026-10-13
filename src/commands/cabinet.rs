// cn cabinet: 現在の保存先のパスを表示する。

use std::io::Write;

use crate::commands::{Context, say};
use crate::error::Result;

pub fn run(ctx: &Context, output: &mut impl Write) -> Result<()> {
    say(output, &ctx.cabinet.display().to_string())
}
