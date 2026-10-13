// 引き出し名の検証。項目ごとの関数を、validate_drawer_name でまとめて呼ぶ。

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum NameError {
    Empty,
    ForbiddenChar(char),
    ControlChar,
    ReservedName(String),
    TooLong(usize),
    EdgeWhitespace,
}

const MAX_NAME_CHARS: usize = 50;

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::Empty => {
                write!(f, "name must not be empty")
            }
            NameError::ForbiddenChar(c) => {
                write!(f, "name contains a forbidden character: '{}'", c)
            }
            NameError::ControlChar => {
                write!(f, "name contains a control character")
            }
            NameError::ReservedName(name) => {
                write!(f, "name is reserved on Windows: {}", name)
            }
            NameError::TooLong(len) => {
                write!(
                    f,
                    "name is too long: {} characters (max {})",
                    len, MAX_NAME_CHARS
                )
            }
            NameError::EdgeWhitespace => {
                write!(f, "name must not start or end with whitespace")
            }
        }
    }
}

// Windowsの予約デバイス名(大文字小文字は区別しない)。
const RESERVED_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

const FORBIDDEN_CHARS: [char; 10] = ['/', '\\', '<', '>', ':', '"', '|', '?', '*', '.'];

pub fn check_not_empty(name: &str) -> Result<(), NameError> {
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    Ok(())
}

pub fn check_no_forbidden_chars(name: &str) -> Result<(), NameError> {
    for c in name.chars() {
        if FORBIDDEN_CHARS.contains(&c) {
            return Err(NameError::ForbiddenChar(c));
        }
    }
    Ok(())
}

pub fn check_no_control_chars(name: &str) -> Result<(), NameError> {
    if name.chars().any(|c| c.is_control()) {
        return Err(NameError::ControlChar);
    }
    Ok(())
}

pub fn check_not_reserved_name(name: &str) -> Result<(), NameError> {
    for r in RESERVED_NAMES {
        if name.eq_ignore_ascii_case(r) {
            return Err(NameError::ReservedName(name.to_string()));
        }
    }
    Ok(())
}

// バイト数ではなく文字数で数える。
pub fn check_length(name: &str) -> Result<(), NameError> {
    let length: usize = name.chars().count();
    if length > MAX_NAME_CHARS {
        return Err(NameError::TooLong(length));
    }
    Ok(())
}

pub fn check_no_edge_whitespace(name: &str) -> Result<(), NameError> {
    if name.trim() != name {
        return Err(NameError::EdgeWhitespace);
    }
    Ok(())
}

// 上から順に確認し、最初に失敗した理由を返す。
pub fn validate_drawer_name(name: &str) -> Result<(), NameError> {
    check_not_empty(name)?;
    check_no_forbidden_chars(name)?;
    check_no_control_chars(name)?;
    check_not_reserved_name(name)?;
    check_length(name)?;
    check_no_edge_whitespace(name)?;
    Ok(())
}
