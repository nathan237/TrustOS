//! Shell Scripting Engine — Variables, Control Flow, Expansion
//!
//! Provides POSIX-like shell scripting capabilities:
//! - `$VAR` and `${VAR}` variable expansion
//! - `export VAR=value` to set variables
//! - `if/elif/else/fi` conditional blocks
//! - `for VAR in LIST; do ... done` loops
//! - `while COND; do ... done` loops
//! - Command substitution: `$(command)`
//! - Arithmetic: `$((expr))`
//! - Special variables: `$?`, `$#`, `$$`, `$0`

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use spin::Mutex;

/// Global shell variable store
static SHELL_VARS: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

/// Last command exit code
static LAST_EXIT_CODE: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);

/// Initialize default shell variables
pub fn init() {
    let mut vars = SHELL_VARS.lock();
    vars.insert(String::from("HOME"), String::from("/"));
    vars.insert(String::from("USER"), String::from("root"));
    vars.insert(String::from("SHELL"), String::from("/bin/tsh"));
    vars.insert(String::from("PATH"), String::from("/bin:/usr/bin:/sbin"));
    vars.insert(String::from("PWD"), String::from("/"));
    vars.insert(String::from("HOSTNAME"), String::from("trustos"));
    vars.insert(String::from("TERM"), String::from("xterm-256color"));
    vars.insert(String::from("LANG"), String::from("en_US.UTF-8"));
    vars.insert(String::from("PS1"), String::from("\\u@\\h:\\w$ "));
    vars.insert(String::from("OSTYPE"), String::from("trustos"));
    vars.insert(String::from("EDITOR"), String::from("trustedit"));
}

/// Set a shell variable
pub fn set_var(name: &str, value: &str) {
    SHELL_VARS.lock().insert(String::from(name), String::from(value));
}

/// Get a shell variable
pub fn get_var(name: &str) -> Option<String> {
    SHELL_VARS.lock().get(name).cloned()
}

/// Remove a shell variable
pub fn unset_var(name: &str) {
    SHELL_VARS.lock().remove(name);
}

/// Get all variables (for `env` / `set` commands)
pub fn all_vars() -> Vec<(String, String)> {
    SHELL_VARS.lock().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

/// Set the last exit code
pub fn set_exit_code(code: i32) {
    LAST_EXIT_CODE.store(code, core::sync::atomic::Ordering::SeqCst);
}

/// Get the last exit code
pub fn get_exit_code() -> i32 {
    LAST_EXIT_CODE.load(core::sync::atomic::Ordering::SeqCst)
}

/// Expand all `$VAR`, `${VAR}`, `$?`, `$$`, `$((expr))` in a string.
/// Does NOT expand `$(command)` — that's handled separately.
pub fn expand_variables(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            let next = bytes[i + 1];

            // Special variables
            if next == b'?' {
                // $? — last exit code
                result.push_str(&format!("{}", get_exit_code()));
                i += 2;
                continue;
            }
            if next == b'$' {
                // $$ — PID (always 1 in kernel mode)
                result.push_str("1");
                i += 2;
                continue;
            }
            if next == b'#' {
                // $# — argument count (0 in interactive)
                result.push_str("0");
                i += 2;
                continue;
            }

            // $((arithmetic))
            if next == b'(' && i + 2 < bytes.len() && bytes[i + 2] == b'(' {
                if let Some(end) = find_matching_double_paren(input, i + 2) {
                    let expr = &input[i + 3..end];
                    let val = eval_arithmetic(expr);
                    result.push_str(&format!("{}", val));
                    i = end + 2; // skip ))
                    continue;
                }
            }

            // $(command substitution)
            if next == b'(' {
                if let Some(end) = find_matching_paren(input, i + 1) {
                    let cmd = &input[i + 2..end];
                    let output = command_substitution(cmd);
                    result.push_str(output.trim());
                    i = end + 1;
                    continue;
                }
            }

            // ${VAR} or ${VAR:-default}
            if next == b'{' {
                if let Some(close) = input[i + 2..].find('}') {
                    let var_spec = &input[i + 2..i + 2 + close];
                    let value = expand_var_spec(var_spec);
                    result.push_str(&value);
                    i = i + 3 + close;
                    continue;
                }
            }

            // $VAR — alphanumeric + underscore
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                let name = &input[start..end];
                if let Some(val) = get_var(name) {
                    result.push_str(&val);
                }
                // If not set, expand to empty string (POSIX behavior)
                i = end;
                continue;
            }

            // Bare $ — just add it
            result.push('$');
            i += 1;
        } else if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Escape sequences
            match bytes[i + 1] {
                b'n' => result.push('\n'),
                b't' => result.push('\t'),
                b'\\' => result.push('\\'),
                b'$' => result.push('$'),
                b'"' => result.push('"'),
                other => {
                    result.push('\\');
                    result.push(other as char);
                }
            }
            i += 2;
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Expand ${VAR}, ${VAR:-default}, ${VAR:=default}, ${VAR:+alt}, ${#VAR}
fn expand_var_spec(spec: &str) -> String {
    // ${#VAR} — length
    if spec.starts_with('#') {
        let name = &spec[1..];
        return format!("{}", get_var(name).map(|v| v.len()).unwrap_or(0));
    }

    // ${VAR:-default} — use default if unset
    if let Some(pos) = spec.find(":-") {
        let name = &spec[..pos];
        let default = &spec[pos + 2..];
        return get_var(name).unwrap_or_else(|| String::from(default));
    }

    // ${VAR:=default} — assign default if unset
    if let Some(pos) = spec.find(":=") {
        let name = &spec[..pos];
        let default = &spec[pos + 2..];
        if let Some(val) = get_var(name) {
            return val;
        }
        set_var(name, default);
        return String::from(default);
    }

    // ${VAR:+alt} — use alt only if set
    if let Some(pos) = spec.find(":+") {
        let name = &spec[..pos];
        let alt = &spec[pos + 2..];
        return if get_var(name).is_some() { String::from(alt) } else { String::new() };
    }

    // Plain ${VAR}
    get_var(spec).unwrap_or_default()
}

/// Execute a command and capture its output (for `$(...)`)
fn command_substitution(cmd: &str) -> String {
    // Use the capture mode infrastructure
    use core::sync::atomic::Ordering;
    super::CAPTURE_MODE.store(true, Ordering::SeqCst);
    {
        let mut buf = super::CAPTURE_BUF.lock();
        buf.clear();
    }

    super::execute_command(cmd);

    super::CAPTURE_MODE.store(false, Ordering::SeqCst);
    let buf = super::CAPTURE_BUF.lock();
    buf.clone()
}

fn find_matching_paren(s: &str, open_pos: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0;
    for i in open_pos..bytes.len() {
        if bytes[i] == b'(' { depth += 1; }
        if bytes[i] == b')' {
            depth -= 1;
            if depth == 0 { return Some(i); }
        }
    }
    None
}

fn find_matching_double_paren(s: &str, open_pos: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0;
    let mut i = open_pos;
    while i < bytes.len() {
        if bytes[i] == b'(' { depth += 1; }
        if bytes[i] == b')' {
            depth -= 1;
            if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b')' {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Simple integer arithmetic evaluator for $((expr))
/// Supports: +, -, *, /, %, parentheses, variable references
pub fn eval_arithmetic(expr: &str) -> i64 {
    let expanded = expand_variables(expr);
    let tokens = tokenize_arith(&expanded);
    parse_expr(&tokens, &mut 0)
}

fn tokenize_arith(expr: &str) -> Vec<ArithToken> {
    let mut tokens = Vec::new();
    let bytes = expr.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b' ' | b'\t' => { i += 1; }
            b'+' => { tokens.push(ArithToken::Plus); i += 1; }
            b'-' => { tokens.push(ArithToken::Minus); i += 1; }
            b'*' => { tokens.push(ArithToken::Mul); i += 1; }
            b'/' => { tokens.push(ArithToken::Div); i += 1; }
            b'%' => { tokens.push(ArithToken::Mod); i += 1; }
            b'(' => { tokens.push(ArithToken::LParen); i += 1; }
            b')' => { tokens.push(ArithToken::RParen); i += 1; }
            b'0'..=b'9' => {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
                let n: i64 = expr[start..i].parse().unwrap_or(0);
                tokens.push(ArithToken::Num(n));
            }
            _ => { i += 1; } // skip unknown
        }
    }
    tokens
}

#[derive(Debug, Clone)]
enum ArithToken { Num(i64), Plus, Minus, Mul, Div, Mod, LParen, RParen }

fn parse_expr(tokens: &[ArithToken], pos: &mut usize) -> i64 {
    let mut left = parse_term(tokens, pos);
    while *pos < tokens.len() {
        match &tokens[*pos] {
            ArithToken::Plus => { *pos += 1; left += parse_term(tokens, pos); }
            ArithToken::Minus => { *pos += 1; left -= parse_term(tokens, pos); }
            _ => break,
        }
    }
    left
}

fn parse_term(tokens: &[ArithToken], pos: &mut usize) -> i64 {
    let mut left = parse_factor(tokens, pos);
    while *pos < tokens.len() {
        match &tokens[*pos] {
            ArithToken::Mul => { *pos += 1; left *= parse_factor(tokens, pos); }
            ArithToken::Div => {
                *pos += 1;
                let r = parse_factor(tokens, pos);
                if r != 0 { left /= r; }
            }
            ArithToken::Mod => {
                *pos += 1;
                let r = parse_factor(tokens, pos);
                if r != 0 { left %= r; }
            }
            _ => break,
        }
    }
    left
}

fn parse_factor(tokens: &[ArithToken], pos: &mut usize) -> i64 {
    if *pos >= tokens.len() { return 0; }
    match &tokens[*pos] {
        ArithToken::Num(n) => { let v = *n; *pos += 1; v }
        ArithToken::Minus => { *pos += 1; -parse_factor(tokens, pos) }
        ArithToken::Plus => { *pos += 1; parse_factor(tokens, pos) }
        ArithToken::LParen => {
            *pos += 1;
            let v = parse_expr(tokens, pos);
            if *pos < tokens.len() { *pos += 1; } // skip RParen
            v
        }
        _ => { *pos += 1; 0 }
    }
}

/// Execute an `if/elif/else/fi` block.
/// Returns true if the block was consumed (input was an if-block).
pub fn execute_if_block(lines: &[&str], idx: &mut usize) -> bool {
    if *idx >= lines.len() { return false; }
    let first = lines[*idx].trim();
    if !first.starts_with("if ") { return false; }

    // Collect the full if/elif/else/fi block
    let mut block_lines: Vec<&str> = Vec::new();
    let mut depth = 0;
    let start = *idx;

    while *idx < lines.len() {
        let line = lines[*idx].trim();
        if line.starts_with("if ") { depth += 1; }
        if line == "fi" { depth -= 1; }
        block_lines.push(line);
        *idx += 1;
        if depth == 0 { break; }
    }

    // Parse into condition → body segments
    let mut segments: Vec<(Option<&str>, Vec<&str>)> = Vec::new();
    let mut current_cond: Option<&str> = None;
    let mut current_body: Vec<&str> = Vec::new();
    let mut inner_depth = 0;

    for &line in &block_lines {
        if inner_depth == 0 {
            if line.starts_with("if ") && segments.is_empty() {
                let cond = line.strip_prefix("if ").unwrap().trim();
                let cond = cond.strip_suffix("; then").or_else(|| cond.strip_suffix(" then")).unwrap_or(cond);
                current_cond = Some(cond);
                inner_depth = 0;
                continue;
            }
            if line == "then" { continue; }
            if line.starts_with("elif ") {
                segments.push((current_cond, core::mem::take(&mut current_body)));
                let cond = line.strip_prefix("elif ").unwrap().trim();
                let cond = cond.strip_suffix("; then").or_else(|| cond.strip_suffix(" then")).unwrap_or(cond);
                current_cond = Some(cond);
                continue;
            }
            if line == "else" {
                segments.push((current_cond, core::mem::take(&mut current_body)));
                current_cond = None; // else always runs
                continue;
            }
            if line == "fi" {
                segments.push((current_cond, core::mem::take(&mut current_body)));
                break;
            }
        }

        // Track nested if depth
        if line.starts_with("if ") { inner_depth += 1; }
        if line == "fi" { inner_depth -= 1; }
        current_body.push(line);
    }

    // Execute: find first matching condition
    for (cond, body) in &segments {
        let should_run = match cond {
            None => true, // else
            Some(c) => eval_condition(c),
        };
        if should_run {
            for &line in body {
                super::execute_command(line);
            }
            break;
        }
    }

    true
}

/// Execute a `for VAR in LIST; do ... done` block.
pub fn execute_for_block(lines: &[&str], idx: &mut usize) -> bool {
    if *idx >= lines.len() { return false; }
    let first = lines[*idx].trim();
    if !first.starts_with("for ") { return false; }

    // Collect the full for/do/done block
    let mut block_lines: Vec<&str> = Vec::new();
    let mut depth = 0;
    while *idx < lines.len() {
        let line = lines[*idx].trim();
        if line.starts_with("for ") || line.starts_with("while ") { depth += 1; }
        if line == "done" { depth -= 1; }
        block_lines.push(line);
        *idx += 1;
        if depth == 0 { break; }
    }

    // Parse: for VAR in item1 item2 ...; do
    let header = block_lines[0];
    let after_for = header.strip_prefix("for ").unwrap().trim();

    // Split on " in "
    let (var_name, list_str) = if let Some(pos) = after_for.find(" in ") {
        (&after_for[..pos], &after_for[pos + 4..])
    } else {
        return true; // Malformed
    };

    let list_str = list_str.strip_suffix("; do")
        .or_else(|| list_str.strip_suffix(" do"))
        .unwrap_or(list_str);

    // Expand variables in list
    let expanded_list = expand_variables(list_str);
    let items: Vec<&str> = expanded_list.split_whitespace().collect();

    // Find body (between "do" and "done")
    let mut body: Vec<&str> = Vec::new();
    let mut in_body = false;
    for &line in &block_lines[1..] {
        if line == "do" { in_body = true; continue; }
        if line == "done" { break; }
        if in_body { body.push(line); }
        // If "do" was on the for line, everything after is body
        if !in_body && block_lines[0].contains("; do") {
            in_body = true;
            body.push(line);
        }
    }
    // Handle "for VAR in LIST; do" (do on same line)
    if block_lines[0].ends_with("; do") || block_lines[0].ends_with(" do") {
        body.clear();
        for &line in &block_lines[1..] {
            if line == "done" { break; }
            body.push(line);
        }
    }

    // Execute body for each item
    for item in &items {
        set_var(var_name, item);
        for &line in &body {
            let expanded = expand_variables(line);
            super::execute_command(&expanded);
        }
    }

    true
}

/// Execute a `while COND; do ... done` block.
pub fn execute_while_block(lines: &[&str], idx: &mut usize) -> bool {
    if *idx >= lines.len() { return false; }
    let first = lines[*idx].trim();
    if !first.starts_with("while ") { return false; }

    // Collect block
    let mut block_lines: Vec<&str> = Vec::new();
    let mut depth = 0;
    while *idx < lines.len() {
        let line = lines[*idx].trim();
        if line.starts_with("for ") || line.starts_with("while ") { depth += 1; }
        if line == "done" { depth -= 1; }
        block_lines.push(line);
        *idx += 1;
        if depth == 0 { break; }
    }

    // Parse condition
    let header = block_lines[0];
    let cond = header.strip_prefix("while ").unwrap().trim();
    let cond = cond.strip_suffix("; do")
        .or_else(|| cond.strip_suffix(" do"))
        .unwrap_or(cond);

    // Find body
    let mut body: Vec<&str> = Vec::new();
    let first_is_do = block_lines[0].contains("; do") || block_lines[0].ends_with(" do");
    if first_is_do {
        for &line in &block_lines[1..] {
            if line == "done" { break; }
            body.push(line);
        }
    } else {
        let mut in_body = false;
        for &line in &block_lines[1..] {
            if line == "do" { in_body = true; continue; }
            if line == "done" { break; }
            if in_body { body.push(line); }
        }
    }

    // Execute loop (max 1000 iterations safety limit)
    let mut iterations = 0;
    while eval_condition(cond) && iterations < 1000 {
        for &line in &body {
            let expanded = expand_variables(line);
            super::execute_command(&expanded);
        }
        iterations += 1;
    }

    true
}

/// Evaluate a condition for if/while.
/// Supports:
/// - `[ expr ]` / `test expr` — POSIX test expressions
/// - `command` — runs command, checks exit code (0 = true)
/// - String comparisons: `=`, `!=`
/// - Numeric comparisons: `-eq`, `-ne`, `-lt`, `-gt`, `-le`, `-ge`
/// - File tests: `-f`, `-d`, `-e`, `-z`, `-n`
fn eval_condition(cond: &str) -> bool {
    let cond = cond.trim();

    // [ ... ] bracket syntax
    if cond.starts_with("[ ") && cond.ends_with(" ]") {
        let inner = &cond[2..cond.len() - 2].trim();
        return eval_test_expr(inner);
    }

    // test ... command
    if cond.starts_with("test ") {
        let inner = &cond[5..].trim();
        return eval_test_expr(inner);
    }

    // true / false literals
    if cond == "true" { return true; }
    if cond == "false" { return false; }

    // Run as command and check exit code
    // (Use capture mode to suppress output)
    let expanded = expand_variables(cond);
    super::CAPTURE_MODE.store(true, core::sync::atomic::Ordering::SeqCst);
    { super::CAPTURE_BUF.lock().clear(); }
    super::execute_command(&expanded);
    super::CAPTURE_MODE.store(false, core::sync::atomic::Ordering::SeqCst);
    get_exit_code() == 0
}

/// Evaluate a test expression (for `[...]` and `test`)
fn eval_test_expr(expr: &str) -> bool {
    let expanded = expand_variables(expr);
    let parts: Vec<&str> = expanded.split_whitespace().collect();
    
    match parts.as_slice() {
        // Unary operators
        ["-z", s] => s.is_empty(),
        ["-n", s] => !s.is_empty(),
        ["-f", path] => crate::ramfs::with_fs(|fs| {
            fs.stat(path).map(|e| e.file_type == crate::ramfs::FileType::File).unwrap_or(false)
        }),
        ["-d", path] => crate::ramfs::with_fs(|fs| {
            fs.stat(path).map(|e| e.file_type == crate::ramfs::FileType::Directory).unwrap_or(false)
        }),
        ["-e", path] => crate::ramfs::with_fs(|fs| fs.exists(path)),
        // String comparison
        [a, "=", b] | [a, "==", b] => a == b,
        [a, "!=", b] => a != b,
        // Numeric comparison
        [a, "-eq", b] => a.parse::<i64>().ok() == b.parse::<i64>().ok(),
        [a, "-ne", b] => a.parse::<i64>().ok() != b.parse::<i64>().ok(),
        [a, "-lt", b] => a.parse::<i64>().unwrap_or(0) < b.parse::<i64>().unwrap_or(0),
        [a, "-gt", b] => a.parse::<i64>().unwrap_or(0) > b.parse::<i64>().unwrap_or(0),
        [a, "-le", b] => a.parse::<i64>().unwrap_or(0) <= b.parse::<i64>().unwrap_or(0),
        [a, "-ge", b] => a.parse::<i64>().unwrap_or(0) >= b.parse::<i64>().unwrap_or(0),
        // Negation
        ["!", rest @ ..] => !eval_test_expr(&rest.join(" ")),
        // Single string: non-empty = true
        [s] => !s.is_empty(),
        _ => false,
    }
}

/// Execute a script string (multi-line). Handles if/for/while blocks.
pub fn execute_script(script: &str) {
    let lines: Vec<&str> = script.lines().collect();
    let mut idx = 0;
    
    while idx < lines.len() {
        let line = lines[idx].trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            idx += 1;
            continue;
        }

        // Try control flow blocks
        if execute_if_block(&lines, &mut idx) { continue; }
        if execute_for_block(&lines, &mut idx) { continue; }
        if execute_while_block(&lines, &mut idx) { continue; }

        // Regular command — expand variables first
        let expanded = expand_variables(line);
        super::execute_command(&expanded);
        idx += 1;
    }
}
