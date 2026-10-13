// メモ本文から最初の # 見出し(レベル1)を取り出す。規則は commands.md の「タイトルの扱い」。

fn is_code_fence(line: &str) -> bool {
    line.starts_with("```") || line.starts_with("~~~")
}

pub fn extract_title(body: &str) -> Option<String> {
    // 先頭のBOMは無視する(Windowsのメモ帳が付けることがある)
    let body = body.strip_prefix('\u{feff}').unwrap_or(body);
    let mut in_code_block = false;
    for line in body.lines() {
        if is_code_fence(line) {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            let title = rest.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}
