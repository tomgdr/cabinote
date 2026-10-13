// コマンドライン引数を Command に変換する(入出力なし)。引き出し名の検証は commands/ が行う。

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Command {
    // cn / cn -h / cn --help(サブコマンドの後ろの -h / --help も同じ)
    Help,
    // cn -V / cn --version
    Version,
    // cn cabinet / cn c
    Cabinet,
    // cn drawer / cn d
    Drawer(DrawerCommand),
    // cn memo / cn m
    Memo(MemoCommand),
}

#[derive(Debug, PartialEq)]
pub enum DrawerCommand {
    // cn drawer
    Show,
    // -l
    List,
    // -a <name>
    Add(String),
    // -c <name>
    SwitchByName(String),
    // -cn <number>(番号は1始まり)
    SwitchByNumber(usize),
    // -r <new name>
    Rename(String),
    // -d <name>
    DeleteByName(String),
    // -dn <number>(番号は1始まり)
    DeleteByNumber(usize),
}

#[derive(Debug, PartialEq)]
pub enum MemoCommand {
    // cn memo <title>
    OpenByTitle(String),
    // -n <number>(番号は1始まり)
    OpenByNumber(usize),
    // -l
    List,
    // -a
    Add,
    // -e
    Editor,
    // -d <title>
    DeleteByTitle(String),
    // -dn <number>(番号は1始まり)
    DeleteByNumber(usize),
}

#[derive(Debug, PartialEq)]
pub enum CliError {
    // 知らないコマンド(cn foo)
    UnknownCommand(String),
    // 知らないオプション(cn memo -x)
    UnknownOption(String),
    // 値が必要なオプションに値が無い(中身はオプション名。タイトルが空のときは "<title>")
    MissingValue(String),
    // 番号が不正(数字でない、または0)。中身は入力された文字列
    InvalidNumber(String),
    // 余分な引数
    UnexpectedArgument(String),
    // オプションが2つ以上指定された
    ConflictingOptions,
    // 引数がUTF-8として読めない(main.rs が引数を文字列にするときに使う)
    NotUnicode,
    // オプションもタイトルも無い(中身はコマンド名。cn memo のみ)
    MissingOption(&'static str),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::UnknownCommand(name) => write!(f, "unknown command: {name}"),
            CliError::UnknownOption(name) => write!(f, "unknown option: {name}"),
            CliError::MissingValue(name) => write!(f, "{name} requires a value"),
            CliError::InvalidNumber(value) => {
                write!(f, "invalid number: {value} (must be 1 or greater)")
            }
            CliError::UnexpectedArgument(value) => write!(f, "unexpected argument: {value}"),
            CliError::ConflictingOptions => write!(f, "only one option can be given at a time"),
            CliError::NotUnicode => write!(f, "arguments must be valid UTF-8"),
            CliError::MissingOption(command) => {
                write!(f, "'cn {command}' requires an option or a title")
            }
        }
    }
}

// 数字でない場合と 0 は InvalidNumber。
fn parse_number(value: &str) -> Result<usize, CliError> {
    match value.parse::<usize>() {
        Ok(number) if number >= 1 => Ok(number),
        _ => Err(CliError::InvalidNumber(value.to_string())),
    }
}

// 値を取らないオプションの後ろに余分な引数が無いことを確認する("-" 始まりは ConflictingOptions)。
fn no_more(rest: &[String]) -> Result<(), CliError> {
    match rest.first() {
        None => Ok(()),
        Some(arg) if arg.starts_with('-') => Err(CliError::ConflictingOptions),
        Some(arg) => Err(CliError::UnexpectedArgument(arg.clone())),
    }
}

// 値を1つ取るオプションの値を取り出す。無い・"-" 始まり・空なら MissingValue。
fn single_value(option: &str, rest: &[String]) -> Result<String, CliError> {
    match rest.first() {
        Some(value) if !value.is_empty() && !value.starts_with('-') => {
            no_more(&rest[1..])?;
            Ok(value.clone())
        }
        _ => Err(CliError::MissingValue(option.to_string())),
    }
}

fn parse_cabinet(rest: &[String]) -> Result<Command, CliError> {
    match rest.first() {
        None => Ok(Command::Cabinet),
        Some(arg) if arg.starts_with('-') => Err(CliError::UnknownOption(arg.clone())),
        Some(arg) => Err(CliError::UnexpectedArgument(arg.clone())),
    }
}

fn parse_drawer(rest: &[String]) -> Result<Command, CliError> {
    let Some(option) = rest.first() else {
        return Ok(Command::Drawer(DrawerCommand::Show));
    };
    let values = &rest[1..];
    let command = match option.as_str() {
        "-l" => {
            no_more(values)?;
            DrawerCommand::List
        }
        "-a" => DrawerCommand::Add(single_value(option, values)?),
        "-c" => DrawerCommand::SwitchByName(single_value(option, values)?),
        "-cn" => DrawerCommand::SwitchByNumber(parse_number(&single_value(option, values)?)?),
        "-r" => DrawerCommand::Rename(single_value(option, values)?),
        "-d" => DrawerCommand::DeleteByName(single_value(option, values)?),
        "-dn" => DrawerCommand::DeleteByNumber(parse_number(&single_value(option, values)?)?),
        _ if option.starts_with('-') => return Err(CliError::UnknownOption(option.clone())),
        _ => return Err(CliError::UnexpectedArgument(option.clone())),
    };
    Ok(Command::Drawer(command))
}

// cn memo の後ろの引数を解析する。無ければ MissingOption。
fn parse_memo(rest: &[String]) -> Result<Command, CliError> {
    let Some(first) = rest.first() else {
        return Err(CliError::MissingOption("memo"));
    };
    let values = &rest[1..];
    let command = match first.as_str() {
        "-l" => {
            no_more(values)?;
            MemoCommand::List
        }
        "-a" => {
            no_more(values)?;
            MemoCommand::Add
        }
        "-e" => {
            no_more(values)?;
            MemoCommand::Editor
        }
        "-n" => MemoCommand::OpenByNumber(parse_number(&single_value(first, values)?)?),
        "-d" => MemoCommand::DeleteByTitle(single_value(first, values)?),
        "-dn" => MemoCommand::DeleteByNumber(parse_number(&single_value(first, values)?)?),
        _ if first.starts_with('-') => return Err(CliError::UnknownOption(first.clone())),
        // "-" で始まらない最初の引数はタイトル
        _ if first.is_empty() => return Err(CliError::MissingValue("<title>".to_string())),
        _ => {
            if let Some(extra) = values.first() {
                return Err(CliError::UnexpectedArgument(extra.clone()));
            }
            MemoCommand::OpenByTitle(first.clone())
        }
    };
    Ok(Command::Memo(command))
}

// args はプログラム名を除いた引数。サブコマンドの後ろの -h / --help もヘルプにする。
pub fn parse_args(args: &[String]) -> Result<Command, CliError> {
    let Some(name) = args.first() else {
        return Ok(Command::Help);
    };
    let rest = &args[1..];
    if name == "-h" || name == "--help" {
        return Ok(Command::Help);
    }
    if name == "-V" || name == "--version" {
        return Ok(Command::Version);
    }
    if let Some(first) = rest.first()
        && (first == "-h" || first == "--help")
    {
        return Ok(Command::Help);
    }
    match name.as_str() {
        "cabinet" | "c" => parse_cabinet(rest),
        "drawer" | "d" => parse_drawer(rest),
        "memo" | "m" => parse_memo(rest),
        _ => Err(CliError::UnknownCommand(name.clone())),
    }
}
