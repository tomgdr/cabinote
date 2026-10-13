// y/n の確認。削除の前に使う。入出力は引数で受け取る。

use std::io::{BufRead, Write};

use crate::error::{CnError, Result};

// 入力が "y"(前後の空白は無視)のときだけ true。EOFや他の入力は false(明示されない限り削除しない)。
pub fn confirm(input: &mut impl BufRead, output: &mut impl Write, message: &str) -> Result<bool> {
    let io_error = |source| CnError::Io {
        context: "failed to read the confirmation".to_string(),
        source,
    };
    write!(output, "{message} [y/N]: ").map_err(io_error)?;
    // 改行なしの表示は、flush しないと入力待ちの前に表示されないことがある
    output.flush().map_err(io_error)?;
    let mut line = String::new();
    input.read_line(&mut line).map_err(io_error)?;
    Ok(line.trim() == "y")
}
