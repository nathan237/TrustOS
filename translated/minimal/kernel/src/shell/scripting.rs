











use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use spin::Mutex;


static QK_: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());


static BAT_: core::sync::atomic::AtomicI32 = core::sync::atomic::AtomicI32::new(0);


pub fn init() {
    let mut qq = QK_.lock();
    qq.insert(String::from("HOME"), String::from("/"));
    qq.insert(String::from("USER"), String::from("root"));
    qq.insert(String::from("SHELL"), String::from("/bin/tsh"));
    qq.insert(String::from("PATH"), String::from("/bin:/usr/bin:/sbin"));
    qq.insert(String::from("PWD"), String::from("/"));
    qq.insert(String::from("HOSTNAME"), String::from("trustos"));
    qq.insert(String::from("TERM"), String::from("xterm-256color"));
    qq.insert(String::from("LANG"), String::from("en_US.UTF-8"));
    qq.insert(String::from("PS1"), String::from("\\u@\\h:\\w$ "));
    qq.insert(String::from("OSTYPE"), String::from("trustos"));
    qq.insert(String::from("EDITOR"), String::from("trustedit"));
}


pub fn cql(name: &str, value: &str) {
    QK_.lock().insert(String::from(name), String::from(value));
}


pub fn axh(name: &str) -> Option<String> {
    QK_.lock().get(name).cloned()
}


pub fn fdx(name: &str) {
    QK_.lock().remove(name);
}


pub fn efi() -> Vec<(String, String)> {
    QK_.lock().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}


pub fn qvv(code: i32) {
    BAT_.store(code, core::sync::atomic::Ordering::SeqCst);
}


pub fn ibm() -> i32 {
    BAT_.load(core::sync::atomic::Ordering::SeqCst)
}



pub fn bbm(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            let next = bytes[i + 1];

            
            if next == b'?' {
                
                result.push_str(&format!("{}", ibm()));
                i += 2;
                continue;
            }
            if next == b'$' {
                
                result.push_str("1");
                i += 2;
                continue;
            }
            if next == b'#' {
                
                result.push_str("0");
                i += 2;
                continue;
            }

            
            if next == b'(' && i + 2 < bytes.len() && bytes[i + 2] == b'(' {
                if let Some(end) = lvx(input, i + 2) {
                    let expr = &input[i + 3..end];
                    let val = dov(expr);
                    result.push_str(&format!("{}", val));
                    i = end + 2; 
                    continue;
                }
            }

            
            if next == b'(' {
                if let Some(end) = lvy(input, i + 1) {
                    let cmd = &input[i + 2..end];
                    let output = kwa(cmd);
                    result.push_str(output.trim());
                    i = end + 1;
                    continue;
                }
            }

            
            if next == b'{' {
                if let Some(close) = input[i + 2..].find('}') {
                    let pre = &input[i + 2..i + 2 + close];
                    let value = lsn(pre);
                    result.push_str(&value);
                    i = i + 3 + close;
                    continue;
                }
            }

            
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                let name = &input[start..end];
                if let Some(val) = axh(name) {
                    result.push_str(&val);
                }
                
                i = end;
                continue;
            }

            
            result.push('$');
            i += 1;
        } else if bytes[i] == b'\\' && i + 1 < bytes.len() {
            
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


fn lsn(ye: &str) -> String {
    
    if ye.starts_with('#') {
        let name = &ye[1..];
        return format!("{}", axh(name).map(|v| v.len()).unwrap_or(0));
    }

    
    if let Some(pos) = ye.find(":-") {
        let name = &ye[..pos];
        let default = &ye[pos + 2..];
        return axh(name).unwrap_or_else(|| String::from(default));
    }

    
    if let Some(pos) = ye.find(":=") {
        let name = &ye[..pos];
        let default = &ye[pos + 2..];
        if let Some(val) = axh(name) {
            return val;
        }
        cql(name, default);
        return String::from(default);
    }

    
    if let Some(pos) = ye.find(":+") {
        let name = &ye[..pos];
        let adf = &ye[pos + 2..];
        return if axh(name).is_some() { String::from(adf) } else { String::new() };
    }

    
    axh(ye).unwrap_or_default()
}


fn kwa(cmd: &str) -> String {
    
    use core::sync::atomic::Ordering;
    super::DL_.store(true, Ordering::SeqCst);
    {
        let mut buf = super::IG_.lock();
        buf.clear();
    }

    super::aav(cmd);

    super::DL_.store(false, Ordering::SeqCst);
    let buf = super::IG_.lock();
    buf.clone()
}

fn lvy(j: &str, open_pos: usize) -> Option<usize> {
    let bytes = j.as_bytes();
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

fn lvx(j: &str, open_pos: usize) -> Option<usize> {
    let bytes = j.as_bytes();
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



pub fn dov(expr: &str) -> i64 {
    let expanded = bbm(expr);
    let tokens = pkv(&expanded);
    parse_expr(&tokens, &mut 0)
}

fn pkv(expr: &str) -> Vec<ArithToken> {
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
                let ae: i64 = expr[start..i].parse().unwrap_or(0);
                tokens.push(ArithToken::Num(ae));
            }
            _ => { i += 1; } 
        }
    }
    tokens
}

#[derive(Debug, Clone)]
enum ArithToken { Num(i64), Plus, Minus, Mul, Div, Mod, LParen, RParen }

fn parse_expr(tokens: &[ArithToken], pos: &mut usize) -> i64 {
    let mut left = cnv(tokens, pos);
    while *pos < tokens.len() {
        match &tokens[*pos] {
            ArithToken::Plus => { *pos += 1; left += cnv(tokens, pos); }
            ArithToken::Minus => { *pos += 1; left -= cnv(tokens, pos); }
            _ => break,
        }
    }
    left
}

fn cnv(tokens: &[ArithToken], pos: &mut usize) -> i64 {
    let mut left = bip(tokens, pos);
    while *pos < tokens.len() {
        match &tokens[*pos] {
            ArithToken::Mul => { *pos += 1; left *= bip(tokens, pos); }
            ArithToken::Div => {
                *pos += 1;
                let r = bip(tokens, pos);
                if r != 0 { left /= r; }
            }
            ArithToken::Mod => {
                *pos += 1;
                let r = bip(tokens, pos);
                if r != 0 { left %= r; }
            }
            _ => break,
        }
    }
    left
}

fn bip(tokens: &[ArithToken], pos: &mut usize) -> i64 {
    if *pos >= tokens.len() { return 0; }
    match &tokens[*pos] {
        ArithToken::Num(ae) => { let v = *ae; *pos += 1; v }
        ArithToken::Minus => { *pos += 1; -bip(tokens, pos) }
        ArithToken::Plus => { *pos += 1; bip(tokens, pos) }
        ArithToken::LParen => {
            *pos += 1;
            let v = parse_expr(tokens, pos);
            if *pos < tokens.len() { *pos += 1; } 
            v
        }
        _ => { *pos += 1; 0 }
    }
}



pub fn lsc(lines: &[&str], idx: &mut usize) -> bool {
    if *idx >= lines.len() { return false; }
    let first = lines[*idx].trim();
    if !first.starts_with("if ") { return false; }

    
    let mut akz: Vec<&str> = Vec::new();
    let mut depth = 0;
    let start = *idx;

    while *idx < lines.len() {
        let line = lines[*idx].trim();
        if line.starts_with("if ") { depth += 1; }
        if line == "fi" { depth -= 1; }
        akz.push(line);
        *idx += 1;
        if depth == 0 { break; }
    }

    
    let mut segments: Vec<(Option<&str>, Vec<&str>)> = Vec::new();
    let mut cwa: Option<&str> = None;
    let mut ejk: Vec<&str> = Vec::new();
    let mut eqq = 0;

    for &line in &akz {
        if eqq == 0 {
            if line.starts_with("if ") && segments.is_empty() {
                let fc = line.strip_prefix("if ").unwrap().trim();
                let fc = fc.strip_suffix("; then").or_else(|| fc.strip_suffix(" then")).unwrap_or(fc);
                cwa = Some(fc);
                eqq = 0;
                continue;
            }
            if line == "then" { continue; }
            if line.starts_with("elif ") {
                segments.push((cwa, core::mem::take(&mut ejk)));
                let fc = line.strip_prefix("elif ").unwrap().trim();
                let fc = fc.strip_suffix("; then").or_else(|| fc.strip_suffix(" then")).unwrap_or(fc);
                cwa = Some(fc);
                continue;
            }
            if line == "else" {
                segments.push((cwa, core::mem::take(&mut ejk)));
                cwa = None; 
                continue;
            }
            if line == "fi" {
                segments.push((cwa, core::mem::take(&mut ejk)));
                break;
            }
        }

        
        if line.starts_with("if ") { eqq += 1; }
        if line == "fi" { eqq -= 1; }
        ejk.push(line);
    }

    
    for (fc, body) in &segments {
        let orw = match fc {
            None => true, 
            Some(c) => hwp(c),
        };
        if orw {
            for &line in body {
                super::aav(line);
            }
            break;
        }
    }

    true
}


pub fn lsb(lines: &[&str], idx: &mut usize) -> bool {
    if *idx >= lines.len() { return false; }
    let first = lines[*idx].trim();
    if !first.starts_with("for ") { return false; }

    
    let mut akz: Vec<&str> = Vec::new();
    let mut depth = 0;
    while *idx < lines.len() {
        let line = lines[*idx].trim();
        if line.starts_with("for ") || line.starts_with("while ") { depth += 1; }
        if line == "done" { depth -= 1; }
        akz.push(line);
        *idx += 1;
        if depth == 0 { break; }
    }

    
    let header = akz[0];
    let fgl = header.strip_prefix("for ").unwrap().trim();

    
    let (edn, dtm) = if let Some(pos) = fgl.find(" in ") {
        (&fgl[..pos], &fgl[pos + 4..])
    } else {
        return true; 
    };

    let dtm = dtm.strip_suffix("; do")
        .or_else(|| dtm.strip_suffix(" do"))
        .unwrap_or(dtm);

    
    let lso = bbm(dtm);
    let items: Vec<&str> = lso.split_whitespace().collect();

    
    let mut body: Vec<&str> = Vec::new();
    let mut cla = false;
    for &line in &akz[1..] {
        if line == "do" { cla = true; continue; }
        if line == "done" { break; }
        if cla { body.push(line); }
        
        if !cla && akz[0].contains("; do") {
            cla = true;
            body.push(line);
        }
    }
    
    if akz[0].ends_with("; do") || akz[0].ends_with(" do") {
        body.clear();
        for &line in &akz[1..] {
            if line == "done" { break; }
            body.push(line);
        }
    }

    
    for item in &items {
        cql(edn, item);
        for &line in &body {
            let expanded = bbm(line);
            super::aav(&expanded);
        }
    }

    true
}


pub fn lsg(lines: &[&str], idx: &mut usize) -> bool {
    if *idx >= lines.len() { return false; }
    let first = lines[*idx].trim();
    if !first.starts_with("while ") { return false; }

    
    let mut akz: Vec<&str> = Vec::new();
    let mut depth = 0;
    while *idx < lines.len() {
        let line = lines[*idx].trim();
        if line.starts_with("for ") || line.starts_with("while ") { depth += 1; }
        if line == "done" { depth -= 1; }
        akz.push(line);
        *idx += 1;
        if depth == 0 { break; }
    }

    
    let header = akz[0];
    let fc = header.strip_prefix("while ").unwrap().trim();
    let fc = fc.strip_suffix("; do")
        .or_else(|| fc.strip_suffix(" do"))
        .unwrap_or(fc);

    
    let mut body: Vec<&str> = Vec::new();
    let lwi = akz[0].contains("; do") || akz[0].ends_with(" do");
    if lwi {
        for &line in &akz[1..] {
            if line == "done" { break; }
            body.push(line);
        }
    } else {
        let mut cla = false;
        for &line in &akz[1..] {
            if line == "do" { cla = true; continue; }
            if line == "done" { break; }
            if cla { body.push(line); }
        }
    }

    
    let mut xe = 0;
    while hwp(fc) && xe < 1000 {
        for &line in &body {
            let expanded = bbm(line);
            super::aav(&expanded);
        }
        xe += 1;
    }

    true
}








fn hwp(fc: &str) -> bool {
    let fc = fc.trim();

    
    if fc.starts_with("[ ") && fc.ends_with(" ]") {
        let inner = &fc[2..fc.len() - 2].trim();
        return fvk(inner);
    }

    
    if fc.starts_with("test ") {
        let inner = &fc[5..].trim();
        return fvk(inner);
    }

    
    if fc == "true" { return true; }
    if fc == "false" { return false; }

    
    
    let expanded = bbm(fc);
    super::DL_.store(true, core::sync::atomic::Ordering::SeqCst);
    { super::IG_.lock().clear(); }
    super::aav(&expanded);
    super::DL_.store(false, core::sync::atomic::Ordering::SeqCst);
    ibm() == 0
}


fn fvk(expr: &str) -> bool {
    let expanded = bbm(expr);
    let au: Vec<&str> = expanded.split_whitespace().collect();
    
    match au.as_slice() {
        
        ["-z", j] => j.is_empty(),
        ["-n", j] => !j.is_empty(),
        ["-f", path] => crate::ramfs::bh(|fs| {
            fs.stat(path).map(|e| e.file_type == crate::ramfs::FileType::File).unwrap_or(false)
        }),
        ["-d", path] => crate::ramfs::bh(|fs| {
            fs.stat(path).map(|e| e.file_type == crate::ramfs::FileType::Directory).unwrap_or(false)
        }),
        ["-e", path] => crate::ramfs::bh(|fs| fs.exists(path)),
        
        [a, "=", b] | [a, "==", b] => a == b,
        [a, "!=", b] => a != b,
        
        [a, "-eq", b] => a.parse::<i64>().ok() == b.parse::<i64>().ok(),
        [a, "-ne", b] => a.parse::<i64>().ok() != b.parse::<i64>().ok(),
        [a, "-lt", b] => a.parse::<i64>().unwrap_or(0) < b.parse::<i64>().unwrap_or(0),
        [a, "-gt", b] => a.parse::<i64>().unwrap_or(0) > b.parse::<i64>().unwrap_or(0),
        [a, "-le", b] => a.parse::<i64>().unwrap_or(0) <= b.parse::<i64>().unwrap_or(0),
        [a, "-ge", b] => a.parse::<i64>().unwrap_or(0) >= b.parse::<i64>().unwrap_or(0),
        
        ["!", ef @ ..] => !fvk(&ef.join(" ")),
        
        [j] => !j.is_empty(),
        _ => false,
    }
}


pub fn lse(script: &str) {
    let lines: Vec<&str> = script.lines().collect();
    let mut idx = 0;
    
    while idx < lines.len() {
        let line = lines[idx].trim();
        
        
        if line.is_empty() || line.starts_with('#') {
            idx += 1;
            continue;
        }

        
        if lsc(&lines, &mut idx) { continue; }
        if lsb(&lines, &mut idx) { continue; }
        if lsg(&lines, &mut idx) { continue; }

        
        let expanded = bbm(line);
        super::aav(&expanded);
        idx += 1;
    }
}
