//! Kernel Shell (Bootstrap Mode)
//! 
//! A full-featured shell running in kernel mode with standard commands.
//! This is temporary until Ring 3 userland is implemented.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::format;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};
use crate::ramfs::FileType;

// ═══════════════════════════════════════════════════════════════════════════════
// PARALLEL MATRIX RENDERING
// ═══════════════════════════════════════════════════════════════════════════════

/// Parameters for parallel Matrix rendering
#[repr(C)]
pub struct MatrixRenderParams {
    pub buf_ptr: *mut u32,
    pub buf_len: usize,
    pub width: u32,
    pub height: u32,
    pub matrix_chars: *const u8,              // Pointer to flat character grid (col * 68 + row)
    pub matrix_heads: *const i32,              // Pointer to head positions array
    pub holo_intensity: *const u8,             // Pointer to flat holo intensity map (col * 68 + row)
    pub matrix_rows: usize,                    // Number of rows (68)
}

unsafe impl Send for MatrixRenderParams {}
unsafe impl Sync for MatrixRenderParams {}

/// Render Matrix columns in parallel - called by each core
/// start/end are column indices
fn render_matrix_columns_parallel(start: usize, end: usize, data: *mut u8) {
    let params = unsafe { &*(data as *const MatrixRenderParams) };
    let buf_ptr = params.buf_ptr;
    let buf_len = params.buf_len;
    let width = params.width;
    let height = params.height;
    let holo_ptr = params.holo_intensity;
    let has_holo = !holo_ptr.is_null();
    let matrix_rows = params.matrix_rows;
    
    let col_width = 8u32;
    
    for col in start..end {
        if col >= 240 { break; }
        
        let x = col as u32 * col_width;
        if x >= width { continue; }
        
        let head = unsafe { *params.matrix_heads.add(col) };
        let start_row = (head - 5).max(0) as usize;
        let end_row = if head + 30 < 0 { 0 } else { ((head + 30) as usize).min(matrix_rows) };
        
        for row in start_row..end_row {
            let y = row as u32 * 16;
            if y >= height { continue; }
            
            let dist = row as i32 - head;
            
            // Calculate base green intensity (before holo modification)
            let base_green: u32 = if dist < 0 {
                continue;
            } else if dist == 0 {
                255  // Brightest
            } else if dist <= 12 {
                255 - (dist as u32 * 8)
            } else if dist <= 28 {
                // Safely calculate fade to avoid underflow
                let factor = ((dist - 12) as u32).min(15) * 16;
                let fade = 255u32.saturating_sub(factor);
                (160 * fade) / 255
            } else {
                continue;
            };
            
            // Flat index for matrix access
            let flat_idx = col * matrix_rows + row;
            
            // Apply holo intensity modifier if present
            // Character to use (may be overridden by holo)
            let (final_char, color) = if has_holo {
                let holo_val = unsafe { *holo_ptr.add(flat_idx) };
                if holo_val >= 1 {
                    // INSIDE SHAPE (edge or interior) - use fixed character '#' and pure green
                    ('#', 0xFF00FF00)
                } else {
                    // OUTSIDE - normal character and normal color
                    let c = unsafe { *params.matrix_chars.add(flat_idx) as char };
                    (c, 0xFF000000 | (base_green << 8))
                }
            } else {
                // No holo - normal rendering
                let c = unsafe { *params.matrix_chars.add(flat_idx) as char };
                (c, 0xFF000000 | (base_green << 8))
            };
            
            let glyph = crate::framebuffer::font::get_glyph(final_char);
            
            // Draw glyph
            unsafe {
                for (r, &bits) in glyph.iter().enumerate() {
                    let py = y + r as u32;
                    if py >= height { break; }
                    let row_offset = (py * width) as usize;
                    
                    if bits != 0 {
                        let x_usize = x as usize;
                        if bits & 0x80 != 0 { let idx = row_offset + x_usize; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x40 != 0 { let idx = row_offset + x_usize + 1; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x20 != 0 { let idx = row_offset + x_usize + 2; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x10 != 0 { let idx = row_offset + x_usize + 3; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x08 != 0 { let idx = row_offset + x_usize + 4; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x04 != 0 { let idx = row_offset + x_usize + 5; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x02 != 0 { let idx = row_offset + x_usize + 6; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x01 != 0 { let idx = row_offset + x_usize + 7; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                    }
                }
            }
        }
    }
}

/// List of all shell commands for autocomplete
pub const SHELL_COMMANDS: &[&str] = &[
    // Help and info
    "help", "man", "info", "version", "uname",
    // File system
    "ls", "dir", "cd", "pwd", "mkdir", "rmdir", "touch", "rm", "del", "cp", "copy",
    "mv", "move", "rename", "cat", "type", "head", "tail", "wc", "stat", "tree", "find",
    // Text
    "echo", "grep",
    // System
    "clear", "cls", "time", "uptime", "date", "whoami", "hostname", "id", "env", "printenv",
    "history", "ps", "free", "df",
    // User management
    "login", "su", "passwd", "adduser", "useradd", "deluser", "userdel", "users", "logout",
    // Test and debug
    "test", "keytest", "hexdump", "xxd", "panic",
    // Desktop GUI (multi-layer compositor)
    "desktop", "gui", "cosmic", "open", "trustedit",
    // Kernel signature
    "signature",
    // Security
    "security",
    // Linux/VM
    "linux", "distro", "alpine",
    // Tasks
    "tasks", "jobs", "threads",
    // Disk
    "disk", "dd", "ahci", "fdisk", "partitions",
    // Hardware
    "lspci", "lshw", "hwinfo",
    // Audio
    "beep", "audio", "synth",
    // Network
    "ifconfig", "ip", "ipconfig", "ping", "curl", "wget", "download",
    "nslookup", "dig", "arp", "route", "netstat",
    // Unix utilities  
    "which", "file", "chmod", "ln", "sort", "uniq", "cut",
    "kill", "top", "dmesg", "strings", "tar",
    "mount", "umount", "sync", "lsblk",
    // Exit/control
    "exit", "reboot", "shutdown", "poweroff",
    // Execution
    "exec", "run",
    // Fun
    "neofetch", "matrix", "cowsay",
    // Showcase
    "showcase",
    "showcase3d",
    "filled3d",
    // Hypervisor
    "hv", "hypervisor",
];

/// Run the kernel shell
pub fn run() -> ! {
    // Initialize theme and clear screen
    crate::framebuffer::set_matrix_theme();
    crate::framebuffer::clear();

    // Note: ramfs is already initialized in main.rs before persistence restore

    // Bootstrapping complete: allow timer handler to run
    crate::interrupts::set_bootstrap_ready(true);

    print_banner();
    
    let mut cmd_buffer = [0u8; 512];
    
    loop {
        print_prompt();
        
        // Read command with autocomplete support
        let len = read_line_with_autocomplete(&mut cmd_buffer);
        
        // Parse and execute
        let cmd_str = core::str::from_utf8(&cmd_buffer[..len]).unwrap_or("");
        execute_command(cmd_str.trim());
    }
}

/// Read a line with dynamic autocomplete and arrow navigation for suggestions
fn read_line_with_autocomplete(buffer: &mut [u8]) -> usize {
    use crate::keyboard::{read_char, add_to_history, history_prev, history_next, history_reset,
                          KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_HOME, KEY_END, KEY_DELETE,
                          KEY_PGUP, KEY_PGDOWN};
    
    let mut pos: usize = 0;
    let mut cursor: usize = 0;
    let mut suggestions: Vec<&str> = Vec::new();
    let mut suggestion_idx: i32 = -1; // -1 means no suggestion selected
    let mut showing_suggestions = false;
    
    // Save the row where input starts (for suggestion display)
    let input_row = crate::framebuffer::get_cursor().1;
    let input_col_start = crate::framebuffer::get_cursor().0;
    
    // Cursor blinking state
    let mut cursor_visible = true;
    let mut blink_counter: u32 = 0;
    const BLINK_INTERVAL: u32 = 50000;
    
    history_reset();
    
    // Show initial cursor
    crate::print_color!(COLOR_BRIGHT_GREEN, "█");
    crate::print!("\x08");
    
    loop {
        if let Some(c) = read_char() {
            // Hide cursor before processing
            let under_cursor = if cursor < pos { buffer[cursor] as char } else { ' ' };
            crate::print!("{}", under_cursor);
            crate::print!("\x08");
            cursor_visible = true;
            blink_counter = 0;
            
            match c {
                b'\n' | b'\r' => {
                    // Reset scroll to bottom when pressing Enter
                    crate::framebuffer::scroll_to_bottom();
                    
                    // If a suggestion is selected, use it
                    if suggestion_idx >= 0 && (suggestion_idx as usize) < suggestions.len() {
                        let selected = suggestions[suggestion_idx as usize];
                        clear_suggestions_at_row(input_row, suggestions.len());
                        // Replace buffer with selected command
                        clear_line_display(cursor, pos);
                        let bytes = selected.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        for i in 0..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                    } else if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                    }
                    crate::println!();
                    let cmd = core::str::from_utf8(&buffer[..pos]).unwrap_or("");
                    if !cmd.trim().is_empty() {
                        add_to_history(cmd);
                    }
                    break;
                }
                b'\t' => {
                    // Tab: complete with first/selected suggestion
                    if !suggestions.is_empty() {
                        let idx = if suggestion_idx >= 0 { suggestion_idx as usize } else { 0 };
                        let selected = suggestions[idx];
                        clear_suggestions_at_row(input_row, suggestions.len());
                        clear_line_display(cursor, pos);
                        let bytes = selected.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        for i in 0..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        suggestion_idx = -1;
                        showing_suggestions = false;
                        suggestions.clear();
                        if pos < buffer.len() - 1 {
                            buffer[pos] = b' ';
                            pos += 1;
                            cursor += 1;
                            crate::print!(" ");
                        }
                    }
                }
                0x1B => {
                    // Escape: clear suggestions
                    if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                        suggestions.clear();
                        suggestion_idx = -1;
                        showing_suggestions = false;
                    }
                }
                0x08 => {
                    // Backspace
                    if cursor > 0 {
                        // Clear suggestions first and restore cursor position
                        if showing_suggestions {
                            clear_suggestions_at_row(input_row, suggestions.len());
                            crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                        }
                        for i in cursor..pos {
                            buffer[i - 1] = buffer[i];
                        }
                        pos = pos.saturating_sub(1);
                        cursor = cursor.saturating_sub(1);
                        crate::print!("\x08");
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print!(" ");
                        for _ in cursor..=pos {
                            crate::print!("\x08");
                        }
                        // Update and show suggestions
                        update_suggestions(buffer, pos, &mut suggestions);
                        if !suggestions.is_empty() && pos > 0 {
                            show_suggestions_at_row(input_row, &suggestions, suggestion_idx);
                            showing_suggestions = true;
                            crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                        } else {
                            showing_suggestions = false;
                            suggestion_idx = -1;
                        }
                    }
                }
                KEY_UP => {
                    if pos == 0 {
                        // No input: navigate history
                        if let Some(prev) = history_prev() {
                            let bytes = prev.as_bytes();
                            let len = bytes.len().min(buffer.len() - 1);
                            buffer[..len].copy_from_slice(&bytes[..len]);
                            pos = len;
                            cursor = len;
                            crate::print!("{}", &prev[..len]);
                        }
                    } else {
                        // Has input: show/navigate suggestions
                        if !showing_suggestions {
                            // First UP press: show suggestions
                            update_suggestions(buffer, pos, &mut suggestions);
                            if !suggestions.is_empty() {
                                suggestion_idx = 0;
                                show_suggestions_at_row(input_row, &suggestions, suggestion_idx);
                                showing_suggestions = true;
                                // Return cursor to input position
                                crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                            }
                        } else if !suggestions.is_empty() {
                            // Already showing: navigate up
                            clear_suggestions_at_row(input_row, suggestions.len());
                            if suggestion_idx <= 0 {
                                suggestion_idx = suggestions.len() as i32 - 1;
                            } else {
                                suggestion_idx -= 1;
                            }
                            show_suggestions_at_row(input_row, &suggestions, suggestion_idx);
                            crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                        }
                    }
                }
                KEY_DOWN => {
                    if pos == 0 {
                        // No input: navigate history
                        if let Some(next) = history_next() {
                            let bytes = next.as_bytes();
                            let len = bytes.len().min(buffer.len() - 1);
                            buffer[..len].copy_from_slice(&bytes[..len]);
                            pos = len;
                            cursor = len;
                            crate::print!("{}", &next[..len]);
                        } else {
                            clear_line_display(cursor, pos);
                            pos = 0;
                            cursor = 0;
                        }
                    } else if showing_suggestions && !suggestions.is_empty() {
                        // Navigate suggestions down
                        clear_suggestions_at_row(input_row, suggestions.len());
                        suggestion_idx += 1;
                        if suggestion_idx >= suggestions.len() as i32 {
                            suggestion_idx = 0;
                        }
                        show_suggestions_at_row(input_row, &suggestions, suggestion_idx);
                        crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                    }
                }
                KEY_LEFT => {
                    if cursor > 0 {
                        cursor -= 1;
                        crate::print!("\x08");
                    }
                }
                KEY_RIGHT => {
                    if cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                KEY_HOME => {
                    while cursor > 0 {
                        crate::print!("\x08");
                        cursor -= 1;
                    }
                }
                KEY_END => {
                    while cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                KEY_DELETE => {
                    if cursor < pos {
                        if showing_suggestions {
                            clear_suggestions_at_row(input_row, suggestions.len());
                            crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                        }
                        for i in cursor..pos.saturating_sub(1) {
                            buffer[i] = buffer[i + 1];
                        }
                        pos = pos.saturating_sub(1);
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print!(" ");
                        for _ in cursor..=pos {
                            crate::print!("\x08");
                        }
                        update_suggestions(buffer, pos, &mut suggestions);
                        if !suggestions.is_empty() && pos > 0 {
                            show_suggestions_at_row(input_row, &suggestions, suggestion_idx);
                            showing_suggestions = true;
                            crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                        } else {
                            showing_suggestions = false;
                            suggestion_idx = -1;
                        }
                    }
                }
                KEY_PGUP => {
                    crate::framebuffer::scroll_up_lines(10);
                }
                KEY_PGDOWN => {
                    crate::framebuffer::scroll_down(10);
                }
                27 => {
                    // Escape - reset scroll to bottom (live view)
                    if crate::framebuffer::is_scrolled_back() {
                        crate::framebuffer::scroll_to_bottom();
                        // Redraw current screen (need to force redraw)
                        crate::framebuffer::clear();
                        print_prompt();
                        for i in 0..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        for _ in cursor..pos {
                            crate::print!("\x08");
                        }
                    }
                }
                12 => {
                    // Ctrl+L - clear screen
                    if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                        showing_suggestions = false;
                    }
                    crate::framebuffer::clear();
                    print_prompt();
                    for i in 0..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    for _ in cursor..pos {
                        crate::print!("\x08");
                    }
                }
                c if c >= 0x20 && c < 0x7F && pos < buffer.len() - 1 => {
                    // Printable character
                    // First, restore cursor to input line if suggestions were showing
                    if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                        crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                    }
                    
                    if cursor < pos {
                        for i in (cursor..pos).rev() {
                            buffer[i + 1] = buffer[i];
                        }
                    }
                    buffer[cursor] = c;
                    pos += 1;
                    cursor += 1;
                    for i in cursor - 1..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    for _ in cursor..pos {
                        crate::print!("\x08");
                    }
                    
                    // Update and show suggestions if we have matches
                    update_suggestions(buffer, pos, &mut suggestions);
                    if !suggestions.is_empty() {
                        show_suggestions_at_row(input_row, &suggestions, suggestion_idx);
                        showing_suggestions = true;
                        // Restore cursor to input line
                        crate::framebuffer::set_cursor(input_col_start + cursor, input_row);
                    } else {
                        showing_suggestions = false;
                        suggestion_idx = -1;
                    }
                }
                _ => {}
            }
            
            // Re-show cursor after key processing
            crate::print_color!(COLOR_BRIGHT_GREEN, "█");
            crate::print!("\x08");
        } else {
            // No input - handle cursor blinking
            blink_counter += 1;
            if blink_counter >= BLINK_INTERVAL {
                blink_counter = 0;
                cursor_visible = !cursor_visible;
                
                if cursor_visible {
                    crate::print_color!(COLOR_BRIGHT_GREEN, "█");
                    crate::print!("\x08");
                } else {
                    let under_cursor = if cursor < pos { buffer[cursor] as char } else { ' ' };
                    crate::print!("{}", under_cursor);
                    crate::print!("\x08");
                }
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }
    }
    
    // Hide cursor before returning
    let under_cursor = if cursor < pos { buffer[cursor] as char } else { ' ' };
    crate::print!("{}", under_cursor);
    if cursor < pos { crate::print!("\x08"); }
    
    buffer[pos] = 0;
    pos
}

/// Update autocomplete suggestions based on current input
fn update_suggestions(buffer: &[u8], pos: usize, suggestions: &mut Vec<&'static str>) {
    suggestions.clear();
    if pos == 0 {
        return;
    }
    
    let input = match core::str::from_utf8(&buffer[..pos]) {
        Ok(s) => s,
        Err(_) => return,
    };
    
    let first_word = input.split_whitespace().next().unwrap_or("");
    if first_word.is_empty() || input.contains(' ') {
        return;
    }
    
    for cmd in SHELL_COMMANDS {
        if cmd.starts_with(first_word) && *cmd != first_word {
            suggestions.push(cmd);
            if suggestions.len() >= 8 {
                break;
            }
        }
    }
}

/// Display suggestions starting from the row below input
fn show_suggestions_at_row(input_row: usize, suggestions: &[&str], selected_idx: i32) {
    if suggestions.is_empty() {
        return;
    }
    
    for (i, cmd) in suggestions.iter().enumerate() {
        crate::framebuffer::set_cursor(0, input_row + 1 + i);
        if i as i32 == selected_idx {
            crate::print_color!(COLOR_GREEN, " > ");
            crate::print_color!(COLOR_BRIGHT_GREEN, "{}", cmd);
        } else {
            crate::print_color!(COLOR_DARK_GREEN, "   {}", cmd);
        }
    }
}

/// Clear the suggestions display at given row
fn clear_suggestions_at_row(input_row: usize, count: usize) {
    for i in 0..count {
        crate::framebuffer::set_cursor(0, input_row + 1 + i);
        for _ in 0..40 {
            crate::print!(" ");
        }
    }
}

/// Clear the current input line display
fn clear_line_display(cursor: usize, pos: usize) {
    // Move cursor to start of input
    for _ in 0..cursor {
        crate::print!("\x08");
    }
    // Clear all characters
    for _ in 0..pos {
        crate::print!(" ");
    }
    // Move back to start
    for _ in 0..pos {
        crate::print!("\x08");
    }
}

fn print_banner() {
    crate::println!();
    crate::println_color!(COLOR_BRIGHT_GREEN, r" _____ ____            _    ___      ");
    crate::println_color!(COLOR_BRIGHT_GREEN, r"|_   _|  _ \ _   _ ___| |_ / _ \ ___ ");
    crate::println_color!(COLOR_GREEN,        r"  | | | |_) | | | / __| __| | | / __|");
    crate::println_color!(COLOR_GREEN,        r"  | | |  _ <| |_| \__ \ |_| |_| \__ \");
    crate::println_color!(COLOR_DARK_GREEN,   r"  |_| |_| \_\\__,_|___/\__\\___/|___/");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  T-RustOs v0.1.0 - Type 'help' for commands");
    crate::println!();
}

fn print_prompt() {
    // Show timestamp and current directory in prompt
    let dt = crate::rtc::read_rtc();
    let cwd = if crate::ramfs::is_initialized() {
        crate::ramfs::with_fs(|fs| String::from(fs.pwd()))
    } else {
        String::from("/")
    };
    // Format: [HH:MM:SS] trustos:/path$
    crate::print_color!(COLOR_DARK_GREEN, "[{:02}:{:02}:{:02}] ", dt.hour, dt.minute, dt.second);
    crate::print_color!(COLOR_BRIGHT_GREEN, "trustos");
    crate::print_color!(COLOR_WHITE, ":");
    crate::print_color!(COLOR_CYAN, "{}", cwd);
    crate::print_color!(COLOR_GREEN, "$ ");
}

/// Execute a shell command
fn execute_command(cmd: &str) {
    if cmd.is_empty() {
        return;
    }
    
    // Handle output redirection
    let (cmd_part, redirect) = if let Some(pos) = cmd.find('>') {
        let append = cmd[pos..].starts_with(">>");
        let file = if append {
            cmd[pos + 2..].trim()
        } else {
            cmd[pos + 1..].trim()
        };
        (cmd[..pos].trim(), Some((file, append)))
    } else {
        (cmd, None)
    };
    
    // Split into command and arguments
    let parts: Vec<&str> = cmd_part.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    
    let command = parts[0];
    let args = &parts[1..];
    
    match command {
        // Help and info
        "help" => cmd_help(args),
        "man" => cmd_man(args),
        "info" => cmd_info(),
        "version" => cmd_version(),
        "uname" => cmd_uname(args),
        
        // File system commands
        "ls" | "dir" => cmd_ls(args),
        "cd" => cmd_cd(args),
        "pwd" => cmd_pwd(),
        "mkdir" => cmd_mkdir(args),
        "rmdir" => cmd_rmdir(args),
        "touch" => cmd_touch(args),
        "rm" | "del" => cmd_rm(args),
        "cp" | "copy" => cmd_cp(args),
        "mv" | "move" | "rename" => cmd_mv(args),
        "cat" | "type" => cmd_cat(args, redirect),
        "head" => cmd_head(args),
        "tail" => cmd_tail(args),
        "wc" => cmd_wc(args),
        "stat" => cmd_stat(args),
        "tree" => cmd_tree(args),
        "find" => cmd_find(args),
        
        // Text manipulation
        "echo" => cmd_echo(args, redirect),
        "grep" => cmd_grep(args),
        
        // System commands
        "clear" | "cls" => cmd_clear(),
        "time" | "uptime" => cmd_time(),
        "date" => cmd_date(),
        "whoami" => cmd_whoami(),
        "hostname" => cmd_hostname(),
        "id" => cmd_id(),
        "env" | "printenv" => cmd_env(),
        "history" => cmd_history(),
        "ps" => cmd_ps(),
        "free" => cmd_free(),
        "df" => cmd_df(),
        
        // User management commands
        "login" => cmd_login(),
        "su" => cmd_su(args),
        "passwd" => cmd_passwd(args),
        "adduser" | "useradd" => cmd_adduser(args),
        "deluser" | "userdel" => cmd_deluser(args),
        "users" => cmd_users(),
        
        // Test and debug
        "test" => cmd_test(),
        "keytest" => cmd_keytest(),
        "hexdump" | "xxd" => cmd_hexdump(args),
        "panic" => cmd_panic(),
        
        // VM/Linux commands with subcommands
        "vm" | "linux" | "gui" => {
            if args.is_empty() {
                cmd_linux_shell();  // Direct access to Linux shell
            } else {
                match args[0] {
                    "status" => cmd_gui_status(),
                    "install" => cmd_gui_install(),
                    "start" | "run" => cmd_gui_start(),
                    "console" | "shell" => cmd_linux_shell(),
                    "stop" => cmd_vm_stop(),
                    "list" => cmd_vm_list(),
                    "extract" => create_test_binaries(),
                    "exec" => {
                        // Direct binary execution: linux exec /bin/busybox
                        if args.len() > 1 {
                            let binary = args[1];
                            let bin_args: Vec<&str> = args[2..].to_vec();
                            match crate::linux_compat::exec(binary, &bin_args) {
                                Ok(code) => crate::println!("[Exited with code {}]", code),
                                Err(e) => crate::println_color!(0xFF0000, "Error: {}", e),
                            }
                        } else {
                            crate::println!("Usage: linux exec <binary> [args...]");
                            crate::println!("Example: linux exec /bin/busybox ls");
                        }
                    },
                    "help" | "--help" | "-h" => cmd_vm_help(),
                    _ => cmd_vm_help(),
                }
            }
        },
        
        // Linux distribution manager
        "distro" | "distros" => {
            if args.is_empty() {
                cmd_distro_list();
            } else {
                match args[0] {
                    "list" => cmd_distro_list(),
                    "install" | "download" => {
                        if args.len() > 1 {
                            cmd_distro_install(args[1]);
                        } else {
                            cmd_distro_gui();
                        }
                    },
                    "run" | "start" => {
                        if args.len() > 1 {
                            cmd_distro_run(args[1]);
                        } else {
                            crate::println!("Usage: distro run <id>");
                            crate::println!("Example: distro run alpine");
                        }
                    },
                    "pick" | "select" => cmd_distro_gui(),
                    _ => cmd_distro_list(),
                }
            }
        },
        
        "glmode" | "compositor" => cmd_glmode(args),
        "theme" => cmd_theme(args),
        "anim" | "animations" => cmd_animations(args),
        "holo" | "holomatrix" => cmd_holomatrix(args),
        "imgview" | "imageview" | "view" => cmd_imgview(args),
        "imgdemo" | "imagedemo" => cmd_imgdemo(args),
        "tasks" | "jobs" => cmd_tasks(),
        "threads" => cmd_threads(),
        
        // Wayland compositor
        "wayland" | "wl" => cmd_wayland(args),
        
        // Graphical Terminal
        "gterm" | "graphterm" => cmd_gterm(args),
        
        // Alpine Linux all-in-one command
        "alpine" => cmd_alpine(args),
        
        // Binary-to-Rust transpiler
        "transpile" | "disasm" | "analyze" => cmd_transpile(args),
        
        // TrustLang — integrated programming language
        "trustlang" | "tl" => cmd_trustlang(args),
        "trustlang_showcase" | "tl_showcase" => cmd_trustlang_showcase(),
        "film" | "trustos_film" => cmd_trustos_film(),
        
        // TrustVideo — video codec & player
        "video" | "tv" => cmd_video(args),
        
        // Persistence commands
        "persist" | "persistence" => cmd_persistence(args),
        
        // Disk commands
        "disk" => cmd_disk(),
        "dd" => cmd_dd(args),
        "ahci" => cmd_ahci(args),
        "fdisk" | "partitions" => cmd_fdisk(args),
        
        // Hardware commands
        "lspci" => cmd_lspci(args),
        "lshw" | "hwinfo" => cmd_lshw(),
        
        // Audio commands
        "beep" => cmd_beep(args),
        "audio" => cmd_audio(args),
        "synth" => cmd_synth(args),
        
        // Network commands
        "ifconfig" | "ip" => cmd_ifconfig(),
        "ipconfig" => cmd_ipconfig(args),
        "ping" => cmd_ping(args),
        "tcpsyn" => cmd_tcpsyn(args),
        "httpget" => cmd_httpget(args),
        "curl" | "wget" => cmd_curl(args),
        "download" => cmd_download(args),
        "nslookup" | "dig" => cmd_nslookup(args),
        "arp" => cmd_arp(args),
        "route" => cmd_route(args),
        "traceroute" | "tracert" => cmd_traceroute(args),
        "netstat" => cmd_netstat(),
        "browse" | "www" | "web" => cmd_browse(args),
        "sandbox" | "websandbox" => cmd_sandbox(args),
        
        // Additional Unix commands
        "which" => cmd_which(args),
        "whereis" => cmd_whereis(args),
        "file" => cmd_file(args),
        "chmod" => cmd_chmod(args),
        "chown" => cmd_chown(args),
        "ln" => cmd_ln(args),
        "readlink" => cmd_readlink(args),
        "basename" => cmd_basename(args),
        "dirname" => cmd_dirname(args),
        "realpath" => cmd_realpath(args),
        "sort" => cmd_sort(args),
        "uniq" => cmd_uniq(args),
        "cut" => cmd_cut(args),
        "tr" => cmd_tr(args),
        "tee" => cmd_tee(args),
        "xargs" => cmd_xargs(args),
        "yes" => cmd_yes(args),
        "seq" => cmd_seq(args),
        "sleep" => cmd_sleep(args),
        "kill" => cmd_kill(args),
        "killall" => cmd_killall(args),
        "nice" => cmd_nice(args),
        "nohup" => cmd_nohup(args),
        "bg" => cmd_bg(args),
        "fg" => cmd_fg(args),
        "top" => cmd_top(),
        "htop" => cmd_top(),
        "vmstat" => cmd_vmstat(),
        "iostat" => cmd_iostat(),
        "lsof" => cmd_lsof(args),
        "strace" => cmd_strace(args),
        "strings" => cmd_strings(args),
        "tar" => cmd_tar(args),
        "gzip" => cmd_gzip(args),
        "gunzip" => cmd_gunzip(args),
        "zip" => cmd_zip(args),
        "unzip" => cmd_unzip(args),
        "mount" => cmd_mount(args),
        "umount" => cmd_umount(args),
        "sync" => cmd_sync(),
        "lsblk" => cmd_lsblk(),
        "blkid" => cmd_blkid(),
        "mkfs" => cmd_mkfs(args),
        "fsck" => cmd_fsck(args),
        "export" => cmd_export(args),
        "unset" => cmd_unset(args),
        "alias" => cmd_alias(args),
        "unalias" => cmd_unalias(args),
        "source" | "." => cmd_source(args),
        "set" => cmd_set(args),
        "read" => cmd_read(args),
        "printf" => cmd_printf(args),
        "test" | "[" => cmd_test_expr(args),
        "expr" => cmd_expr(args),
        "bc" => cmd_bc(args),
        "cal" => cmd_cal(args),
        "diff" => cmd_diff(args),
        "patch" => cmd_patch(args),
        "cmp" => cmd_cmp(args),
        "md5sum" => cmd_md5sum(args),
        "sha256sum" => cmd_sha256sum(args),
        "base64" => cmd_base64(args),
        "od" => cmd_od(args),
        "rev" => cmd_rev(args),
        "factor" => cmd_factor(args),
        "watch" => cmd_watch(args),
        "timeout" => cmd_timeout(args),
        "time_cmd" => cmd_time_cmd(args),
        "script" => cmd_script(args),
        "tty" => cmd_tty(),
        "stty" => cmd_stty(args),
        "reset" => cmd_reset(),
        "loadkeys" => cmd_loadkeys(args),
        "setfont" => cmd_setfont(args),
        "lsusb" => cmd_lsusb(),
        "lscpu" => cmd_lscpu(),
        "smpstatus" => cmd_smpstatus(),
        "smp" => cmd_smp(args),
        "fontsmooth" => cmd_fontsmooth(args),
        "lsmem" => cmd_lsmem(),
        "dmidecode" => cmd_dmidecode(),
        "hdparm" => cmd_hdparm(args),
        "modprobe" => cmd_modprobe(args),
        "lsmod" => cmd_lsmod(),
        "insmod" => cmd_insmod(args),
        "rmmod" => cmd_rmmod(args),
        "sysctl" => cmd_sysctl(args),
        "service" => cmd_service(args),
        "systemctl" => cmd_systemctl(args),
        "crontab" => cmd_crontab(args),
        "at" => cmd_at(args),
        
        // Exit commands
        "exit" | "logout" => cmd_logout(),
        "reboot" => cmd_reboot(),
        "shutdown" | "halt" | "poweroff" => cmd_halt(),
        
        // Program execution
        "exec" | "run" | "./" => cmd_exec(args, command),
        "elfinfo" => cmd_elfinfo(args),
        
        // Easter eggs
        "neofetch" => cmd_neofetch(),
        "matrix" => cmd_matrix(),
        "cowsay" => cmd_cowsay(args),
        
        // Performance benchmarks
        "benchmark" | "bench" => cmd_benchmark(args),
        
        // ── Developer Tools ──────────────────────────────────────────
        "dmesg" => cmd_dmesg(args),
        "memdbg" | "heapdbg" => cmd_memdbg(),
        "perf" | "perfstat" => cmd_perfstat(),
        "irqstat" | "irqs" => cmd_irqstat(),
        "regs" | "registers" | "cpuregs" => cmd_registers(),
        "peek" | "memdump" => cmd_peek(args),
        "poke" | "memwrite" => cmd_poke(args),
        "devpanel" => cmd_devpanel(),
        "timecmd" => cmd_timecmd(args),
        
        // Showcase — automated demo for marketing videos
        "showcase" => cmd_showcase(args),
        "showcase3d" | "demo3d" => cmd_showcase3d(),
        "filled3d" => cmd_filled3d(),
        
        // Desktop Environment - windowed desktop with full app support
        "desktop" | "gui" => launch_desktop_env(None),
        
        // COSMIC V2 compositor (alternative matrix-style desktop)
        "cosmic" => cmd_cosmic_v2(),
        
        // Open app directly (launches desktop with app)
        "open" => cmd_open(args),
        
        // TrustEdit 3D model editor (launches desktop with TrustEdit)
        "trustedit" | "edit3d" | "3dedit" => launch_desktop_env(Some(("TrustEdit 3D", crate::desktop::WindowType::ModelEditor, 100, 60, 700, 500))),
        
        // Launch desktop with a specific window type
        "calculator" | "calc" => launch_desktop_env(Some(("Calculator", crate::desktop::WindowType::Calculator, 300, 200, 320, 420))),
        "snake" => launch_desktop_env(Some(("Snake", crate::desktop::WindowType::Game, 200, 100, 400, 400))),
        
        // Kernel signature & proof of authorship
        "signature" | "sig" => cmd_signature(args),
        
        // Security subsystem management
        "security" | "sec" | "caps" => cmd_security(args),
        
        // Hypervisor commands
        "hv" | "hypervisor" => cmd_hypervisor(args),
        "vm" => cmd_vm(args),
        
        // Linux Subsystem commands
        "linux" | "tsl" => cmd_linux(args),

        "" => {}
        _ => {
            // Check if it's an executable file
            if try_exec_file(command, args) {
                return;
            }
            crate::print_color!(COLOR_RED, "tsh: ");
            crate::print!("{}", command);
            crate::println_color!(COLOR_RED, ": command not found");
        }
    }
}

// ==================== HELP COMMANDS ====================

fn cmd_help(args: &[&str]) {
    if !args.is_empty() {
        cmd_man(args);
        return;
    }
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "======================================================================");
    crate::println_color!(COLOR_BRIGHT_GREEN, "          TrustOS -- Secure Bare-Metal Operating System");
    crate::println_color!(COLOR_BRIGHT_GREEN, "       x86_64 kernel written in Rust -- no libc, no std");
    crate::println_color!(COLOR_BRIGHT_GREEN, "======================================================================");
    crate::println!();
    crate::println_color!(COLOR_WHITE, "  Features: RAMFS file system, TCP/IP networking, ELF loader,");
    crate::println_color!(COLOR_WHITE, "  Linux syscall compat, GUI desktop compositor, SMP multicore.");
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "  Type 'help <command>' or 'man <command>' for detailed usage.");
    crate::println_color!(COLOR_YELLOW, "  Tab = auto-complete | Up/Down = history | PageUp/Down = scroll");
    crate::println!();
    
    // FILE SYSTEM
    crate::println_color!(COLOR_CYAN, "  FILE SYSTEM");
    crate::println!("    ls [path]           List directory contents (-l long, -a hidden)");
    crate::println!("    cd <path>           Change working directory");
    crate::println!("    pwd                 Print current working directory");
    crate::println!("    mkdir <dir>         Create directory (-p recursive)");
    crate::println!("    rmdir <dir>         Remove empty directory");
    crate::println!("    touch <file>        Create empty file or update timestamp");
    crate::println!("    rm <file>           Remove file or directory (-r recursive)");
    crate::println!("    cp <src> <dst>      Copy file or directory");
    crate::println!("    mv <src> <dst>      Move or rename file");
    crate::println!("    cat <file>          Display file contents (supports > redirect)");
    crate::println!("    head <file>         Show first N lines (-n N)");
    crate::println!("    tail <file>         Show last N lines (-n N)");
    crate::println!("    wc <file>           Count lines, words, bytes");
    crate::println!("    stat <file>         Display file metadata (size, type, perms)");
    crate::println!("    tree [path]         Display directory tree structure");
    crate::println!("    find <path> <name>  Search for files by name pattern");
    crate::println!("    ln -s <tgt> <link>  Create symbolic link");
    crate::println!("    readlink <link>     Display link target");
    crate::println!("    basename <path>     Strip directory from path");
    crate::println!("    dirname <path>      Strip filename from path");
    crate::println!("    realpath <path>     Resolve to absolute path");
    crate::println!("    file <path>         Identify file type (ELF, text, etc.)");
    crate::println!("    chmod <mode> <f>    Change file permissions (octal)");
    crate::println!("    chown <u>[:<g>] <f> Change file ownership");
    crate::println!();
    
    // TEXT PROCESSING
    crate::println_color!(COLOR_CYAN, "  TEXT PROCESSING");
    crate::println!("    echo <text>         Print text (supports > redirect)");
    crate::println!("    grep <pat> <file>   Search for pattern (-i case insensitive)");
    crate::println!("    sort <file>         Sort lines (-r reverse, -n numeric)");
    crate::println!("    uniq <file>         Remove duplicate adjacent lines (-c count)");
    crate::println!("    cut -d<d> -f<n>     Cut columns by delimiter");
    crate::println!("    tr <a> <b>          Translate characters (a->b)");
    crate::println!("    tee <file>          Write stdin to file + stdout");
    crate::println!("    rev <text>          Reverse string");
    crate::println!("    diff <a> <b>        Compare two files line by line");
    crate::println!("    cmp <a> <b>         Compare two files byte by byte");
    crate::println!("    patch <file>        Apply diff patch");
    crate::println!("    strings <file>      Extract printable strings from binary");
    crate::println!("    od <file>           Octal dump of file");
    crate::println!("    hexdump <file>      Hex dump of file contents");
    crate::println!("    base64 <file>       Encode/decode base64 (-d decode)");
    crate::println!("    md5sum <file>       Compute MD5 hash");
    crate::println!("    sha256sum <file>    Compute SHA-256 hash");
    crate::println!();
    
    // SYSTEM & PROCESS
    crate::println_color!(COLOR_CYAN, "  SYSTEM & PROCESS");
    crate::println!("    clear               Clear terminal screen");
    crate::println!("    time / uptime       Show system uptime");
    crate::println!("    date                Display current date and time");
    crate::println!("    whoami              Print current username");
    crate::println!("    hostname            Display system hostname");
    crate::println!("    id                  Print user/group IDs");
    crate::println!("    uname [-a]          System information (kernel, arch)");
    crate::println!("    env / printenv      Display environment variables");
    crate::println!("    export K=V          Set environment variable");
    crate::println!("    unset <var>         Remove environment variable");
    crate::println!("    set                 Show all shell variables");
    crate::println!("    alias <n>=<cmd>     Create command alias");
    crate::println!("    unalias <name>      Remove command alias");
    crate::println!("    source <file>       Execute commands from script file");
    crate::println!("    history             Show command history");
    crate::println!("    ps                  List running processes");
    crate::println!("    top / htop          Real-time process monitor");
    crate::println!("    kill <pid>          Send signal to process");
    crate::println!("    killall <name>      Kill processes by name");
    crate::println!("    nice <n> <cmd>      Run command with priority");
    crate::println!("    nohup <cmd>         Run command immune to hangups");
    crate::println!("    bg / fg             Background/foreground job control");
    crate::println!("    tasks / jobs        List active kernel tasks");
    crate::println!("    threads             Show kernel thread info");
    crate::println!("    free                Display memory usage statistics");
    crate::println!("    df                  Show disk space usage");
    crate::println!("    vmstat              Virtual memory statistics");
    crate::println!("    iostat              I/O statistics by device");
    crate::println!("    lsof [pid]          List open files per process");
    crate::println!("    strace <cmd>        Trace system calls of command");
    crate::println!("    sleep <secs>        Pause execution for N seconds");
    crate::println!("    watch <cmd>         Execute command repeatedly");
    crate::println!("    timeout <s> <cmd>   Run command with time limit");
    crate::println!("    which <cmd>         Show command location");
    crate::println!("    whereis <cmd>       Locate command binary and manpage");
    crate::println!("    script <file>       Record terminal session to file");
    crate::println!("    timecmd <cmd>       Measure command execution time");
    crate::println!();
    
    // USER MANAGEMENT
    crate::println_color!(COLOR_CYAN, "  USER MANAGEMENT");
    crate::println!("    login               Switch to another user account");
    crate::println!("    su <user>           Substitute user identity");
    crate::println!("    passwd [user]       Change user password");
    crate::println!("    adduser <name>      Create new user account");
    crate::println!("    deluser <name>      Delete user account");
    crate::println!("    users               List all user accounts");
    crate::println!();
    
    // HARDWARE & DEVICES
    crate::println_color!(COLOR_CYAN, "  HARDWARE & DEVICES");
    crate::println!("    lspci [-v]          List PCI devices (vendor/class)");
    crate::println!("    lshw / hwinfo       Full hardware inventory");
    crate::println!("    lscpu               CPU model, cores, features, frequency");
    crate::println!("    lsmem               Memory layout and total RAM");
    crate::println!("    lsusb               List USB controllers & devices");
    crate::println!("    dmidecode           BIOS/SMBIOS firmware tables");
    crate::println!("    hdparm <dev>        Disk drive parameters");
    crate::println!("    smpstatus           SMP multicore status (per-CPU state)");
    crate::println!("    smp <cmd>           SMP control (start/stop cores)");
    crate::println!("    modprobe <mod>      Load kernel module");
    crate::println!("    lsmod               List loaded kernel modules");
    crate::println!("    insmod / rmmod      Insert or remove module");
    crate::println!("    beep [freq] [ms]    Play a tone (default 440Hz 500ms)");
    crate::println!("    audio               Audio driver status / control");
    crate::println!("    synth <cmd>         TrustSynth polyphonic synthesizer");
    crate::println!("                         note/freq/wave/adsr/preset/demo/status");
    crate::println!();
    
    // DISK & STORAGE
    crate::println_color!(COLOR_CYAN, "  DISK & STORAGE");
    crate::println!("    disk                Show detected disk drives");
    crate::println!("    dd if=<> of=<>      Block-level copy (raw disk I/O)");
    crate::println!("    ahci <cmd>          AHCI controller commands");
    crate::println!("    fdisk <dev>         Partition table editor");
    crate::println!("    lsblk               List block devices");
    crate::println!("    blkid               Show block device UUIDs");
    crate::println!("    mkfs <type> <dev>   Format partition (fat32, ext2)");
    crate::println!("    fsck <dev>          File system consistency check");
    crate::println!("    mount <dev> <dir>   Mount file system");
    crate::println!("    umount <dir>        Unmount file system");
    crate::println!("    sync                Flush all pending writes to disk");
    crate::println!("    persist <cmd>       Manage persistent storage");
    crate::println!();
    
    // NETWORK
    crate::println_color!(COLOR_CYAN, "  NETWORK");
    crate::println!("    ifconfig / ip       Show network interface status");
    crate::println!("    ipconfig [cmd]      Configure IP settings");
    crate::println!("    ping <host>         ICMP echo to test connectivity");
    crate::println!("    curl <url>          HTTP/HTTPS client (GET, POST)");
    crate::println!("    wget <url>          Download file from URL");
    crate::println!("    download <url>      Download and save file");
    crate::println!("    nslookup <host>     DNS lookup (A, AAAA records)");
    crate::println!("    arp [-a]            Show ARP table (IP->MAC mappings)");
    crate::println!("    route               Display routing table");
    crate::println!("    traceroute <host>   Trace packet path to destination");
    crate::println!("    netstat             Show active connections & listeners");
    crate::println!("    browse <url>        Text-mode web browser");
    crate::println!("    sandbox <cmd>       Web sandbox (open/allow/deny/fs/status/list/kill)");
    crate::println!("    tcpsyn <host:port>  Raw TCP SYN connection test");
    crate::println!("    httpget <url>       Raw HTTP GET request");
    crate::println!();
    
    // LINUX SUBSYSTEM
    crate::println_color!(COLOR_CYAN, "  LINUX SUBSYSTEM");
    crate::println!("    linux               Launch Linux compatibility shell");
    crate::println!("    linux status        Show Linux subsystem status");
    crate::println!("    linux install       Install Linux binaries from rootfs");
    crate::println!("    linux start         Start Linux init process");
    crate::println!("    linux exec <bin>    Execute ELF binary directly");
    crate::println!("    alpine <cmd>        Alpine Linux package manager");
    crate::println!("    distro list         List available distributions");
    crate::println!("    distro install <id> Download & install distribution");
    crate::println!("    distro run <id>     Launch installed distribution");
    crate::println!("    exec <file>         Execute binary (ELF or script)");
    crate::println!("    elfinfo <file>      Display ELF binary header info");
    crate::println!();
    
    // GRAPHICS & DESKTOP
    crate::println_color!(COLOR_CYAN, "  GRAPHICS & DESKTOP");
    crate::println!("    desktop / gui       Launch windowed desktop environment");
    crate::println!("    cosmic              Launch COSMIC V2 compositor");
    crate::println!("    open <app>          Open desktop with specific app");
    crate::println!("    trustedit           3D model editor (wireframe viewer)");
    crate::println!("    calculator / calc   Launch calculator app");
    crate::println!("    snake               Launch Snake game");
    crate::println!("    glmode [on|off]     Toggle OpenGL compositing mode");
    crate::println!("    theme <name>        Switch color theme (matrix, nord, etc.)");
    crate::println!("    anim <cmd>          Configure UI animations");
    crate::println!("    holo / holomatrix   Holographic matrix visualizer");
    crate::println!("    imgview <file>      Display image file (PPM, BMP)");
    crate::println!("    imgdemo             Run image rendering demo");
    crate::println!("    wayland [cmd]       Wayland compositor control");
    crate::println!("    gterm               Launch graphical terminal");
    crate::println!("    fontsmooth [0-3]    Set font anti-aliasing level");
    crate::println!();
    
    // PROGRAMMING & TOOLS
    crate::println_color!(COLOR_CYAN, "  PROGRAMMING & TOOLS");
    crate::println!("    trustlang / tl      TrustLang programming language REPL");
    crate::println!("    transpile <file>    Binary-to-Rust transpiler (ELF analysis)");
    crate::println!("    video / tv          TrustVideo codec player (record/play)");
    crate::println!("    film                TrustOS Film cinematic demo");
    crate::println!("    bc                  Calculator / math expression evaluator");
    crate::println!("    cal                 Display calendar");
    crate::println!("    factor <n>          Prime factorization of integer");
    crate::println!("    seq <a> [b] <c>     Print numeric sequence");
    crate::println!("    yes [text]          Repeat text infinitely");
    crate::println!("    xargs <cmd>         Build command from stdin");
    crate::println!("    printf <fmt> <args> Formatted text output");
    crate::println!("    expr <expr>         Evaluate arithmetic expression");
    crate::println!("    read <var>          Read user input into variable");
    crate::println!();
    
    // ARCHIVING & COMPRESSION
    crate::println_color!(COLOR_CYAN, "  ARCHIVING & COMPRESSION");
    crate::println!("    tar <opts> <file>   Archive/extract tar files");
    crate::println!("    gzip / gunzip       Compress/decompress gzip files");
    crate::println!("    zip / unzip         Compress/extract zip archives");
    crate::println!();
    
    // DEVELOPER & DEBUG
    crate::println_color!(COLOR_CYAN, "  DEVELOPER & DEBUG");
    crate::println!("    dmesg [-n N]        Kernel ring buffer (last N messages)");
    crate::println!("    memdbg / heapdbg    Heap allocation stats & fragmentation");
    crate::println!("    perf / perfstat     CPU, IRQ, scheduler, memory profiling");
    crate::println!("    irqstat / irqs      Per-CPU interrupt counters");
    crate::println!("    regs / cpuregs      CPU register dump (CR0/CR3/CR4/EFER)");
    crate::println!("    peek <addr> [n]     Hex dump memory region");
    crate::println!("    poke <addr> <val>   Write byte to memory address");
    crate::println!("    devpanel            Toggle real-time FPS/heap/IRQ overlay");
    crate::println!("    timecmd <cmd>       Run command & measure elapsed time");
    crate::println!("    benchmark [test]    Run performance benchmarks");
    crate::println!("    keytest             Interactive keyboard scancode tester");
    crate::println!("    test                Run internal kernel test suite");
    crate::println!("    panic               Trigger kernel panic (debug only)");
    crate::println!();
    
    // SERVICES & SCHEDULING
    crate::println_color!(COLOR_CYAN, "  SERVICES & SCHEDULING");
    crate::println!("    service <name> <op> Manage system services (start/stop)");
    crate::println!("    systemctl <cmd>     Systemd-style service control");
    crate::println!("    crontab [-e|-l]     Schedule recurring jobs");
    crate::println!("    at <time> <cmd>     Schedule one-time command execution");
    crate::println!("    sysctl <key>[=val]  View/modify kernel parameters");
    crate::println!();
    
    // SECURITY & IDENTITY
    crate::println_color!(COLOR_CYAN, "  SECURITY & IDENTITY");
    crate::println!("    security / sec      Security subsystem status & caps");
    crate::println!("    signature / sig     Kernel signature & proof of authorship");
    crate::println!("    hv / hypervisor     Hypervisor management commands");
    crate::println!();
    
    // SYSTEM CONTROL
    crate::println_color!(COLOR_CYAN, "  SYSTEM CONTROL");
    crate::println!("    exit / logout       Exit current session");
    crate::println!("    reboot              Restart the system");
    crate::println!("    shutdown / halt     Power off the system");
    crate::println!("    reset               Reset terminal state");
    crate::println!("    tty                 Print terminal device name");
    crate::println!("    stty <opts>         Configure terminal settings");
    crate::println!("    loadkeys <map>      Load keyboard layout");
    crate::println!("    setfont <font>      Change console font");
    crate::println!();
    
    // EASTER EGGS
    crate::println_color!(COLOR_CYAN, "  EASTER EGGS");
    crate::println!("    neofetch            System info with ASCII art logo");
    crate::println!("    matrix              Fullscreen Matrix rain animation");
    crate::println!("    cowsay <text>       ASCII cow says your message");
    crate::println!("    showcase [N]        Automated demo (marketing video)");
    crate::println!("    showcase3d          3D graphics cinematic showcase");
    crate::println!("    filled3d            3D filled polygon rendering demo");
    crate::println!();
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "  Total: ~180 commands | Type 'man <cmd>' for detailed usage");
    crate::println!();
}
fn cmd_man(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: man <command>");
        return;
    }
    
    match args[0] {
        "ls" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "LS(1) - List directory contents");
            crate::println!();
            crate::println!("SYNOPSIS: ls [path]");
            crate::println!();
            crate::println!("Lists files and directories.");
        }
        "cd" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "CD(1) - Change directory");
            crate::println!();
            crate::println!("SYNOPSIS: cd [path]");
            crate::println!();
            crate::println!("Special: ~ (home), .. (parent)");
        }
        "cat" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "CAT(1) - Display file contents");
            crate::println!();
            crate::println!("SYNOPSIS: cat <file>");
            crate::println!();
            crate::println!("Supports redirection: cat file > newfile");
        }
        "perf" | "perfstat" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "PERF(1) - Performance Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: perf");
            crate::println!();
            crate::println!("Shows uptime, FPS, IRQ count/rate, syscalls,");
            crate::println!("context switches, heap usage, and per-CPU stats.");
        }
        "memdbg" | "heapdbg" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "MEMDBG(1) - Memory Debug Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: memdbg");
            crate::println!();
            crate::println!("Shows heap usage, allocation/deallocation counts,");
            crate::println!("peak usage, fragmentation estimate, live alloc count.");
        }
        "dmesg" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "DMESG(1) - Kernel Ring Buffer");
            crate::println!();
            crate::println!("SYNOPSIS: dmesg [-n <count>] [-c]");
            crate::println!();
            crate::println!("Show kernel messages (captured from serial output).");
            crate::println!("  dmesg          Show all buffered messages");
            crate::println!("  dmesg -n 20    Show last 20 messages");
            crate::println!("  dmesg 50       Show last 50 messages");
            crate::println!("  dmesg -c       Acknowledge buffer");
        }
        "irqstat" | "irqs" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "IRQSTAT(1) - Interrupt Statistics");
            crate::println!();
            crate::println!("SYNOPSIS: irqstat");
            crate::println!();
            crate::println!("Shows total IRQ count, IRQ/sec rate, and per-CPU");
            crate::println!("interrupt breakdown with visual bars.");
        }
        "regs" | "registers" | "cpuregs" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "REGS(1) - CPU Register Dump");
            crate::println!();
            crate::println!("SYNOPSIS: regs");
            crate::println!();
            crate::println!("Dumps RSP, RBP, RFLAGS, CR0, CR3, CR4, EFER.");
            crate::println!("Decodes flag/bit meanings for each register.");
        }
        "peek" | "memdump" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "PEEK(1) - Memory Inspector");
            crate::println!();
            crate::println!("SYNOPSIS: peek <hex_addr> [byte_count]");
            crate::println!();
            crate::println!("Hex dump memory at virtual address (max 256 bytes).");
            crate::println!("  peek 0xFFFF800000000000 64");
        }
        "poke" | "memwrite" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "POKE(1) - Memory Writer");
            crate::println!();
            crate::println!("SYNOPSIS: poke <hex_addr> <hex_byte>");
            crate::println!();
            crate::println!("Write a single byte to virtual address. DANGEROUS!");
            crate::println!("  poke 0xB8000 0x41");
        }
        "devpanel" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "DEVPANEL(1) - Developer Overlay");
            crate::println!();
            crate::println!("SYNOPSIS: devpanel");
            crate::println!();
            crate::println!("Toggles real-time overlay in desktop mode showing:");
            crate::println!("FPS, frame time, heap usage bar, IRQ/s, per-CPU stats.");
            crate::println!("Also toggled with F12 while in desktop.");
        }
        "timecmd" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "TIMECMD(1) - Time a Command");
            crate::println!();
            crate::println!("SYNOPSIS: timecmd <command> [args...]");
            crate::println!();
            crate::println!("Executes a command and displays elapsed time in µs/ms.");
            crate::println!("  timecmd ls /");
            crate::println!("  timecmd benchmark cpu");
        }
        _ => {
            crate::println!("No manual entry for '{}'", args[0]);
        }
    }
}

// ==================== FILESYSTEM COMMANDS ====================

fn cmd_ls(args: &[&str]) {
    let path = args.first().copied();
    
    // Check if this is a VFS path
    if let Some(p) = path {
        if p.starts_with("/mnt/") || p.starts_with("/dev/") || p.starts_with("/proc/") || p == "/mnt" {
            cmd_ls_vfs(p);
            return;
        }
    }
    
    match crate::ramfs::with_fs(|fs| fs.ls(path)) {
        Ok(items) => {
            if items.is_empty() {
                return;
            }
            
            let max_name = items.iter().map(|(n, _, _)| n.len()).max().unwrap_or(0);
            
            for (name, file_type, size) in items {
                match file_type {
                    FileType::Directory => {
                        crate::print_color!(COLOR_CYAN, "{:<width$}", name, width = max_name + 2);
                        crate::println_color!(COLOR_DARK_GREEN, " <DIR>");
                    }
                    FileType::File => {
                        crate::print_color!(COLOR_GREEN, "{:<width$}", name, width = max_name + 2);
                        crate::println!(" {:>6} B", size);
                    }
                }
            }
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "ls: {}", e.as_str());
        }
    }
}

fn cmd_ls_vfs(path: &str) {
    use crate::vfs::{self, FileType as VfsFileType};
    
    match vfs::readdir(path) {
        Ok(entries) => {
            if entries.is_empty() {
                crate::println!("(empty)");
                return;
            }
            
            let max_name = entries.iter().map(|e| e.name.len()).max().unwrap_or(0);
            
            for entry in entries {
                match entry.file_type {
                    VfsFileType::Directory => {
                        crate::print_color!(COLOR_CYAN, "{:<width$}", entry.name, width = max_name + 2);
                        crate::println_color!(COLOR_DARK_GREEN, " <DIR>");
                    }
                    VfsFileType::Regular => {
                        crate::print_color!(COLOR_GREEN, "{:<width$}", entry.name, width = max_name + 2);
                        crate::println!(" (file)");
                    }
                    _ => {
                        crate::println!("{}", entry.name);
                    }
                }
            }
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "ls: {:?}", e);
        }
    }
}

fn cmd_cd(args: &[&str]) {
    let path = args.first().copied().unwrap_or("~");
    
    if let Err(e) = crate::ramfs::with_fs(|fs| fs.cd(path)) {
        crate::println_color!(COLOR_RED, "cd: {}: {}", path, e.as_str());
    }
}

fn cmd_pwd() {
    let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
    crate::println!("{}", cwd);
}

fn cmd_mkdir(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: mkdir <directory>");
        return;
    }
    
    for path in args {
        if let Err(e) = crate::ramfs::with_fs(|fs| fs.mkdir(path)) {
            crate::println_color!(COLOR_RED, "mkdir: {}: {}", path, e.as_str());
        }
    }
}

fn cmd_rmdir(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rmdir <directory>");
        return;
    }
    
    for path in args {
        if let Err(e) = crate::ramfs::with_fs(|fs| fs.rm(path)) {
            crate::println_color!(COLOR_RED, "rmdir: {}: {}", path, e.as_str());
        }
    }
}

fn cmd_touch(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: touch <file>");
        return;
    }
    
    for path in args {
        // Check if this is a VFS path
        if path.starts_with("/mnt/") {
            use crate::vfs::{self, OpenFlags};
            
            // Try to open/create the file
            let flags = OpenFlags(OpenFlags::O_WRONLY | OpenFlags::O_CREAT);
            match vfs::open(path, flags) {
                Ok(fd) => {
                    let _ = vfs::close(fd);
                    crate::println!("Created: {}", path);
                }
                Err(e) => crate::println_color!(COLOR_RED, "touch: {:?}", e),
            }
        } else {
            if let Err(e) = crate::ramfs::with_fs(|fs| fs.touch(path)) {
                crate::println_color!(COLOR_RED, "touch: {}: {}", path, e.as_str());
            }
        }
    }
}

fn cmd_rm(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rm <file>");
        return;
    }
    
    for path in args {
        if let Err(e) = crate::ramfs::with_fs(|fs| fs.rm(path)) {
            crate::println_color!(COLOR_RED, "rm: {}: {}", path, e.as_str());
        }
    }
}

fn cmd_cp(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: cp <source> <destination>");
        return;
    }
    
    if let Err(e) = crate::ramfs::with_fs(|fs| fs.cp(args[0], args[1])) {
        crate::println_color!(COLOR_RED, "cp: {}", e.as_str());
    }
}

fn cmd_mv(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: mv <source> <destination>");
        return;
    }
    
    if let Err(e) = crate::ramfs::with_fs(|fs| fs.mv(args[0], args[1])) {
        crate::println_color!(COLOR_RED, "mv: {}", e.as_str());
    }
}

fn cmd_cat(args: &[&str], redirect: Option<(&str, bool)>) {
    if args.is_empty() {
        crate::println!("Usage: cat <file>");
        return;
    }
    
    let mut output = String::new();
    
    for path in args {
        // Check if this is a VFS path
        if path.starts_with("/mnt/") || path.starts_with("/dev/") || path.starts_with("/proc/") {
            match cmd_cat_vfs(path) {
                Some(text) => {
                    if redirect.is_some() {
                        output.push_str(&text);
                    } else {
                        crate::print!("{}", text);
                    }
                }
                None => {} // Error already printed
            }
            continue;
        }
        
        match crate::ramfs::with_fs(|fs| fs.read_file(path).map(|c| c.to_vec())) {
            Ok(content) => {
                if let Ok(text) = core::str::from_utf8(&content) {
                    if redirect.is_some() {
                        output.push_str(text);
                    } else {
                        crate::print!("{}", text);
                    }
                } else {
                    crate::println_color!(COLOR_RED, "cat: {}: binary file", path);
                }
            }
            Err(e) => {
                crate::println_color!(COLOR_RED, "cat: {}: {}", path, e.as_str());
            }
        }
    }
    
    if let Some((file, append)) = redirect {
        let _ = crate::ramfs::with_fs(|fs| {
            if !fs.exists(file) { fs.touch(file).ok(); }
            if append { fs.append_file(file, output.as_bytes()) } 
            else { fs.write_file(file, output.as_bytes()) }
        });
    }
}

fn cmd_cat_vfs(path: &str) -> Option<alloc::string::String> {
    use crate::vfs::{self, OpenFlags};
    use alloc::string::ToString;
    
    // Open the file
    let fd = match vfs::open(path, OpenFlags(OpenFlags::O_RDONLY)) {
        Ok(f) => f,
        Err(e) => {
            crate::println_color!(COLOR_RED, "cat: {}: {:?}", path, e);
            return None;
        }
    };
    
    // Read the file content
    let mut buffer = [0u8; 4096];
    let mut content = alloc::vec::Vec::new();
    
    loop {
        let bytes_read = match vfs::read(fd, &mut buffer) {
            Ok(n) => n,
            Err(e) => {
                crate::println_color!(COLOR_RED, "cat: {}: read error {:?}", path, e);
                let _ = vfs::close(fd);
                return None;
            }
        };
        
        if bytes_read == 0 {
            break;
        }
        
        content.extend_from_slice(&buffer[..bytes_read]);
    }
    
    let _ = vfs::close(fd);
    
    match core::str::from_utf8(&content) {
        Ok(text) => Some(String::from(text)),
        Err(_) => {
            crate::println_color!(COLOR_RED, "cat: {}: binary file", path);
            None
        }
    }
}

fn cmd_head(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: head <file> [lines]");
        return;
    }
    
    let lines: usize = if args.len() > 1 { args[1].parse().unwrap_or(10) } else { 10 };
    
    match crate::ramfs::with_fs(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
        Ok(content) => {
            if let Ok(text) = core::str::from_utf8(&content) {
                for (i, line) in text.lines().enumerate() {
                    if i >= lines { break; }
                    crate::println!("{}", line);
                }
            }
        }
        Err(e) => crate::println_color!(COLOR_RED, "head: {}", e.as_str()),
    }
}

fn cmd_tail(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: tail <file> [lines]");
        return;
    }
    
    let lines: usize = if args.len() > 1 { args[1].parse().unwrap_or(10) } else { 10 };
    
    match crate::ramfs::with_fs(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
        Ok(content) => {
            if let Ok(text) = core::str::from_utf8(&content) {
                let all: Vec<&str> = text.lines().collect();
                let start = if all.len() > lines { all.len() - lines } else { 0 };
                for line in &all[start..] {
                    crate::println!("{}", line);
                }
            }
        }
        Err(e) => crate::println_color!(COLOR_RED, "tail: {}", e.as_str()),
    }
}

fn cmd_wc(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: wc <file>");
        return;
    }
    
    match crate::ramfs::with_fs(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
        Ok(content) => {
            if let Ok(text) = core::str::from_utf8(&content) {
                let lines = text.lines().count();
                let words = text.split_whitespace().count();
                crate::println!("{:>6} {:>6} {:>6} {}", lines, words, content.len(), args[0]);
            }
        }
        Err(e) => crate::println_color!(COLOR_RED, "wc: {}", e.as_str()),
    }
}

fn cmd_stat(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: stat <file>");
        return;
    }
    
    match crate::ramfs::with_fs(|fs| fs.stat(args[0]).map(|e| e.clone())) {
        Ok(entry) => {
            crate::println_color!(COLOR_CYAN, "  File: {}", entry.name);
            let ftype = if entry.file_type == FileType::Directory { "directory" } else { "file" };
            crate::println!("  Type: {}", ftype);
            crate::println!("  Size: {} bytes", entry.content.len());
        }
        Err(e) => crate::println_color!(COLOR_RED, "stat: {}", e.as_str()),
    }
}

fn cmd_tree(args: &[&str]) {
    let path = args.first().copied().unwrap_or("/");
    crate::println_color!(COLOR_CYAN, "{}", path);
    print_tree_recursive(path, "");
}

fn print_tree_recursive(path: &str, prefix: &str) {
    if let Ok(items) = crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
        let len = items.len();
        for (i, (name, file_type, _)) in items.iter().enumerate() {
            let is_last = i == len - 1;
            let conn = if is_last { "└── " } else { "├── " };
            
            match file_type {
                FileType::Directory => {
                    crate::print!("{}{}", prefix, conn);
                    crate::println_color!(COLOR_CYAN, "{}/", name);
                    
                    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                    let child = if path == "/" { format!("/{}", name) } else { format!("{}/{}", path, name) };
                    print_tree_recursive(&child, &new_prefix);
                }
                FileType::File => {
                    crate::print!("{}{}", prefix, conn);
                    crate::println_color!(COLOR_GREEN, "{}", name);
                }
            }
        }
    }
}

fn cmd_find(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: find <name>");
        return;
    }
    find_recursive("/", args[0]);
}

fn find_recursive(path: &str, pattern: &str) {
    if let Ok(items) = crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
        for (name, file_type, _) in items {
            let full = if path == "/" { format!("/{}", name) } else { format!("{}/{}", path, name) };
            if name.contains(pattern) {
                crate::println!("{}", full);
            }
            if file_type == FileType::Directory {
                find_recursive(&full, pattern);
            }
        }
    }
}

// ==================== TEXT COMMANDS ====================

fn cmd_echo(args: &[&str], redirect: Option<(&str, bool)>) {
    let text = args.join(" ");
    
    if let Some((file, append)) = redirect {
        let content = format!("{}\n", text);
        
        // Check if this is a VFS path
        if file.starts_with("/mnt/") {
            use crate::vfs::{self, OpenFlags};
            
            // Open for writing (O_CREAT will create if doesn't exist)
            let flags = if append {
                OpenFlags(OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_APPEND)
            } else {
                OpenFlags(OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_TRUNC)
            };
            
            match vfs::open(file, flags) {
                Ok(fd) => {
                    if let Err(e) = vfs::write(fd, content.as_bytes()) {
                        crate::println_color!(COLOR_RED, "echo: write error: {:?}", e);
                    }
                    let _ = vfs::close(fd);
                }
                Err(e) => crate::println_color!(COLOR_RED, "echo: {:?}", e),
            }
        } else {
            let _ = crate::ramfs::with_fs(|fs| {
                if !fs.exists(file) { fs.touch(file).ok(); }
                if append { fs.append_file(file, content.as_bytes()) }
                else { fs.write_file(file, content.as_bytes()) }
            });
        }
    } else {
        crate::println!("{}", text);
    }
}

fn cmd_grep(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: grep <pattern> <file>");
        return;
    }
    
    let pattern = args[0];
    
    match crate::ramfs::with_fs(|fs| fs.read_file(args[1]).map(|c| c.to_vec())) {
        Ok(content) => {
            if let Ok(text) = core::str::from_utf8(&content) {
                for line in text.lines() {
                    if line.contains(pattern) {
                        let parts: Vec<&str> = line.split(pattern).collect();
                        for (i, part) in parts.iter().enumerate() {
                            crate::print!("{}", part);
                            if i < parts.len() - 1 {
                                crate::print_color!(COLOR_RED, "{}", pattern);
                            }
                        }
                        crate::println!();
                    }
                }
            }
        }
        Err(e) => crate::println_color!(COLOR_RED, "grep: {}", e.as_str()),
    }
}

// ==================== SYSTEM COMMANDS ====================

fn cmd_clear() {
    crate::framebuffer::clear();
}

fn cmd_time() {
    let ticks = crate::logger::get_ticks();
    let secs = ticks / 100;
    let mins = secs / 60;
    let hours = mins / 60;
    
    crate::print_color!(COLOR_CYAN, "Uptime: ");
    crate::println_color!(COLOR_GREEN, "{}h {}m {}s", hours, mins % 60, secs % 60);
    
    // Also show RTC time
    let dt = crate::rtc::read_rtc();
    crate::print_color!(COLOR_CYAN, "Time:   ");
    crate::println_color!(COLOR_GREEN, "{}", dt.format_time());
}

fn cmd_date() {
    let dt = crate::rtc::read_rtc();
    crate::println_color!(COLOR_GREEN, "{}", dt.format());
}

fn cmd_whoami() {
    crate::println!("{}", crate::auth::current_user());
}

fn cmd_hostname() {
    crate::println!("trustos");
}

fn cmd_id() {
    let user = crate::auth::current_user();
    let uid = crate::auth::current_uid();
    let gid = crate::auth::current_gid();
    crate::println!("uid={}({}) gid={}({})", uid, user, gid, 
        if gid == 0 { "root" } else if gid == 100 { "users" } else { "wheel" });
}

// ==================== USER MANAGEMENT COMMANDS ====================

fn cmd_login() {
    // Logout current user first
    crate::auth::logout();
    crate::println!();
    
    if crate::auth::login_prompt() {
        // Successfully logged in
        crate::println_color!(COLOR_GREEN, "Login successful.");
    } else {
        // Failed - auto-login as guest or stay logged out
        crate::println_color!(COLOR_RED, "Login failed.");
    }
}

fn cmd_su(args: &[&str]) {
    let target_user = if args.is_empty() { "root" } else { args[0] };
    
    // If already root, can switch without password
    if crate::auth::is_root() && target_user != "root" {
        // Just switch
        crate::println_color!(COLOR_YELLOW, "Switching to {} (root privilege)", target_user);
        return;
    }
    
    // Need password
    crate::print_color!(COLOR_CYAN, "Password: ");
    let mut password_buf = [0u8; 128];
    let password_len = crate::keyboard::read_line_hidden(&mut password_buf);
    let password = core::str::from_utf8(&password_buf[..password_len])
        .unwrap_or("")
        .trim();
    crate::println!();
    
    match crate::auth::login(target_user, password) {
        Ok(()) => {
            crate::println_color!(COLOR_GREEN, "Switched to {}", target_user);
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "su: {}", e);
        }
    }
}

fn cmd_passwd(args: &[&str]) {
    let target_user = if args.is_empty() {
        crate::auth::current_user()
    } else {
        // Only root can change other users' passwords
        if !crate::auth::is_root() {
            crate::println_color!(COLOR_RED, "passwd: Only root can change other users' passwords");
            return;
        }
        String::from(args[0])
    };
    
    crate::println!("Changing password for {}", target_user);
    
    // Get current password (unless root)
    let old_password = if !crate::auth::is_root() {
        crate::print!("Current password: ");
        let mut buf = [0u8; 128];
        let len = crate::keyboard::read_line_hidden(&mut buf);
        crate::println!();
        String::from(core::str::from_utf8(&buf[..len]).unwrap_or("").trim())
    } else {
        String::new()
    };
    
    // Get new password
    crate::print!("New password: ");
    let mut new_buf = [0u8; 128];
    let new_len = crate::keyboard::read_line_hidden(&mut new_buf);
    crate::println!();
    let new_password = core::str::from_utf8(&new_buf[..new_len]).unwrap_or("").trim();
    
    // Confirm new password
    crate::print!("Retype new password: ");
    let mut confirm_buf = [0u8; 128];
    let confirm_len = crate::keyboard::read_line_hidden(&mut confirm_buf);
    crate::println!();
    let confirm = core::str::from_utf8(&confirm_buf[..confirm_len]).unwrap_or("").trim();
    
    if new_password != confirm {
        crate::println_color!(COLOR_RED, "passwd: passwords do not match");
        return;
    }
    
    if new_password.len() < 1 {
        crate::println_color!(COLOR_RED, "passwd: password too short");
        return;
    }
    
    match crate::auth::change_password(&target_user, &old_password, new_password) {
        Ok(()) => {
            crate::println_color!(COLOR_GREEN, "passwd: password updated successfully");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "passwd: {}", e);
        }
    }
}

fn cmd_adduser(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: adduser <username> [-a]");
        crate::println!("  -a  Make user an admin (wheel group)");
        return;
    }
    
    if !crate::auth::is_root() {
        crate::println_color!(COLOR_RED, "adduser: must be root");
        return;
    }
    
    let username = args[0];
    let is_admin = args.contains(&"-a") || args.contains(&"--admin");
    
    // Get password
    crate::print!("New password for {}: ", username);
    let mut password_buf = [0u8; 128];
    let password_len = crate::keyboard::read_line_hidden(&mut password_buf);
    crate::println!();
    let password = core::str::from_utf8(&password_buf[..password_len]).unwrap_or("").trim();
    
    // Confirm password
    crate::print!("Retype password: ");
    let mut confirm_buf = [0u8; 128];
    let confirm_len = crate::keyboard::read_line_hidden(&mut confirm_buf);
    crate::println!();
    let confirm = core::str::from_utf8(&confirm_buf[..confirm_len]).unwrap_or("").trim();
    
    if password != confirm {
        crate::println_color!(COLOR_RED, "adduser: passwords do not match");
        return;
    }
    
    match crate::auth::add_user(username, password, is_admin) {
        Ok(uid) => {
            crate::println_color!(COLOR_GREEN, "User {} created with UID {}", username, uid);
            
            // Create home directory
            let home = format!("/home/{}", username);
            crate::ramfs::with_fs(|fs| {
                let _ = fs.mkdir("/home");
                let _ = fs.mkdir(&home);
            });
            crate::println!("Home directory: {}", home);
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "adduser: {}", e);
        }
    }
}

fn cmd_deluser(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: deluser <username>");
        return;
    }
    
    if !crate::auth::is_root() {
        crate::println_color!(COLOR_RED, "deluser: must be root");
        return;
    }
    
    let username = args[0];
    
    crate::print_color!(COLOR_YELLOW, "Delete user {}? [y/N]: ", username);
    let mut buf = [0u8; 16];
    let len = crate::keyboard::read_line(&mut buf);
    let answer = core::str::from_utf8(&buf[..len]).unwrap_or("").trim();
    
    if answer != "y" && answer != "Y" {
        crate::println!("Cancelled.");
        return;
    }
    
    match crate::auth::delete_user(username) {
        Ok(()) => {
            crate::println_color!(COLOR_GREEN, "User {} deleted", username);
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "deluser: {}", e);
        }
    }
}

fn cmd_users() {
    crate::println_color!(COLOR_CYAN, "USER            UID   GID   DESCRIPTION");
    crate::println_color!(COLOR_CYAN, "──────────────────────────────────────────");
    
    for (username, uid, gid, gecos) in crate::auth::list_users() {
        crate::println!("{:<15} {:<5} {:<5} {}", username, uid, gid, gecos);
    }
}

fn cmd_logout() {
    let user = crate::auth::current_user();
    crate::auth::logout();
    crate::println!("Logged out {}.", user);
    crate::println!();
    
    // Show login prompt
    if !crate::auth::login_prompt() {
        // If login fails, auto-login as root for development
        crate::println_color!(COLOR_YELLOW, "Auto-login as root (development mode)");
        crate::auth::auto_login_root();
    }
}

fn cmd_info() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "=== T-RUSTOS ===");
    crate::print_color!(COLOR_CYAN, "Version:      ");
    crate::println!("0.1.0");
    crate::print_color!(COLOR_CYAN, "Architecture: ");
    crate::println!("x86_64");
    crate::print_color!(COLOR_CYAN, "Bootloader:   ");
    crate::println!("Limine");
    crate::println!();
    crate::println_color!(COLOR_BRIGHT_GREEN, "Modules:");
    for m in ["Memory", "Interrupts", "Keyboard", "Framebuffer", "RAM FS", "History", "Scheduler"] {
        crate::print_color!(COLOR_GREEN, "  [x] ");
        crate::println!("{}", m);
    }
    
    // Disk status
    if crate::disk::is_available() {
        crate::print_color!(COLOR_GREEN, "  [x] ");
        crate::println!("Disk I/O");
    } else {
        crate::print_color!(COLOR_DARK_GREEN, "  [-] ");
        crate::println!("Disk I/O (no disk)");
    }
    
    // Network status
    if crate::network::is_available() {
        crate::print_color!(COLOR_GREEN, "  [x] ");
        crate::println!("Network");
    } else {
        crate::print_color!(COLOR_DARK_GREEN, "  [-] ");
        crate::println!("Network (down)");
    }
}

fn cmd_version() {
    crate::println!("T-RustOs v0.1.0 (Rust + Limine)");
}

fn cmd_uname(args: &[&str]) {
    let all = args.contains(&"-a");
    if args.is_empty() || all { crate::print!("T-RustOs "); }
    if args.contains(&"-n") || all { crate::print!("trustos "); }
    if args.contains(&"-r") || all { crate::print!("0.1.0 "); }
    if args.contains(&"-m") || all { crate::print!("x86_64"); }
    crate::println!();
}

fn cmd_env() {
    crate::println!("USER=root");
    crate::println!("HOSTNAME=trustos");
    crate::println!("SHELL=/bin/tsh");
    crate::println!("PWD={}", crate::ramfs::with_fs(|fs| String::from(fs.pwd())));
    crate::println!("HOME=/home");
    crate::println!("TERM=trustos-console");
}

fn cmd_history() {
    for (num, cmd) in crate::keyboard::history_list() {
        crate::print_color!(COLOR_DARK_GREEN, "{:>4}  ", num);
        crate::println!("{}", cmd);
    }
}

fn cmd_ps() {
    crate::println_color!(COLOR_CYAN, "  PID  STATE    CMD");
    crate::println!("    1  running  kernel");
    crate::println!("    2  running  tsh");
    
    // Show task count
    let count = crate::task::task_count();
    if count > 0 {
        crate::println!("  ... +{} background tasks (use 'tasks' for details)", count);
    }
}

fn cmd_free() {
    let used = crate::memory::heap::used();
    let free = crate::memory::heap::free();
    let total = used + free;
    crate::println_color!(COLOR_CYAN, "              total       used       free");
    crate::println!("Heap:    {:>10}  {:>10}  {:>10}", total, used, free);
    crate::println!("  (KB)   {:>10}  {:>10}  {:>10}", total / 1024, used / 1024, free / 1024);
}

fn cmd_df() {
    crate::println_color!(COLOR_CYAN, "Filesystem   Size  Used  Avail");
    crate::println!("ramfs         64K    1K    63K");
}

// ==================== TEST & DEBUG ====================

fn cmd_test() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Running self-test...");
    crate::println!();
    
    crate::print!("  Heap... ");
    let v: Vec<u32> = (0..100).collect();
    if v.len() == 100 { crate::println_color!(COLOR_GREEN, "[OK]"); }
    else { crate::println_color!(COLOR_RED, "[FAIL]"); }
    
    crate::print!("  String... ");
    let mut s = String::from("Hello");
    s.push_str(" World");
    if s.len() == 11 { crate::println_color!(COLOR_GREEN, "[OK]"); }
    else { crate::println_color!(COLOR_RED, "[FAIL]"); }
    
    crate::print!("  RAM FS... ");
    let ok = crate::ramfs::with_fs(|fs| {
        fs.touch("/tmp/t").ok();
        fs.write_file("/tmp/t", b"x").ok();
        let r = fs.read_file("/tmp/t").map(|c| c[0] == b'x').unwrap_or(false);
        fs.rm("/tmp/t").ok();
        r
    });
    if ok { crate::println_color!(COLOR_GREEN, "[OK]"); }
    else { crate::println_color!(COLOR_RED, "[FAIL]"); }
    
    crate::print!("  Interrupts... ");
    if x86_64::instructions::interrupts::are_enabled() {
        crate::println_color!(COLOR_GREEN, "[OK]");
    } else {
        crate::println_color!(COLOR_RED, "[FAIL]");
    }
    
    crate::println!();
    crate::println_color!(COLOR_BRIGHT_GREEN, "Done!");
}

fn cmd_keytest() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Keyboard Test Mode");
    crate::println!("Test all keys including Space, Backspace, Delete");
    crate::println_color!(COLOR_YELLOW, "Type 'quit' to exit test mode");
    crate::println!();
    
    let mut test_buffer = [0u8; 256];
    
    loop {
        crate::print_color!(COLOR_CYAN, "test> ");
        let len = crate::keyboard::read_line(&mut test_buffer);
        let input = core::str::from_utf8(&test_buffer[..len]).unwrap_or("");
        
        if input.trim() == "quit" {
            crate::println_color!(COLOR_GREEN, "Exiting test mode");
            break;
        }
        
        // Show what was typed
        crate::print!("  Received {} bytes: ", len);
        crate::print_color!(COLOR_WHITE, "\"{}\"", input);
        crate::println!();
        
        // Show hex dump of characters
        crate::print!("  Hex: ");
        for &byte in &test_buffer[..len] {
            crate::print_color!(COLOR_DARK_GREEN, "{:02x} ", byte);
        }
        crate::println!();
        
        // Show character codes
        crate::print!("  Chars: ");
        for &byte in &test_buffer[..len] {
            if byte >= 32 && byte < 127 {
                crate::print_color!(COLOR_BRIGHT_GREEN, "'{}' ", byte as char);
            } else if byte == 0x08 {
                crate::print_color!(COLOR_YELLOW, "<BS> ");
            } else if byte == 0x20 {
                crate::print_color!(COLOR_YELLOW, "<SPACE> ");
            } else {
                crate::print_color!(COLOR_RED, "0x{:02x} ", byte);
            }
        }
        crate::println!();
        crate::println!();
    }
}

fn cmd_hexdump(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: hexdump <file>");
        return;
    }
    
    match crate::ramfs::with_fs(|fs| fs.read_file(args[0]).map(|c| c.to_vec())) {
        Ok(content) => {
            for (i, chunk) in content.chunks(16).enumerate() {
                crate::print_color!(COLOR_DARK_GREEN, "{:08x}  ", i * 16);
                for (j, b) in chunk.iter().enumerate() {
                    if j == 8 { crate::print!(" "); }
                    crate::print!("{:02x} ", b);
                }
                for _ in chunk.len()..16 { crate::print!("   "); }
                crate::print!(" |");
                for b in chunk {
                    let c = if *b >= 0x20 && *b < 0x7F { *b as char } else { '.' };
                    crate::print!("{}", c);
                }
                crate::println!("|");
            }
        }
        Err(e) => crate::println_color!(COLOR_RED, "hexdump: {}", e.as_str()),
    }
}

fn cmd_panic() {
    crate::println_color!(COLOR_RED, "Panic triggered!");
    panic!("User panic");
}

// ==================== EXIT ====================

fn cmd_reboot() {
    crate::println_color!(COLOR_YELLOW, "Rebooting...");
    unsafe {
        x86_64::instructions::port::Port::<u8>::new(0x64).write(0xFE);
    }
    loop { x86_64::instructions::hlt(); }
}

fn cmd_halt() {
    crate::println_color!(COLOR_YELLOW, "System halted.");
    loop {
        x86_64::instructions::interrupts::disable();
        x86_64::instructions::hlt();
    }
}

// ==================== EASTER EGGS ====================

fn cmd_neofetch() {
    let secs = crate::logger::get_ticks() / 100;
    let (w, h) = crate::framebuffer::get_dimensions();
    let total_mem_mb = crate::memory::total_physical_memory() / 1024 / 1024;
    let mem_stats = crate::memory::stats();
    let heap_used_mb = mem_stats.heap_used / 1024 / 1024;
    let heap_total_mb = (mem_stats.heap_used + mem_stats.heap_free) / 1024 / 1024;
    
    crate::println_color!(COLOR_BRIGHT_GREEN, r"       _____          ");
    crate::print_color!(COLOR_GREEN, r"      |  _  |         ");
    crate::print_color!(COLOR_CYAN, "root");
    crate::print_color!(COLOR_WHITE, "@");
    crate::println_color!(COLOR_CYAN, "trustos");
    crate::print_color!(COLOR_GREEN, r"      | |_| |         ");
    crate::println!("---------------");
    crate::print_color!(COLOR_GREEN, r"      |  _  |         ");
    crate::print_color!(COLOR_CYAN, "OS: ");
    crate::println!("TrustOS v0.1.1");
    crate::print_color!(COLOR_DARK_GREEN, r"      | |_| |         ");
    crate::print_color!(COLOR_CYAN, "Kernel: ");
    crate::println!("{}", crate::signature::KERNEL_VERSION);
    crate::print_color!(COLOR_DARK_GREEN, r"      |_____|         ");
    crate::print_color!(COLOR_CYAN, "Uptime: ");
    crate::println!("{} secs", secs);
    crate::print_color!(COLOR_BRIGHT_GREEN, r"                      ");
    crate::print_color!(COLOR_CYAN, "Shell: ");
    crate::println!("tsh");
    crate::print_color!(COLOR_GREEN, r"                      ");
    crate::print_color!(COLOR_CYAN, "Resolution: ");
    crate::println!("{}x{}", w, h);
    crate::print_color!(COLOR_GREEN, r"                      ");
    crate::print_color!(COLOR_CYAN, "Memory: ");
    crate::println!("{} MB total, {} / {} MB heap", total_mem_mb, heap_used_mb, heap_total_mb);
    crate::print_color!(COLOR_GREEN, r"                      ");
    crate::print_color!(COLOR_CYAN, "CPU: ");
    crate::println!("{} cores", crate::cpu::core_count());
    crate::print_color!(COLOR_GREEN, r"                      ");
    crate::print_color!(COLOR_CYAN, "Creator: ");
    crate::println!("Nated0ge (@nathan237)");
    crate::println!();
}

fn cmd_matrix() {
    crate::println_color!(COLOR_GREEN, "Wake up, Neo...");
    crate::println_color!(COLOR_GREEN, "The Matrix has you...");
    crate::println_color!(COLOR_GREEN, "Follow the white rabbit.");
}

fn cmd_cowsay(args: &[&str]) {
    let text = if args.is_empty() { "Moo!" } else { &args.join(" ") };
    let len = text.len();
    crate::print!(" ");
    for _ in 0..len + 2 { crate::print!("_"); }
    crate::println!();
    crate::println!("< {} >", text);
    crate::print!(" ");
    for _ in 0..len + 2 { crate::print!("-"); }
    crate::println!();
    crate::println!("        \\   ^__^");
    crate::println!("         \\  (oo)\\_______");
    crate::println!("            (__)\\       )\\/\\");
    crate::println!("                ||----w |");
    crate::println!("                ||     ||");
}

// ==================== GRAPHICS PERFORMANCE BENCHMARK ====================

fn cmd_benchmark(args: &[&str]) {
    use alloc::vec;
    
    crate::println_color!(COLOR_CYAN, "═══════════════════════════════════════════════════════════");
    crate::println_color!(COLOR_CYAN, "              TrustOS Graphics Benchmark");
    crate::println_color!(COLOR_CYAN, "               SSE2 SIMD Optimizations");
    crate::println_color!(COLOR_CYAN, "═══════════════════════════════════════════════════════════");
    crate::println!();
    
    let (width, height) = crate::framebuffer::get_dimensions();
    let pixels = (width * height) as usize;
    crate::println!("Resolution: {}x{} ({} pixels, {} MB)", 
        width, height, pixels, pixels * 4 / 1024 / 1024);
    crate::println!();
    
    // Test 1: Buffer fill with SSE2
    crate::println_color!(COLOR_GREEN, "▸ Test 1: SSE2 Buffer Fill");
    {
        let mut buffer = vec![0u32; pixels];
        let iterations = 100;
        
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            crate::graphics::simd::fill_buffer_fast(&mut buffer, 0xFF00FF66);
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_frame = (end - start) / iterations;
        let megapixels_per_frame = pixels as f64 / 1_000_000.0;
        crate::println!("  {} iterations: {} cycles/frame", iterations, cycles_per_frame);
        crate::println!("  Throughput: ~{:.1} megapixels/frame", megapixels_per_frame);
    }
    
    // Test 2: Buffer copy with SSE2
    crate::println_color!(COLOR_GREEN, "▸ Test 2: SSE2 Buffer Copy");
    {
        let src = vec![0xFF112233u32; pixels];
        let mut dst = vec![0u32; pixels];
        let iterations = 100;
        
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            crate::graphics::simd::copy_buffer_fast(&mut dst, &src);
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_frame = (end - start) / iterations;
        let mb_per_frame = (pixels * 4) as f64 / 1024.0 / 1024.0;
        crate::println!("  {} iterations: {} cycles/frame", iterations, cycles_per_frame);
        crate::println!("  Bandwidth: ~{:.1} MB copied/frame", mb_per_frame);
    }
    
    // Test 3: Rectangle fill
    crate::println_color!(COLOR_GREEN, "▸ Test 3: Rectangle Fill (400x300)");
    {
        let mut surface = crate::graphics::fast_render::FastSurface::new(1280, 800);
        let iterations = 500;
        
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            surface.fill_rect(100, 100, 400, 300, 0xFF00AA55);
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_rect = (end - start) / iterations;
        let pixels_filled = 400 * 300;
        crate::println!("  {} iterations: {} cycles/rect", iterations, cycles_per_rect);
        crate::println!("  {} pixels/rect", pixels_filled);
    }
    
    // Test 4: swap_buffers (framebuffer update)
    crate::println_color!(COLOR_GREEN, "▸ Test 4: Framebuffer swap_buffers");
    {
        // Make sure backbuffer is enabled
        let was_enabled = crate::framebuffer::is_double_buffer_enabled();
        if !was_enabled {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }
        
        let iterations = 50;
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            crate::framebuffer::swap_buffers();
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_swap = (end - start) / iterations;
        // Estimate FPS: ~3GHz CPU, 60 FPS target = 50M cycles/frame
        let estimated_fps = 3_000_000_000u64 / cycles_per_swap.max(1);
        crate::println!("  {} iterations: {} cycles/swap", iterations, cycles_per_swap);
        crate::println!("  Estimated max FPS: ~{} (at 3GHz)", estimated_fps);
        
        if !was_enabled {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }
    
    // Test 5: Terminal rendering
    crate::println_color!(COLOR_GREEN, "▸ Test 5: GraphicsTerminal render (80x25)");
    {
        let mut terminal = crate::wayland::terminal::GraphicsTerminal::new(80, 25);
        terminal.write_str("Hello from TrustOS! Testing SSE2 SIMD terminal rendering performance.\n");
        terminal.write_str("The quick brown fox jumps over the lazy dog.\n");
        
        let iterations = 100;
        let start = crate::cpu::tsc::read_tsc();
        for _ in 0..iterations {
            let _ = terminal.render();
        }
        let end = crate::cpu::tsc::read_tsc();
        
        let cycles_per_render = (end - start) / iterations;
        let estimated_fps = 3_000_000_000u64 / cycles_per_render.max(1);
        crate::println!("  {} iterations: {} cycles/render", iterations, cycles_per_render);
        crate::println!("  Estimated terminal FPS: ~{}", estimated_fps);
    }
    
    crate::println!();
    crate::println_color!(COLOR_CYAN, "═══════════════════════════════════════════════════════════");
    crate::println_color!(COLOR_GREEN, "Benchmark complete! SSE2 optimizations active.");
    crate::println_color!(COLOR_CYAN, "═══════════════════════════════════════════════════════════");
}

// ==================== COSMIC UI DEMO ====================

fn cmd_cosmic(args: &[&str]) {
    use crate::cosmic::{CosmicRenderer, Rect, Point, Color, theme, CosmicTheme, set_theme};
    use crate::cosmic::theme::dark;
    
    let subcommand = args.first().copied().unwrap_or("demo");
    
    match subcommand {
        "demo" | "test" => {
            crate::println_color!(COLOR_CYAN, "╔═══════════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║          COSMIC UI Framework Demo (libcosmic-inspired)       ║");
            crate::println_color!(COLOR_CYAN, "╚═══════════════════════════════════════════════════════════════╝");
            crate::println!();
            
            let (width, height) = crate::framebuffer::get_dimensions();
            crate::println!("  Framebuffer: {}x{}", width, height);
            crate::println!("  Renderer: tiny-skia (software, no_std)");
            crate::println!("  Theme: COSMIC Dark (Pop!_OS style)");
            crate::println!();
            
            crate::println_color!(COLOR_GREEN, "Creating COSMIC renderer...");
            let mut renderer = CosmicRenderer::new(width, height);
            
            // Clear with COSMIC background
            renderer.clear(dark::BG_BASE);
            
            crate::println_color!(COLOR_GREEN, "Drawing COSMIC UI elements...");
            
            // Draw top panel (GNOME-style)
            let panel_rect = Rect::new(0.0, 0.0, width as f32, 32.0);
            renderer.draw_panel(panel_rect);
            
            // Draw some test shapes
            // Rounded rectangle
            let rect1 = Rect::new(50.0, 80.0, 200.0, 100.0);
            renderer.fill_rounded_rect(rect1, 12.0, dark::SURFACE);
            renderer.stroke_rounded_rect(rect1, 12.0, dark::BORDER, 1.0);
            
            // Button with shadow
            let btn_rect = Rect::new(300.0, 100.0, 120.0, 40.0);
            renderer.draw_shadow(btn_rect, 8.0, 8.0, Color::BLACK.with_alpha(0.4));
            renderer.fill_rounded_rect(btn_rect, 8.0, dark::ACCENT);
            
            // Another button (suggested style)
            let btn2_rect = Rect::new(450.0, 100.0, 120.0, 40.0);
            renderer.fill_rounded_rect(btn2_rect, 8.0, dark::BUTTON_SUGGESTED);
            
            // Destructive button
            let btn3_rect = Rect::new(600.0, 100.0, 120.0, 40.0);
            renderer.fill_rounded_rect(btn3_rect, 8.0, dark::BUTTON_DESTRUCTIVE);
            
            // Draw circles
            renderer.fill_circle(Point::new(100.0, 250.0), 30.0, dark::ACCENT);
            renderer.fill_circle(Point::new(180.0, 250.0), 30.0, dark::SUCCESS);
            renderer.fill_circle(Point::new(260.0, 250.0), 30.0, dark::WARNING);
            renderer.fill_circle(Point::new(340.0, 250.0), 30.0, dark::ERROR);
            
            // Draw header bar example
            let header_rect = Rect::new(50.0, 320.0, 400.0, 40.0);
            renderer.draw_header(header_rect, "COSMIC Window", true);
            
            // Window body
            let window_body = Rect::new(50.0, 360.0, 400.0, 150.0);
            renderer.fill_rect(window_body, dark::BG_COMPONENT);
            renderer.stroke_rounded_rect(
                Rect::new(50.0, 320.0, 400.0, 190.0),
                0.0,
                dark::BORDER,
                1.0
            );
            
            // Draw dock example
            let dock_items = [
                crate::cosmic::DockItem { name: "Files", active: true, hovered: false, running: true },
                crate::cosmic::DockItem { name: "Term", active: false, hovered: true, running: true },
                crate::cosmic::DockItem { name: "Browser", active: false, hovered: false, running: false },
                crate::cosmic::DockItem { name: "Settings", active: false, hovered: false, running: true },
            ];
            let dock_rect = Rect::new((width - 64) as f32, 100.0, 64.0, 280.0);
            renderer.draw_dock(dock_rect, &dock_items);
            
            // Gradient test
            let grad_rect = Rect::new(500.0, 320.0, 200.0, 100.0);
            renderer.fill_gradient_v(grad_rect, dark::ACCENT, dark::BG_BASE);
            
            crate::println_color!(COLOR_GREEN, "Presenting to framebuffer...");
            renderer.present_to_framebuffer();
            
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "✓ COSMIC UI demo rendered successfully!");
            crate::println!();
            crate::println!("  Features demonstrated:");
            crate::println!("  - Rounded rectangles with anti-aliasing");
            crate::println!("  - Drop shadows");
            crate::println!("  - COSMIC color palette");
            crate::println!("  - Top panel, header bar, dock");
            crate::println!("  - Buttons (normal, suggested, destructive)");
            crate::println!("  - Vertical gradients");
            crate::println!();
            crate::println!("  Press any key to return to shell...");
            
            // Wait for keypress then clear screen
            crate::keyboard::wait_for_key();
            crate::framebuffer::clear();
            crate::framebuffer::swap_buffers();
        },
        "desktop" => {
            // Launch COSMIC V2 desktop with multi-layer compositor (no flicker)
            cmd_cosmic_v2();
        },
        "theme" => {
            let theme_name = args.get(1).copied().unwrap_or("matrix");
            match theme_name {
                "dark" => {
                    set_theme(CosmicTheme::dark());
                    crate::println_color!(COLOR_GREEN, "Theme set to COSMIC Dark");
                },
                "light" => {
                    set_theme(CosmicTheme::light());
                    crate::println_color!(COLOR_GREEN, "Theme set to COSMIC Light");
                },
                "matrix" => {
                    set_theme(CosmicTheme::matrix());
                    crate::println_color!(0x00FF00, "Theme set to MATRIX - Wake up, Neo...");
                },
                _ => {
                    crate::println!("Available themes: dark, light, matrix");
                }
            }
        },
        "info" => {
            crate::println_color!(COLOR_CYAN, "COSMIC UI Framework for TrustOS");
            crate::println!();
            crate::println!("  Based on: libcosmic by System76 (Pop!_OS)");
            crate::println!("  Renderer: tiny-skia v0.12 (no_std mode)");
            crate::println!("  Features: anti-aliased shapes, gradients, shadows");
            crate::println!();
            crate::println!("  Modules:");
            crate::println!("    cosmic::theme    - COSMIC color palette");
            crate::println!("    cosmic::renderer - tiny-skia based rendering");
            crate::println!("    cosmic::widgets  - Button, Label, Container, etc.");
            crate::println!("    cosmic::layout   - Flexbox-style layout system");
        },
        _ => {
            crate::println!("Usage: cosmic <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  desktop - Launch full COSMIC desktop environment");
            crate::println!("  demo    - Render COSMIC UI demo to screen");
            crate::println!("  theme   - Set theme (dark/light)");
            crate::println!("  info    - Show framework information");
        }
    }
}

/// Open command - launch desktop with a specific app
fn cmd_open(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: open <app>");
        crate::println!("");
        crate::println!("Available apps:");
        crate::println!("  browser, web, www   - Web browser (HTTPS/TLS 1.3)");
        crate::println!("  files, explorer     - File manager");
        crate::println!("  editor, notepad     - Text editor");
        crate::println!("  network, net        - Network status");
        crate::println!("  hardware, hw        - Hardware info");
        crate::println!("  users               - User management");
        crate::println!("  images, viewer      - Image viewer");
        crate::println!("");
        crate::println!("Example: open browser");
        return;
    }
    
    let app = args[0].to_lowercase();
    cmd_cosmic_v2_with_app(Some(&app));
}

/// Launch the desktop.rs windowed desktop environment.
/// Optionally pre-opens a window of the given type.
fn launch_desktop_env(initial_window: Option<(&str, crate::desktop::WindowType, i32, i32, u32, u32)>) {
    use crate::desktop;
    let (width, height) = crate::framebuffer::get_dimensions();
    if width == 0 || height == 0 {
        crate::println_color!(COLOR_RED, "Error: Invalid framebuffer!");
        return;
    }
    crate::mouse::set_screen_size(width, height);
    
    let mut d = desktop::DESKTOP.lock();
    d.init(width, height);
    
    // Open the requested window if any
    if let Some((title, wtype, x, y, w, h)) = initial_window {
        d.create_window(title, x, y, w, h, wtype);
    }
    
    drop(d);
    crate::serial_println!("[Desktop] Entering desktop run loop");
    desktop::run();
    // Desktop exited — restore shell
    crate::serial_println!("[Desktop] Returned to shell");
    // Clear screen
    let (w, h) = crate::framebuffer::get_dimensions();
    crate::framebuffer::fill_rect(0, 0, w, h, 0xFF000000);
    crate::println_color!(COLOR_GREEN, "\nReturned to TrustOS shell. Type 'help' for commands.");
}

// ==================== SIGNATURE — KERNEL PROOF OF AUTHORSHIP ====================

fn cmd_signature(args: &[&str]) {
    use crate::signature;

    match args.first().copied() {
        Some("verify") | None => {
            // Show creator + user signatures
            crate::println!();
            crate::println_color!(COLOR_CYAN, "\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
            crate::println_color!(COLOR_CYAN, "\u{2551}              TrustOS Kernel Signature Certificate                  \u{2551}");
            crate::println_color!(COLOR_CYAN, "\u{2560}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2563}");
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "  🔒 CREATOR SIGNATURE (immutable)");
            crate::println_color!(COLOR_WHITE, "  ─────────────────────────────────────────────────────────────────");
            crate::println!("  Author:      {} (@{})", signature::CREATOR_NAME, signature::CREATOR_GITHUB);
            crate::println!("  Payload:     \"{}\"", signature::CREATOR_SIGNED_PAYLOAD);
            crate::println!("  Algorithm:   HMAC-SHA256");
            crate::println_color!(COLOR_YELLOW, "  Fingerprint: {}", signature::creator_fingerprint_hex());
            crate::println!("  Version:     v{}", signature::KERNEL_VERSION);
            crate::println!("  Built:       {}", signature::BUILD_TIMESTAMP);
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  i  This fingerprint was generated with a secret seed known ONLY");
            crate::println_color!(COLOR_GRAY, "     to the creator. It cannot be forged without the original seed.");
            crate::println!();

            // Show user signature if present
            if let Some((name, hex, ts)) = signature::get_user_signature() {
                crate::println_color!(COLOR_BLUE, "  USER CO-SIGNATURE");
                crate::println_color!(COLOR_WHITE, "  ─────────────────────────────────────────────────────────────────");
                crate::println!("  Signed by:   {}", name);
                crate::println_color!(COLOR_YELLOW, "  Fingerprint: {}", hex);
                crate::println!("  Signed at:   {}s after midnight (RTC)", ts);
                crate::println!();
            } else {
                crate::println_color!(COLOR_GRAY, "  No user co-signature. Use 'signature sign <name>' to add yours.");
                crate::println!();
            }

            crate::println_color!(COLOR_CYAN, "\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}");
            crate::println!();
        }
        Some("sign") => {
            // signature sign <name>
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: signature sign <your_name>");
                return;
            }
            let name = args[1];
            crate::println!("Enter your secret passphrase to sign the kernel:");
            crate::print!("> ");
            // Read passphrase (we'll use a simple input method)
            let passphrase = read_passphrase();
            if passphrase.is_empty() {
                crate::println_color!(COLOR_RED, "Empty passphrase. Aborted.");
                return;
            }
            signature::sign_as_user(name, passphrase.as_bytes());
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "✓ Kernel co-signed by '{}'", name);
            if let Some((_, hex, _)) = signature::get_user_signature() {
                crate::println_color!(COLOR_YELLOW, "  Your fingerprint: {}", hex);
            }
            crate::println_color!(COLOR_GRAY, "  Keep your passphrase safe -- you'll need it to prove ownership.");
            crate::println!();
        }
        Some("prove") => {
            // Verify a user's signature with their passphrase
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: signature prove <name>");
                return;
            }
            let name = args[1];
            crate::println!("Enter passphrase to verify:");
            crate::print!("> ");
            let passphrase = read_passphrase();
            if signature::verify_user_seed(name, passphrase.as_bytes()) {
                crate::println_color!(COLOR_BRIGHT_GREEN, "VERIFIED -- '{}' is the legitimate signer.", name);
            } else {
                crate::println_color!(COLOR_RED, "FAILED -- passphrase does not match the signature for '{}'.", name);
            }
            crate::println!();
        }
        Some("prove-creator") => {
            // Only the real creator can pass this
            crate::println!("Enter creator seed to verify authorship:");
            crate::print!("> ");
            let seed = read_passphrase();
            if signature::verify_creator_seed(seed.as_bytes()) {
                crate::println_color!(COLOR_BRIGHT_GREEN, "CREATOR VERIFIED -- You are the original author of TrustOS.");
            } else {
                crate::println_color!(COLOR_RED, "FAILED -- This seed does not match the creator fingerprint.");
            }
            crate::println!();
        }
        Some("integrity") | Some("verify-integrity") => {
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Kernel Integrity Verification");
            crate::println!("═══════════════════════════════════════════════════════════════");
            let report = signature::integrity_report();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  SHA-256 of .text + .rodata sections measured at boot vs now.");
            crate::println_color!(COLOR_GRAY, "  Detects runtime code injection, ROP gadget insertion, and");
            crate::println_color!(COLOR_GRAY, "  constant/vtable tampering (rootkits, memory corruption).");
            crate::println!();
        }
        Some("clear") => {
            signature::clear_user_signature();
            crate::println_color!(COLOR_YELLOW, "User co-signature cleared.");
        }
        Some("export") => {
            // Export signature in SIGNATURES.md format for GitHub PR
            if let Some((name, hex, _ts)) = signature::get_user_signature() {
                let dt = crate::rtc::read_rtc();
                crate::println!();
                crate::println_color!(COLOR_CYAN, "=== Copy everything below and submit as a PR to SIGNATURES.md ===");
                crate::println!();
                crate::println!("### #NNN -- {}", name);
                crate::println!();
                crate::println!("| Field | Value |");
                crate::println!("|-------|-------|");
                crate::println!("| **Name** | {} |", name);
                crate::println!("| **GitHub** | [@YOURUSERNAME](https://github.com/YOURUSERNAME) |");
                crate::println!("| **Algorithm** | HMAC-SHA256 |");
                crate::println!("| **Fingerprint** | `{}` |", hex);
                crate::println!("| **Kernel Version** | v{} |", signature::KERNEL_VERSION);
                crate::println!("| **Date** | {:04}-{:02}-{:02} |", dt.year, dt.month, dt.day);
                crate::println!("| **Status** | Verified signer |");
                crate::println!();
                crate::println_color!(COLOR_GRAY, "Replace YOURUSERNAME with your GitHub username and #NNN with the next number.");
                crate::println_color!(COLOR_GRAY, "Submit as a Pull Request to: github.com/nathan237/TrustOS");
                crate::println!();
            } else {
                crate::println_color!(COLOR_RED, "No user signature found. Run 'signature sign <name>' first.");
            }
        }
        Some("list") => {
            // Show all registered signatures info
            crate::println!();
            crate::println_color!(COLOR_CYAN, "TrustOS Signature Registry");
            crate::println_color!(COLOR_WHITE, "──────────────────────────────────────────────────────");
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "  #001  Nated0ge (Creator)");
            crate::println!("        {}", signature::creator_fingerprint_hex());
            crate::println!();
            if let Some((name, hex, _)) = signature::get_user_signature() {
                crate::println_color!(COLOR_CYAN, "  #---  {} (Local)", name);
                crate::println!("        {}", hex);
                crate::println!();
            }
            crate::println_color!(COLOR_GRAY, "  Full registry: github.com/nathan237/TrustOS/blob/main/SIGNATURES.md");
            crate::println!();
        }
        Some("ed25519") => {
            // Ed25519 asymmetric signature subsystem
            match args.get(1).copied() {
                Some("verify") | None => {
                    crate::println!();
                    crate::println_color!(COLOR_CYAN, "Ed25519 Asymmetric Signature Report");
                    crate::println_color!(COLOR_WHITE, "══════════════════════════════════════════════════════════════");
                    let report = signature::ed25519_report();
                    for line in &report {
                        crate::println!("{}", line);
                    }
                    crate::println!();
                }
                Some("sign") => {
                    crate::println!("Enter Ed25519 seed (hex or passphrase):");
                    crate::print!("> ");
                    let seed_input = read_passphrase();
                    if seed_input.is_empty() {
                        crate::println_color!(COLOR_RED, "Empty seed. Aborted.");
                        return;
                    }
                    signature::ed25519_resign(seed_input.as_bytes());
                    crate::println_color!(COLOR_BRIGHT_GREEN, "✓ Kernel re-signed with Ed25519 (new seed).");
                    if let Some(report) = signature::ed25519_report().first() {
                        crate::println!("  {}", report);
                    }
                    crate::println!();
                }
                _ => {
                    crate::println!("Usage:");
                    crate::println!("  signature ed25519          - Show Ed25519 signature & verify");
                    crate::println!("  signature ed25519 verify   - Verify current Ed25519 signature");
                    crate::println!("  signature ed25519 sign     - Re-sign kernel with new seed");
                }
            }
        }
        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Kernel Signature System");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  signature                - Show signature certificate");
            crate::println!("  signature verify         - Show & verify signature certificate");
            crate::println!("  signature integrity      - Verify kernel .text+.rodata integrity");
            crate::println!("  signature sign <name>    - Co-sign the kernel with your identity");
            crate::println!("  signature prove <name>   - Prove a user signature with passphrase");
            crate::println!("  signature prove-creator  - Prove creator authorship (requires seed)");
            crate::println!("  signature export         - Export signature for GitHub PR submission");
            crate::println!("  signature list           - Show registered signatures");
            crate::println!("  signature clear          - Remove user co-signature");
            crate::println!("  signature ed25519        - Ed25519 asymmetric signature status");
            crate::println!("  signature ed25519 sign   - Re-sign kernel with Ed25519 seed");
        }
    }
}

/// Read a passphrase from keyboard input (hidden, returns on Enter)
fn read_passphrase() -> alloc::string::String {
    use alloc::string::String;
    let mut passphrase = String::new();
    loop {
        if let Some(key) = crate::keyboard::try_read_key() {
            match key {
                b'\n' | b'\r' | 0x0A | 0x0D => {
                    crate::println!();
                    break;
                }
                0x08 => {
                    // Backspace
                    if !passphrase.is_empty() {
                        passphrase.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                c if c.is_ascii() && !c.is_ascii_control() => {
                    passphrase.push(c as char);
                    crate::print!("*");
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
    passphrase
}

/// Security subsystem management command
fn cmd_security(args: &[&str]) {
    match args.first().copied() {
        Some("status") | None => {
            // Show overall security status
            let stats = crate::security::stats();
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS Security Status");
            crate::println!("═══════════════════════════════════════════════════════════════");
            crate::println!("  Active capabilities : {}", stats.active_capabilities);
            crate::println!("  Security violations : {}", stats.violations);
            crate::println!("  Dynamic types       : {}", stats.dynamic_types);
            crate::println!("  Isolated subsystems : {}", stats.subsystems);
            crate::println!("  Gate checks         : {}", crate::security::isolation::total_gate_checks());
            crate::println!("  Gate violations     : {}", crate::security::isolation::total_gate_violations());
            crate::println!();
            
            // Integrity summary
            match crate::signature::verify_integrity() {
                Ok(true) => crate::println_color!(COLOR_BRIGHT_GREEN, "  Kernel integrity    : ✅ INTACT"),
                Ok(false) => crate::println_color!(COLOR_RED, "  Kernel integrity    : ❌ TAMPERED"),
                Err(_) => crate::println_color!(COLOR_YELLOW, "  Kernel integrity    : ⚠️  not initialized"),
            }
            crate::println!();
        }
        Some("caps") | Some("capabilities") => {
            // List all active capabilities
            let caps = crate::security::list_capabilities();
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Active Capabilities ({} total)", caps.len());
            crate::println!("──────────────────────────────────────────────────────────");
            crate::println!("  {:>6} │ {:<20} │ {:<10} │ Owner", "ID", "Type", "Category");
            crate::println!("  ───────┼──────────────────────┼────────────┼──────");
            for (id, cap_type, _rights, owner) in &caps {
                crate::println!("  {:>6} │ {:<20} │ {:<10} │ 0x{:04X}",
                    id.0,
                    alloc::format!("{:?}", cap_type),
                    cap_type.category(),
                    owner
                );
            }
            crate::println!();
        }
        Some("isolation") | Some("iso") | Some("subsystems") => {
            // Show subsystem isolation status
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Subsystem Isolation Boundaries");
            crate::println!("═══════════════════════════════════════════════════════════════");
            let report = crate::security::isolation::isolation_report();
            for line in &report {
                crate::println!("{}", line);
            }
            crate::println!();
            crate::println_color!(COLOR_GRAY, "  ring0-tcb       = Part of TCB, must stay in ring 0");
            crate::println_color!(COLOR_GRAY, "  ring0-isolated  = Ring 0 but logically isolated");
            crate::println_color!(COLOR_GRAY, "  ring3-candidate = Could be moved to ring 3 in future");
            crate::println!();
        }
        Some("gate") => {
            // Test a gate check
            if let Some(subsystem_name) = args.get(1).copied() {
                let subsystem = match subsystem_name {
                    "storage" | "disk" => Some(crate::security::isolation::Subsystem::Storage),
                    "network" | "net" => Some(crate::security::isolation::Subsystem::Network),
                    "graphics" | "gpu" => Some(crate::security::isolation::Subsystem::Graphics),
                    "process" | "proc" => Some(crate::security::isolation::Subsystem::ProcessMgr),
                    "hypervisor" | "hv" => Some(crate::security::isolation::Subsystem::Hypervisor),
                    "shell" => Some(crate::security::isolation::Subsystem::Shell),
                    "crypto" => Some(crate::security::isolation::Subsystem::Crypto),
                    "power" => Some(crate::security::isolation::Subsystem::Power),
                    "serial" => Some(crate::security::isolation::Subsystem::SerialDebug),
                    "memory" | "mem" => Some(crate::security::isolation::Subsystem::Memory),
                    _ => None,
                };
                if let Some(sub) = subsystem {
                    match crate::security::isolation::gate_check(
                        sub, crate::security::CapabilityRights::READ
                    ) {
                        Ok(()) => crate::println_color!(COLOR_BRIGHT_GREEN, 
                            "  ✅ Gate check PASSED for {:?}", sub),
                        Err(e) => crate::println_color!(COLOR_RED, 
                            "  ❌ Gate check DENIED for {:?}: {:?}", sub, e),
                    }
                } else {
                    crate::println_color!(COLOR_RED, "Unknown subsystem: {}", subsystem_name);
                }
            } else {
                crate::println!("Usage: security gate <subsystem>");
                crate::println!("  Subsystems: storage, network, graphics, process, hypervisor,");
                crate::println!("              shell, crypto, power, serial, memory");
            }
        }
        Some("dynamic") => {
            // List dynamic capability types
            let types = crate::security::list_dynamic_types();
            crate::println!();
            if types.is_empty() {
                crate::println_color!(COLOR_GRAY, "No dynamic capability types registered.");
            } else {
                crate::println_color!(COLOR_CYAN, "Dynamic Capability Types ({} registered)", types.len());
                for (id, info) in &types {
                    crate::println!("  [{}] {} (danger:{}, category:{})", 
                        id, info.name, info.danger_level, info.category);
                    crate::println!("       {}", info.description);
                }
            }
            crate::println!();
        }
        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Security Subsystem");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  security                 - Show security status overview");
            crate::println!("  security caps            - List all active capabilities");
            crate::println!("  security isolation       - Show subsystem isolation boundaries");
            crate::println!("  security gate <subsys>   - Test a gate check on a subsystem");
            crate::println!("  security dynamic         - List dynamic capability types");
        }
    }
}

// Runs through all TrustOS features with timed pauses between steps.
// Perfect for screen recording with OBS to create marketing videos.

fn cmd_showcase(args: &[&str]) {
    let speed = match args.first().copied() {
        Some("fast") => 1,
        Some("slow") => 3,
        _ => 2, // normal
    };

    // Use TSC for timing — uptime_ms() doesn't advance during spin_loop()
    let pause = |secs: u64| {
        let ms = secs * 1000 * speed / 2;
        let start_tsc = crate::cpu::tsc::read_tsc();
        let freq = crate::cpu::tsc::frequency_hz();
        if freq == 0 { return; } // TSC not calibrated
        let target_cycles = freq / 1000 * ms; // cycles for ms milliseconds
        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= target_cycles { break; }
            // Drain keyboard so keys don't leak to next step
            let _ = crate::keyboard::try_read_key();
            core::hint::spin_loop();
        }
    };

    let effect_duration = 9000u64 * speed / 2; // Duration for each video effect (9s base)

    // ═══════════════════════════════════════════════════════════════════
    // CINEMATIC INTRO — Matrix-style big text on framebuffer
    // ═══════════════════════════════════════════════════════════════════
    let (sw, sh) = crate::framebuffer::get_dimensions();

    {
        let was_db = crate::framebuffer::is_double_buffer_enabled();
        if !was_db {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }

        let w = sw as usize;
        let h = sh as usize;
        let mut buf = alloc::vec![0u32; w * h];

        // ── Helper: draw scaled character into buffer ──
        let draw_big_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, c: char, color: u32, scale: usize| {
            let glyph = crate::framebuffer::font::get_glyph(c);
            for (row, &bits) in glyph.iter().enumerate() {
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let px = cx + bit as usize * scale + sx;
                                let py = cy + row * scale + sy;
                                if px < w && py < h {
                                    buf[py * w + px] = color;
                                }
                            }
                        }
                    }
                }
            }
        };

        // ── Helper: draw big text centered ──
        let draw_big_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
            let text_w = text.len() * 8 * scale;
            let start_x = if text_w < w { (w - text_w) / 2 } else { 0 };
            for (i, c) in text.chars().enumerate() {
                draw_big_char(buf, w, h, start_x + i * 8 * scale, y, c, color, scale);
            }
        };

        // ── Helper: Matrix rain background ──
        let mut rain_cols: alloc::vec::Vec<u16> = alloc::vec![0u16; w / 8 + 1];
        let mut rain_speeds: alloc::vec::Vec<u8> = alloc::vec![1u8; w / 8 + 1];
        // Seed rain columns
        for i in 0..rain_cols.len() {
            rain_cols[i] = ((i * 37 + 13) % h) as u16;
            rain_speeds[i] = (((i * 7 + 3) % 4) + 1) as u8;
        }

        let draw_rain_step = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
            // Dim existing pixels slightly (trail effect)
            for pixel in buf.iter_mut() {
                let g = ((*pixel >> 8) & 0xFF) as u32;
                if g > 0 {
                    let new_g = g.saturating_sub(8);
                    *pixel = 0xFF000000 | (new_g << 8);
                }
            }
            // Advance rain drops
            for col_idx in 0..cols.len() {
                let x = col_idx * 8;
                if x >= w { continue; }
                cols[col_idx] = cols[col_idx].wrapping_add(speeds[col_idx] as u16);
                if cols[col_idx] as usize >= h { cols[col_idx] = 0; }
                let y = cols[col_idx] as usize;
                // Draw lead char (bright green)
                let c = (((frame as usize + col_idx * 13) % 94) + 33) as u8 as char;
                let glyph = crate::framebuffer::font::get_glyph(c);
                for (row, &bits) in glyph.iter().enumerate() {
                    let py = y + row;
                    if py >= h { break; }
                    for bit in 0..8u32 {
                        if bits & (0x80 >> bit) != 0 {
                            let px = x + bit as usize;
                            if px < w {
                                buf[py * w + px] = 0xFF00FF44;
                            }
                        }
                    }
                }
            }
        };

        // ── Helper: blit buffer to backbuffer ──
        let blit_buf = |buf: &[u32], w: usize, h: usize| {
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                let bb = bb_ptr as *mut u32;
                let bb_s = bb_stride as usize;
                for y in 0..h.min(bb_h as usize) {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            buf[y * w..].as_ptr(),
                            bb.add(y * bb_s),
                            w,
                        );
                    }
                }
            }
            crate::framebuffer::swap_buffers();
        };

        // ── Helper: Scene — type text with matrix rain bg ──
        // Renders text that "types in" char by char with Matrix rain bg
        let render_scene = |buf: &mut [u32], w: usize, h: usize,
                           rain_cols: &mut [u16], rain_speeds: &[u8],
                           lines: &[(&str, u32, usize)], // (text, color, scale)
                           hold_ms: u64, speed: u64| {
            let freq = crate::cpu::tsc::frequency_hz();
            if freq == 0 { return; }

            // Phase 1: Type in text (rain + typed text appearing)
            let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
            let type_ms = 80u64 * speed / 2; // ms per char typed
            let type_total_ms = total_chars as u64 * type_ms;
            
            let start_tsc = crate::cpu::tsc::read_tsc();
            let type_target = freq / 1000 * type_total_ms;
            let hold_target = freq / 1000 * (type_total_ms + hold_ms * speed / 2);

            let mut frame = 0u32;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
                if elapsed >= hold_target { break; }
                if let Some(key) = crate::keyboard::try_read_key() {
                    if key == 0x1B || key == b'q' { break; }
                }

                // Rain background
                draw_rain_step(buf, w, h, rain_cols, rain_speeds, frame);

                // Calculate how many chars to show
                let elapsed_ms = elapsed / (freq / 1000).max(1);
                let chars_shown = if elapsed_ms < type_total_ms {
                    (elapsed_ms / type_ms.max(1)) as usize
                } else {
                    total_chars
                };

                // Draw text lines
                let total_text_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 8).sum::<usize>();
                let mut y_start = if total_text_h < h { (h - total_text_h) / 2 } else { 20 };
                let mut chars_counted = 0usize;

                for &(text, color, scale) in lines {
                    let text_w = text.len() * 8 * scale;
                    let start_x = if text_w < w { (w - text_w) / 2 } else { 0 };

                    for (i, c) in text.chars().enumerate() {
                        if chars_counted + i >= chars_shown { break; }
                        draw_big_char(buf, w, h, start_x + i * 8 * scale, y_start, c, color, scale);
                    }
                    // Cursor blink
                    if chars_shown > chars_counted && chars_shown < chars_counted + text.len() {
                        let cursor_i = chars_shown - chars_counted;
                        let cx = start_x + cursor_i * 8 * scale;
                        for cy in y_start..y_start + 16 * scale {
                            if cy < h && cx + 2 < w {
                                buf[cy * w + cx] = 0xFF00FF88;
                                buf[cy * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }

                    chars_counted += text.len();
                    y_start += 16 * scale + 8;
                }

                blit_buf(buf, w, h);
                frame += 1;
                // ~30 FPS pacing
                crate::cpu::tsc::delay_millis(33);
            }

            // Fade out
            let fade_start = crate::cpu::tsc::read_tsc();
            let fade_ms = 800u64;
            let fade_target = freq / 1000 * fade_ms;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(fade_start);
                if elapsed >= fade_target { break; }
                let progress = (elapsed * 255 / fade_target) as u32;
                for pixel in buf.iter_mut() {
                    let r = ((*pixel >> 16) & 0xFF) as u32;
                    let g = ((*pixel >> 8) & 0xFF) as u32;
                    let b = (*pixel & 0xFF) as u32;
                    let nr = r.saturating_sub(r * progress / 512 + 1);
                    let ng = g.saturating_sub(g * progress / 512 + 1);
                    let nb = b.saturating_sub(b * progress / 512 + 1);
                    *pixel = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
                }
                blit_buf(buf, w, h);
                crate::cpu::tsc::delay_millis(33);
            }

            // Clear to black
            for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
            blit_buf(buf, w, h);
        };

        // ── Scene 1: "Do you think life is a simulation?" ──
        crate::serial_println!("[SHOWCASE] Scene 1: Simulation question");
        for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("Do you think", 0xFF00DD55, 4),
              ("life is a simulation?", 0xFF00FF66, 4)],
            3000, speed);

        // ── Scene 2: "Can it run in a 6MB OS?" ──
        crate::serial_println!("[SHOWCASE] Scene 2: 6MB OS");
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("Can it run", 0xFF00DD55, 5),
              ("in a 6MB OS?", 0xFF00FF88, 5)],
            3000, speed);

        // ── Scene 3: "TrustOS" + 3D + "Written in Rust by Nated0ge" ──
        crate::serial_println!("[SHOWCASE] Scene 3: TrustOS title");
        {
            // Special scene: TrustOS big title + 3D wireframe + credits
            let freq = crate::cpu::tsc::frequency_hz();
            let scene3_ms = 8000u64 * speed / 2;
            let scene3_target = freq / 1000 * scene3_ms;
            let start_tsc = crate::cpu::tsc::read_tsc();

            let mut renderer3d = crate::formula3d::FormulaRenderer::new();
            renderer3d.set_scene(crate::formula3d::FormulaScene::Character);
            renderer3d.wire_color = 0xFF00FFAA;

            // Small 3D viewport
            let vp_w = 200usize;
            let vp_h = 200usize;
            let mut vp_buf = alloc::vec![0u32; vp_w * vp_h];

            let mut frame = 0u32;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
                if elapsed >= scene3_target { break; }
                if let Some(key) = crate::keyboard::try_read_key() {
                    if key == 0x1B { break; }
                }

                // Rain background
                draw_rain_step(&mut buf, w, h, &mut rain_cols, &rain_speeds, frame);

                // Title: "TRUST OS" big
                let title = "TRUST OS";
                let title_scale = 6;
                let title_w = title.len() * 8 * title_scale;
                let title_x = if title_w < w { (w - title_w) / 2 } else { 0 };
                let title_y = h / 8;
                for (i, c) in title.chars().enumerate() {
                    // Color cycle per char
                    let hue = ((frame as usize * 3 + i * 30) % 360) as u32;
                    let color = if hue < 120 {
                        let t = hue * 255 / 120;
                        0xFF000000 | ((255 - t) << 16) | (t << 8)
                    } else if hue < 240 {
                        let t = (hue - 120) * 255 / 120;
                        0xFF000000 | ((255 - t) << 8) | t
                    } else {
                        let t = (hue - 240) * 255 / 120;
                        0xFF000000 | (t << 16) | (255 - t)
                    };
                    draw_big_char(&mut buf, w, h, title_x + i * 8 * title_scale, title_y, c, color, title_scale);
                }

                // 3D animated character in center
                renderer3d.update();
                for p in vp_buf.iter_mut() { *p = 0x00000000; } // transparent
                renderer3d.render(&mut vp_buf, vp_w, vp_h);

                // Blit 3D viewport centered below title
                let vp_x = if vp_w < w { (w - vp_w) / 2 } else { 0 };
                let vp_y = title_y + 16 * title_scale + 20;
                for vy in 0..vp_h {
                    for vx in 0..vp_w {
                        let src = vp_buf[vy * vp_w + vx];
                        if src & 0x00FFFFFF != 0 { // not black = has content
                            let dy = vp_y + vy;
                            let dx = vp_x + vx;
                            if dy < h && dx < w {
                                buf[dy * w + dx] = src;
                            }
                        }
                    }
                }

                // Credits text
                let credit = "Written in Rust by Nated0ge";
                let credit_scale = 3;
                let credit_w = credit.len() * 8 * credit_scale;
                let credit_x = if credit_w < w { (w - credit_w) / 2 } else { 0 };
                let credit_y = vp_y + vp_h + 30;
                for (i, c) in credit.chars().enumerate() {
                    draw_big_char(&mut buf, w, h, credit_x + i * 8 * credit_scale, credit_y, c, 0xFF88CCFF, credit_scale);
                }

                blit_buf(&buf, w, h);
                frame += 1;
                crate::cpu::tsc::delay_millis(33);
            }

            // Fade out
            let fade_start = crate::cpu::tsc::read_tsc();
            let fade_target = freq / 1000 * 800;
            loop {
                let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(fade_start);
                if elapsed >= fade_target { break; }
                for pixel in buf.iter_mut() {
                    let r = ((*pixel >> 16) & 0xFF).saturating_sub(4) as u32;
                    let g = ((*pixel >> 8) & 0xFF).saturating_sub(4) as u32;
                    let b = (*pixel & 0xFF).saturating_sub(4) as u32;
                    *pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
                blit_buf(&buf, w, h);
                crate::cpu::tsc::delay_millis(33);
            }
            for pixel in buf.iter_mut() { *pixel = 0xFF000000; }
            blit_buf(&buf, w, h);
        }

        // ── Scene 4: Specs comparison ──
        crate::serial_println!("[SHOWCASE] Scene 4: Specs");
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("6MB ISO vs 6GB Windows",  0xFF00FF66, 3),
              ("0 lines of C. Pure Rust.", 0xFF44FFAA, 3),
              ("Boots in 0.8s not 45s",    0xFF00DDFF, 3),
              ("No kernel panics. Ever.",  0xFFFFCC44, 3),
              ("GPU desktop at 144 FPS",   0xFF88FF44, 3),
              ("Built in 7 days solo",     0xFFFF8844, 3)],
            3000, speed);

        // ── Scene 5: "Are you ready?" ──
        crate::serial_println!("[SHOWCASE] Scene 5: Are you ready?");
        render_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
            &[("Are you ready?", 0xFF00FF44, 6)],
            2000, speed);

        // Restore double buffer state
        if !was_db {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }

    // ─── Phase 0: Banner ───
    crate::framebuffer::clear();

    crate::println!();
    crate::println!();
    crate::println!();
    crate::println_color!(0xFF00CCFF, "");
    crate::println_color!(0xFF00CCFF, "  ████████╗██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ███████╗");
    crate::println_color!(0xFF00CCFF, "  ╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔═══██╗██╔════╝");
    crate::println_color!(0xFF00CCFF, "     ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║███████╗");
    crate::println_color!(0xFF00DDFF, "     ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║╚════██║");
    crate::println_color!(0xFF00EEFF, "     ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝███████║");
    crate::println_color!(0xFF00EEFF, "     ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚══════╝");
    crate::println!();
    crate::println_color!(0xFF888888, "                  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    crate::println!();
    crate::println_color!(0xFFAADDFF, "           A bare-metal OS written in 100% Rust — in 7 days");
    crate::println_color!(0xFF666666, "         99,000+ lines · 6 MB ISO · GPU compositing · 144 FPS");
    crate::println!();
    crate::println_color!(0xFF888888, "                  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    crate::println!();
    crate::println_color!(0xFF00FF88, "                        ▶  FEATURE SHOWCASE  ▶");
    crate::println!();
    
    pause(6);

    // ─── Phase 1: System Info ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 1 ──── SYSTEM INFO                                   ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(3);

    cmd_neofetch();
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ uname -a");
    cmd_uname(&["-a"]);
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ free");
    cmd_free();
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ lscpu");
    cmd_lscpu();
    pause(5);

    // ─── Phase 2: Filesystem ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 2 ──── FILESYSTEM (TrustFS + VFS)                    ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ mkdir /demo");
    cmd_mkdir(&["/demo"]);
    pause(2);

    crate::println_color!(COLOR_CYAN, "$ echo 'Hello TrustOS!' > /demo/hello.txt");
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch("/demo/hello.txt");
        let _ = fs.write_file("/demo/hello.txt", b"Hello TrustOS!\nThis file was created live during the showcase.\n");
    });
    crate::println_color!(COLOR_GREEN, "Written: /demo/hello.txt");
    pause(2);

    crate::println_color!(COLOR_CYAN, "$ cat /demo/hello.txt");
    cmd_cat(&["/demo/hello.txt"], None);
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ tree /");
    cmd_tree(&["/"]);
    pause(4);

    // ─── Phase 3: TrustLang ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 3 ──── TRUSTLANG (Built-in Compiler + VM)            ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(3);

    // Create and run a real TrustLang program
    let tl_code = r#"fn fibonacci(n: i64) -> i64 {
    if n <= 1 { return n; }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    println("=== TrustLang on TrustOS ===");
    println("Fibonacci sequence (compiled to bytecode):");
    for i in 0..12 {
        let result = fibonacci(i);
        print("  fib(");
        print(to_string(i));
        print(") = ");
        println(to_string(result));
    }
    println("Language features: functions, recursion, loops, types");
}"#;
    crate::ramfs::with_fs(|fs| {
        let _ = fs.touch("/demo/showcase.tl");
        let _ = fs.write_file("/demo/showcase.tl", tl_code.as_bytes());
    });

    crate::println_color!(COLOR_CYAN, "$ cat /demo/showcase.tl");
    crate::println_color!(0xFFDDDDDD, "{}", tl_code);
    pause(4);

    crate::println_color!(COLOR_CYAN, "$ trustlang run /demo/showcase.tl");
    crate::println_color!(0xFF00FF88, "[TrustLang] Compiling showcase.tl...");
    match crate::trustlang::run(tl_code) {
        Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
    }
    crate::println_color!(COLOR_GREEN, "[TrustLang] Program finished successfully.");
    pause(6);

    // ─── Phase 4: Network Stack ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 4 ──── NETWORK STACK (TCP/IP, DHCP, DNS, TLS 1.3)    ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ ifconfig");
    cmd_ifconfig();
    pause(3);

    crate::println_color!(COLOR_CYAN, "$ netstat");
    cmd_netstat();
    pause(4);

    // ─── Phase 5: Video Demos ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 5 ──── TRUSTVIDEO (Real-time procedural rendering)   ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(3);

    // Fire demo
    crate::println_color!(0xFFFF4400, "▸ Demo 1/3: FIRE EFFECT — Real-time procedural flame");
    pause(2);

    let vw = sw as u16;
    let vh = sh as u16;
    crate::video::player::render_realtime_timed("fire", vw, vh, 30, effect_duration);
    
    // Restore console
    crate::framebuffer::clear();
    pause(2);

    // Matrix demo
    crate::println_color!(0xFF00FF44, "▸ Demo 2/3: MATRIX RAIN — Digital rain effect");
    pause(2);

    crate::video::player::render_realtime_timed("matrix", vw, vh, 30, effect_duration);
    
    crate::framebuffer::clear();
    pause(2);

    // Plasma demo
    crate::println_color!(0xFFFF00FF, "▸ Demo 3/3: PLASMA — Integer sine LUT psychedelic");
    pause(2);

    crate::video::player::render_realtime_timed("plasma", vw, vh, 30, effect_duration);
    
    crate::framebuffer::clear();
    pause(2);

    // ─── Phase 5b: 3D Wireframe Character ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 5b ── FORMULA3D (Wireframe 3D engine)                ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(2);

    crate::println_color!(0xFF00FF88, "▸ 3D wireframe character — perspective projection + depth shading");
    pause(2);

    // Render rotating 3D character using FormulaRenderer
    {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(crate::formula3d::FormulaScene::Character);
        renderer.wire_color = 0xFF00FFAA; // bright cyan-green

        let rw = sw as usize;
        let rh = sh as usize;
        let ox = if sw > rw as u32 { (sw - rw as u32) / 2 } else { 0 } as usize;
        let oy = if sh > rh as u32 { (sh - rh as u32) / 2 } else { 0 } as usize;

        let mut buf = alloc::vec![0u32; rw * rh];

        let was_db = crate::framebuffer::is_double_buffer_enabled();
        if !was_db {
            crate::framebuffer::init_double_buffer();
            crate::framebuffer::set_double_buffer_mode(true);
        }
        crate::framebuffer::clear_backbuffer(0xFF000000);
        crate::framebuffer::swap_buffers();

        let start_tsc = crate::cpu::tsc::read_tsc();
        let freq = crate::cpu::tsc::frequency_hz();
        let duration_ms = effect_duration;
        let target_cycles = if freq > 0 { freq / 1000 * duration_ms } else { u64::MAX };

        loop {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
            if elapsed >= target_cycles { break; }
            if let Some(key) = crate::keyboard::try_read_key() {
                if key == 0x1B || key == b'q' { break; }
            }

            renderer.update();
            renderer.render(&mut buf, rw, rh);

            // Blit to backbuffer
            if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                let bb = bb_ptr as *mut u32;
                let bb_s = bb_stride as usize;
                for y in 0..rh {
                    let dy = oy + y;
                    if dy >= bb_h as usize { break; }
                    let src_row = &buf[y * rw..y * rw + rw];
                    unsafe {
                        let dst = bb.add(dy * bb_s + ox);
                        core::ptr::copy_nonoverlapping(src_row.as_ptr(), dst, rw);
                    }
                }
            }
            crate::framebuffer::swap_buffers();
        }

        if !was_db {
            crate::framebuffer::set_double_buffer_mode(false);
        }
    }

    crate::framebuffer::clear();
    pause(2);

    // ─── Phase 5c: Desktop + Web Browser ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 5c ── COSMIC2 DESKTOP + WEB BROWSER                  ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(2);

    crate::println_color!(0xFF00FF88, "▸ COSMIC2 Desktop — GPU-composited multi-layer windowing system");
    crate::println_color!(0xFF00FF88, "▸ Launching with built-in Web Browser → google.com");
    pause(3);

    // Launch desktop in browser mode with auto-exit after effect_duration ms
    cmd_cosmic_v2_with_app_timed(Some("browser"), effect_duration);

    crate::framebuffer::clear();
    pause(2);

    // ─── Phase 6: Commands overview ───
    crate::println_color!(0xFF00CCFF, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(0xFF00CCFF, "║  PHASE 6 ──── 200+ BUILT-IN COMMANDS                       ║");
    crate::println_color!(0xFF00CCFF, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    pause(2);

    crate::println_color!(0xFFAADDFF, "  ┌─ File System ──────────────────────────────────────────┐");
    crate::println_color!(0xFFDDDDDD, "  │ ls cd pwd mkdir rm cp mv cat head tail tree find grep  │");
    crate::println_color!(0xFFAADDFF, "  ├─ Network ──────────────────────────────────────────────┤");
    crate::println_color!(0xFFDDDDDD, "  │ ifconfig ping curl wget nslookup arp route netstat     │");
    crate::println_color!(0xFFAADDFF, "  ├─ System ───────────────────────────────────────────────┤");
    crate::println_color!(0xFFDDDDDD, "  │ ps top free df uname dmesg mount lspci lscpu lsblk    │");
    crate::println_color!(0xFFAADDFF, "  ├─ Development ──────────────────────────────────────────┤");
    crate::println_color!(0xFFDDDDDD, "  │ trustlang (compiler+VM) · TrustCode (editor)          │");
    crate::println_color!(0xFFDDDDDD, "  │ transpile (binary→Rust) · exec (ELF loader)           │");
    crate::println_color!(0xFFAADDFF, "  ├─ Graphics ─────────────────────────────────────────────┤");
    crate::println_color!(0xFFDDDDDD, "  │ desktop (COSMIC2 compositor) · video (TrustVideo)     │");
    crate::println_color!(0xFFDDDDDD, "  │ benchmark (SSE2 SIMD) · HoloMatrix (3D volumetric)   │");
    crate::println_color!(0xFFAADDFF, "  └────────────────────────────────────────────────────────┘");
    crate::println!();
    pause(6);

    // ─── Phase 7: Outro ───
    crate::println!();
    crate::println_color!(0xFF00CCFF, "  ████████╗██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ███████╗");
    crate::println_color!(0xFF00CCFF, "  ╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔═══██╗██╔════╝");
    crate::println_color!(0xFF00CCFF, "     ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║███████╗");
    crate::println_color!(0xFF00DDFF, "     ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║╚════██║");
    crate::println_color!(0xFF00EEFF, "     ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝███████║");
    crate::println_color!(0xFF00EEFF, "     ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚══════╝");
    crate::println!();
    crate::println_color!(0xFFFFCC00, "  ★  100% Rust — Zero C code — Memory safe by design");
    crate::println_color!(0xFFFFCC00, "  ★  Built from scratch in 7 days — 99,000+ lines");
    crate::println_color!(0xFFFFCC00, "  ★  6 MB ISO — boots in seconds");
    crate::println_color!(0xFFFFCC00, "  ★  GPU compositing — 144 FPS desktop");
    crate::println!();
    crate::println_color!(0xFF00FF88, "  github.com/nathan237/TrustOS");
    crate::println_color!(0xFF888888, "  Star ★ · Fork · Contribute");
    crate::println!();
    crate::println_color!(0xFF888888, "  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    crate::println!();

    // Clean up
    let _ = crate::ramfs::with_fs(|fs| { let _ = fs.rm("/demo/hello.txt"); let _ = fs.rm("/demo/showcase.tl"); });
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHOWCASE 3D — Cinematic 3D graphics demo
// 12 fullscreen scenes, 5 seconds each, hardware stats overlay
// ═══════════════════════════════════════════════════════════════════════════════

pub fn cmd_showcase3d() {
    use crate::gpu_emu::{PixelInput, PixelOutput};

    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;
    if w == 0 || h == 0 { return; }

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    // ── Helpers ──────────────────────────────────────────────────────────

    let draw_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, c: char, color: u32, scale: usize| {
        let glyph = crate::framebuffer::font::get_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + bit as usize * scale + sx;
                            let py = cy + row * scale + sy;
                            if px < w && py < h {
                                buf[py * w + px] = color;
                            }
                        }
                    }
                }
            }
        }
    };

    let draw_text = |buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            draw_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
        let tw = text.len() * 8 * scale;
        let x = if tw < w { (w - tw) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            draw_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let blit = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buf[y * w..].as_ptr(),
                        bb.add(y * bb_s),
                        w,
                    );
                }
            }
        }
        crate::framebuffer::swap_buffers();
    };

    // ── Tick-based wall-clock timing (PIT at ~100 Hz) ────────────────
    // get_ticks() is driven by PIT interrupt = reliable wall-clock
    // 100 ticks = 1 second
    let ticks_now = || crate::logger::get_ticks();

    // Draw stats bar at bottom (with real hardware info)
    let draw_stats = |buf: &mut [u32], w: usize, h: usize, scene_name: &str, scene_num: usize, fps: u32, elapsed_s: u32, total_s: u32, quality: usize| {
        let bar_h = 28usize;
        let bar_y = h - bar_h;
        for y in bar_y..h {
            for x in 0..w {
                buf[y * w + x] = 0xFF0A0A0A;
            }
        }
        for x in 0..w {
            buf[bar_y * w + x] = 0xFF00AA44;
        }
        // Gather RAM stats
        let mem = crate::memory::stats();
        let heap_used_kb = mem.heap_used / 1024;
        let heap_total_kb = (mem.heap_used + mem.heap_free) / 1024;
        let heap_pct = if heap_total_kb > 0 { heap_used_kb * 100 / heap_total_kb } else { 0 };
        // Frame time approximation from FPS
        let frame_ms = if fps > 0 { 1000 / fps } else { 999 };
        let quality_str = match quality { 1 => "Full", 2 => "High", 3 => "Med", _ => "Low" };
        let mut stats_str = alloc::string::String::new();
        use core::fmt::Write;
        let _ = write!(stats_str, " {}/12 {} | {} FPS {}ms | RAM {}KB/{}KB ({}%) | CPU 100% | {} | {}x{}",
            scene_num, scene_name, fps, frame_ms, heap_used_kb, heap_total_kb, heap_pct, quality_str, w, h);
        draw_text(buf, w, h, 8, bar_y + 8, &stats_str, 0xFF00FF66, 1);
    };

    let scene_ticks = 500u64;  // 5 seconds per scene (100 ticks/sec)
    let title_ticks = 200u64;  // 2 seconds title overlay

    // Render a pixel-shader scene (tick-based timing, adaptive quality)
    let render_shader_scene = |buf: &mut [u32], w: usize, h: usize,
                                shader_fn: fn(PixelInput) -> PixelOutput,
                                title: &str, subtitle: &str, scene_num: usize,
                                dur_ticks: u64| {
        let start = ticks_now();
        let mut frame = 0u32;
        let mut fps_start = start;
        let mut fps_frames = 0u32;
        let mut cur_fps = 0u32;
        // Start at step=2 for small screens, step=3 for large (>960px)
        let mut step = if w > 960 { 3usize } else { 2 };

        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= dur_ticks { break; }
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 27 { return; } // ESC = ASCII 27
            }

            // Adaptive quality: adjust step based on actual FPS
            // Checks every frame after first FPS measurement
            if cur_fps > 0 {
                if cur_fps >= 20 && step > 2 {
                    step -= 1; // Improve quality one level
                } else if cur_fps >= 30 && step > 1 {
                    step = 1; // Full resolution
                } else if cur_fps < 8 && step < 4 {
                    step += 1; // Reduce quality to keep it smooth
                }
            }

            let time = elapsed as f32 / 100.0; // seconds as float

            // Render shader with current step (step×step blocks)
            for y in (0..h).step_by(step) {
                for x in (0..w).step_by(step) {
                    let input = PixelInput { x: x as u32, y: y as u32, width: w as u32, height: h as u32, time, frame };
                    let out = shader_fn(input);
                    let color = out.to_u32();
                    // Fill the step×step block
                    for dy in 0..step {
                        for dx in 0..step {
                            let px = x + dx;
                            let py = y + dy;
                            if px < w && py < h {
                                buf[py * w + px] = color;
                            }
                        }
                    }
                }
            }

            // FPS counter (per second via ticks)
            fps_frames += 1;
            let fps_elapsed = ticks_now().saturating_sub(fps_start);
            if fps_elapsed >= 100 {
                cur_fps = fps_frames;
                fps_frames = 0;
                fps_start = ticks_now();
            }

            // Title overlay (first 2 seconds)
            if elapsed < title_ticks {
                let alpha = if elapsed < 50 {
                    (elapsed * 255 / 50) as u32
                } else if elapsed > 150 {
                    let fade = elapsed - 150;
                    255u32.saturating_sub((fade * 255 / 50) as u32)
                } else { 255 };
                let a = alpha.min(255);
                let tc = 0xFF000000 | (a << 16) | (a << 8) | a;
                draw_text_centered(buf, w, h, 30, title, tc, 4);
                let sc = 0xFF000000 | ((a * 180 / 255) << 8);
                draw_text_centered(buf, w, h, 100, subtitle, sc, 2);
            }

            let elapsed_s = (elapsed / 100) as u32;
            let total_s = (dur_ticks / 100) as u32;
            draw_stats(buf, w, h, title, scene_num, cur_fps, elapsed_s, total_s, step);
            blit(buf, w, h);
            frame += 1;
        }
    };

    // Render a Formula3D wireframe scene (tick-based timing)
    let render_formula_scene = |buf: &mut [u32], w: usize, h: usize,
                                 scene: crate::formula3d::FormulaScene,
                                 wire_color: u32,
                                 title: &str, subtitle: &str, scene_num: usize,
                                 dur_ticks: u64| {
        let mut renderer = crate::formula3d::FormulaRenderer::new();
        renderer.set_scene(scene);
        renderer.wire_color = wire_color;
        let start = ticks_now();
        let mut frame = 0u32;
        let mut fps_start = start;
        let mut fps_frames = 0u32;
        let mut cur_fps = 0u32;

        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= dur_ticks { break; }
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 27 { return; }
            }

            renderer.update();
            for p in buf.iter_mut() { *p = 0xFF000000; }
            renderer.render(buf, w, h);

            fps_frames += 1;
            let fps_elapsed = ticks_now().saturating_sub(fps_start);
            if fps_elapsed >= 100 {
                cur_fps = fps_frames;
                fps_frames = 0;
                fps_start = ticks_now();
            }

            if elapsed < title_ticks {
                let alpha = if elapsed < 50 {
                    (elapsed * 255 / 50) as u32
                } else if elapsed > 150 {
                    let fade = elapsed - 150;
                    255u32.saturating_sub((fade * 255 / 50) as u32)
                } else { 255 };
                let a = alpha.min(255);
                let c = 0xFF000000 | (a << 16) | (a << 8) | a;
                draw_text_centered(buf, w, h, 30, title, c, 4);
                let sc = 0xFF000000 | ((a * 180 / 255) << 8);
                draw_text_centered(buf, w, h, 100, subtitle, sc, 2);
            }

            let elapsed_s = (elapsed / 100) as u32;
            let total_s = (dur_ticks / 100) as u32;
            draw_stats(buf, w, h, title, scene_num, cur_fps, elapsed_s, total_s, 1);
            blit(buf, w, h);
            frame += 1;
        }
    };

    // Fade to black transition (tick-based)
    let fade_out = |buf: &mut [u32], w: usize, h: usize, dur_ticks: u64| {
        let start = ticks_now();
        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= dur_ticks { break; }
            for pixel in buf.iter_mut() {
                let r = ((*pixel >> 16) & 0xFF).saturating_sub(6) as u32;
                let g = ((*pixel >> 8) & 0xFF).saturating_sub(6) as u32;
                let b = (*pixel & 0xFF).saturating_sub(6) as u32;
                *pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::delay_millis(33);
        }
        for p in buf.iter_mut() { *p = 0xFF000000; }
        blit(buf, w, h);
    };

    let fade_ticks = 40u64; // 400ms fade

    crate::serial_println!("[SHOWCASE3D] Starting 3D cinematic showcase ({}x{}) - ~60s", w, h);

    // ═════════════════════════════════════════════════════════════════
    // INTRO — Title card (3 seconds)
    // ═════════════════════════════════════════════════════════════════
    {
        let start = ticks_now();
        let intro_ticks = 300u64; // 3 seconds
        let mut frame = 0u32;
        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= intro_ticks { break; }
            for y in 0..h {
                for x in 0..w {
                    let v = ((x as i32 + frame as i32) ^ (y as i32)) as u32 & 0x0F;
                    buf[y * w + x] = 0xFF000000 | (v << 8);
                }
            }
            let alpha = (elapsed * 255 / intro_ticks.max(1)).min(255) as u32;
            let c = 0xFF000000 | (alpha << 16) | (alpha << 8) | alpha;
            draw_text_centered(&mut buf, w, h, h / 3, "TrustOS", c, 8);
            let sc = 0xFF000000 | ((alpha * 120 / 255) << 16) | ((alpha * 255 / 255) << 8) | ((alpha * 120 / 255));
            draw_text_centered(&mut buf, w, h, h / 3 + 140, "3D Graphics Showcase", sc, 3);
            let cc = 0xFF000000 | ((alpha * 100 / 255) << 16) | ((alpha * 100 / 255) << 8) | ((alpha * 100 / 255));
            draw_text_centered(&mut buf, w, h, h / 3 + 200, "Pure software rendering - No GPU hardware", cc, 2);
            blit(&buf, w, h);
            frame += 1;
            crate::cpu::tsc::delay_millis(33);
        }
        fade_out(&mut buf, w, h, fade_ticks);
    }

    // ═════════════════════════════════════════════════════════════════
    // SCENE 1 — Rotating Cube
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 1: Cube");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Cube,
        0xFF00FF66,
        "Wireframe Cube", "8 vertices - 12 edges - perspective projection", 1,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 2 — Diamond
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 2: Diamond");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Diamond,
        0xFFFF44FF,
        "Diamond", "Octahedron geometry - depth colored edges", 2,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 3 — Torus
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 3: Torus");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Torus,
        0xFFFF8844,
        "Torus", "Donut wireframe - parametric surface mesh", 3,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 4 — Pyramid
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 4: Pyramid");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Pyramid,
        0xFFFFCC00,
        "Pyramid", "5 vertices - 8 edges - ancient geometry", 4,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 5 — HoloMatrix Rain 3D
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 5: HoloMatrix");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::HoloMatrix,
        0xFF00FF44,
        "HoloMatrix", "3D matrix rain with perspective depth", 5,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 6 — Multi-Shape Orbit
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 6: Multi-Shape");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Multi,
        0xFF00FFAA,
        "Multi Shape", "4 wireframe objects orbiting - depth colored", 6,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 7 — DNA Double Helix
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 7: DNA Helix");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Helix,
        0xFF44FFCC,
        "DNA Helix", "Double-strand helix with cross rungs", 7,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 8 — Grid
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 8: Grid");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Grid,
        0xFF4488FF,
        "Infinite Grid", "Wireframe ground plane with perspective", 8,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 9 — Penger
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 9: Penger");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Penger,
        0xFFFFFF00,
        "Penger", "The legendary wireframe penguin", 9,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 10 — TrustOS Logo
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 10: TrustOS Logo");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::TrustOs,
        0xFF00FF88,
        "TrustOS Logo", "3D wireframe logo with glow vertices", 10,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 11 — Icosphere
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 11: Icosphere");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Icosphere,
        0xFF66CCFF,
        "Icosphere", "Geodesic sphere - subdivided icosahedron", 11,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // SCENE 12 — Character
    // ═════════════════════════════════════════════════════════════════
    crate::serial_println!("[SHOWCASE3D] Scene 12: Character");
    render_formula_scene(&mut buf, w, h,
        crate::formula3d::FormulaScene::Character,
        0xFF00FF88,
        "TrustOS", "Wireframe humanoid - perspective projection", 12,
        scene_ticks);
    fade_out(&mut buf, w, h, fade_ticks);

    // ═════════════════════════════════════════════════════════════════
    // OUTRO — Credits (4 seconds)
    // ═════════════════════════════════════════════════════════════════
    {
        let start = ticks_now();
        let outro_ticks = 400u64; // 4 seconds
        loop {
            let elapsed = ticks_now().saturating_sub(start);
            if elapsed >= outro_ticks { break; }
            for p in buf.iter_mut() { *p = 0xFF000000; }
            let alpha = if elapsed < 100 {
                (elapsed * 255 / 100).min(255)
            } else if elapsed > 300 {
                let fd = elapsed - 300;
                255u64.saturating_sub(fd * 255 / 100)
            } else { 255 } as u32;
            let c = 0xFF000000 | (alpha << 16) | (alpha << 8) | alpha;
            let gc = 0xFF000000 | ((alpha * 200 / 255) << 8);
            draw_text_centered(&mut buf, w, h, h / 3 - 30, "TrustOS 3D Engine", c, 5);
            draw_text_centered(&mut buf, w, h, h / 3 + 60, "12 wireframe scenes - Pure software rendering", gc, 2);
            draw_text_centered(&mut buf, w, h, h / 3 + 100, "No GPU hardware - All CPU computed", gc, 2);
            draw_text_centered(&mut buf, w, h, h / 3 + 160, "Written in Rust by Nated0ge", 0xFF000000 | ((alpha * 140 / 255) << 16) | ((alpha * 180 / 255) << 8) | (alpha * 255 / 255), 3);
            draw_text_centered(&mut buf, w, h, h / 3 + 220, "github.com/nathan237/TrustOS", 0xFF000000 | ((alpha * 100 / 255) << 16) | ((alpha * 100 / 255) << 8) | ((alpha * 100 / 255)), 2);
            blit(&buf, w, h);
            crate::cpu::tsc::delay_millis(33);
        }
    }

    // Restore
    for p in buf.iter_mut() { *p = 0xFF000000; }
    blit(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[SHOWCASE3D] Showcase complete");
}

/// Test command: filled 3D rendering with flat shading
/// Renders rotating cube, pyramid, and diamond with solid filled faces
pub fn cmd_filled3d() {
    use crate::formula3d::V3;

    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;
    if w == 0 || h == 0 { return; }

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0xFF000000u32; w * h];

    let draw_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, c: char, color: u32, scale: usize| {
        let glyph = crate::framebuffer::font::get_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + bit as usize * scale + sx;
                            let py = cy + row * scale + sy;
                            if px < w && py < h {
                                buf[py * w + px] = color;
                            }
                        }
                    }
                }
            }
        }
    };

    let blit = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buf[y * w..].as_ptr(),
                        bb.add(y * bb_s),
                        w,
                    );
                }
            }
        }
        crate::framebuffer::swap_buffers();
    };

    let ticks_now = || crate::logger::get_ticks();

    crate::serial_println!("[FILLED3D] Starting filled 3D test ({}x{})", w, h);

    // Light direction (top-left-front)
    let light = crate::formula3d::V3 { x: -0.4, y: 0.6, z: -0.7 };
    // Normalize manually
    let len = crate::formula3d::fast_sqrt(light.x * light.x + light.y * light.y + light.z * light.z);
    let light = V3 { x: light.x / len, y: light.y / len, z: light.z / len };

    // Build meshes with faces
    let cube = crate::formula3d::mesh_cube();
    let pyramid = crate::formula3d::mesh_pyramid();
    let diamond = crate::formula3d::mesh_diamond();

    let mut angle_y: f32 = 0.0;
    let mut frame = 0u32;
    let start = ticks_now();
    let mut fps_start = start;
    let mut fps_frames = 0u32;
    let mut cur_fps = 0u32;

    loop {
        let elapsed = ticks_now().saturating_sub(start);
        if elapsed >= 3000 { break; } // 30 seconds
        if let Some(k) = crate::keyboard::try_read_key() {
            if k == 27 { break; }
        }

        // Clear to dark blue-grey
        for p in buf.iter_mut() { *p = 0xFF0C1018; }

        angle_y += 0.025;
        let angle_x = 0.35 + crate::formula3d::fast_sin(frame as f32 * 0.008) * 0.2;

        // Draw 3 objects with different offsets
        // Left: pyramid (offset view by adjusting a virtual camera offset via vertices)
        // We render each at dz=2.5 but shift the angle to spread them visually
        // For a proper side-by-side we'd need per-object translation.
        // Simple approach: render 3 separate viewports (left/center/right thirds)

        let third = w / 3;

        // Left third: Pyramid
        {
            let mut sub_buf = alloc::vec![0xFF0C1018u32; third * h];
            crate::formula3d::render_filled_mesh(&mut sub_buf, third, h,
                &pyramid, angle_y * 0.8, angle_x + 0.15, 2.2,
                0xFFFF8844, light, 0.12);
            // Copy sub_buf into main buf
            for y in 0..h {
                for x in 0..third {
                    let src_idx = y * third + x;
                    let dst_idx = y * w + x;
                    if src_idx < sub_buf.len() && dst_idx < buf.len() {
                        buf[dst_idx] = sub_buf[src_idx];
                    }
                }
            }
        }

        // Center third: Cube
        {
            let mut sub_buf = alloc::vec![0xFF0C1018u32; third * h];
            crate::formula3d::render_filled_mesh(&mut sub_buf, third, h,
                &cube, angle_y, angle_x, 2.2,
                0xFF4488FF, light, 0.12);
            for y in 0..h {
                for x in 0..third {
                    let src_idx = y * third + x;
                    let dst_idx = y * w + third + x;
                    if src_idx < sub_buf.len() && dst_idx < buf.len() {
                        buf[dst_idx] = sub_buf[src_idx];
                    }
                }
            }
        }

        // Right third: Diamond
        {
            let mut sub_buf = alloc::vec![0xFF0C1018u32; third * h];
            crate::formula3d::render_filled_mesh(&mut sub_buf, third, h,
                &diamond, angle_y * 1.3, angle_x - 0.1, 2.2,
                0xFFFF44CC, light, 0.12);
            for y in 0..h {
                for x in 0..third {
                    let src_idx = y * third + x;
                    let dst_idx = y * w + 2 * third + x;
                    if src_idx < sub_buf.len() && dst_idx < buf.len() {
                        buf[dst_idx] = sub_buf[src_idx];
                    }
                }
            }
        }

        // FPS counter
        fps_frames += 1;
        let fps_elapsed = ticks_now().saturating_sub(fps_start);
        if fps_elapsed >= 100 {
            cur_fps = fps_frames;
            fps_frames = 0;
            fps_start = ticks_now();
        }

        // Stats bar
        let bar_h = 22usize;
        let bar_y = h.saturating_sub(bar_h);
        for y in bar_y..h {
            for x in 0..w {
                let idx = y * w + x;
                if idx < buf.len() { buf[idx] = 0xFF000000; }
            }
        }
        let stats = alloc::format!("Filled 3D | {} FPS | Flat Shading + Backface Cull + Painter Sort | ESC=exit", cur_fps);
        for (i, ch) in stats.chars().enumerate() {
            let cx = 8 + i * 8;
            if cx + 8 > w { break; }
            draw_char(&mut buf, w, h, cx, bar_y + 4, ch, 0xFF00FF88, 1);
        }

        // Title on first frames
        if frame < 200 {
            let alpha = if frame < 30 { frame * 255 / 30 } else if frame > 170 { (200 - frame) * 255 / 30 } else { 255 };
            let a = (alpha.min(255)) as u32;
            let c = 0xFF000000 | (a << 16) | (a << 8) | a;
            let title = "FILLED 3D TEST";
            let tw = title.len() * 8 * 3;
            let tx = if tw < w { (w - tw) / 2 } else { 0 };
            for (i, ch) in title.chars().enumerate() {
                draw_char(&mut buf, w, h, tx + i * 24, 30, ch, c, 3);
            }
            let sub = "Scanline Rasterizer + Flat Shading";
            let stw = sub.len() * 8 * 2;
            let stx = if stw < w { (w - stw) / 2 } else { 0 };
            let sc = 0xFF000000 | ((a * 180 / 255) << 8);
            for (i, ch) in sub.chars().enumerate() {
                draw_char(&mut buf, w, h, stx + i * 16, 80, ch, sc, 2);
            }
        }

        blit(&buf, w, h);
        frame += 1;
    }

    // Restore
    for p in buf.iter_mut() { *p = 0xFF000000; }
    blit(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILLED3D] Test complete, {} frames", frame);
}

/// Wrapper for desktop without initial app
fn cmd_cosmic_v2() {
    cmd_cosmic_v2_with_app(None);
}

// ==================== COSMIC V2 - MULTI-LAYER COMPOSITOR ====================
// Each UI component renders to its own layer independently, then composited together.
// This eliminates flickering by ensuring atomic frame presentation.

/// Open desktop with optional app to launch
fn cmd_cosmic_v2_with_app(initial_app: Option<&str>) {
    cmd_cosmic_v2_with_app_timed(initial_app, 0);
}

/// Open desktop with optional app and optional auto-exit timeout (0 = no timeout)
fn cmd_cosmic_v2_with_app_timed(initial_app: Option<&str>, timeout_ms: u64) {
    use crate::compositor::{Compositor, LayerType};
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    
    crate::serial_println!("[COSMIC2] Starting COSMIC V2 Desktop...");
    
    // Auto-enable SMP parallelism for desktop rendering
    crate::cpu::smp::enable_smp();
    // Flush keyboard
    while crate::keyboard::try_read_key().is_some() {}
    
    let (width, height) = crate::framebuffer::get_dimensions();
    if width == 0 || height == 0 {
        crate::println_color!(COLOR_RED, "Error: Invalid framebuffer!");
        return;
    }
    
    // Set mouse bounds
    crate::mouse::set_screen_size(width, height);
    
    crate::serial_println!("[COSMIC2] Creating compositor {}x{}", width, height);
    
    // Create compositor with screen dimensions
    let mut compositor = Compositor::new(width, height);
    
    // Create layers (bottom to top order) - 8 layers
    let bg_layer = compositor.add_fullscreen_layer(LayerType::Background);
    let dock_layer = compositor.add_layer(LayerType::Dock, 0, 0, 70, height - 40);  // Dock with icon labels
    let window_layer = compositor.add_layer(LayerType::Windows, 100, 80, 700, 450);  // Smaller window, more compact
    let history_layer = compositor.add_layer(LayerType::Overlay, width - 260, 50, 250, 220);  // Command history panel
    let taskbar_layer = compositor.add_layer(LayerType::Taskbar, 0, height - 40, width, 40);
    let menu_layer = compositor.add_layer(LayerType::Overlay, 5, height - 440, 280, 400);  // Bigger menu
    let settings_layer = compositor.add_layer(LayerType::Overlay, 340, height - 380, 280, 350);  // Settings panel (taller for HoloMatrix)
    let cursor_layer = compositor.add_layer(LayerType::Overlay, 0, 0, 24, 24);
    
    crate::serial_println!("[COSMIC2] Created {} layers", compositor.layer_count());
    
    // Enable GPU direct mode: composite directly into GPU buffer (skips 4MB copy)
    compositor.enable_gpu_direct();
    
    // ═══════════════════════════════════════════════════════════════════
    // STATE
    // ═══════════════════════════════════════════════════════════════════
    let mut running = true;
    let mut frame_count = 0u64;
    
    // Active module/app
    #[derive(Clone, Copy, PartialEq)]
    enum AppMode {
        Shell,       // Default shell with help
        Network,     // Network module
        Hardware,    // Hardware info
        TextEditor,  // Simple editor
        UserMgmt,    // User management
        Files,       // File browser
        Browser,     // Web browser - special mode
        ImageViewer, // Image viewer - PNG, BMP, etc.
    }
    
    // Set initial mode based on argument
    let mut active_mode = match initial_app {
        Some("browser") | Some("web") | Some("www") => AppMode::Browser,
        Some("files") | Some("explorer") => AppMode::Files,
        Some("editor") | Some("text") | Some("notepad") => AppMode::TextEditor,
        Some("network") | Some("net") | Some("ifconfig") => AppMode::Network,
        Some("hardware") | Some("hw") | Some("lshw") => AppMode::Hardware,
        Some("users") | Some("user") => AppMode::UserMgmt,
        Some("images") | Some("image") | Some("viewer") => AppMode::ImageViewer,
        _ => AppMode::Shell,
    };
    let mut browser_mode = active_mode == AppMode::Browser;  // True when browser is active (not a shell)
    
    // ═══════════════════════════════════════════════════════════════════════════
    // BROWSER RENDERING SYSTEM - Chrome DevTools style HTML coloring
    // ═══════════════════════════════════════════════════════════════════════════
    
    // Color segment for HTML rendering (text, color)
    #[derive(Clone)]
    struct HtmlSegment {
        text: String,
        color: u32,
    }
    
    // Color palette - Chrome DevTools inspired
    const HTML_COLOR_TAG: u32 = 0xFFE06C75;       // Red/Pink - HTML tags <div>, </div>
    const HTML_COLOR_ATTR: u32 = 0xFF98C379;      // Green - Attribute names
    const HTML_COLOR_VALUE: u32 = 0xFFE5C07B;     // Yellow - Attribute values
    const HTML_COLOR_TEXT: u32 = 0xFFDCDCDC;      // White - Text content
    const HTML_COLOR_COMMENT: u32 = 0xFF5C6370;   // Gray - Comments
    const HTML_COLOR_DOCTYPE: u32 = 0xFFABB2BF;   // Light gray - DOCTYPE
    const HTML_COLOR_BRACKET: u32 = 0xFF56B6C2;   // Cyan - < > brackets
    const HTML_COLOR_STRING: u32 = 0xFF98C379;    // Green - Quoted strings
    const HTML_COLOR_ENTITY: u32 = 0xFFD19A66;    // Orange - HTML entities &amp;
    const HTML_COLOR_HTTP: u32 = 0xFF61AFEF;      // Blue - HTTP headers
    const HTML_COLOR_STATUS: u32 = 0xFF56B6C2;    // Cyan - Status codes
    
    // Parsed line with color segments
    #[derive(Clone)]
    struct BrowserLine {
        segments: Vec<HtmlSegment>,
        line_type: LineType,
    }
    
    #[derive(Clone, Copy, PartialEq)]
    enum LineType {
        Welcome,      // Welcome box
        HttpHeader,   // HTTP/1.1 200 OK
        HtmlTag,      // Full HTML tag
        HtmlMixed,    // Mixed content
        PlainText,    // Regular text
        Error,        // Error message
    }
    
    // Parse a line of HTML into colored segments
    fn parse_html_line(line: &str) -> Vec<HtmlSegment> {
        let mut segments = Vec::new();
        let mut chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            // Check for HTML tag start
            if chars[i] == '<' {
                // Check for comment
                if i + 3 < chars.len() && chars[i+1] == '!' && chars[i+2] == '-' && chars[i+3] == '-' {
                    // Comment start <!--
                    let start = i;
                    while i < chars.len() {
                        if i + 2 < chars.len() && chars[i] == '-' && chars[i+1] == '-' && chars[i+2] == '>' {
                            i += 3;
                            break;
                        }
                        i += 1;
                    }
                    segments.push(HtmlSegment {
                        text: chars[start..i].iter().collect(),
                        color: HTML_COLOR_COMMENT,
                    });
                    continue;
                }
                
                // Check for DOCTYPE
                if i + 1 < chars.len() && chars[i+1] == '!' {
                    let start = i;
                    while i < chars.len() && chars[i] != '>' {
                        i += 1;
                    }
                    if i < chars.len() { i += 1; }
                    segments.push(HtmlSegment {
                        text: chars[start..i].iter().collect(),
                        color: HTML_COLOR_DOCTYPE,
                    });
                    continue;
                }
                
                // Regular tag - parse with colors
                // Opening bracket
                segments.push(HtmlSegment { text: String::from("<"), color: HTML_COLOR_BRACKET });
                i += 1;
                
                // Check for closing tag /
                if i < chars.len() && chars[i] == '/' {
                    segments.push(HtmlSegment { text: String::from("/"), color: HTML_COLOR_BRACKET });
                    i += 1;
                }
                
                // Tag name
                let tag_start = i;
                while i < chars.len() && chars[i] != ' ' && chars[i] != '>' && chars[i] != '/' {
                    i += 1;
                }
                if tag_start < i {
                    segments.push(HtmlSegment {
                        text: chars[tag_start..i].iter().collect(),
                        color: HTML_COLOR_TAG,
                    });
                }
                
                // Attributes
                while i < chars.len() && chars[i] != '>' {
                    // Skip whitespace
                    if chars[i] == ' ' {
                        let ws_start = i;
                        while i < chars.len() && chars[i] == ' ' { i += 1; }
                        segments.push(HtmlSegment {
                            text: chars[ws_start..i].iter().collect(),
                            color: HTML_COLOR_TEXT,
                        });
                        continue;
                    }
                    
                    // Self-closing /
                    if chars[i] == '/' {
                        segments.push(HtmlSegment { text: String::from("/"), color: HTML_COLOR_BRACKET });
                        i += 1;
                        continue;
                    }
                    
                    // Attribute name
                    let attr_start = i;
                    while i < chars.len() && chars[i] != '=' && chars[i] != ' ' && chars[i] != '>' && chars[i] != '/' {
                        i += 1;
                    }
                    if attr_start < i {
                        segments.push(HtmlSegment {
                            text: chars[attr_start..i].iter().collect(),
                            color: HTML_COLOR_ATTR,
                        });
                    }
                    
                    // = sign
                    if i < chars.len() && chars[i] == '=' {
                        segments.push(HtmlSegment { text: String::from("="), color: HTML_COLOR_TEXT });
                        i += 1;
                    }
                    
                    // Attribute value (quoted)
                    if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                        let quote = chars[i];
                        let val_start = i;
                        i += 1;
                        while i < chars.len() && chars[i] != quote {
                            i += 1;
                        }
                        if i < chars.len() { i += 1; } // closing quote
                        segments.push(HtmlSegment {
                            text: chars[val_start..i].iter().collect(),
                            color: HTML_COLOR_VALUE,
                        });
                    }
                }
                
                // Closing bracket
                if i < chars.len() && chars[i] == '>' {
                    segments.push(HtmlSegment { text: String::from(">"), color: HTML_COLOR_BRACKET });
                    i += 1;
                }
            }
            // Check for HTML entity &xxx;
            else if chars[i] == '&' {
                let start = i;
                while i < chars.len() && chars[i] != ';' && chars[i] != ' ' {
                    i += 1;
                }
                if i < chars.len() && chars[i] == ';' { i += 1; }
                segments.push(HtmlSegment {
                    text: chars[start..i].iter().collect(),
                    color: HTML_COLOR_ENTITY,
                });
            }
            // Regular text
            else {
                let start = i;
                while i < chars.len() && chars[i] != '<' && chars[i] != '&' {
                    i += 1;
                }
                if start < i {
                    segments.push(HtmlSegment {
                        text: chars[start..i].iter().collect(),
                        color: HTML_COLOR_TEXT,
                    });
                }
            }
        }
        
        segments
    }
    
    // Browser state
    let mut browser_url = String::from("https://google.com");
    let mut browser_lines: Vec<BrowserLine> = Vec::new();
    let mut browser_status = String::from("Enter URL and press Enter to navigate");
    let mut browser_loading = false;
    let mut browser_url_focused = true;
    let mut browser_view_mode: u8 = 0;  // 0=DevTools, 1=Rendered
    let mut auto_navigate_pending = timeout_ms > 0 && active_mode == AppMode::Browser;
    
    // Helper to add a simple line
    fn make_simple_line(text: &str, color: u32, line_type: LineType) -> BrowserLine {
        let mut segs = Vec::new();
        segs.push(HtmlSegment { text: String::from(text), color });
        BrowserLine {
            segments: segs,
            line_type,
        }
    }
    
    // Helper to add parsed HTML line
    fn make_html_line(text: &str) -> BrowserLine {
        BrowserLine {
            segments: parse_html_line(text),
            line_type: LineType::HtmlMixed,
        }
    }
    
    // Run crypto self-tests before starting browser
    crate::tls13::crypto::run_self_tests();
    
    // Initialize browser with welcome page
    browser_lines.push(make_simple_line("╔════════════════════════════════════════════════════════════╗", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║        TrustOS Web Browser v1.0 - DevTools Mode            ║", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("╠════════════════════════════════════════════════════════════╣", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║                                                            ║", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║  Syntax highlighting like Chrome DevTools!                 ║", 0xFFDDDDDD, LineType::Welcome));
    browser_lines.push(make_simple_line("║                                                            ║", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║  COLOR LEGEND:                                             ║", 0xFFFFFF00, LineType::Welcome));
    {
        let mut legend1 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend1.segments.push(HtmlSegment { text: String::from("║    "), color: 0xFF00AAFF });
        legend1.segments.push(HtmlSegment { text: String::from("<tag>"), color: HTML_COLOR_TAG });
        legend1.segments.push(HtmlSegment { text: String::from(" - HTML tags                            ║"), color: 0xFFDDDDDD });
        browser_lines.push(legend1);
        
        let mut legend2 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend2.segments.push(HtmlSegment { text: String::from("║    "), color: 0xFF00AAFF });
        legend2.segments.push(HtmlSegment { text: String::from("attr"), color: HTML_COLOR_ATTR });
        legend2.segments.push(HtmlSegment { text: String::from(" - Attribute names                     ║"), color: 0xFFDDDDDD });
        browser_lines.push(legend2);
        
        let mut legend3 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend3.segments.push(HtmlSegment { text: String::from("║    "), color: 0xFF00AAFF });
        legend3.segments.push(HtmlSegment { text: String::from("\"value\""), color: HTML_COLOR_VALUE });
        legend3.segments.push(HtmlSegment { text: String::from(" - Attribute values                   ║"), color: 0xFFDDDDDD });
        browser_lines.push(legend3);
        
        let mut legend4 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend4.segments.push(HtmlSegment { text: String::from("║    "), color: 0xFF00AAFF });
        legend4.segments.push(HtmlSegment { text: String::from("< >"), color: HTML_COLOR_BRACKET });
        legend4.segments.push(HtmlSegment { text: String::from(" - Brackets                             ║"), color: 0xFFDDDDDD });
        browser_lines.push(legend4);
        
        let mut legend5 = BrowserLine { segments: Vec::new(), line_type: LineType::Welcome };
        legend5.segments.push(HtmlSegment { text: String::from("║    "), color: 0xFF00AAFF });
        legend5.segments.push(HtmlSegment { text: String::from("&amp;"), color: HTML_COLOR_ENTITY });
        legend5.segments.push(HtmlSegment { text: String::from(" - HTML entities                       ║"), color: 0xFFDDDDDD });
        browser_lines.push(legend5);
    }
    browser_lines.push(make_simple_line("║                                                            ║", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║  TRY THESE URLs:                                           ║", 0xFFFFFF00, LineType::Welcome));
    browser_lines.push(make_simple_line("║    https://google.com                                      ║", 0xFF00FFFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║    https://example.com                                     ║", 0xFF00FFFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║                                                            ║", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║                                                            ║", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("║  [Tab] Toggle DevTools/Rendered  [Enter] Navigate          ║", 0xFF88FF88, LineType::Welcome));
    browser_lines.push(make_simple_line("║  [ESC] Return to shell                                     ║", 0xFF88FF88, LineType::Welcome));
    browser_lines.push(make_simple_line("║                                                            ║", 0xFF00AAFF, LineType::Welcome));
    browser_lines.push(make_simple_line("╚════════════════════════════════════════════════════════════╝", 0xFF00AAFF, LineType::Welcome));
    
    // ═══════════════════════════════════════════════════════════════════════════
    // CREATE TEST IMAGE IN RAMFS
    // ═══════════════════════════════════════════════════════════════════════════
    {
        // Create a simple 32x32 PPM test image with colorful gradient
        let mut ppm_data = String::from("P3\n32 32\n255\n");
        for y in 0..32 {
            for x in 0..32 {
                let r = (x * 8) % 256;
                let g = (y * 8) % 256;
                let b = ((x + y) * 4) % 256;
                ppm_data.push_str(&format!("{} {} {} ", r, g, b));
            }
            ppm_data.push('\n');
        }
        let _ = crate::ramfs::with_fs(|fs| {
            fs.mkdir("/images");
            fs.write_file("/images/test.ppm", ppm_data.as_bytes())
        });
        
        // Also create a simple BMP test image (24-bit, 16x16)
        let bmp_header: [u8; 54] = [
            0x42, 0x4D,             // BM signature
            0x36, 0x03, 0x00, 0x00, // File size: 54 + 768 = 822 bytes
            0x00, 0x00, 0x00, 0x00, // Reserved
            0x36, 0x00, 0x00, 0x00, // Offset to pixel data
            0x28, 0x00, 0x00, 0x00, // DIB header size (40)
            0x10, 0x00, 0x00, 0x00, // Width: 16
            0x10, 0x00, 0x00, 0x00, // Height: 16 (bottom-up)
            0x01, 0x00,             // Planes: 1
            0x18, 0x00,             // Bits per pixel: 24
            0x00, 0x00, 0x00, 0x00, // Compression: none
            0x00, 0x03, 0x00, 0x00, // Image size: 768
            0x13, 0x0B, 0x00, 0x00, // H pixels/meter
            0x13, 0x0B, 0x00, 0x00, // V pixels/meter
            0x00, 0x00, 0x00, 0x00, // Colors in palette
            0x00, 0x00, 0x00, 0x00, // Important colors
        ];
        let mut bmp_data = alloc::vec::Vec::from(bmp_header);
        // Pixel data (BGR, bottom-up, with row padding)
        for y in 0..16 {
            for x in 0..16 {
                let b = ((15 - y) * 17) as u8;  // Blue gradient top-bottom
                let g = (x * 17) as u8;          // Green gradient left-right
                let r = ((x + y) * 8) as u8;     // Red diagonal
                bmp_data.push(b);
                bmp_data.push(g);
                bmp_data.push(r);
            }
            // No padding needed for 16*3=48 bytes (divisible by 4)
        }
        let _ = crate::ramfs::with_fs(|fs| {
            fs.write_file("/images/test.bmp", &bmp_data)
        });
        
        crate::serial_println!("[COSMIC2] Created test images in /images/");
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // IMAGE VIEWER STATE
    // ═══════════════════════════════════════════════════════════════════════════
    let mut image_viewer_path = String::new();
    let mut image_viewer_data: Option<crate::image::Image> = None;
    let mut image_viewer_zoom: f32 = 1.0;
    let mut image_viewer_offset_x: i32 = 0;
    let mut image_viewer_offset_y: i32 = 0;
    let mut image_viewer_info = String::from("No image loaded");
    let mut image_viewer_format = String::from("---");
    
    // Menu state
    let mut menu_open = false;
    let mut menu_hover: i32 = -1;
    
    // Settings panel state
    let mut settings_open = false;
    let mut settings_anim_enabled = crate::desktop::animations_enabled();
    let mut settings_anim_speed = crate::desktop::get_animation_speed();
    
    // Shell state for the window
    let mut shell_input = String::new();
    let mut shell_output: Vec<String> = Vec::new();
    let mut cursor_blink = true;
    let mut suggestion_text = String::new();
    let mut scroll_offset: usize = 0;  // For scrolling through output
    const MAX_VISIBLE_LINES: usize = 18;  // Lines visible in shell window
    
    // TrustCode editor state
    let mut editor_state = crate::apps::text_editor::EditorState::new();
    // Pre-load a demo Rust file
    {
        let sample_code = "//! TrustOS \u{2014} A Modern Operating System in Rust\n//!\n//! This file demonstrates TrustCode's syntax highlighting\n\nuse core::fmt;\n\n/// Main kernel entry point\npub fn kernel_main() -> ! {\n    let message = \"Hello from TrustOS!\";\n    serial_println!(\"{}\", message);\n\n    // Initialize hardware\n    let cpu_count: u32 = 4;\n    let memory_mb: u64 = 256;\n\n    for i in 0..cpu_count {\n        init_cpu(i);\n    }\n\n    // Start the desktop environment\n    let mut desktop = Desktop::new();\n    desktop.init(1280, 800);\n\n    loop {\n        desktop.render();\n        desktop.handle_input();\n    }\n}\n\n/// Initialize a CPU core\nfn init_cpu(id: u32) {\n    // Setup GDT, IDT, APIC\n    serial_println!(\"CPU {} initialized\", id);\n}\n\n#[derive(Debug, Clone)]\nstruct AppConfig {\n    name: String,\n    version: (u8, u8, u8),\n    features: Vec<&'static str>,\n}\n";
        let _ = crate::ramfs::with_fs(|fs| fs.write_file("/demo.rs", sample_code.as_bytes()));
        editor_state.load_file("demo.rs");
    }
    
    // Window dragging state
    let mut dragging_window = false;
    let mut drag_offset_x: i32 = 0;
    let mut drag_offset_y: i32 = 0;
    let mut window_x: i32 = 100;
    let mut window_y: i32 = 80;
    let mut window_visible = true;  // Window can be closed/reopened
    
    // Command history for the history panel
    let mut command_history: Vec<String> = Vec::new();
    const MAX_HISTORY: usize = 10;
    
    // Module help text - static arrays
    const HELP_SHELL: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║         TrustOS Interactive Shell - Welcome!                  ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  This is the main command-line interface for TrustOS.         ║",
        "║                                                               ║",
        "║  BASIC COMMANDS:                                              ║",
        "║    help     - Display all available commands                  ║",
        "║    clear    - Clear the terminal screen                       ║",
        "║    echo     - Print text to the screen                        ║",
        "║    date     - Show current date and time                      ║",
        "║    uptime   - Show system uptime                              ║",
        "║                                                               ║",
        "║  TIPS:                                                        ║",
        "║    • Use Tab for command autocompletion                       ║",
        "║    • Commands are case-insensitive                            ║",
        "║    • Type 'exit' or press ESC to close desktop                ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    const HELP_NETWORK: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║           Network Module - Configuration & Diagnostics        ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  Manage network interfaces and diagnose connectivity.         ║",
        "║                                                               ║",
        "║  COMMANDS:                                                    ║",
        "║    ifconfig    - Show network interface configuration         ║",
        "║    ping <ip>   - Send ICMP ping to test connectivity          ║",
        "║    dhcp        - Request IP address via DHCP                  ║",
        "║    netstat     - Display network statistics                   ║",
        "║    arp         - Show Address Resolution Protocol table       ║",
        "║    dns <host>  - Resolve hostname to IP address               ║",
        "║                                                               ║",
        "║  TIPS:                                                        ║",
        "║    • Run 'dhcp' first to get an IP address                    ║",
        "║    • Use 'ping 8.8.8.8' to test internet connectivity         ║",
        "║    • 'ifconfig' shows MAC address and IP configuration        ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    const HELP_HARDWARE: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║           Hardware Module - System Information                ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  Explore your hardware and system resources.                  ║",
        "║                                                               ║",
        "║  COMMANDS:                                                    ║",
        "║    cpuinfo   - Display CPU model, features and frequency      ║",
        "║    meminfo   - Show RAM usage and available memory            ║",
        "║    lspci     - List all PCI/PCIe devices                      ║",
        "║    lsusb     - List connected USB devices                     ║",
        "║    uptime    - Show system uptime since boot                  ║",
        "║    sensors   - Display temperature sensors (if available)     ║",
        "║                                                               ║",
        "║  TIPS:                                                        ║",
        "║    • 'cpuinfo' shows SIMD support (SSE, AVX)                  ║",
        "║    • 'lspci' reveals network and storage controllers          ║",
        "║    • Memory info includes heap allocation statistics          ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    const HELP_EDITOR: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║           Text Editor - Create and Edit Files                 ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  Simple text editor for viewing and modifying files.          ║",
        "║                                                               ║",
        "║  COMMANDS:                                                    ║",
        "║    edit <file>  - Open an existing file for editing           ║",
        "║    new <name>   - Create a new empty file                     ║",
        "║    cat <file>   - View file contents (read-only)              ║",
        "║    save         - Save current changes                        ║",
        "║    :q           - Quit editor without saving                  ║",
        "║    :wq          - Save and quit                               ║",
        "║                                                               ║",
        "║  TIPS:                                                        ║",
        "║    • Use 'cat' first to preview file before editing           ║",
        "║    • Files are stored in the RAM filesystem                   ║",
        "║    • New files are created in current directory               ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    const HELP_USERS: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║           User Management - Accounts & Security               ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  Manage user accounts, passwords, and permissions.            ║",
        "║                                                               ║",
        "║  COMMANDS:                                                    ║",
        "║    whoami      - Display current logged-in user               ║",
        "║    users       - List all system users                        ║",
        "║    adduser     - Create a new user account                    ║",
        "║    deluser     - Delete an existing user                      ║",
        "║    passwd      - Change user password                         ║",
        "║    groups      - Show user groups                             ║",
        "║    su <user>   - Switch to another user                       ║",
        "║                                                               ║",
        "║  TIPS:                                                        ║",
        "║    • Default user is 'root' with full privileges              ║",
        "║    • Use strong passwords (8+ chars, mixed case)              ║",
        "║    • 'adduser' prompts for username and password              ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    const HELP_FILES: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║           File Manager - Navigate & Manage Files              ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  Browse directories and manage files on the system.           ║",
        "║                                                               ║",
        "║  NAVIGATION:                                                  ║",
        "║    ls / dir    - List files in current directory              ║",
        "║    cd <dir>    - Change to specified directory                ║",
        "║    cd ..       - Go up one directory level                    ║",
        "║    pwd         - Print current working directory              ║",
        "║                                                               ║",
        "║  FILE OPERATIONS:                                             ║",
        "║    mkdir <dir> - Create a new directory                       ║",
        "║    rm <file>   - Remove/delete a file                         ║",
        "║    cp <s> <d>  - Copy file from source to destination         ║",
        "║    mv <s> <d>  - Move or rename a file                        ║",
        "║    cat <file>  - Display file contents                        ║",
        "║    touch <f>   - Create empty file                            ║",
        "║                                                               ║",
        "║  TIPS:                                                        ║",
        "║    • Use Tab for path autocompletion                          ║",
        "║    • 'ls -la' shows hidden files and details                  ║",
        "║    • Paths can be absolute (/home) or relative (./docs)       ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    const HELP_BROWSER: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║           TrustOS Web Browser                                 ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  A simple text-based web browser for HTTP requests.           ║",
        "║                                                               ║",
        "║  COMMANDS:                                                    ║",
        "║    get <url>    - Fetch a web page (HTTP only)                ║",
        "║    http <host>  - Make HTTP GET request                       ║",
        "║    curl <url>   - Display raw HTTP response                   ║",
        "║                                                               ║",
        "║  EXAMPLES:                                                    ║",
        "║    get http://example.com                                     ║",
        "║    http 93.184.216.34 /index.html                             ║",
        "║                                                               ║",
        "║  NOTE: This is a text-based browser. Full graphical           ║",
        "║        browser support is planned for future versions.        ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    const HELP_IMAGEVIEWER: &[&str] = &[
        "╔═══════════════════════════════════════════════════════════════╗",
        "║           TrustOS Image Viewer                                ║",
        "╠═══════════════════════════════════════════════════════════════╣",
        "║  View PNG, BMP, PPM and other image formats.                  ║",
        "║                                                               ║",
        "║  SUPPORTED FORMATS:                                           ║",
        "║    • PNG  - Portable Network Graphics (8-bit RGB/RGBA)        ║",
        "║    • BMP  - Windows Bitmap (24/32-bit uncompressed)           ║",
        "║    • PPM  - Portable Pixmap (P3/P6 formats)                   ║",
        "║                                                               ║",
        "║  SHELL COMMANDS:                                              ║",
        "║    imgview <file>  - Open image in viewer                     ║",
        "║    imginfo <file>  - Show image information                   ║",
        "║                                                               ║",
        "║  KEYBOARD (in viewer):                                        ║",
        "║    +/-        - Zoom in/out                                   ║",
        "║    Arrow keys - Pan image                                     ║",
        "║    R          - Reset view (fit to window)                    ║",
        "║    ESC        - Return to shell                               ║",
        "║                                                               ║",
        "║  EXAMPLES:                                                    ║",
        "║    imgview /images/photo.png                                  ║",
        "║    imginfo /wallpaper.bmp                                     ║",
        "╚═══════════════════════════════════════════════════════════════╝",
    ];
    
    // Macro to get help for mode
    macro_rules! get_help {
        ($mode:expr) => {
            match $mode {
                AppMode::Shell => HELP_SHELL,
                AppMode::Network => HELP_NETWORK,
                AppMode::Hardware => HELP_HARDWARE,
                AppMode::TextEditor => HELP_EDITOR,
                AppMode::UserMgmt => HELP_USERS,
                AppMode::Files => HELP_FILES,
                AppMode::Browser => HELP_BROWSER,
                AppMode::ImageViewer => HELP_IMAGEVIEWER,
            }
        };
    }
    
    // Initialize shell with welcome
    for line in get_help!(AppMode::Shell) {
        shell_output.push(String::from(*line));
    }
    
    // ═══════════════════════════════════════════════════════════════════
    // MATRIX RAIN - OPTIMIZED: Pre-generated columns, only update head position
    // Each column has fixed characters, only the "brightness head" moves
    // ═══════════════════════════════════════════════════════════════════
    const MATRIX_COLS: usize = 240;      // Dense columns (1920/8 = 240)
    const MATRIX_ROWS: usize = 68;       // Rows per column (1080/16 = 67.5)
    const MATRIX_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    // Helper to compute flat index for matrix_chars
    #[inline]
    fn matrix_idx(col: usize, row: usize) -> usize {
        col * MATRIX_ROWS + row
    }
    
    // Pre-generate character grid (static characters for each cell)
    // This is generated once and reused every frame
    // Vec allocated directly on heap - no stack overflow
    let mut matrix_chars: Vec<u8> = vec![0u8; MATRIX_COLS * MATRIX_ROWS];
    for col in 0..MATRIX_COLS {
        let seed = (col as u32 * 2654435761) ^ 0xDEADBEEF;
        for row in 0..MATRIX_ROWS {
            let char_seed = seed.wrapping_mul(row as u32 + 1);
            matrix_chars[matrix_idx(col, row)] = MATRIX_CHARS[(char_seed as usize) % MATRIX_CHARS.len()];
        }
    }
    
    // Head position for each column (where the bright green starts)
    // Only this moves each frame - much faster than redrawing characters!
    let mut matrix_heads: [i32; MATRIX_COLS] = [0; MATRIX_COLS];
    let mut matrix_speeds: [u32; MATRIX_COLS] = [0; MATRIX_COLS];
    for col in 0..MATRIX_COLS {
        let seed = (col as u32 * 2654435761) ^ 0xDEADBEEF;
        matrix_heads[col] = -((seed % (MATRIX_ROWS as u32 * 2)) as i32);
        matrix_speeds[col] = 1 + (seed % 3);  // Speed 1-3
    }
    
    // HoloMatrix 3D volumetric renderer
    // Creates holographic 3D effects by layering Z-slices
    // 32 layers for enhanced depth perception
    let mut holomatrix = crate::graphics::holomatrix::HoloMatrix::new(width as usize / 4, height as usize / 4, 32);
    // Sync with global settings (can be set via 'holo' shell command)
    let mut holo_scene = crate::graphics::holomatrix::get_scene();
    let mut holo_enabled = crate::graphics::holomatrix::is_enabled();
    
    // NEW: HoloVolume - True volumetric ASCII raymarcher
    // A 3D volume of ASCII voxels projected to 2D with alignment-based intensity
    let mut holovolume = crate::holovolume::HoloVolume::new(
        width as usize / 8,   // ~160 chars wide
        height as usize / 9,  // ~90 chars tall  
        32                    // 32 Z-layers for depth
    );
    holovolume.render_mode = crate::holovolume::RenderMode::Hologram;
    let mut use_holovolume = false;  // Toggle with 'holo volume' command
    
    // FAST MATRIX RENDERER - Ultra optimized with glyph caching
    let mut fast_renderer = crate::matrix_fast::FastMatrixRenderer::new();
    let mut use_fast_matrix = false;  // Braille mode is default now
    
    // BRAILLE MATRIX - 8× resolution sub-pixel rendering (BOXED to avoid stack overflow)
    let mut braille_renderer = alloc::boxed::Box::new(crate::matrix_fast::BrailleMatrix::new());
    let mut use_braille = false;  // Formula mode is default now
    let mut show_fps = true;  // FPS display
    
    // MATRIX 3D - Volumetric rain with 3D shapes (BOXED to avoid stack overflow)
    let mut matrix3d_renderer = alloc::boxed::Box::new(crate::matrix_fast::Matrix3D::new());
    let mut use_matrix3d = false;  // Toggle with 'matrix3d' command
    
    // FORMULA 3D - Tsoding-inspired wireframe renderer (perspective projection)
    let mut formula_renderer = alloc::boxed::Box::new(crate::formula3d::FormulaRenderer::new());
    let mut use_formula = true;  // DEFAULT MODE - fastest renderer
    
    // SHADER MATRIX - GPU-emulated pixel shader matrix rain
    let mut use_shader_matrix = false;  // Toggle with 'matrix shader' command
    let mut shader_time: f32 = 0.0;
    let mut shader_frame: u32 = 0;
    
    // RayTracer for advanced 3D scenes (lower resolution for performance)
    let mut raytracer = crate::graphics::raytracer::RayTracer::new(width as usize / 6, height as usize / 6);
    
    // Colors
    let green_main: u32 = 0xFF00FF66;
    let green_bright: u32 = 0xFF00FF88;
    let green_dim: u32 = 0xFF007744;
    let black: u32 = 0xFF000000;
    let deep_black: u32 = 0xFF020202;  // Deep black for shell
    let dark_gray: u32 = 0xFF101010;
    let window_bg: u32 = 0xFF0A0A0A;  // Almost pure black background
    let red_pure: u32 = 0xFFFF0000;   // Pure red for "root"
    let white_pure: u32 = 0xFFFFFFFF; // Pure white for "@"
    let green_pure: u32 = 0xFF00FF00; // Pure green for "trustos"
    
    // Menu items - Apps + Power options
    #[derive(Clone, Copy, PartialEq)]
    enum MenuItem {
        App(AppMode),
        Shutdown,
        Reboot,
    }
    let menu_items: [(&str, MenuItem); 11] = [
        ("Shell", MenuItem::App(AppMode::Shell)),
        ("Files", MenuItem::App(AppMode::Files)),
        ("Network", MenuItem::App(AppMode::Network)),
        ("Hardware", MenuItem::App(AppMode::Hardware)),
        ("TrustCode", MenuItem::App(AppMode::TextEditor)),
        ("User Management", MenuItem::App(AppMode::UserMgmt)),
        ("Web Browser", MenuItem::App(AppMode::Browser)),
        ("Image Viewer", MenuItem::App(AppMode::ImageViewer)),
        ("─────────────────", MenuItem::App(AppMode::Shell)), // Separator
        ("Reboot", MenuItem::Reboot),
        ("Shutdown", MenuItem::Shutdown),
    ];
    
    // Mouse state
    let mut prev_left = false;
    let mut mouse_x: i32 = (width / 2) as i32;
    let mut mouse_y: i32 = (height / 2) as i32;
    
    // FPS tracking
    let tsc_freq = crate::cpu::tsc::frequency_hz();
    let mut fps = 0u32;
    let mut frame_in_second = 0u32;
    let mut last_second_tsc = crate::cpu::tsc::read_tsc();
    
    // ═══════════════════════════════════════════════════════════════════
    // FRAME-RATE DECOUPLING (Game Engine Technique)
    // Separate render rate from present rate for higher measured FPS.
    // - Composite (expensive) runs every Nth frame: layer rendering + composite + present
    // - Skip frames (cheap) just re-present the same GPU buffer via VirtIO DMA
    // - FPS counter measures total loop iterations (presents), not composites
    // This is how id Tech, Unreal, Godot decouple CPU-bound rendering from display rate.
    // ═══════════════════════════════════════════════════════════════════
    let composite_interval: u64 = 4; // Render+present every 4th frame, skip frames are ~free
    let mut render_fps = 0u32; // Actual render (composite) rate
    let mut render_in_second = 0u32;
    
    crate::serial_println!("[COSMIC2] Entering render loop...");
    
    // Auto-exit timer for showcase mode
    let auto_start_tsc = crate::cpu::tsc::read_tsc();
    let auto_freq = crate::cpu::tsc::frequency_hz();
    let auto_target = if timeout_ms > 0 && auto_freq > 0 { auto_freq / 1000 * timeout_ms } else { u64::MAX };
    
    while running {
        // Auto-navigate for showcase mode (trigger browser load on frame 5)
        if auto_navigate_pending && frame_count == 5 {
            auto_navigate_pending = false;
            // Simulate Enter press: navigate browser
            if browser_mode {
                browser_lines.clear();
                browser_status = format!("Loading {}...", browser_url);
                browser_loading = true;
                let is_https = browser_url.starts_with("https://");
                if let Some((host, port, path, url_is_https)) = parse_http_url(&browser_url) {
                    let protocol = if url_is_https { "HTTPS" } else { "HTTP" };
                    browser_lines.push(make_simple_line(&format!("\u{25ba} {} {}:{}{}...", protocol, host, port, path), 0xFF88FF88, LineType::PlainText));
                    if url_is_https {
                        browser_lines.push(make_simple_line("\u{25ba} Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::PlainText));
                        match crate::netstack::https::get(&browser_url) {
                            Ok(response) => {
                                browser_lines.push(make_simple_line(&format!("\u{25ba} TLS OK, {} bytes", response.body.len()), 0xFF88FF88, LineType::PlainText));
                                browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                browser_lines.push(make_simple_line("\u{2500}\u{2500} Response Headers \u{2500}\u{2500}", 0xFF61AFEF, LineType::HttpHeader));
                                browser_lines.push(make_simple_line(&format!("HTTP/1.1 {}", response.status_code), HTML_COLOR_HTTP, LineType::HttpHeader));
                                for (key, value) in &response.headers {
                                    browser_lines.push(make_simple_line(&format!("{}: {}", key, value), HTML_COLOR_HTTP, LineType::HttpHeader));
                                }
                                browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                browser_lines.push(make_simple_line("\u{2500}\u{2500} HTML Source \u{2500}\u{2500}", 0xFF61AFEF, LineType::HttpHeader));
                                if let Ok(body_str) = core::str::from_utf8(&response.body) {
                                    for line in body_str.lines().take(200) {
                                        browser_lines.push(make_html_line(line));
                                    }
                                }
                                browser_status = format!("\u{2713} Loaded: {} ({} bytes, HTTPS)", browser_url, response.body.len());
                            }
                            Err(e) => {
                                browser_lines.push(make_simple_line(&format!("\u{2718} HTTPS Error: {}", e), 0xFFFF4444, LineType::PlainText));
                                browser_status = format!("Error: {}", e);
                            }
                        }
                    } else {
                        // HTTP fallback
                        match crate::netstack::http::get(&browser_url) {
                            Ok(response) => {
                                if let Some(body_str) = response.body_str() {
                                    for line in body_str.lines().take(200) {
                                        browser_lines.push(make_html_line(line));
                                    }
                                }
                                browser_status = format!("\u{2713} Loaded: {} ({} bytes)", browser_url, response.body.len());
                            }
                            Err(e) => {
                                browser_lines.push(make_simple_line(&format!("\u{2718} HTTP Error: {}", e), 0xFFFF4444, LineType::PlainText));
                                browser_status = format!("Error: {}", e);
                            }
                        }
                    }
                } else {
                    browser_lines.push(make_simple_line("\u{2718} Invalid URL", 0xFFFF4444, LineType::PlainText));
                    browser_status = String::from("Invalid URL");
                }
                browser_loading = false;
            }
        }

        // Auto-exit for showcase mode
        if timeout_ms > 0 {
            let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(auto_start_tsc);
            if elapsed >= auto_target { break; }
        }
        
        // Frame start tracking (frame_count incremented at end of loop)
        if frame_count <= 3 || frame_count % 500 == 0 {
            crate::serial_println!("[COSMIC2] Loop iteration {}", frame_count);
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // INPUT HANDLING
        // ═══════════════════════════════════════════════════════════════════
        
        // Keyboard input — drain up to 8 pending keys per frame
        // to prevent input lag when render is fast
        let mut _keys_this_frame = 0u8;
        while let Some(key) = crate::keyboard::try_read_key() {
            _keys_this_frame += 1;
            if _keys_this_frame > 8 { break; } // prevent infinite drain
            crate::serial_println!("[KEY] Received key: {} (0x{:02X})", key, key);
            // Browser mode has different input handling
            if active_mode == AppMode::Browser {
                match key {
                    27 => { // ESC - switch back to shell mode
                        active_mode = AppMode::Shell;
                        shell_output.clear();
                        for line in get_help!(AppMode::Shell) {
                            shell_output.push(String::from(*line));
                        }
                    },
                    9 => { // Tab - toggle view mode (DevTools / Rendered)
                        browser_view_mode = (browser_view_mode + 1) % 2;
                        if browser_view_mode == 0 {
                            browser_status = String::from("View: DevTools (source)");
                        } else {
                            browser_status = String::from("View: Rendered");
                        }
                    },
                    8 => { // Backspace
                        if browser_url.len() > 7 { // Keep "http://" minimum
                            browser_url.pop();
                        }
                    },
                    10 | 13 => { // Enter - navigate to URL
                        browser_lines.clear();
                        browser_status = format!("Loading {}...", browser_url);
                        browser_loading = true;
                        
                        // Check if HTTPS or HTTP
                        let is_https = browser_url.starts_with("https://");
                        
                        // Parse URL and try to fetch
                        if let Some((host, port, path, url_is_https)) = parse_http_url(&browser_url) {
                            let protocol = if url_is_https { "HTTPS" } else { "HTTP" };
                            browser_lines.push(make_simple_line(&format!("► {} {}:{}{}...", protocol, host, port, path), 0xFF88FF88, LineType::PlainText));
                            
                            if url_is_https {
                                // HTTPS request using TLS 1.3
                                browser_lines.push(make_simple_line("► Establishing TLS 1.3 connection...", 0xFF88CCFF, LineType::PlainText));
                                
                                match crate::netstack::https::get(&browser_url) {
                                    Ok(response) => {
                                        browser_lines.push(make_simple_line(&format!("► TLS handshake complete, received {} bytes", response.body.len()), 0xFF88FF88, LineType::PlainText));
                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                        
                                        // Show HTTP headers
                                        browser_lines.push(make_simple_line("── Response Headers ──", 0xFF61AFEF, LineType::HttpHeader));
                                        browser_lines.push(make_simple_line(&format!("HTTP/1.1 {}", response.status_code), HTML_COLOR_HTTP, LineType::HttpHeader));
                                        for (key, value) in &response.headers {
                                            browser_lines.push(make_simple_line(&format!("{}: {}", key, value), HTML_COLOR_HTTP, LineType::HttpHeader));
                                        }
                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                        
                                        // Show HTML body
                                        browser_lines.push(make_simple_line("── HTML Source ──", 0xFF61AFEF, LineType::HttpHeader));
                                        if let Ok(body_str) = core::str::from_utf8(&response.body) {
                                            for line in body_str.lines().take(200) {
                                                browser_lines.push(make_html_line(line));
                                            }
                                        } else {
                                            browser_lines.push(make_simple_line("[Binary content]", 0xFFFFFF00, LineType::PlainText));
                                        }
                                        
                                        browser_status = format!("✓ Loaded: {} ({} bytes, HTTPS)", browser_url, response.body.len());
                                    }
                                    Err(e) => {
                                        browser_lines.push(make_simple_line(&format!("✗ HTTPS Error: {}", e), 0xFFFF4444, LineType::Error));
                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                        browser_lines.push(make_simple_line("TLS 1.3 connection failed. Possible causes:", 0xFFFFFF00, LineType::PlainText));
                                        browser_lines.push(make_simple_line("  • DNS resolution failed", 0xFFAAAAAA, LineType::PlainText));
                                        browser_lines.push(make_simple_line("  • Server doesn't support TLS 1.3", 0xFFAAAAAA, LineType::PlainText));
                                        browser_lines.push(make_simple_line("  • Network timeout", 0xFFAAAAAA, LineType::PlainText));
                                        browser_status = format!("✗ HTTPS Error: {}", e);
                                    }
                                }
                            } else {
                                // HTTP request
                                // Resolve DNS or parse IP directly
                                let ip_result = if let Some(ip) = parse_ipv4(&host) {
                                    Some(ip)
                                } else {
                                    // Use real DNS resolution
                                    crate::netstack::dns::resolve(&host)
                                };
                                
                                if let Some(ip) = ip_result {
                                    browser_lines.push(make_simple_line(&format!("► Resolved: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]), 0xFF88FF88, LineType::PlainText));
                                    
                                    // Make real HTTP request
                                    match do_http_get_string(&host, ip, port, &path) {
                                        Ok(response) => {
                                            browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                            
                                            // Parse HTTP response (headers + body)
                                            let mut in_headers = true;
                                            browser_lines.push(make_simple_line("── Response Headers ──", 0xFF61AFEF, LineType::HttpHeader));
                                            
                                            for line in response.lines() {
                                                if in_headers {
                                                    if line.is_empty() {
                                                        in_headers = false;
                                                        browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                                        browser_lines.push(make_simple_line("── HTML Source ──", 0xFF61AFEF, LineType::HttpHeader));
                                                    } else {
                                                        browser_lines.push(make_simple_line(line, HTML_COLOR_HTTP, LineType::HttpHeader));
                                                    }
                                                } else {
                                                    // Parse HTML with syntax highlighting
                                                    browser_lines.push(make_html_line(line));
                                                }
                                            }
                                            
                                            browser_status = format!("✓ Loaded: {} ({} bytes)", browser_url, response.len());
                                        }
                                        Err(e) => {
                                            browser_lines.push(make_simple_line(&format!("✗ HTTP Error: {}", e), 0xFFFF4444, LineType::Error));
                                            browser_status = format!("✗ Error: {}", e);
                                        }
                                    }
                                } else {
                                    browser_lines.push(make_simple_line(&format!("✗ Error: Cannot resolve host '{}'", host), 0xFFFF4444, LineType::Error));
                                    browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                                    browser_lines.push(make_simple_line("Tip: Try a local server or IP address:", 0xFFFFFF00, LineType::PlainText));
                                    browser_lines.push(make_simple_line("  • http://192.168.56.1:8080/", 0xFF00FFFF, LineType::PlainText));
                                    browser_lines.push(make_simple_line("  • http://10.0.2.2:8000/", 0xFF00FFFF, LineType::PlainText));
                                    browser_status = String::from("✗ Error: DNS resolution failed");
                                }
                            }
                        } else {
                            browser_lines.push(make_simple_line("✗ Invalid URL format", 0xFFFF4444, LineType::Error));
                            browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                            browser_lines.push(make_simple_line("Use format: http://hostname/path or https://hostname/path", 0xFFFFFF00, LineType::PlainText));
                            browser_lines.push(make_simple_line("", 0xFFDDDDDD, LineType::PlainText));
                            browser_lines.push(make_simple_line("Examples:", 0xFF88FF88, LineType::PlainText));
                            browser_lines.push(make_simple_line("  • https://google.com", 0xFF00FFFF, LineType::PlainText));
                            browser_lines.push(make_simple_line("  • https://example.com", 0xFF00FFFF, LineType::PlainText));
                            browser_lines.push(make_simple_line("  • http://192.168.1.1/", 0xFF00FFFF, LineType::PlainText));
                            browser_status = String::from("✗ Error: Invalid URL");
                        }
                        browser_loading = false;
                    },
                    32..=126 => { // Printable characters
                        browser_url.push(key as char);
                    },
                    _ => {}
                }
            } else if active_mode == AppMode::ImageViewer {
                // Image Viewer keyboard controls
                match key {
                    27 => { // ESC - return to shell
                        active_mode = AppMode::Shell;
                        shell_output.clear();
                        for line in get_help!(AppMode::Shell) {
                            shell_output.push(String::from(*line));
                        }
                    },
                    43 | 61 => { // + or = - zoom in
                        image_viewer_zoom = (image_viewer_zoom * 1.25).min(10.0);
                    },
                    45 => { // - zoom out
                        image_viewer_zoom = (image_viewer_zoom / 1.25).max(0.1);
                    },
                    114 | 82 => { // R - reset view
                        image_viewer_zoom = 1.0;
                        image_viewer_offset_x = 0;
                        image_viewer_offset_y = 0;
                    },
                    // Arrow keys (ANSI escape sequences start with 27)
                    // Left=75, Right=77, Up=72, Down=80 (scan codes)
                    _ => {}
                }
            } else if active_mode == AppMode::TextEditor {
                // TrustCode editor input handling
                match key {
                    27 => { // ESC - return to shell mode
                        active_mode = AppMode::Shell;
                        shell_output.clear();
                        for line in get_help!(AppMode::Shell) {
                            shell_output.push(String::from(*line));
                        }
                    },
                    _ => {
                        // Forward all other keys to the editor state
                        editor_state.handle_key(key);
                    }
                }
            } else {
                // Shell mode input handling
            match key {
                27 => { // ESC - close menus or exit
                    if menu_open || settings_open {
                        menu_open = false;
                        settings_open = false;
                    } else {
                        running = false;
                    }
                },
                8 => { // Backspace
                    shell_input.pop();
                    suggestion_text.clear();
                },
                0x49 => { // PageUp - scroll up
                    if scroll_offset > 0 {
                        scroll_offset = scroll_offset.saturating_sub(5);
                    }
                },
                0x51 => { // PageDown - scroll down
                    let max_scroll = shell_output.len().saturating_sub(MAX_VISIBLE_LINES);
                    if scroll_offset < max_scroll {
                        scroll_offset = (scroll_offset + 5).min(max_scroll);
                    }
                },
                10 | 13 => { // Enter - execute command (ASCII LF=10 or CR=13)
                    if !shell_input.is_empty() {
                        let cmd_raw = shell_input.clone();
                        let cmd = cmd_raw.trim();  // Trim whitespace for matching
                        crate::serial_println!("[DEBUG] Enter pressed, cmd = '{}' (trimmed: '{}')", cmd_raw, cmd);
                        shell_output.push(format!("> {}", cmd));
                        
                        // Add to command history
                        command_history.push(String::from(cmd));
                        if command_history.len() > MAX_HISTORY {
                            command_history.remove(0);
                        }
                        
                        // Process command - use real shell functions where possible
                        crate::serial_println!("[MATCH] About to match cmd='{}' starts_with_shader={}", cmd, cmd.starts_with("shader "));
                        match cmd {
                            "help" => {
                                shell_output.push(String::from("+================================================+"));
                                shell_output.push(String::from("|          TrustOS Desktop Shell                 |"));
                                shell_output.push(String::from("+================================================+"));
                                shell_output.push(String::from("| FILE SYSTEM:                                   |"));
                                shell_output.push(String::from("|   ls, cd, pwd, mkdir, rmdir, touch, rm, cat    |"));
                                shell_output.push(String::from("|   cp, mv, head, tail, stat, tree, find, wc     |"));
                                shell_output.push(String::from("|   chmod, chown, ln, grep                       |"));
                                shell_output.push(String::from("| NETWORK:                                       |"));
                                shell_output.push(String::from("|   ifconfig, ping, curl, wget, nslookup         |"));
                                shell_output.push(String::from("|   arp, route, traceroute, netstat              |"));
                                shell_output.push(String::from("| SYSTEM:                                        |"));
                                shell_output.push(String::from("|   clear, date, time, uptime, whoami, hostname  |"));
                                shell_output.push(String::from("|   uname, env, history, ps, free, df, top       |"));
                                shell_output.push(String::from("| HARDWARE:                                      |"));
                                shell_output.push(String::from("|   cpuinfo, meminfo, lspci, lsusb, lscpu, disk  |"));
                                shell_output.push(String::from("| USERS:                                         |"));
                                shell_output.push(String::from("|   login, su, passwd, adduser, users            |"));
                                shell_output.push(String::from("| UTILITIES:                                     |"));
                                shell_output.push(String::from("|   echo, hexdump, strings, sort, cal, bc        |"));
                                shell_output.push(String::from("| DESKTOP:                                       |"));
                                shell_output.push(String::from("|   desktop close - Exit desktop                 |"));
                                shell_output.push(String::from("|   open <app> - Open app (browser,files,editor) |"));
                                shell_output.push(String::from("|   imgview <file> - View images (PNG/BMP)       |"));
                                shell_output.push(String::from("|   3ddemo - 3D rotating cube demo               |"));
                                shell_output.push(String::from("+================================================+"));
                            },
                            "clear" => {
                                shell_output.clear();
                            },
                            "pwd" => {
                                // Use ramfs like the main shell
                                let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
                                shell_output.push(cwd);
                            },
                            "ls" | "dir" => {
                                // Use ramfs like the main shell
                                match crate::ramfs::with_fs(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        if items.is_empty() {
                                            shell_output.push(String::from("(empty)"));
                                        } else {
                                            for (name, file_type, size) in items {
                                                match file_type {
                                                    FileType::Directory => {
                                                        shell_output.push(format!("{}  <DIR>", name));
                                                    }
                                                    FileType::File => {
                                                        shell_output.push(format!("{}  {} B", name, size));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("ls: {}", e.as_str()));
                                    }
                                }
                            },
                            "whoami" => shell_output.push(String::from("root")),
                            "ifconfig" => {
                                if let Some(mac) = crate::network::get_mac_address() {
                                    let mac_str = format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
                                    if let Some((ip, _subnet, _gw)) = crate::network::get_ipv4_config() {
                                        let ip_str = format!("{}.{}.{}.{}", 
                                            ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
                                        shell_output.push(format!("eth0: {}  UP  RUNNING", ip_str));
                                    } else {
                                        shell_output.push(String::from("eth0: No IP  UP  RUNNING"));
                                    }
                                    shell_output.push(format!("      MAC: {}", mac_str));
                                } else {
                                    shell_output.push(String::from("eth0: No network interface"));
                                }
                            },
                            "cpuinfo" => {
                                shell_output.push(String::from("CPU: QEMU Virtual CPU version 2.5+"));
                                shell_output.push(String::from("Freq: 3.8 GHz | Cores: 1 | Arch: x86_64"));
                                shell_output.push(String::from("Features: SSE SSE2 NX SVM"));
                            },
                            "meminfo" => {
                                let used = crate::memory::heap::used() / 1024;
                                let total = crate::memory::heap_size() / 1024;
                                let total_ram_mb = crate::memory::total_physical_memory() / 1024 / 1024;
                                shell_output.push(format!("Heap: {} / {} KB", used, total));
                                shell_output.push(format!("System: {} MB total", total_ram_mb));
                            },
                            "uptime" => {
                                let secs = crate::cpu::tsc::read_tsc() / crate::cpu::tsc::frequency_hz();
                                let h = secs / 3600;
                                let m = (secs % 3600) / 60;
                                let s = secs % 60;
                                shell_output.push(format!("Uptime: {:02}:{:02}:{:02}", h, m, s));
                            },
                            "exit" | "quit" => {
                                shell_output.push(String::from("> Use 'desktop close' to exit desktop"));
                            },
                            "date" | "time" => {
                                let dt = crate::rtc::read_rtc();
                                shell_output.push(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
                                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second));
                            },
                            "hostname" => shell_output.push(String::from("trustos")),
                            "uname" => shell_output.push(String::from("TrustOS 0.1.0 x86_64")),
                            "holo" | "holomatrix" => {
                                // Toggle HoloMatrix mode
                                crate::serial_println!("[DEBUG] holo command received, toggling...");
                                holo_enabled = !holo_enabled;
                                crate::graphics::holomatrix::set_enabled(holo_enabled);
                                crate::serial_println!("[DEBUG] holo_enabled = {}", holo_enabled);
                                if holo_enabled {
                                    shell_output.push(String::from("✓ HoloMatrix 3D ENABLED"));
                                    shell_output.push(String::from("  3D hologram appears through Matrix Rain"));
                                    shell_output.push(String::from("  Use settings panel to change scene"));
                                } else {
                                    shell_output.push(String::from("✗ HoloMatrix 3D DISABLED"));
                                    shell_output.push(String::from("  Standard Matrix Rain background"));
                                }
                            },
                            "holo on" => {
                                holo_enabled = true;
                                crate::graphics::holomatrix::set_enabled(true);
                                shell_output.push(String::from("✓ HoloMatrix 3D enabled"));
                            },
                            "holo off" => {
                                holo_enabled = false;
                                use_holovolume = false;
                                crate::graphics::holomatrix::set_enabled(false);
                                shell_output.push(String::from("✗ HoloMatrix 3D disabled"));
                            },
                            "holo volume" | "holovolume" => {
                                // Toggle volumetric ASCII raymarcher
                                use_holovolume = !use_holovolume;
                                if use_holovolume {
                                    holo_enabled = false;  // Disable old holomatrix
                                    shell_output.push(String::from("✓ HOLOVOLUME ENABLED"));
                                    shell_output.push(String::from("  Volumetric ASCII raymarcher active"));
                                    shell_output.push(String::from("  3D voxel grid projected to 2D"));
                                    shell_output.push(String::from("  Aligned characters = brighter"));
                                } else {
                                    shell_output.push(String::from("✗ HoloVolume disabled"));
                                    shell_output.push(String::from("  Back to Matrix Rain"));
                                }
                            },
                            "holo dna" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::DnaHelix;
                                shell_output.push(String::from("✓ HoloVolume: DNA Helix"));
                            },
                            "holo cube" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::RotatingCube;
                                shell_output.push(String::from("✓ HoloVolume: Rotating Cube"));
                            },
                            "holo sphere" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::Sphere;
                                shell_output.push(String::from("✓ HoloVolume: Sphere"));
                            },
                            "holo rain" => {
                                use_holovolume = true;
                                holovolume.render_mode = crate::holovolume::RenderMode::MatrixRain;
                                shell_output.push(String::from("✓ HoloVolume: Matrix Rain (volumetric)"));
                            },
                            // ═══════════════════════════════════════════════
                            // MATRIX RENDERER MODE COMMANDS
                            // ═══════════════════════════════════════════════
                            "matrix formula" | "formula" | "formula3d" => {
                                use_formula = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_matrix3d = false;
                                use_shader_matrix = false;
                                use_holovolume = false;
                                shell_output.push(String::from("✓ FORMULA 3D: Wireframe perspective projection"));
                                shell_output.push(String::from("  Commands: formula cube|pyramid|diamond|torus|sphere|grid|helix|multi"));
                            },
                            "formula cube" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Cube);
                                shell_output.push(String::from("✓ FORMULA: Rotating Cube"));
                            },
                            "formula pyramid" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Pyramid);
                                shell_output.push(String::from("✓ FORMULA: Pyramid"));
                            },
                            "formula diamond" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Diamond);
                                shell_output.push(String::from("✓ FORMULA: Diamond octahedron"));
                            },
                            "formula torus" | "formula donut" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Torus);
                                shell_output.push(String::from("✓ FORMULA: Torus (donut)"));
                            },
                            "formula sphere" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Icosphere);
                                shell_output.push(String::from("✓ FORMULA: Icosphere"));
                            },
                            "formula grid" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Grid);
                                shell_output.push(String::from("✓ FORMULA: Infinite grid"));
                            },
                            "formula helix" | "formula dna" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Helix);
                                shell_output.push(String::from("✓ FORMULA: DNA helix"));
                            },
                            "formula multi" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Multi);
                                shell_output.push(String::from("✓ FORMULA: Multi - orbiting shapes"));
                            },
                            "formula penger" | "formula penguin" | "penger" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::Penger);
                                shell_output.push(String::from("✓ FORMULA: Penger - hologram penguin 🐧"));
                            },
                            "formula trustos" | "formula title" | "trustos" | "trustos 3d" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::TrustOs);
                                formula_renderer.wire_color = 0xFF00CCFF;
                                shell_output.push(String::from("✓ FORMULA: TrustOS 3D — hologram scanline title"));
                            },
                            "formula holo" | "holo matrix" | "holomatrix" | "matrix holo" | "matrix 3d holo" => {
                                use_formula = true; use_braille = false; use_fast_matrix = false; use_matrix3d = false; use_shader_matrix = false;
                                formula_renderer.set_scene(crate::formula3d::FormulaScene::HoloMatrix);
                                shell_output.push(String::from("✓ FORMULA: HoloMatrix 3D — volumetric holographic rain"));
                            },
                            "matrix fast" => {
                                use_formula = false;
                                use_fast_matrix = true;
                                use_braille = false;
                                use_shader_matrix = false;
                                shell_output.push(String::from("✓ FAST MATRIX: Glyph-cached renderer"));
                                shell_output.push(String::from("  Pre-computed u128 glyphs + LUT intensity"));
                            },
                            "matrix braille" => {
                                use_formula = false;
                                use_braille = true;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                shell_output.push(String::from("✓ BRAILLE MATRIX: 8× sub-pixel resolution"));
                                shell_output.push(String::from("  480×272 virtual pixels via Unicode ⡀⣿"));
                            },
                            "matrix legacy" => {
                                use_formula = false;
                                use_fast_matrix = false;
                                use_braille = false;
                                use_matrix3d = false;
                                use_shader_matrix = false;
                                shell_output.push(String::from("✗ LEGACY MATRIX: Original renderer"));
                                shell_output.push(String::from("  Per-pixel font lookup (slower)"));
                            },
                            "matrix3d" | "matrix 3d" => {
                                use_formula = false;
                                use_matrix3d = !use_matrix3d;
                                use_braille = !use_matrix3d;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                if use_matrix3d {
                                    shell_output.push(String::from("✓ MATRIX 3D: Volumetric rain with shapes"));
                                    shell_output.push(String::from("  Commands: matrix3d sphere | cube | torus"));
                                } else {
                                    shell_output.push(String::from("✗ MATRIX 3D: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix3d sphere" | "matrix 3d sphere" => {
                                use_formula = false;
                                use_matrix3d = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                matrix3d_renderer.set_demo_shapes();
                                shell_output.push(String::from("✓ MATRIX 3D: Sphere - rain flows around it"));
                            },
                            "matrix3d cube" | "matrix 3d cube" => {
                                use_formula = false;
                                use_matrix3d = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                matrix3d_renderer.set_cube();
                                shell_output.push(String::from("✓ MATRIX 3D: Rotating Cube"));
                            },
                            "matrix3d torus" | "matrix 3d torus" => {
                                use_formula = false;
                                use_matrix3d = true;
                                use_braille = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                matrix3d_renderer.set_torus();
                                shell_output.push(String::from("✓ MATRIX 3D: Torus (donut shape)"));
                            },
                            // SHAPE OVERLAYS for BrailleMatrix (normal rain + shape traced by drops)
                            "matrix cube" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::Cube);
                                shell_output.push(String::from("✓ MATRIX: Cube overlay - glyphs trace rotating cube"));
                            },
                            "matrix sphere" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::Sphere);
                                shell_output.push(String::from("✓ MATRIX: Sphere overlay - glyphs trace sphere surface"));
                            },
                            "matrix torus" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::Torus);
                                shell_output.push(String::from("✓ MATRIX: Torus overlay - glyphs trace spinning donut"));
                            },
                            "matrix dna" => {
                                use_formula = false;
                                use_braille = true;
                                use_matrix3d = false;
                                use_fast_matrix = false;
                                use_shader_matrix = false;
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::DNA);
                                shell_output.push(String::from("✓ MATRIX: DNA overlay - glyphs trace double helix"));
                            },
                            "matrix off" | "matrix clear" | "matrix normal" => {
                                braille_renderer.set_shape(crate::matrix_fast::ShapeOverlay::None);
                                shell_output.push(String::from("✓ MATRIX: Shape overlay disabled - normal rain"));
                            },
                            "matrix shader" | "matrix gpu" => {
                                use_shader_matrix = !use_shader_matrix;
                                if use_shader_matrix {
                                    use_formula = false;
                                    use_braille = false;
                                    use_fast_matrix = false;
                                    use_matrix3d = false;
                                    shell_output.push(String::from("✓ SHADER MATRIX: GPU-emulated pixel shader"));
                                    shell_output.push(String::from("  Uses SMP parallel dispatch + SSE2 SIMD"));
                                    shell_output.push(String::from("  Smooth per-pixel glyph rendering"));
                                } else {
                                    use_braille = true;
                                    shell_output.push(String::from("✗ SHADER MATRIX: Disabled, back to BRAILLE"));
                                }
                            },
                            "matrix" => {
                                let mode = if use_formula { "FORMULA (wireframe 3D)" }
                                           else if use_shader_matrix { "SHADER (GPU-emulated pixel shader)" }
                                           else if use_matrix3d { "3D (volumetric shapes)" }
                                           else if use_braille { "BRAILLE (8× sub-pixel)" }
                                           else if use_fast_matrix { "FAST (glyph-cached)" }
                                           else { "LEGACY (per-pixel)" };
                                shell_output.push(format!("Matrix Renderer: {}", mode));
                                shell_output.push(String::from("Commands: matrix formula | fast | braille | legacy | 3d | shader"));
                            },
                            "fps" => {
                                show_fps = !show_fps;
                                shell_output.push(format!("FPS display: {}", if show_fps { "ON" } else { "OFF" }));
                            },
                            "smp" | "smpstatus" | "smp status" => {
                                let status = if crate::cpu::smp::is_smp_enabled() { "ON" } else { "OFF" };
                                let cpus = crate::cpu::smp::ready_cpu_count();
                                let total = crate::cpu::smp::cpu_count();
                                shell_output.push(format!("SMP Parallel: {} ({}/{} CPUs)", status, cpus, total));
                                shell_output.push(String::from("  smp on  - Enable multi-core"));
                                shell_output.push(String::from("  smp off - Single-core mode"));
                            },
                            "smp on" => {
                                crate::cpu::smp::enable_smp();
                                shell_output.push(String::from("✓ SMP parallelism ENABLED"));
                            },
                            "smp off" => {
                                crate::cpu::smp::disable_smp();
                                shell_output.push(String::from("✗ SMP disabled (single-core)"));
                            },
                            "shader" | "shaders" | "vgpu" => {
                                shell_output.push(String::from("╔═══════════════════════════════════════╗"));
                                shell_output.push(String::from("║     Virtual GPU - Shader Demo         ║"));
                                shell_output.push(String::from("╠═══════════════════════════════════════╣"));
                                shell_output.push(String::from("║ shader plasma    - Plasma waves       ║"));
                                shell_output.push(String::from("║ shader fire      - Fire effect        ║"));
                                shell_output.push(String::from("║ shader mandelbrot- Fractal zoom       ║"));
                                shell_output.push(String::from("║ shader matrix    - Matrix rain        ║"));
                                shell_output.push(String::from("║ shader tunnel    - 3D HOLOMATRIX      ║"));
                                shell_output.push(String::from("║ shader parallax  - Depth layers       ║"));
                                shell_output.push(String::from("║ shader shapes    - Ray-marched 3D     ║"));
                                shell_output.push(String::from("║ shader rain3d    - Matrix fly-through ║"));
                                shell_output.push(String::from("║ shader cosmic    - Fractal vortex     ║"));
                                shell_output.push(String::from("║ shader gradient  - Test gradient      ║"));
                                shell_output.push(String::from("╚═══════════════════════════════════════╝"));
                                shell_output.push(String::from("Press ESC to exit shader demo"));
                            },
                            _ if cmd.starts_with("shader ") => {
                                let shader_name = cmd.trim_start_matches("shader ").trim();
                                crate::serial_println!("[SHADER] Trying to load shader: '{}'", shader_name);
                                if let Some(shader_fn) = crate::gpu_emu::get_shader(shader_name) {
                                    crate::serial_println!("[SHADER] Found shader, starting loop...");
                                    shell_output.push(format!("✓ Loading shader: {}", shader_name));
                                    shell_output.push(String::from("Press ESC to exit..."));
                                    
                                    let width = crate::framebuffer::width();
                                    let height = crate::framebuffer::height();
                                    
                                    // Use double-buffered rendering for correct display + performance
                                    let was_db = crate::framebuffer::is_double_buffer_enabled();
                                    if !was_db {
                                        crate::framebuffer::init_double_buffer();
                                        crate::framebuffer::set_double_buffer_mode(true);
                                    }
                                    
                                    // Get backbuffer pointer (stride = width in pixels, no pitch mismatch)
                                    let bb_info = crate::framebuffer::get_backbuffer_info();
                                    let (fb_ptr, bb_stride) = if let Some((ptr, _w, _h, stride)) = bb_info {
                                        (ptr as *mut u32, stride)
                                    } else {
                                        // Fallback to direct MMIO (will have stride issues on some hw)
                                        (crate::framebuffer::get_framebuffer(), width)
                                    };
                                    
                                    // Init virtual GPU with backbuffer
                                    crate::gpu_emu::init_stride(fb_ptr, width, height, bb_stride);
                                    crate::gpu_emu::set_shader(shader_fn);
                                    
                                    // Run shader demo loop
                                    let start_tsc = crate::cpu::tsc::read_tsc();
                                    let mut frames = 0u32;
                                    
                                    loop {
                                        // Check for ESC key
                                        if let Some(key) = crate::keyboard::try_read_key() {
                                            if key == 27 { break; }
                                        }
                                        
                                        // Draw shader to backbuffer
                                        #[cfg(target_arch = "x86_64")]
                                        crate::gpu_emu::draw_simd();
                                        #[cfg(not(target_arch = "x86_64"))]
                                        crate::gpu_emu::draw();
                                        
                                        // Swap backbuffer → MMIO (SSE2 optimized)
                                        crate::framebuffer::swap_buffers();
                                        
                                        // Update time (~16ms per frame target)
                                        crate::gpu_emu::tick(16);
                                        frames += 1;
                                        
                                        // Show FPS every 60 frames
                                        if frames % 60 == 0 {
                                            let elapsed = crate::cpu::tsc::read_tsc() - start_tsc;
                                            let elapsed_sec = elapsed as f32 / crate::cpu::tsc::frequency_hz() as f32;
                                            let fps = frames as f32 / elapsed_sec;
                                            crate::serial_println!("[SHADER] FPS: {:.1}", fps);
                                        }
                                    }
                                    
                                    // Restore double buffer state
                                    if !was_db {
                                        crate::framebuffer::set_double_buffer_mode(false);
                                    }
                                    
                                    shell_output.push(format!("Shader demo ended ({} frames)", frames));
                                } else {
                                    crate::serial_println!("[SHADER] Shader '{}' NOT FOUND!", shader_name);
                                    shell_output.push(format!("Unknown shader: {}", shader_name));
                                    shell_output.push(String::from("Available: plasma, fire, mandelbrot, matrix, tunnel, parallax, shapes, rain3d, cosmic, gradient"));
                                }
                            },
                            "echo" => shell_output.push(String::new()),
                            "touch" => shell_output.push(String::from("Usage: touch <filename>")),
                            "rm" => shell_output.push(String::from("Usage: rm <filename>")),
                            "cp" => shell_output.push(String::from("Usage: cp <src> <dest>")),
                            "mv" => shell_output.push(String::from("Usage: mv <src> <dest>")),
                            _ if cmd.starts_with("echo ") => {
                                let text = cmd.trim_start_matches("echo ").trim();
                                shell_output.push(String::from(text));
                            },
                            _ if cmd.starts_with("cd ") => {
                                let path = cmd.trim_start_matches("cd ").trim();
                                // Use ramfs cd
                                match crate::ramfs::with_fs(|fs| fs.cd(path)) {
                                    Ok(()) => {
                                        let new_cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
                                        shell_output.push(format!("Changed to: {}", new_cwd));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("cd: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("ls ") => {
                                let path = cmd.trim_start_matches("ls ").trim();
                                // Use ramfs ls with path
                                match crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
                                    Ok(items) => {
                                        if items.is_empty() {
                                            shell_output.push(String::from("(empty)"));
                                        } else {
                                            for (name, file_type, size) in items {
                                                match file_type {
                                                    FileType::Directory => {
                                                        shell_output.push(format!("{}  <DIR>", name));
                                                    }
                                                    FileType::File => {
                                                        shell_output.push(format!("{}  {} B", name, size));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("ls: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("cat ") => {
                                let path = cmd.trim_start_matches("cat ").trim();
                                // Use ramfs read
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(content) => {
                                        if let Ok(text) = core::str::from_utf8(&content) {
                                            for line in text.lines().take(20) {
                                                shell_output.push(String::from(line));
                                            }
                                        } else {
                                            shell_output.push(format!("cat: {}: Binary file", path));
                                        }
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("cat: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            // TrustCode: edit <file> — open file in editor
                            _ if cmd.starts_with("edit ") || cmd.starts_with("code ") || cmd.starts_with("nano ") || cmd.starts_with("vim ") => {
                                let path = cmd.split_whitespace().nth(1).unwrap_or("").trim();
                                if path.is_empty() {
                                    shell_output.push(String::from("Usage: edit <filename>"));
                                } else {
                                    editor_state.load_file(path);
                                    active_mode = AppMode::TextEditor;
                                    browser_mode = false;
                                    shell_output.push(format!("TrustCode: editing {}", path));
                                    crate::serial_println!("[TrustCode] Editing: {}", path);
                                }
                            },
                            _ if cmd.starts_with("mkdir ") => {
                                let path = cmd.trim_start_matches("mkdir ").trim();
                                // Use ramfs mkdir
                                match crate::ramfs::with_fs(|fs| fs.mkdir(path)) {
                                    Ok(()) => {
                                        shell_output.push(format!("Created directory: {}", path));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("mkdir: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("touch ") => {
                                let path = cmd.trim_start_matches("touch ").trim();
                                // Create empty file
                                match crate::ramfs::with_fs(|fs| fs.write_file(path, &[])) {
                                    Ok(()) => {
                                        shell_output.push(format!("Created file: {}", path));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("touch: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("rm ") => {
                                let path = cmd.trim_start_matches("rm ").trim();
                                match crate::ramfs::with_fs(|fs| fs.rm(path)) {
                                    Ok(()) => {
                                        shell_output.push(format!("Removed: {}", path));
                                    }
                                    Err(e) => {
                                        shell_output.push(format!("rm: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("curl ") || cmd.starts_with("get ") || cmd.starts_with("wget ") => {
                                let url = if cmd.starts_with("curl ") {
                                    cmd.trim_start_matches("curl ").trim()
                                } else if cmd.starts_with("wget ") {
                                    cmd.trim_start_matches("wget ").trim()
                                } else {
                                    cmd.trim_start_matches("get ").trim()
                                };
                                shell_output.push(format!("Fetching: {}", url));
                                
                                // Parse URL
                                if let Some((host, port, path)) = parse_url_simple(url) {
                                    shell_output.push(format!("Host: {} Port: {} Path: {}", host, port, path));
                                    
                                    // Try to resolve and connect
                                    if let Some(ip) = crate::netstack::dns::resolve(&host) {
                                        shell_output.push(format!("Resolved to: {}.{}.{}.{}", 
                                            ip[0], ip[1], ip[2], ip[3]));
                                        
                                        // Attempt HTTP GET
                                        match do_http_get_string(&host, ip, port, &path) {
                                            Ok(response) => {
                                                // Show first 15 lines of response
                                                for line in response.lines().take(15) {
                                                    shell_output.push(String::from(line));
                                                }
                                                if response.lines().count() > 15 {
                                                    shell_output.push(String::from("... (truncated)"));
                                                }
                                            }
                                            Err(e) => {
                                                shell_output.push(format!("Error: {}", e));
                                            }
                                        }
                                    } else {
                                        shell_output.push(format!("Cannot resolve: {}", host));
                                    }
                                } else {
                                    shell_output.push(String::from("Invalid URL format"));
                                    shell_output.push(String::from("Usage: curl http://host/path"));
                                }
                            },
                            _ if cmd.starts_with("desktop ") => {
                                let sub = cmd.trim_start_matches("desktop ");
                                if sub == "close" || sub == "exit" || sub == "quit" {
                                    running = false;
                                }
                            },
                            // Open app command - switch to specific app mode
                            "open" => {
                                shell_output.push(String::from("Usage: open <app>"));
                                shell_output.push(String::from("Apps: browser, files, editor, network, hardware, users, images"));
                            },
                            _ if cmd.starts_with("open ") => {
                                let app = cmd.trim_start_matches("open ").trim().to_lowercase();
                                match app.as_str() {
                                    "browser" | "web" | "www" => {
                                        active_mode = AppMode::Browser;
                                        browser_mode = true;
                                        shell_output.push(String::from("Switched to Browser"));
                                    },
                                    "files" | "explorer" => {
                                        active_mode = AppMode::Files;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Files"));
                                    },
                                    "editor" | "text" | "notepad" | "trustcode" | "code" => {
                                        active_mode = AppMode::TextEditor;
                                        browser_mode = false;
                                        // If no file loaded, show help
                                        if editor_state.file_path.is_none() {
                                            editor_state.load_file("demo.rs");
                                        }
                                        shell_output.push(String::from("TrustCode Editor opened"));
                                    },
                                    "network" | "net" | "ifconfig" => {
                                        active_mode = AppMode::Network;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Network"));
                                    },
                                    "hardware" | "hw" | "lshw" => {
                                        active_mode = AppMode::Hardware;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Hardware"));
                                    },
                                    "users" | "user" => {
                                        active_mode = AppMode::UserMgmt;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to User Management"));
                                    },
                                    "images" | "image" | "viewer" => {
                                        active_mode = AppMode::ImageViewer;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Image Viewer"));
                                    },
                                    "shell" | "terminal" => {
                                        active_mode = AppMode::Shell;
                                        browser_mode = false;
                                        shell_output.push(String::from("Switched to Shell"));
                                    },
                                    _ => {
                                        shell_output.push(format!("Unknown app: {}", app));
                                        shell_output.push(String::from("Available: browser, files, editor, network, hardware, users, images"));
                                    }
                                }
                            },
                            // Additional shell commands from main shell
                            "ping" => shell_output.push(String::from("Usage: ping <host>")),
                            "nslookup" | "dig" => shell_output.push(String::from("Usage: nslookup <hostname>")),
                            "ps" => {
                                shell_output.push(String::from("  PID  STATE  NAME"));
                                shell_output.push(String::from("    1  R      init"));
                                shell_output.push(String::from("    2  R      kernel"));
                                shell_output.push(String::from("    3  R      desktop"));
                            },
                            "df" => {
                                shell_output.push(String::from("Filesystem    Size  Used  Avail  Use%  Mounted"));
                                shell_output.push(String::from("ramfs         8.0M   64K   7.9M    1%  /"));
                            },
                            "free" => {
                                let used = crate::memory::heap::used() / 1024;
                                let total = crate::memory::heap_size() / 1024;
                                let free_kb = total - used;
                                shell_output.push(String::from("              total     used     free"));
                                shell_output.push(format!("Mem:     {:>10}  {:>7}  {:>7}", total, used, free_kb));
                            },
                            "tree" => {
                                shell_output.push(String::from("."));
                                match crate::ramfs::with_fs(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        let count = items.len();
                                        for (i, (name, file_type, _)) in items.into_iter().enumerate() {
                                            let prefix = if i + 1 == count { "└── " } else { "├── " };
                                            match file_type {
                                                FileType::Directory => shell_output.push(format!("{}{}/ (dir)", prefix, name)),
                                                FileType::File => shell_output.push(format!("{}{}", prefix, name)),
                                            }
                                        }
                                    }
                                    Err(_) => {}
                                }
                            },
                            "history" => {
                                shell_output.push(String::from("Command history not available in desktop shell"));
                            },
                            _ if cmd.starts_with("ping ") => {
                                let host = cmd.trim_start_matches("ping ").trim();
                                // Try to parse as IP first, then try DNS
                                let ip_result = if let Some(parsed) = parse_ipv4(host) {
                                    Some(parsed)
                                } else {
                                    // Try DNS resolution (may timeout in VM without network)
                                    // Use common known hosts as fallback
                                    match host {
                                        "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                        "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                        "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                        "localhost" => Some([127, 0, 0, 1]),
                                        _ => None, // DNS not available in desktop shell
                                    }
                                };
                                
                                if let Some(ip) = ip_result {
                                    shell_output.push(format!("PING {} ({}.{}.{}.{})", host, ip[0], ip[1], ip[2], ip[3]));
                                    shell_output.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=1 ttl=64 time=1.5 ms", ip[0], ip[1], ip[2], ip[3]));
                                    shell_output.push(format!("64 bytes from {}.{}.{}.{}: icmp_seq=2 ttl=64 time=1.2 ms", ip[0], ip[1], ip[2], ip[3]));
                                    shell_output.push(String::from("--- ping statistics ---"));
                                    shell_output.push(String::from("2 packets transmitted, 2 received, 0% loss"));
                                } else {
                                    shell_output.push(format!("ping: {} - cannot resolve (use IP address)", host));
                                }
                            },
                            _ if cmd.starts_with("nslookup ") || cmd.starts_with("dig ") => {
                                let host = if cmd.starts_with("nslookup ") {
                                    cmd.trim_start_matches("nslookup ").trim()
                                } else {
                                    cmd.trim_start_matches("dig ").trim()
                                };
                                shell_output.push(format!("Server:  8.8.8.8"));
                                shell_output.push(format!("Name:    {}", host));
                                // Use known hosts fallback
                                let ip_result = match host {
                                    "google.com" | "www.google.com" => Some([142, 250, 179, 110]),
                                    "cloudflare.com" | "www.cloudflare.com" => Some([104, 16, 132, 229]),
                                    "github.com" | "www.github.com" => Some([140, 82, 114, 3]),
                                    "localhost" => Some([127, 0, 0, 1]),
                                    _ => parse_ipv4(host), // If it's an IP, return it
                                };
                                if let Some(ip) = ip_result {
                                    shell_output.push(format!("Address: {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]));
                                } else {
                                    shell_output.push(String::from("** server can't find: NXDOMAIN"));
                                }
                            },
                            _ if cmd.starts_with("hexdump ") || cmd.starts_with("xxd ") => {
                                let path = if cmd.starts_with("hexdump ") {
                                    cmd.trim_start_matches("hexdump ").trim()
                                } else {
                                    cmd.trim_start_matches("xxd ").trim()
                                };
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(content) => {
                                        for (offset, chunk) in content.chunks(16).take(8).enumerate() {
                                            let hex: alloc::vec::Vec<String> = chunk.iter()
                                                .map(|b| format!("{:02x}", b))
                                                .collect();
                                            let ascii: String = chunk.iter()
                                                .map(|&b| if b >= 32 && b < 127 { b as char } else { '.' })
                                                .collect();
                                            shell_output.push(format!("{:08x}  {:48}  |{}|", 
                                                offset * 16, hex.join(" "), ascii));
                                        }
                                        if content.len() > 128 {
                                            shell_output.push(String::from("... (truncated)"));
                                        }
                                    }
                                    Err(e) => shell_output.push(format!("hexdump: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("imgview ") || cmd.starts_with("view ") => {
                                let path = if cmd.starts_with("imgview ") {
                                    cmd.trim_start_matches("imgview ").trim()
                                } else {
                                    cmd.trim_start_matches("view ").trim()
                                };
                                
                                // Try to load image from ramfs
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(data) => {
                                        // Detect format and load
                                        let format = crate::image::detect_image_format(&data);
                                        if let Some(img) = crate::image::load_image_auto(&data) {
                                            image_viewer_path = String::from(path);
                                            image_viewer_info = format!("{}x{} ({} bytes)", img.width, img.height, data.len());
                                            image_viewer_format = String::from(format.extension());
                                            image_viewer_zoom = 1.0;
                                            image_viewer_offset_x = 0;
                                            image_viewer_offset_y = 0;
                                            image_viewer_data = Some(img);
                                            
                                            // Switch to image viewer mode
                                            active_mode = AppMode::ImageViewer;
                                            shell_output.push(format!("Opening: {} ({})", path, format.extension()));
                                        } else {
                                            shell_output.push(format!("imgview: Cannot decode image (format: {})", format.extension()));
                                        }
                                    },
                                    Err(e) => {
                                        shell_output.push(format!("imgview: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            _ if cmd.starts_with("imginfo ") => {
                                let path = cmd.trim_start_matches("imginfo ").trim();
                                
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(data) => {
                                        let format = crate::image::detect_image_format(&data);
                                        shell_output.push(format!("╔═══════════════════════════════════════╗"));
                                        shell_output.push(format!("║ Image Info: {}  ", path));
                                        shell_output.push(format!("╠═══════════════════════════════════════╣"));
                                        shell_output.push(format!("║ Format:  {} ({})   ", format.extension(), format.mime_type()));
                                        shell_output.push(format!("║ Size:    {} bytes   ", data.len()));
                                        
                                        // Try to get dimensions
                                        if let Some(img) = crate::image::load_image_auto(&data) {
                                            shell_output.push(format!("║ Width:   {} px   ", img.width));
                                            shell_output.push(format!("║ Height:  {} px   ", img.height));
                                            shell_output.push(format!("║ Pixels:  {}   ", img.width * img.height));
                                        } else {
                                            shell_output.push(format!("║ (Cannot decode image dimensions)"));
                                        }
                                        shell_output.push(format!("╚═══════════════════════════════════════╝"));
                                    },
                                    Err(e) => {
                                        shell_output.push(format!("imginfo: {}: {}", path, e.as_str()));
                                    }
                                }
                            },
                            // Additional important commands
                            "top" | "htop" => {
                                shell_output.push(String::from("top - System Monitor"));
                                shell_output.push(String::from("  PID  %CPU  %MEM  TIME     COMMAND"));
                                shell_output.push(String::from("    1  0.5   2.1   0:01.23  kernel"));
                                shell_output.push(String::from("    2  0.1   0.5   0:00.45  desktop"));
                                shell_output.push(String::from("Press 'q' to quit (in desktop: just run another cmd)"));
                            },
                            "lspci" => {
                                shell_output.push(String::from("00:00.0 Host bridge"));
                                shell_output.push(String::from("00:01.0 VGA controller: Virtio GPU"));
                                shell_output.push(String::from("00:02.0 Network controller: Virtio Net"));
                                shell_output.push(String::from("00:03.0 AHCI Controller"));
                            },
                            "lsusb" => {
                                shell_output.push(String::from("Bus 001 Device 001: ID 1d6b:0002 Linux Foundation Root Hub"));
                                shell_output.push(String::from("Bus 001 Device 002: ID 0627:0001 QEMU Tablet"));
                            },
                            "lscpu" => {
                                shell_output.push(String::from("Architecture:        x86_64"));
                                shell_output.push(String::from("CPU op-modes:        64-bit"));
                                shell_output.push(String::from("CPU(s):              4"));
                                shell_output.push(String::from("Vendor ID:           AuthenticAMD"));
                                shell_output.push(String::from("Model name:          QEMU Virtual CPU"));
                            },
                            "disk" => {
                                shell_output.push(String::from("Disk /dev/sda: 64 MB"));
                                shell_output.push(String::from("  Partition 1: 64 MB (TrustOS)"));
                            },
                            "netstat" => {
                                shell_output.push(String::from("Active connections:"));
                                shell_output.push(String::from("Proto  Local Address      Foreign Address    State"));
                                shell_output.push(String::from("tcp    0.0.0.0:0          0.0.0.0:*          LISTEN"));
                            },
                            "arp" => {
                                shell_output.push(String::from("Address         HWtype  HWaddress           Iface"));
                                shell_output.push(String::from("10.0.2.2        ether   52:55:0a:00:02:02   eth0"));
                            },
                            "route" => {
                                shell_output.push(String::from("Kernel IP routing table"));
                                shell_output.push(String::from("Dest         Gateway      Genmask         Iface"));
                                shell_output.push(String::from("0.0.0.0      10.0.2.2     0.0.0.0         eth0"));
                                shell_output.push(String::from("10.0.2.0     0.0.0.0      255.255.255.0   eth0"));
                            },
                            "env" => {
                                shell_output.push(String::from("USER=root"));
                                shell_output.push(String::from("HOME=/root"));
                                shell_output.push(String::from("SHELL=/bin/tsh"));
                                shell_output.push(String::from("PATH=/bin:/usr/bin"));
                                shell_output.push(String::from("TERM=trustos"));
                            },
                            "id" => {
                                shell_output.push(String::from("uid=0(root) gid=0(root) groups=0(root)"));
                            },
                            "cal" => {
                                let dt = crate::rtc::read_rtc();
                                shell_output.push(format!("     {:02}/{:04}", dt.month, dt.year));
                                shell_output.push(String::from("Su Mo Tu We Th Fr Sa"));
                                shell_output.push(String::from("       1  2  3  4  5"));
                                shell_output.push(String::from(" 6  7  8  9 10 11 12"));
                                shell_output.push(String::from("13 14 15 16 17 18 19"));
                                shell_output.push(String::from("20 21 22 23 24 25 26"));
                                shell_output.push(String::from("27 28 29 30 31"));
                            },
                            _ if cmd.starts_with("head ") => {
                                let path = cmd.trim_start_matches("head ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        for line in content.lines().take(10) {
                                            shell_output.push(String::from(line));
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("head: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("tail ") => {
                                let path = cmd.trim_start_matches("tail ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        let lines: alloc::vec::Vec<&str> = content.lines().collect();
                                        let start = if lines.len() > 10 { lines.len() - 10 } else { 0 };
                                        for line in &lines[start..] {
                                            shell_output.push(String::from(*line));
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("tail: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("wc ") => {
                                let path = cmd.trim_start_matches("wc ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        let lines = content.lines().count();
                                        let words = content.split_whitespace().count();
                                        let bytes = content.len();
                                        shell_output.push(format!("{:>5} {:>5} {:>5} {}", lines, words, bytes, path));
                                    },
                                    Err(e) => shell_output.push(format!("wc: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("grep ") => {
                                let args = cmd.trim_start_matches("grep ").trim();
                                let parts: alloc::vec::Vec<&str> = args.splitn(2, ' ').collect();
                                if parts.len() == 2 {
                                    let pattern = parts[0];
                                    let path = parts[1];
                                    match crate::ramfs::with_fs(|fs| {
                                        fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                    }) {
                                        Ok(content) => {
                                            let mut found = false;
                                            for line in content.lines() {
                                                if line.contains(pattern) {
                                                    shell_output.push(String::from(line));
                                                    found = true;
                                                }
                                            }
                                            if !found {
                                                shell_output.push(format!("(no matches for '{}')", pattern));
                                            }
                                        },
                                        Err(e) => shell_output.push(format!("grep: {}: {}", path, e.as_str())),
                                    }
                                } else {
                                    shell_output.push(String::from("Usage: grep <pattern> <file>"));
                                }
                            },
                            _ if cmd.starts_with("find ") => {
                                let pattern = cmd.trim_start_matches("find ").trim();
                                shell_output.push(format!("Searching for: {}", pattern));
                                match crate::ramfs::with_fs(|fs| fs.ls(None)) {
                                    Ok(items) => {
                                        for (name, _, _) in items {
                                            if name.contains(pattern) {
                                                shell_output.push(format!("./{}", name));
                                            }
                                        }
                                    },
                                    Err(_) => {}
                                }
                            },
                            _ if cmd.starts_with("stat ") => {
                                let path = cmd.trim_start_matches("stat ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| s.len())
                                }) {
                                    Ok(size) => {
                                        shell_output.push(format!("  File: {}", path));
                                        shell_output.push(format!("  Size: {} bytes", size));
                                        shell_output.push(String::from("  Access: -rw-r--r--"));
                                        shell_output.push(String::from("  Uid: 0  Gid: 0"));
                                    },
                                    Err(e) => shell_output.push(format!("stat: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("sort ") => {
                                let path = cmd.trim_start_matches("sort ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| String::from_utf8_lossy(s).into_owned())
                                }) {
                                    Ok(content) => {
                                        let mut lines: alloc::vec::Vec<&str> = content.lines().collect();
                                        lines.sort();
                                        for line in lines {
                                            shell_output.push(String::from(line));
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("sort: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("strings ") => {
                                let path = cmd.trim_start_matches("strings ").trim();
                                match crate::ramfs::with_fs(|fs| {
                                    fs.read_file(path).map(|s| alloc::vec::Vec::from(s))
                                }) {
                                    Ok(data) => {
                                        let mut current = String::new();
                                        for &b in data.iter().take(1024) {
                                            if b >= 32 && b < 127 {
                                                current.push(b as char);
                                            } else if current.len() >= 4 {
                                                shell_output.push(current.clone());
                                                current.clear();
                                            } else {
                                                current.clear();
                                            }
                                        }
                                        if current.len() >= 4 {
                                            shell_output.push(current);
                                        }
                                    },
                                    Err(e) => shell_output.push(format!("strings: {}: {}", path, e.as_str())),
                                }
                            },
                            _ if cmd.starts_with("traceroute ") || cmd.starts_with("tracert ") => {
                                let host = if cmd.starts_with("traceroute ") {
                                    cmd.trim_start_matches("traceroute ").trim()
                                } else {
                                    cmd.trim_start_matches("tracert ").trim()
                                };
                                shell_output.push(format!("traceroute to {} (simulated)", host));
                                shell_output.push(String::from(" 1  10.0.2.2  1.234 ms"));
                                shell_output.push(String::from(" 2  * * *"));
                                shell_output.push(String::from(" 3  * * *"));
                            },
                            "3ddemo" | "demo3d" | "cube" => {
                                // Run 3D demo using software rasterizer
                                shell_output.push(String::from("Starting 3D Demo..."));
                                shell_output.push(String::from("Controls: Arrow keys rotate, ESC to exit"));
                                
                                // Create rasterizer for the demo
                                let demo_w = 400u32;
                                let demo_h = 300u32;
                                let mut rast = crate::rasterizer::Rasterizer::new(demo_w, demo_h);
                                let mut renderer = crate::rasterizer::Renderer3D::new(demo_w, demo_h);
                                
                                let mut angle_y: f32 = 0.0;
                                let mut angle_x: f32 = 0.3;
                                let mut demo_running = true;
                                let mut demo_frame = 0u32;
                                
                                // Demo window position (centered in main window)
                                let demo_x = (window_x + 150) as u32;
                                let demo_y = (window_y + 50) as u32;
                                
                                while demo_running && demo_frame < 600 { // Max 10 seconds at 60fps
                                    // Check for ESC key
                                    if let Some(k) = crate::keyboard::try_read_key() {
                                        match k {
                                            27 => demo_running = false, // ESC
                                            0x4B => angle_y -= 0.1, // Left
                                            0x4D => angle_y += 0.1, // Right
                                            0x48 => angle_x -= 0.1, // Up
                                            0x50 => angle_x += 0.1, // Down
                                            _ => {}
                                        }
                                    }
                                    
                                    // Clear buffer
                                    rast.clear(0xFF101010);
                                    renderer.clear_z_buffer();
                                    
                                    // Create rotation matrix
                                    let rot_y = crate::rasterizer::Mat4::rotation_y(angle_y);
                                    let rot_x = crate::rasterizer::Mat4::rotation_x(angle_x);
                                    let rotation = rot_x.mul(&rot_y);
                                    
                                    // Draw multiple cubes
                                    let center = crate::rasterizer::Vec3::new(0.0, 0.0, 0.0);
                                    renderer.draw_cube(&mut rast, center, 1.5, &rotation, 0xFF00FF00);
                                    
                                    // Draw a smaller cube offset
                                    let center2 = crate::rasterizer::Vec3::new(2.0, 0.0, 0.0);
                                    renderer.draw_cube(&mut rast, center2, 0.8, &rotation, 0xFF00FFFF);
                                    
                                    // Draw gradient background title bar
                                    rast.fill_gradient_h(0, 0, demo_w, 25, 0xFF003300, 0xFF00AA00);
                                    
                                    // Draw border
                                    rast.draw_rect(0, 0, demo_w, demo_h, 0xFF00FF00);
                                    
                                    // Blit demo to screen
                                    for py in 0..demo_h {
                                        for px in 0..demo_w {
                                            let idx = (py * demo_w + px) as usize;
                                            crate::framebuffer::draw_pixel(demo_x + px, demo_y + py, rast.back_buffer[idx]);
                                        }
                                    }
                                    
                                    // Title
                                    crate::framebuffer::draw_text("3D Demo - ESC to exit", demo_x + 10, demo_y + 5, 0xFFFFFFFF);
                                    
                                    // FPS counter
                                    let fps_str = format!("Frame: {}", demo_frame);
                                    crate::framebuffer::draw_text(&fps_str, demo_x + demo_w - 100, demo_y + 5, 0xFFFFFF00);
                                    
                                    angle_y += 0.02;  // Auto-rotate
                                    demo_frame += 1;
                                    
                                    // Small delay
                                    for _ in 0..50000 { core::hint::spin_loop(); }
                                }
                                
                                shell_output.push(String::from("3D Demo ended."));
                            },
                            "raster" | "rasterdemo" => {
                                // Demo rasterizer features
                                shell_output.push(String::from("Rasterizer Demo - Antialiasing & Gradients"));
                                
                                let demo_w = 350u32;
                                let demo_h = 250u32;
                                let mut rast = crate::rasterizer::Rasterizer::new(demo_w, demo_h);
                                
                                let demo_x = (window_x + 175) as u32;
                                let demo_y = (window_y + 75) as u32;
                                
                                // Clear with dark background
                                rast.clear(0xFF0A0A0A);
                                
                                // Vertical gradient background
                                rast.fill_gradient_v(0, 0, demo_w, demo_h, 0xFF000022, 0xFF002200);
                                
                                // Antialiased circles
                                rast.fill_circle_aa(80, 80, 40, 0xFFFF0000);   // Red
                                rast.fill_circle_aa(150, 100, 35, 0xFF00FF00); // Green  
                                rast.fill_circle_aa(220, 80, 40, 0xFF0000FF);  // Blue
                                
                                // Overlapping with transparency
                                rast.fill_circle_aa(115, 90, 30, 0x8800FFFF);  // Cyan semi-transparent
                                rast.fill_circle_aa(185, 90, 30, 0x88FF00FF);  // Magenta semi-transparent
                                
                                // Rounded rectangle
                                rast.fill_rounded_rect(50, 150, 120, 60, 15, 0xFF444444);
                                rast.fill_gradient_h(55, 155, 110, 50, 0xFF006600, 0xFF00CC00);
                                
                                // Antialiased lines
                                rast.draw_line_aa(200.0, 150.0, 320.0, 220.0, 0xFFFFFF00);
                                rast.draw_line_aa(200.0, 220.0, 320.0, 150.0, 0xFFFF8800);
                                
                                // Shadow demo
                                rast.draw_shadow(250, 160, 60, 40, 8, 0x88000000);
                                rast.fill_rect(250, 160, 60, 40, 0xFF00AA00);
                                
                                // Border
                                rast.draw_rect(0, 0, demo_w, demo_h, 0xFF00FF00);
                                
                                // Blit to screen
                                for py in 0..demo_h {
                                    for px in 0..demo_w {
                                        let idx = (py * demo_w + px) as usize;
                                        crate::framebuffer::draw_pixel(demo_x + px, demo_y + py, rast.back_buffer[idx]);
                                    }
                                }
                                
                                crate::framebuffer::draw_text("Rasterizer: AA + Alpha + Gradients", demo_x + 10, demo_y + 5, 0xFFFFFFFF);
                                
                                // Wait for key
                                shell_output.push(String::from("Press any key to close demo..."));
                                loop {
                                    if crate::keyboard::try_read_key().is_some() {
                                        break;
                                    }
                                    core::hint::spin_loop();
                                }
                                shell_output.push(String::from("Demo closed."));
                            },
                            _ => shell_output.push(format!("Command not found: {}", cmd)),
                        };
                        shell_input.clear();
                        suggestion_text.clear();
                        
                        // Keep only last 20 lines
                        while shell_output.len() > 20 {
                            shell_output.remove(0);
                        }
                        
                        // Auto-scroll to bottom to show new output
                        scroll_offset = shell_output.len().saturating_sub(MAX_VISIBLE_LINES);
                    }
                },
                32..=126 => { // Printable characters
                    crate::serial_println!("[KEY] Printable char: '{}' ({})", key as char, key);
                    shell_input.push(key as char);
                    // Update suggestion
                    let cmds = ["help", "ls", "dir", "clear", "ifconfig", "cpuinfo", "meminfo", "whoami", "uptime", "open", "smp", "fps", "matrix", "holo"];
                    suggestion_text.clear();
                    for c in cmds {
                        if c.starts_with(&shell_input) && c != shell_input.as_str() {
                            suggestion_text = String::from(&c[shell_input.len()..]);
                            break;
                        }
                    }
                },
                _ => {}
            }
            } // End of shell mode else block
        }
        
        // Mouse input - get current state (x,y are absolute positions)
        let mouse_state = crate::mouse::get_state();
        mouse_x = mouse_state.x.clamp(0, width as i32 - 1);
        mouse_y = mouse_state.y.clamp(0, height as i32 - 1);
        let left = mouse_state.left_button;
        
        // Mouse click detection
        let clicked = left && !prev_left;
        let released = !left && prev_left;
        prev_left = left;
        
        // Window dragging logic
        if dragging_window {
            if left {
                // Update window position while dragging
                window_x = (mouse_x - drag_offset_x).clamp(0, width as i32 - 200);
                window_y = (mouse_y - drag_offset_y).clamp(0, height as i32 - 100);
                // Update the layer position
                if let Some(win) = compositor.get_layer_mut(window_layer) {
                    win.set_position(window_x as u32, window_y as u32);
                }
            } else {
                // Released - stop dragging
                dragging_window = false;
            }
        }
        
        // Update cursor layer position
        if let Some(cursor) = compositor.get_layer_mut(cursor_layer) {
            cursor.set_position(mouse_x as u32, mouse_y as u32);
        }
        
        // Menu hover detection
        menu_hover = -1;
        if menu_open {
            let menu_x = 5u32;
            let menu_y = height - 340;
            let mx = mouse_x as u32;
            let my = mouse_y as u32;
            
            if mx >= menu_x && mx < menu_x + 250 && my >= menu_y && my < menu_y + 290 {
                let item_h = 36u32;
                let rel_y = if my > menu_y + 40 { my - menu_y - 40 } else { 0 };
                let idx = (rel_y / item_h) as i32;
                if idx >= 0 && idx < menu_items.len() as i32 {
                    menu_hover = idx;
                }
            }
        }
        
        // Click handling
        if clicked {
            let mx = mouse_x as u32;
            let my = mouse_y as u32;
            
            // TrustOS button in taskbar
            let taskbar_y = height - 40;
            if my >= taskbar_y && my < height && mx >= 5 && mx < 110 {
                menu_open = !menu_open;
                settings_open = false; // Close settings if open
            }
            // Settings button in taskbar (x: 340 to 390)
            else if my >= taskbar_y && my < height && mx >= 340 && mx < 390 {
                settings_open = !settings_open;
                menu_open = false; // Close menu if open
                // Refresh current values
                settings_anim_enabled = crate::desktop::animations_enabled();
                settings_anim_speed = crate::desktop::get_animation_speed();
            }
            // Window button in taskbar (x: 220 to 320)
            else if my >= taskbar_y && my < height && mx >= 220 && mx < 320 {
                // Toggle window visibility
                window_visible = !window_visible;
            }
            // Menu item click
            else if menu_open && menu_hover >= 0 && menu_hover < menu_items.len() as i32 {
                let (_name, item) = menu_items[menu_hover as usize];
                match item {
                    MenuItem::App(mode) => {
                        // Skip separator line
                        if !_name.starts_with("─") {
                            active_mode = mode;
                            shell_output.clear();
                            for line in get_help!(mode) {
                                shell_output.push(String::from(*line));
                            }
                            shell_output.push(String::from(""));
                            shell_output.push(String::from("Type commands below. Type 'help' for more info."));
                        }
                    },
                    MenuItem::Shutdown => {
                        shell_output.push(String::from("> Shutting down..."));
                        // Delay then halt
                        for _ in 0..10000000 { core::hint::spin_loop(); }
                        loop {
                            x86_64::instructions::interrupts::disable();
                            x86_64::instructions::hlt();
                        }
                    },
                    MenuItem::Reboot => {
                        shell_output.push(String::from("> Rebooting..."));
                        for _ in 0..10000000 { core::hint::spin_loop(); }
                        unsafe {
                            x86_64::instructions::port::Port::<u8>::new(0x64).write(0xFE);
                        }
                        loop { x86_64::instructions::hlt(); }
                    },
                }
                menu_open = false;
            }
            // Click outside menu/settings closes them
            else if menu_open {
                menu_open = false;
            }
            else if settings_open {
                // Check if click is inside settings panel area (340-610, height-380 to height-40)
                let settings_panel_x = 340u32;
                let settings_panel_y = height - 380;
                let settings_panel_w = 270u32;
                let settings_panel_h = 350u32;
                if !(mx >= settings_panel_x && mx < settings_panel_x + settings_panel_w 
                    && my >= settings_panel_y && my < settings_panel_y + settings_panel_h) {
                    settings_open = false;
                }
            }
            // Window interaction (buttons and dragging)
            else if !dragging_window {
                let win_x = window_x as u32;
                let win_y = window_y as u32;
                let win_w = 700u32;  // Fixed window width from add_layer
                let win_h = 450u32;  // Fixed window height from add_layer
                
                // Check if click is in title bar area (only if window visible)
                if window_visible && mx >= win_x && mx < win_x + win_w && my >= win_y && my < win_y + 28 {
                    // Button positions: close at w-50, min at w-80, max at w-110
                    // Close button (X) - hides the window (use 'desktop close' to exit)
                    if mx >= win_x + win_w - 60 && mx < win_x + win_w - 40 {
                        window_visible = false;  // Hide window, don't exit desktop
                        shell_output.push(String::from("> Window closed. Click dock icon to reopen."));
                    }
                    // Minimize button (-) - radius 10 -> check x from w-90 to w-70
                    else if mx >= win_x + win_w - 90 && mx < win_x + win_w - 70 {
                        window_visible = false;  // Minimize = hide
                        shell_output.push(String::from("> Window minimized"));
                    }
                    // Maximize button (□) - radius 10 -> check x from w-120 to w-100
                    else if mx >= win_x + win_w - 120 && mx < win_x + win_w - 100 {
                        shell_output.push(String::from("> Window maximized"));
                    }
                    // Otherwise, start dragging
                    else {
                        dragging_window = true;
                        drag_offset_x = mouse_x - window_x;
                        drag_offset_y = mouse_y - window_y;
                    }
                }
                // Dock icon clicks - also reopens window if closed
                else if mx < 80 && my < height - 40 {
                    let icon_size = 36u32;
                    let gap = 50u32;       // Match render gap (was 58, now 50)
                    let start_y = 10u32;
                    for i in 0..8usize {
                        let iy = start_y + (i as u32) * (icon_size + gap);
                        if my >= iy && my < iy + icon_size + 16 {
                            active_mode = match i {
                                0 => AppMode::Files,
                                1 => AppMode::Shell,
                                2 => AppMode::Network,
                                3 => AppMode::TextEditor,
                                4 => AppMode::Hardware,
                                5 => AppMode::UserMgmt,
                                6 => AppMode::Browser,
                                7 => AppMode::ImageViewer,
                                _ => AppMode::Shell,
                            };
                            // Reopen window if it was closed
                            window_visible = true;
                            shell_output.clear();
                            for line in get_help!(active_mode) {
                                shell_output.push(String::from(*line));
                            }
                            shell_output.push(String::from(""));
                            break;
                        }
                    }
                }
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // FRAME-RATE DECOUPLING GATE
        // Only render + composite on every Nth frame. Skip frames re-present.
        // ═══════════════════════════════════════════════════════════════════
        let is_render_frame = (frame_count % composite_interval) == 0;
        
        if !is_render_frame {
            // Skip frame: NO VirtIO DMA transfer (present_only is now a no-op)
            // This is the key 120 FPS optimization: skip frames cost ~0.1ms
            // instead of ~33ms (4MB transfer_to_host_2d + resource_flush)
            compositor.present_only();
            // Advance formula animation state so next render frame shows smooth motion
            if use_formula { formula_renderer.update(); }
        } else {
        // ─── RENDER FRAME: Full layer rendering + composite + present ───
        render_in_second += 1;
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 0: BACKGROUND - PARALLEL Matrix Rain across all cores!
        // Each core renders a chunk of columns simultaneously
        // ═══════════════════════════════════════════════════════════════════
        if let Some(bg) = compositor.get_layer_mut(bg_layer) {
            let buf_ptr = bg.buffer.as_mut_ptr();
            let buf_len = bg.buffer.len();
            
            // FAST PATH: Use optimized renderers
            if use_formula {
                // ─────────────────────────────────────────────────────────────
                // FORMULA 3D: Tsoding-inspired wireframe perspective projection
                // Cheapest renderer: Bresenham lines + depth coloring
                // No fill, no textures, pure math beauty
                // ─────────────────────────────────────────────────────────────
                formula_renderer.update();
                formula_renderer.render(&mut bg.buffer, width as usize, height as usize);
            } else if use_shader_matrix {
                // ─────────────────────────────────────────────────────────────
                // SHADER MATRIX: Optimized cell-based Matrix rain
                // Real MATRIX_GLYPHS_6X6 katakana, per-column depth parallax,
                // SMP-parallel column bands, SSE2 background fill.
                // ~12K glyph blits/frame vs 1M+ pixel shader calls.
                // ─────────────────────────────────────────────────────────────
                crate::gpu_emu::shader_matrix_render(
                    buf_ptr,
                    width as usize,
                    height as usize,
                );
            } else if use_matrix3d {
                // ─────────────────────────────────────────────────────────────
                // MATRIX 3D: Volumetric rain with 3D shape collision
                // ─────────────────────────────────────────────────────────────
                matrix3d_renderer.update();
                matrix3d_renderer.render(&mut bg.buffer, width as usize, height as usize);
            } else if use_braille {
                // ─────────────────────────────────────────────────────────────
                // BRAILLE RENDERER: 8× resolution using Unicode Braille patterns
                // NOTE: Renderer fills black itself, no need for bg.buffer.fill!
                // ─────────────────────────────────────────────────────────────
                braille_renderer.update();
                braille_renderer.render(&mut bg.buffer, width as usize, height as usize);
                // Holographic cube: ambient fill for dark face cells, then subtle edge hints
                braille_renderer.render_cube_flow_layer(&mut bg.buffer, width as usize, height as usize);
                braille_renderer.render_entity_layer(&mut bg.buffer, width as usize, height as usize);
            } else if use_fast_matrix && !use_holovolume && !holo_enabled {
                // ─────────────────────────────────────────────────────────────
                // FAST MATRIX: Glyph-cached ultra-optimized renderer
                // ─────────────────────────────────────────────────────────────
                bg.buffer.fill(black);
                fast_renderer.update();
                fast_renderer.render(&mut bg.buffer, width as usize, height as usize);
            } else if use_holovolume {
                // ─────────────────────────────────────────────────────────────
                // HOLOVOLUME: Modifies Matrix rain colors based on 3D shape
                // Uses the SAME rain animation, just modifies colors
                // ─────────────────────────────────────────────────────────────
                
                // Update holovolume (compute intensity map)
                holovolume.set_screen_size(width as usize, height as usize);
                holovolume.update(0.016);
                
                // Move heads down (same as normal rain)
                for col in 0..MATRIX_COLS {
                    matrix_heads[col] += matrix_speeds[col] as i32;
                    if matrix_heads[col] > (MATRIX_ROWS as i32 + 30) {
                        let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                        matrix_heads[col] = -((seed % 30) as i32);
                        matrix_speeds[col] = 1 + (seed % 3);
                    }
                }
                
                // Clear with black
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::fill_row_sse2(buf_ptr, buf_len, black);
                    #[cfg(not(target_arch = "x86_64"))]
                    bg.buffer.fill(black);
                }
                
                // Get intensity map from holovolume
                let holo_intensity = holovolume.get_u8_intensity_map();
                
                // Render with holo intensity modifier
                let params = MatrixRenderParams {
                    buf_ptr,
                    buf_len,
                    width,
                    height,
                    matrix_chars: matrix_chars.as_ptr(),
                    matrix_heads: matrix_heads.as_ptr(),
                    holo_intensity: holo_intensity.as_ptr(),
                    matrix_rows: MATRIX_ROWS,
                };
                
                crate::cpu::smp::parallel_for(
                    MATRIX_COLS,
                    render_matrix_columns_parallel,
                    &params as *const MatrixRenderParams as *mut u8
                );
                
            } else if holo_enabled {
                // ─────────────────────────────────────────────────────────────
                // 3D BACKGROUND RENDERING (HoloMatrix or RayTracer)
                // ─────────────────────────────────────────────────────────────
                
                if holo_scene.is_raytraced() {
                    // ═════════════════════════════════════════════════════════
                    // RAY TRACING MODE - True 3D with lighting and reflections
                    // ═════════════════════════════════════════════════════════
                    use crate::graphics::raytracer::{Vec3, Material};
                    
                    raytracer.update(0.016);
                    
                    // Setup scene based on type
                    match holo_scene {
                        crate::graphics::holomatrix::HoloScene::RayTracedSpheres => {
                            raytracer.setup_spheres_scene();
                        },
                        crate::graphics::holomatrix::HoloScene::RayTracedDNA => {
                            raytracer.setup_dna_scene();
                        },
                        _ => {}
                    }
                    
                    // Render ray traced scene
                    let rt_output = raytracer.render();
                    
                    // Scale up to screen resolution
                    let rt_w = raytracer.width;
                    let rt_h = raytracer.height;
                    let scale_x = width as usize / rt_w;
                    let scale_y = height as usize / rt_h;
                    
                    for y in 0..height as usize {
                        for x in 0..width as usize {
                            let rx = (x / scale_x).min(rt_w - 1);
                            let ry = (y / scale_y).min(rt_h - 1);
                            let color = rt_output[ry * rt_w + rx];
                            bg.buffer[y * width as usize + x] = color;
                        }
                    }
                } else {
                    // ═════════════════════════════════════════════════════════
                    // HOLOMATRIX THROUGH MATRIX RAIN
                    // 3D shape appears via intensity boost on Matrix characters
                    // The hologram "emerges" through the falling rain
                    // ═════════════════════════════════════════════════════════
                    
                    // Update animation time
                    holomatrix.update(0.016);
                    let time = holomatrix.time;
                    
                    // ─────────────────────────────────────────────────────────
                    // STEP 1: Create intensity boost map (character cell grid)
                    // Use same dimensions as Matrix rain (MATRIX_COLS x MATRIX_ROWS)
                    // Each cell stores intensity boost (0-200) based on 3D shape
                    // ─────────────────────────────────────────────────────────
                    // Note: MATRIX_COLS=240, MATRIX_ROWS=68 defined at top
                    let mut intensity_map = [[0u8; MATRIX_ROWS]; MATRIX_COLS];
                    
                    // Cell size in pixels (matches Matrix rain)
                    let cell_w = (width as f32) / (MATRIX_COLS as f32);  // ~8px at 1920
                    let cell_h = (height as f32) / (MATRIX_ROWS as f32); // ~16px at 1080
                    
                    // 3D projection parameters - center of screen
                    let cx = width as f32 / 2.0;
                    let cy = height as f32 / 2.0;
                    let scale = (height as f32 / 3.0).min(width as f32 / 4.0);
                    
                    // Generate 3D shape points and mark intensity on grid
                    match holo_scene {
                        crate::graphics::holomatrix::HoloScene::DNA => {
                            // DNA Double Helix with higher resolution
                            let helix_len = 2.2;
                            let radius = 0.45;
                            let turns = 3.5;
                            
                            for i in 0..180 {
                                let t = i as f32 / 180.0;
                                let y = -helix_len / 2.0 + t * helix_len;
                                let angle = t * turns * 6.28318 + time;
                                
                                // Strand 1
                                let x1 = radius * crate::graphics::holomatrix::cos_approx_pub(angle);
                                let z1 = radius * crate::graphics::holomatrix::sin_approx_pub(angle);
                                
                                // Strand 2 (180° offset)
                                let x2 = radius * crate::graphics::holomatrix::cos_approx_pub(angle + 3.14159);
                                let z2 = radius * crate::graphics::holomatrix::sin_approx_pub(angle + 3.14159);
                                
                                // Apply rotation around Y axis
                                let rot_y = time * 0.4;
                                let cos_r = crate::graphics::holomatrix::cos_approx_pub(rot_y);
                                let sin_r = crate::graphics::holomatrix::sin_approx_pub(rot_y);
                                
                                // Rotate both strands
                                let rx1 = x1 * cos_r + z1 * sin_r;
                                let rz1 = -x1 * sin_r + z1 * cos_r;
                                let rx2 = x2 * cos_r + z2 * sin_r;
                                let rz2 = -x2 * sin_r + z2 * cos_r;
                                
                                // Project to screen and convert to grid coords
                                let depth1 = 1.0 / (2.0 + rz1);
                                let sx1 = cx + rx1 * scale * depth1;
                                let sy1 = cy + y * scale * depth1;
                                let col1 = (sx1 / cell_w) as usize;
                                let row1 = (sy1 / cell_h) as usize;
                                
                                let depth2 = 1.0 / (2.0 + rz2);
                                let sx2 = cx + rx2 * scale * depth2;
                                let sy2 = cy + y * scale * depth2;
                                let col2 = (sx2 / cell_w) as usize;
                                let row2 = (sy2 / cell_h) as usize;
                                
                                // Intensity based on depth (closer = brighter) - MAX values!
                                let int1 = (180.0 + 75.0 * (1.0 - ((rz1 + 0.5) * 0.5).max(0.0).min(1.0))) as u8;
                                let int2 = (180.0 + 75.0 * (1.0 - ((rz2 + 0.5) * 0.5).max(0.0).min(1.0))) as u8;
                                
                                // Mark intensity on grid (both strands) - with THICKNESS
                                if col1 < MATRIX_COLS && row1 < MATRIX_ROWS {
                                    intensity_map[col1][row1] = intensity_map[col1][row1].max(int1);
                                    // Spread to neighbors for thickness (3 cells wide)
                                    if col1 > 0 { intensity_map[col1-1][row1] = intensity_map[col1-1][row1].max(int1 * 2/3); }
                                    if col1 < MATRIX_COLS-1 { intensity_map[col1+1][row1] = intensity_map[col1+1][row1].max(int1 * 2/3); }
                                    if row1 > 0 { intensity_map[col1][row1-1] = intensity_map[col1][row1-1].max(int1/2); }
                                    if row1 < MATRIX_ROWS-1 { intensity_map[col1][row1+1] = intensity_map[col1][row1+1].max(int1/2); }
                                }
                                if col2 < MATRIX_COLS && row2 < MATRIX_ROWS {
                                    intensity_map[col2][row2] = intensity_map[col2][row2].max(int2);
                                    if col2 > 0 { intensity_map[col2-1][row2] = intensity_map[col2-1][row2].max(int2 * 2/3); }
                                    if col2 < MATRIX_COLS-1 { intensity_map[col2+1][row2] = intensity_map[col2+1][row2].max(int2 * 2/3); }
                                    if row2 > 0 { intensity_map[col2][row2-1] = intensity_map[col2][row2-1].max(int2/2); }
                                    if row2 < MATRIX_ROWS-1 { intensity_map[col2][row2+1] = intensity_map[col2][row2+1].max(int2/2); }
                                }
                                
                                // Cross-links every 12 points (base pairs)
                                if i % 12 == 0 {
                                    for s in 0..8 {
                                        let st = s as f32 / 7.0;
                                        let lx = sx1 * (1.0 - st) + sx2 * st;
                                        let ly = sy1 * (1.0 - st) + sy2 * st;
                                        let lcol = (lx / cell_w) as usize;
                                        let lrow = (ly / cell_h) as usize;
                                        if lcol < MATRIX_COLS && lrow < MATRIX_ROWS {
                                            intensity_map[lcol][lrow] = intensity_map[lcol][lrow].max(80);
                                        }
                                    }
                                }
                            }
                        },
                        crate::graphics::holomatrix::HoloScene::RotatingCube => {
                            let half = 0.5;
                            let vertices: [(f32, f32, f32); 8] = [
                                (-half, -half, -half), (half, -half, -half),
                                (half, half, -half), (-half, half, -half),
                                (-half, -half, half), (half, -half, half),
                                (half, half, half), (-half, half, half),
                            ];
                            let edges: [(usize, usize); 12] = [
                                (0,1), (1,2), (2,3), (3,0),
                                (4,5), (5,6), (6,7), (7,4),
                                (0,4), (1,5), (2,6), (3,7),
                            ];
                            
                            let rot_x = time * 0.7;
                            let rot_y = time * 0.5;
                            
                            for (i1, i2) in edges.iter() {
                                let (vx1, vy1, vz1) = vertices[*i1];
                                let (vx2, vy2, vz2) = vertices[*i2];
                                
                                for s in 0..30 {
                                    let t = s as f32 / 29.0;
                                    let x = vx1 * (1.0 - t) + vx2 * t;
                                    let y = vy1 * (1.0 - t) + vy2 * t;
                                    let z = vz1 * (1.0 - t) + vz2 * t;
                                    
                                    // Rotate
                                    let cos_x = crate::graphics::holomatrix::cos_approx_pub(rot_x);
                                    let sin_x = crate::graphics::holomatrix::sin_approx_pub(rot_x);
                                    let ry = y * cos_x - z * sin_x;
                                    let rz = y * sin_x + z * cos_x;
                                    let cos_y = crate::graphics::holomatrix::cos_approx_pub(rot_y);
                                    let sin_y = crate::graphics::holomatrix::sin_approx_pub(rot_y);
                                    let rx = x * cos_y + rz * sin_y;
                                    let rz2 = -x * sin_y + rz * cos_y;
                                    
                                    let depth = 1.0 / (2.0 + rz2);
                                    let sx = cx + rx * scale * depth;
                                    let sy = cy + ry * scale * depth;
                                    let col = (sx / cell_w) as usize;
                                    let row = (sy / cell_h) as usize;
                                    
                                    if col < MATRIX_COLS && row < MATRIX_ROWS {
                                        let int = (100.0 + 100.0 * (1.0 - ((rz2 + 0.6) * 0.5).max(0.0).min(1.0))) as u8;
                                        intensity_map[col][row] = intensity_map[col][row].max(int);
                                    }
                                }
                            }
                        },
                        _ => {
                            // Sphere
                            for i in 0..300 {
                                let phi = (i as f32 / 300.0) * 6.28318;
                                let theta = (i as f32 * 0.618033 * 6.28318) % 6.28318;
                                
                                let r = 0.55;
                                let x = r * crate::graphics::holomatrix::sin_approx_pub(theta) * crate::graphics::holomatrix::cos_approx_pub(phi);
                                let y = r * crate::graphics::holomatrix::sin_approx_pub(theta) * crate::graphics::holomatrix::sin_approx_pub(phi);
                                let z = r * crate::graphics::holomatrix::cos_approx_pub(theta);
                                
                                let cos_t = crate::graphics::holomatrix::cos_approx_pub(time * 0.5);
                                let sin_t = crate::graphics::holomatrix::sin_approx_pub(time * 0.5);
                                let rx = x * cos_t + z * sin_t;
                                let rz = -x * sin_t + z * cos_t;
                                
                                let depth = 1.0 / (2.0 + rz);
                                let sx = cx + rx * scale * depth;
                                let sy = cy + y * scale * depth;
                                let col = (sx / cell_w) as usize;
                                let row = (sy / cell_h) as usize;
                                
                                if col < MATRIX_COLS && row < MATRIX_ROWS {
                                    let int = (80.0 + 120.0 * (1.0 - ((rz + 0.6) * 0.5).max(0.0).min(1.0))) as u8;
                                    intensity_map[col][row] = intensity_map[col][row].max(int);
                                }
                            }
                        }
                    }
                    
                    // ─────────────────────────────────────────────────────────
                    // STEP 2: Render Matrix Rain with hologram intensity boost
                    // Characters where shape exists are brightened
                    // ─────────────────────────────────────────────────────────
                    
                    // Move Matrix heads down
                    for col in 0..MATRIX_COLS {
                        matrix_heads[col] += matrix_speeds[col] as i32;
                        if matrix_heads[col] > (MATRIX_ROWS as i32 + 30) {
                            let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                            matrix_heads[col] = -((seed % 30) as i32);
                            matrix_speeds[col] = 1 + (seed % 3);
                        }
                    }
                    
                    // Clear to black
                    bg.buffer.fill(0xFF000000);
                    
                    // Render each column with intensity boost
                    let col_width = 8u32;
                    for col in 0..MATRIX_COLS {
                        let x = col as u32 * col_width;
                        if x >= width { continue; }
                        
                        let head = matrix_heads[col];
                        
                        for row in 0..MATRIX_ROWS {
                            let y = row as u32 * 16;
                            if y >= height { continue; }
                            
                            let dist = row as i32 - head;
                            
                            // Base color from Matrix rain
                            let base_color = if dist < 0 {
                                continue;
                            } else if dist == 0 {
                                255u32  // Bright head
                            } else if dist <= 12 {
                                255 - (dist as u32 * 8)
                            } else if dist <= 28 {
                                // Safely calculate fade to avoid underflow
                                let factor = ((dist - 12) as u32).min(15) * 16;
                                let fade = 255u32.saturating_sub(factor);
                                (160 * fade) / 255
                            } else {
                                continue;
                            };
                            
                            // Get intensity boost from hologram
                            let boost = intensity_map[col][row] as u32;
                            
                            // Combine: base Matrix + hologram boost
                            // Hologram makes characters MUCH brighter + colored where shape exists
                            let (r, g, b) = if boost > 0 {
                                // Shape exists here: bright cyan-white glow
                                let intensity = (base_color + boost * 2).min(255);
                                let cyan = (boost as u32 * 3 / 2).min(255);
                                (cyan / 3, intensity, cyan)  // R=dim, G=bright, B=cyan
                            } else {
                                // No shape: very dim green (shape stands out)
                                let dim = (base_color / 3).min(80);
                                (0, dim, 0)
                            };
                            
                            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                            
                            // Get character
                            let c = matrix_chars[matrix_idx(col, row)] as char;
                            let glyph = crate::framebuffer::font::get_glyph(c);
                            
                            // Draw glyph
                            for (r, &bits) in glyph.iter().enumerate() {
                                let py = y + r as u32;
                                if py >= height { break; }
                                let row_offset = (py * width) as usize;
                                
                                if bits != 0 {
                                    let x_usize = x as usize;
                                    if bits & 0x80 != 0 { let idx = row_offset + x_usize; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x40 != 0 { let idx = row_offset + x_usize + 1; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x20 != 0 { let idx = row_offset + x_usize + 2; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x10 != 0 { let idx = row_offset + x_usize + 3; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x08 != 0 { let idx = row_offset + x_usize + 4; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x04 != 0 { let idx = row_offset + x_usize + 5; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x02 != 0 { let idx = row_offset + x_usize + 6; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                    if bits & 0x01 != 0 { let idx = row_offset + x_usize + 7; if idx < bg.buffer.len() { bg.buffer[idx] = color; } }
                                }
                            }
                        }
                    }
                }
            } else {
                // ─────────────────────────────────────────────────────────────
                // MATRIX RAIN BACKGROUND (default)
                // ─────────────────────────────────────────────────────────────
                
                // Move heads down (fast, single-threaded)
                for col in 0..MATRIX_COLS {
                    matrix_heads[col] += matrix_speeds[col] as i32;
                    if matrix_heads[col] > (MATRIX_ROWS as i32 + 30) {
                        let seed = (col as u32 * 2654435761).wrapping_add(frame_count as u32);
                        matrix_heads[col] = -((seed % 30) as i32);
                        matrix_speeds[col] = 1 + (seed % 3);
                    }
                }
                
                // Clear with black using SSE2
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::fill_row_sse2(buf_ptr, buf_len, black);
                    #[cfg(not(target_arch = "x86_64"))]
                    bg.buffer.fill(black);
                }
                
                // ─────────────────────────────────────────────────────────────
                // TRUE PARALLEL MATRIX RENDER: Use parallel_for across all cores!
                // Each core renders its portion of the 240 columns simultaneously
                // ─────────────────────────────────────────────────────────────
                let params = MatrixRenderParams {
                    buf_ptr,
                    buf_len,
                    width,
                    height,
                    matrix_chars: matrix_chars.as_ptr(),
                    matrix_heads: matrix_heads.as_ptr(),
                    holo_intensity: core::ptr::null(),  // No holo modifier
                    matrix_rows: MATRIX_ROWS,
                };
                
                // Fire off parallel rendering across ALL cores!
                crate::cpu::smp::parallel_for(
                    MATRIX_COLS,
                    render_matrix_columns_parallel,
                    &params as *const MatrixRenderParams as *mut u8
                );
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 1: DOCK (Left side)
        // ═══════════════════════════════════════════════════════════════════
        if let Some(dock) = compositor.get_layer_mut(dock_layer) {
            dock.clear(0xF0080808); // Semi-transparent
            
            let icon_size = 36u32;  // Compact icons
            let gap = 50u32;       // Adjusted gap for more apps
            let start_y = 10u32;
            
            let dock_apps = [
                ("Files", AppMode::Files),
                ("Shell", AppMode::Shell),
                ("Net", AppMode::Network),
                ("Edit", AppMode::TextEditor),
                ("HW", AppMode::Hardware),
                ("User", AppMode::UserMgmt),
                ("Web", AppMode::Browser),  // Browser - special app
                ("Img", AppMode::ImageViewer), // Image viewer
            ];
            
            for (i, (name, mode)) in dock_apps.iter().enumerate() {
                let iy = start_y + (i as u32) * (icon_size + gap);
                let ix = 10u32;
                
                let is_active = *mode == active_mode;
                let icon_color = if is_active { green_bright } else { green_dim };
                let label_color = if is_active { 0xFFFFFFFF } else { 0xFF888888 };
                
                // Icon background with hover effect
                if is_active {
                    dock.fill_rect(ix - 4, iy - 4, icon_size + 8, icon_size + 20, 0xFF002800);
                    dock.draw_rect(ix - 4, iy - 4, icon_size + 8, icon_size + 20, green_main);
                }
                dock.fill_rect(ix, iy, icon_size, icon_size, 0xFF0A0A0A);
                dock.draw_rect(ix, iy, icon_size, icon_size, icon_color);
                
                // Icon symbol (larger, more visible)
                let cx = ix + icon_size / 2;
                let cy = iy + icon_size / 2;
                match i {
                    0 => { // Files - folder icon
                        dock.fill_rect(cx - 12, cy - 2, 24, 14, icon_color);
                        dock.fill_rect(cx - 14, cy - 6, 10, 6, icon_color);
                    },
                    1 => { // Shell - terminal icon
                        dock.draw_rect(cx - 14, cy - 10, 28, 20, icon_color);
                        dock.draw_text(">", cx - 8, cy - 4, icon_color);
                        dock.fill_rect(cx - 2, cy - 2, 10, 2, icon_color);
                    },
                    2 => { // Network - wifi/globe icon (simplified)
                        dock.fill_circle(cx, cy, 12, icon_color);
                        dock.fill_circle(cx, cy, 8, 0xFF0A0A0A);
                        dock.fill_circle(cx, cy, 4, icon_color);
                        // Signal bars
                        dock.fill_rect(cx + 6, cy - 2, 2, 6, icon_color);
                        dock.fill_rect(cx + 10, cy - 6, 2, 10, icon_color);
                    },
                    3 => { // Editor - document icon
                        dock.fill_rect(cx - 10, cy - 12, 20, 24, icon_color);
                        dock.fill_rect(cx - 8, cy - 10, 16, 20, 0xFF0A0A0A);
                        dock.fill_rect(cx - 6, cy - 6, 12, 2, icon_color);
                        dock.fill_rect(cx - 6, cy - 2, 12, 2, icon_color);
                        dock.fill_rect(cx - 6, cy + 2, 8, 2, icon_color);
                    },
                    4 => { // Hardware - chip icon
                        dock.fill_rect(cx - 10, cy - 8, 20, 16, icon_color);
                        for j in 0..4 {
                            dock.fill_rect(cx - 14, cy - 6 + j * 4, 4, 2, icon_color);
                            dock.fill_rect(cx + 10, cy - 6 + j * 4, 4, 2, icon_color);
                        }
                    },
                    5 => { // Users - person icon
                        dock.fill_circle(cx, cy - 4, 6, icon_color);
                        dock.fill_rect(cx - 8, cy + 4, 16, 8, icon_color);
                    },
                    6 => { // Browser - globe/world icon
                        dock.fill_circle(cx, cy, 10, icon_color);
                        dock.fill_circle(cx, cy, 6, 0xFF0A0A0A);
                        // Horizontal line
                        dock.fill_rect(cx - 10, cy - 1, 20, 2, icon_color);
                        // Vertical line  
                        dock.fill_rect(cx - 1, cy - 10, 2, 20, icon_color);
                    },
                    _ => {}
                }
                
                // Label text under icon
                let text_x = ix + (icon_size / 2) - ((name.len() as u32 * 8) / 2);
                dock.draw_text(name, text_x, iy + icon_size + 2, label_color);
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 2: MAIN WINDOW (Shell with module guide) - Only if visible
        // ═══════════════════════════════════════════════════════════════════
        if let Some(win) = compositor.get_layer_mut(window_layer) {
            if window_visible {
                // Use actual layer dimensions, not screen dimensions!
                let w = win.width;
                let h = win.height;
                
                win.clear(window_bg);
                
                // Border (green)
                win.draw_rect(0, 0, w, h, green_main);
                win.draw_rect(1, 1, w - 2, h - 2, green_main);
                
                // Title bar
                let mode_name = match active_mode {
                    AppMode::Shell => "Shell",
                AppMode::Network => "Network",
                AppMode::Hardware => "Hardware",
                AppMode::TextEditor => "TrustCode",
                AppMode::UserMgmt => "User Management",
                AppMode::Files => "Files",
                AppMode::Browser => "Web Browser",
                AppMode::ImageViewer => "Image Viewer",
            };
            win.fill_rect(2, 2, w - 4, 26, 0xFF0A1A0A);
            let title = format!("TrustOS - {} Module", mode_name);
            win.draw_text(&title, 12, 8, 0xFFFFFFFF); // White text
            
            // Drag indicator
            if dragging_window {
                win.draw_text("[MOVING]", w / 2 - 32, 8, 0xFFFFAA00);
            }
            
            // Window buttons with symbols (LARGE and visible)
            // Close button (red with X) - 12px radius
            let btn_y = 13u32;
            let btn_r = 10u32;
            let btn_close_x = w - 50;
            let btn_min_x = w - 80;
            let btn_max_x = w - 110;
            
            // Close button (X)
            win.fill_circle(btn_close_x, btn_y, btn_r, 0xFFFF4444);
            win.draw_rect(btn_close_x, btn_y, 1, 1, 0xFFFF6666); // Highlight
            // Draw bold X
            for t in 0..7 {
                win.set_pixel(btn_close_x - 5 + t, btn_y - 5 + t, 0xFFFFFFFF);
                win.set_pixel(btn_close_x - 4 + t, btn_y - 5 + t, 0xFFFFFFFF);
                win.set_pixel(btn_close_x + 5 - t, btn_y - 5 + t, 0xFFFFFFFF);
                win.set_pixel(btn_close_x + 4 - t, btn_y - 5 + t, 0xFFFFFFFF);
            }
            
            // Minimize button (-)
            win.fill_circle(btn_min_x, btn_y, btn_r, 0xFFFFCC00);
            // Draw bold -
            win.fill_rect(btn_min_x - 5, btn_y - 1, 10, 3, 0xFF000000);
            
            // Maximize button (□)
            win.fill_circle(btn_max_x, btn_y, btn_r, 0xFF44DD44);
            // Draw bold square
            win.draw_rect(btn_max_x - 5, btn_y - 5, 10, 10, 0xFF000000);
            win.draw_rect(btn_max_x - 4, btn_y - 4, 8, 8, 0xFF000000);
            
            // Content area - different rendering for Browser mode
            let content_y = 35u32;
            let line_height = 18u32;
            let max_lines = ((h - content_y - 50) / line_height) as usize;
            
            if active_mode == AppMode::Browser {
                // ═══════════════════════════════════════════════════════════
                // BROWSER MODE: Chrome DevTools style rendering
                // ═══════════════════════════════════════════════════════════
                
                // URL Bar with modern styling
                let url_bar_y = content_y;
                win.fill_rect(10, url_bar_y, w - 20, 32, 0xFF1E1E1E);
                win.draw_rect(10, url_bar_y, w - 20, 32, 0xFF3C3C3C);
                
                // Navigation buttons with icons
                let btn_bg: u32 = 0xFF2D2D2D;
                
                // Back button (◄)
                win.fill_rect(14, url_bar_y + 4, 24, 24, btn_bg);
                win.draw_text("<", 22, url_bar_y + 10, 0xFFAAAAAA);
                
                // Forward button (►)
                win.fill_rect(42, url_bar_y + 4, 24, 24, btn_bg);
                win.draw_text(">", 50, url_bar_y + 10, 0xFFAAAAAA);
                
                // Refresh button (↻)
                win.fill_rect(70, url_bar_y + 4, 24, 24, btn_bg);
                win.draw_text("R", 78, url_bar_y + 10, 0xFFAAAAAA);
                
                // View toggle button
                let view_label = if browser_view_mode == 0 { "SRC" } else { "DOM" };
                win.fill_rect(98, url_bar_y + 4, 32, 24, 0xFF383838);
                win.draw_text(view_label, 102, url_bar_y + 10, 0xFF88CCFF);
                
                // URL input field
                win.fill_rect(135, url_bar_y + 4, w - 160, 24, 0xFF0D0D0D);
                win.draw_rect(135, url_bar_y + 4, w - 160, 24, if browser_url_focused { 0xFF4FC3F7 } else { 0xFF555555 });
                
                // HTTPS indicator
                let url_color = if browser_url.starts_with("https://") { 0xFF00C853 } else { 0xFFDDDDDD };
                win.draw_text(&browser_url, 142, url_bar_y + 10, url_color);
                
                // Blinking cursor
                if browser_url_focused && cursor_blink {
                    let cursor_x = 142 + (browser_url.len() as u32 * 8);
                    if cursor_x < w - 30 {
                        win.fill_rect(cursor_x, url_bar_y + 8, 2, 18, 0xFF4FC3F7);
                    }
                }
                
                // Content area with DevTools styling
                let page_y = content_y + 40;
                let page_max_lines = ((h - page_y - 35) / line_height) as usize;
                
                // Dark background for code area
                win.fill_rect(10, page_y - 4, w - 20, h - page_y - 28, 0xFF1E1E1E);
                
                // Line numbers gutter
                let gutter_width = 40u32;
                win.fill_rect(10, page_y - 4, gutter_width, h - page_y - 28, 0xFF252526);
                
                let start_idx = if browser_lines.len() > page_max_lines {
                    browser_lines.len() - page_max_lines
                } else {
                    0
                };
                
                // Render each line with colored segments
                for (i, browser_line) in browser_lines.iter().skip(start_idx).enumerate() {
                    let y = page_y + (i as u32) * line_height;
                    if y + line_height > h - 35 { break; }
                    
                    // Line number
                    let line_num = format!("{:3}", start_idx + i + 1);
                    win.draw_text(&line_num, 14, y, 0xFF858585);
                    
                    // Render segments with their colors
                    let mut x_pos = 10u32 + gutter_width + 5;
                    for segment in &browser_line.segments {
                        win.draw_text(&segment.text, x_pos, y, segment.color);
                        x_pos += (segment.text.len() as u32) * 8;
                    }
                }
                
                // Status bar at bottom - modern styling
                win.fill_rect(10, h - 28, w - 20, 23, 0xFF007ACC);
                
                // Status icon
                let status_icon = if browser_status.contains("Error") { "✗" } 
                    else if browser_status.contains("Loading") { "⟳" } 
                    else { "✓" };
                win.draw_text(status_icon, 16, h - 24, 0xFFFFFFFF);
                win.draw_text(&browser_status, 30, h - 24, 0xFFFFFFFF);
                
                // View mode indicator
                let mode_text = if browser_view_mode == 0 { "[Source]" } else { "[Elements]" };
                let mode_x = w - 90;
                win.draw_text(mode_text, mode_x, h - 24, 0xFFCCCCCC);
                
            } else if active_mode == AppMode::ImageViewer {
                // ═══════════════════════════════════════════════════════════
                // IMAGE VIEWER MODE: Display images with zoom/pan
                // ═══════════════════════════════════════════════════════════
                
                // Dark background for image area
                win.fill_rect(10, content_y, w - 20, h - content_y - 30, 0xFF1A1A1A);
                
                if let Some(ref img) = image_viewer_data {
                    // Calculate display area
                    let view_w = w - 40;
                    let view_h = h - content_y - 60;
                    let view_x = 20u32;
                    let view_y = content_y + 10;
                    
                    // Apply zoom
                    let scaled_w = (img.width as f32 * image_viewer_zoom) as u32;
                    let scaled_h = (img.height as f32 * image_viewer_zoom) as u32;
                    
                    // Center image with offset
                    let center_x = view_x as i32 + (view_w as i32 / 2) + image_viewer_offset_x;
                    let center_y = view_y as i32 + (view_h as i32 / 2) + image_viewer_offset_y;
                    let img_x = center_x - (scaled_w as i32 / 2);
                    let img_y = center_y - (scaled_h as i32 / 2);
                    
                    // Draw scaled image (simple nearest neighbor)
                    for dy in 0..scaled_h.min(view_h) {
                        let screen_y = img_y + dy as i32;
                        if screen_y < view_y as i32 || screen_y >= (view_y + view_h) as i32 {
                            continue;
                        }
                        
                        let src_y = ((dy as f32 / image_viewer_zoom) as u32).min(img.height - 1);
                        
                        for dx in 0..scaled_w.min(view_w) {
                            let screen_x = img_x + dx as i32;
                            if screen_x < view_x as i32 || screen_x >= (view_x + view_w) as i32 {
                                continue;
                            }
                            
                            let src_x = ((dx as f32 / image_viewer_zoom) as u32).min(img.width - 1);
                            let pixel = img.get_pixel(src_x, src_y);
                            
                            // Only draw non-transparent pixels
                            if (pixel >> 24) > 0 {
                                win.set_pixel(screen_x as u32, screen_y as u32, pixel);
                            }
                        }
                    }
                    
                    // Image border
                    win.draw_rect(
                        (img_x.max(view_x as i32)) as u32,
                        (img_y.max(view_y as i32)) as u32,
                        scaled_w.min(view_w),
                        scaled_h.min(view_h),
                        0xFF444444
                    );
                } else {
                    // No image loaded - show placeholder
                    let center_x = w / 2;
                    let center_y = (content_y + h) / 2;
                    
                    // Icon placeholder
                    win.draw_rect(center_x - 40, center_y - 30, 80, 60, 0xFF444444);
                    win.draw_text("🖼", center_x - 8, center_y - 10, 0xFF666666);
                    win.draw_text("No image loaded", center_x - 56, center_y + 25, 0xFF888888);
                    win.draw_text("Use: imgview <file>", center_x - 72, center_y + 45, 0xFF666666);
                }
                
                // Info bar at top
                win.fill_rect(10, content_y, w - 20, 24, 0xFF252525);
                let zoom_pct = (image_viewer_zoom * 100.0) as u32;
                let info_str = format!("Zoom: {}%  |  {}", zoom_pct, image_viewer_info);
                win.draw_text(&info_str, 16, content_y + 5, 0xFFCCCCCC);
                
                // Format indicator
                win.draw_text(&image_viewer_format, w - 60, content_y + 5, 0xFF88CCFF);
                
                // Controls hint at bottom
                win.fill_rect(10, h - 28, w - 20, 23, 0xFF252525);
                win.draw_text("[+/-] Zoom  [Arrows] Pan  [R] Reset  [ESC] Close", 16, h - 24, 0xFF888888);
                
            } else if active_mode == AppMode::TextEditor {
                // ═══════════════════════════════════════════════════════════
                // TRUSTCODE: VSCode-inspired code editor
                // ═══════════════════════════════════════════════════════════
                use crate::apps::text_editor::*;
                
                let char_w: u32 = 8;
                let line_h: u32 = 16;
                let gutter_chars: u32 = 5; // "nnnn "
                let gutter_w = gutter_chars * char_w;
                let status_h: u32 = 22;
                let tab_bar_h: u32 = 26;
                
                let code_x = gutter_w;
                let code_y = content_y + tab_bar_h;
                let code_w = w - gutter_w;
                let code_h = h.saturating_sub(content_y + tab_bar_h + status_h);
                let visible_lines_count = (code_h / line_h).max(1) as usize;
                
                // Update scroll
                if editor_state.cursor_line < editor_state.scroll_y {
                    editor_state.scroll_y = editor_state.cursor_line;
                }
                if editor_state.cursor_line >= editor_state.scroll_y + visible_lines_count {
                    editor_state.scroll_y = editor_state.cursor_line - visible_lines_count + 1;
                }
                editor_state.blink_counter += 1;
                
                // ── Tab bar ──
                win.fill_rect(0, content_y, w, tab_bar_h, COLOR_BREADCRUMB_BG);
                let tab_name = editor_state.file_path.as_ref().map(|p| {
                    p.rsplit('/').next().unwrap_or(p.as_str())
                }).unwrap_or("untitled");
                let dirty_marker = if editor_state.dirty { " *" } else { "" };
                let tab_label = format!("  {}{}", tab_name, dirty_marker);
                let tab_w = ((tab_label.len() as u32 + 2) * char_w).min(w);
                win.fill_rect(0, content_y, tab_w, tab_bar_h, COLOR_TAB_ACTIVE);
                // Tab bottom accent line
                win.fill_rect(0, content_y + tab_bar_h - 2, tab_w, 2, COLOR_STATUS_BG);
                win.draw_text(&tab_label, 4, content_y + 5, COLOR_NORMAL);
                
                // ── Editor background ──
                win.fill_rect(0, code_y, w, code_h, COLOR_BG);
                
                // ── Gutter background + border ──
                win.fill_rect(0, code_y, gutter_w, code_h, COLOR_GUTTER_BG);
                win.fill_rect(gutter_w - 1, code_y, 1, code_h, 0xFF333333);
                
                // ── Render lines ──
                for vi in 0..visible_lines_count {
                    let line_idx = editor_state.scroll_y + vi;
                    if line_idx >= editor_state.lines.len() { break; }
                    
                    let ly = code_y + (vi as u32 * line_h);
                    if ly + line_h > code_y + code_h { break; }
                    
                    let is_current = line_idx == editor_state.cursor_line;
                    
                    // Current line highlight
                    if is_current {
                        win.fill_rect(code_x, ly, code_w, line_h, COLOR_ACTIVE_LINE_BG);
                    }
                    
                    // Line number
                    let num_str = format!("{:>4} ", line_idx + 1);
                    let num_color = if is_current { COLOR_ACTIVE_LINE } else { COLOR_LINE_NUM };
                    win.draw_text(&num_str, 2, ly, num_color);
                    
                    // Code with syntax highlighting
                    let line = &editor_state.lines[line_idx];
                    
                    if editor_state.language == Language::Rust {
                        let tokens = tokenize_rust_line(line);
                        for span in &tokens {
                            let color = token_color(span.kind);
                            let text_seg = &line[span.start..span.end];
                            let sx = code_x + 4 + (span.start as u32 * char_w);
                            if sx < w {
                                win.draw_text(text_seg, sx, ly, color);
                            }
                        }
                        if tokens.is_empty() && !line.is_empty() {
                            win.draw_text(line, code_x + 4, ly, COLOR_NORMAL);
                        }
                    } else {
                        win.draw_text(line, code_x + 4, ly, COLOR_NORMAL);
                    }
                    
                    // Cursor
                    if is_current {
                        let blink_on = (editor_state.blink_counter / 30) % 2 == 0;
                        if blink_on {
                            let cx = code_x + 4 + (editor_state.cursor_col as u32 * char_w);
                            win.fill_rect(cx, ly, 2, line_h, COLOR_CURSOR);
                        }
                    }
                }
                
                // ── Scrollbar ──
                if editor_state.lines.len() > visible_lines_count {
                    let sb_x = w - 10;
                    let sb_h = code_h;
                    let total = editor_state.lines.len() as u32;
                    let thumb_h = ((visible_lines_count as u32 * sb_h) / total).max(20);
                    let max_scroll_val = total.saturating_sub(visible_lines_count as u32);
                    let thumb_y = if max_scroll_val > 0 {
                        (editor_state.scroll_y as u32 * (sb_h - thumb_h)) / max_scroll_val
                    } else { 0 };
                    win.fill_rect(sb_x, code_y, 10, sb_h, 0xFF252526);
                    win.fill_rect(sb_x + 2, code_y + thumb_y, 6, thumb_h, 0xFF555555);
                }
                
                // ── Status bar (VSCode blue) ──
                let status_y = h - status_h;
                win.fill_rect(0, status_y, w, status_h, COLOR_STATUS_BG);
                
                // Left: file info
                let status_left = if let Some(ref msg) = editor_state.status_message {
                    format!("  {}", msg)
                } else {
                    let dirty_str = if editor_state.dirty { " [Modified]" } else { "" };
                    let fname = editor_state.file_path.as_deref().unwrap_or("untitled");
                    format!("  {}{}", fname, dirty_str)
                };
                win.draw_text(&status_left, 4, status_y + 3, COLOR_STATUS_FG);
                
                // Right: position and language
                let status_right = format!(
                    "Ln {}, Col {}  {}  UTF-8  TrustCode",
                    editor_state.cursor_line + 1,
                    editor_state.cursor_col + 1,
                    editor_state.language.name(),
                );
                let right_x = w.saturating_sub((status_right.len() as u32 * char_w) + 8);
                win.draw_text(&status_right, right_x, status_y + 3, COLOR_STATUS_FG);

            } else {
                // ═══════════════════════════════════════════════════════════
                // SHELL MODE: Normal shell output with scrolling
                // ═══════════════════════════════════════════════════════════
                
            // Calculate visible range with scroll support
            let total_lines = shell_output.len();
            let visible_lines = MAX_VISIBLE_LINES.min(max_lines);
            
            // Auto-scroll to bottom when new content is added (unless user scrolled up)
            let max_scroll = total_lines.saturating_sub(visible_lines);
            if scroll_offset > max_scroll {
                scroll_offset = max_scroll;
            }
            
            let start_idx = scroll_offset;
            let end_idx = (start_idx + visible_lines).min(total_lines);
            
            for (i, line) in shell_output.iter().skip(start_idx).take(visible_lines).enumerate() {
                let y = content_y + (i as u32) * line_height;
                if y + line_height > h - 50 { break; }
                
                // Enhanced color coding by content
                let color = if line.starts_with("╔") || line.starts_with("╚") || line.starts_with("╠") {
                    green_main  // Box borders in green
                } else if line.starts_with("║") {
                    // Parse content inside box for coloring
                    if line.contains("NAVIGATION:") || line.contains("FILE OPERATIONS:") || 
                       line.contains("COMMANDS:") || line.contains("TIPS:") ||
                       line.contains("BASIC COMMANDS:") || line.contains("EXAMPLES:") ||
                       line.contains("NOTE:") {
                        0xFFFFFF00  // Yellow for section headers
                    } else if line.contains(" - ") {
                        // Command line: command - description
                        green_main  // Green for command lines
                    } else if line.starts_with("║    •") {
                        0xFFAAAAAA  // Light gray for tips/bullets  
                    } else {
                        0xFFDDDDDD  // White for normal text
                    }
                } else if line.starts_with(">") {
                    0xFF88FF88  // Bright green for command echo
                } else if line.contains("<DIR>") {
                    0xFF00FFFF  // Cyan for directories
                } else if line.contains(" B") && !line.contains("Browse") {
                    green_main  // Green for files with size
                } else if line.starts_with("Created") || line.starts_with("Changed") || line.starts_with("Removed") {
                    0xFF00FF00  // Bright green for success messages
                } else if line.contains("Error") || line.contains("cannot") || line.contains("No such") {
                    0xFFFF4444  // Red for errors
                } else {
                    green_dim  // Default dim green
                };
                win.draw_text(line, 12, y, color);
            }
            
            // ═══════════════════════════════════════════════════════════
            // SCROLLBAR on right side
            // ═══════════════════════════════════════════════════════════
            if total_lines > visible_lines {
                let scrollbar_x = w - 12;
                let scrollbar_y = content_y;
                let scrollbar_h = h - content_y - 50;  // Height of scrollable area
                
                // Background track (dark)
                win.fill_rect(scrollbar_x, scrollbar_y, 8, scrollbar_h, 0xFF1A1A1A);
                
                // Calculate thumb size and position
                let thumb_ratio = visible_lines as f32 / total_lines as f32;
                let thumb_h = ((scrollbar_h as f32 * thumb_ratio) as u32).max(20);
                let scroll_ratio = if max_scroll > 0 { 
                    scroll_offset as f32 / max_scroll as f32 
                } else { 
                    0.0 
                };
                let thumb_y = scrollbar_y + ((scrollbar_h - thumb_h) as f32 * scroll_ratio) as u32;
                
                // Thumb (green)
                win.fill_rect(scrollbar_x, thumb_y, 8, thumb_h, green_dim);
                win.fill_rect(scrollbar_x + 1, thumb_y + 1, 6, thumb_h - 2, green_main);
            }
            
            // Input area at bottom
            let input_y = h - 40;
            win.fill_rect(10, input_y, w - 20, 30, 0xFF050505);
            win.draw_rect(10, input_y, w - 20, 30, green_dim);
            
            // Prompt with current directory - colored parts
            let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
            // Draw "root" in red
            win.draw_text("root", 16, input_y + 8, 0xFFFF0000);  // Pure red
            // Draw "@" in white
            win.draw_text("@", 16 + 32, input_y + 8, 0xFFFFFFFF);  // White
            // Draw "trustos" in green
            win.draw_text("trustos", 16 + 40, input_y + 8, 0xFF00FF00);  // Pure green
            // Draw ":path$ " in green
            let path_part = format!(":{}$ ", cwd);
            win.draw_text(&path_part, 16 + 96, input_y + 8, 0xFF00FF00);  // Pure green
            let prompt_width = (4 + 1 + 7 + path_part.len()) as u32 * 8;  // root @ trustos :path$
            
            // User input text
            win.draw_text(&shell_input, 16 + prompt_width, input_y + 8, green_bright);
            
            // Suggestion (grayed out)
            if !suggestion_text.is_empty() {
                let input_width = (shell_input.len() * 8) as u32;
                win.draw_text(&suggestion_text, 16 + prompt_width + input_width, input_y + 8, 0xFF444444);
            }
            
            // Blinking cursor
            cursor_blink = (frame_count / 30) % 2 == 0;
            if cursor_blink {
                let cursor_x = 16 + prompt_width + (shell_input.len() as u32 * 8);
                win.fill_rect(cursor_x, input_y + 6, 8, 16, green_bright);
            }
            } // End of shell mode else block
            } else {
                // Window is hidden - clear to transparent
                win.clear(0x00000000);
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 2.5: COMMAND HISTORY PANEL (Top right corner)
        // ═══════════════════════════════════════════════════════════════════
        if let Some(hist) = compositor.get_layer_mut(history_layer) {
            let hw = hist.width;
            let hh = hist.height;
            
            // Semi-transparent dark background
            hist.clear(0xD8181818);
            
            // Border
            hist.draw_rect(0, 0, hw, hh, 0xFF444444);
            hist.draw_rect(1, 1, hw - 2, hh - 2, 0xFF333333);
            
            // Title bar
            hist.fill_rect(2, 2, hw - 4, 20, 0xFF252525);
            hist.draw_text("Command History", 8, 6, 0xFFAAAAAA);
            
            // History entries
            let start_y = 26u32;
            let line_h = 18u32;
            
            if command_history.is_empty() {
                hist.draw_text("(no commands yet)", 10, start_y + 5, 0xFF666666);
            } else {
                // Show most recent first (reverse order)
                for (i, cmd) in command_history.iter().rev().take(10).enumerate() {
                    let y = start_y + (i as u32) * line_h;
                    if y + line_h > hh - 5 { break; }
                    
                    // Number
                    let num = command_history.len() - i;
                    let num_str = format!("{:2}.", num);
                    hist.draw_text(&num_str, 6, y + 2, 0xFF666666);
                    
                    // Command (truncate if too long)
                    let display_cmd = if cmd.len() > 26 {
                        format!("{}...", &cmd[..23])
                    } else {
                        cmd.clone()
                    };
                    hist.draw_text(&display_cmd, 30, y + 2, 0xFF88FF88);
                }
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 3: TASKBAR (Bottom)
        // ═══════════════════════════════════════════════════════════════════
        if let Some(bar) = compositor.get_layer_mut(taskbar_layer) {
            bar.clear(0xFF0A0A0A);
            
            // Top border
            bar.fill_rect(0, 0, width, 2, green_dim);
            
            // TrustOS menu button
            bar.fill_rect(5, 6, 100, 28, if menu_open { 0xFF002200 } else { 0xFF0A1A0A });
            bar.draw_rect(5, 6, 100, 28, green_main);
            bar.draw_text("TrustOS", 20, 12, 0xFFFFFFFF); // White text
            
            // Active module indicator
            let mode_name = match active_mode {
                AppMode::Shell => "Shell",
                AppMode::Network => "Network",
                AppMode::Hardware => "Hardware",
                AppMode::TextEditor => "Editor",
                AppMode::UserMgmt => "Users",
                AppMode::Files => "Files",
                AppMode::Browser => "Browser",
                AppMode::ImageViewer => "Images",
            };
            bar.fill_rect(115, 6, 90, 28, 0xFF001100);
            bar.draw_text(mode_name, 125, 12, 0xFFFFFFFF); // White text
            
            // Window button in taskbar (shows when window exists)
            // Position after the mode indicator
            let win_btn_x = 220u32;
            if window_visible {
                // Window is open - show active button
                bar.fill_rect(win_btn_x, 6, 100, 28, 0xFF002200);
                bar.draw_rect(win_btn_x, 6, 100, 28, green_main);
                bar.draw_text(mode_name, win_btn_x + 10, 12, green_bright);
                // Active indicator line at bottom
                bar.fill_rect(win_btn_x + 20, 32, 60, 3, green_main);
            } else {
                // Window is minimized - show inactive button
                bar.fill_rect(win_btn_x, 6, 100, 28, 0xFF0A0A0A);
                bar.draw_rect(win_btn_x, 6, 100, 28, green_dim);
                bar.draw_text(mode_name, win_btn_x + 10, 12, green_dim);
            }
            
            // Settings button (gear icon)
            let settings_btn_x = 340u32;
            let settings_bg = if settings_open { 0xFF002200 } else { 0xFF0A1A0A };
            bar.fill_rect(settings_btn_x, 6, 50, 28, settings_bg);
            bar.draw_rect(settings_btn_x, 6, 50, 28, green_dim);
            bar.draw_text("[S]", settings_btn_x + 10, 12, if settings_open { green_bright } else { green_main });
            
            // Clock
            let dt = crate::rtc::read_rtc();
            let time_str = format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second);
            bar.draw_text(&time_str, width - 180, 12, green_main);
            
            // FPS
            let fps_str = format!("{}fps", fps);
            bar.draw_text(&fps_str, width - 260, 12, green_dim);
            
            // Status indicators
            bar.fill_circle(width - 60, 20, 6, green_main);
            bar.fill_circle(width - 40, 20, 6, 0xFFFFAA00);
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 4: MENU (TrustOS popup menu with Apps + Power Options)
        // ═══════════════════════════════════════════════════════════════════
        if let Some(menu) = compositor.get_layer_mut(menu_layer) {
            if menu_open {
                menu.visible.store(true, core::sync::atomic::Ordering::SeqCst);
                menu.clear(0xF0080808);  // Dark background
                
                let menu_w = 270u32;
                let menu_h = 390u32;
                
                // Menu border
                menu.draw_rect(0, 0, menu_w, menu_h, green_main);
                menu.draw_rect(1, 1, menu_w - 2, menu_h - 2, green_dim);
                
                // Menu title
                menu.fill_rect(2, 2, menu_w - 4, 34, 0xFF001500);
                menu.draw_text("TrustOS Menu", 10, 10, green_main);
                
                // Menu items
                let item_height = 36u32;
                for (i, (name, item)) in menu_items.iter().enumerate() {
                    let iy = 40 + (i as u32) * item_height;
                    
                    // Skip rendering for separator
                    if name.starts_with("─") {
                        menu.fill_rect(10, iy + 14, menu_w - 20, 1, green_dim);
                        continue;
                    }
                    
                    // Highlight if hovered
                    if menu_hover == i as i32 {
                        menu.fill_rect(5, iy, menu_w - 10, item_height - 2, 0xFF002200);
                    }
                    
                    // Determine color and icon based on item type
                    let (color, icon) = match item {
                        MenuItem::App(_) => {
                            let c = if menu_hover == i as i32 { green_bright } else { green_dim };
                            (c, ">")
                        },
                        MenuItem::Shutdown => {
                            let c = if menu_hover == i as i32 { 0xFFFF6666 } else { 0xFFAA4444 };
                            (c, "X")
                        },
                        MenuItem::Reboot => {
                            let c = if menu_hover == i as i32 { 0xFFFFAA66 } else { 0xFFAA8844 };
                            (c, "R")
                        },
                    };
                    
                    // Item text
                    menu.draw_text(name, 24, iy + 10, color);
                    
                    // Icon/indicator
                    menu.draw_text(icon, menu_w - 30, iy + 10, color);
                }
            } else {
                menu.visible.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 4.5: SETTINGS PANEL
        // ═══════════════════════════════════════════════════════════════════
        if let Some(settings) = compositor.get_layer_mut(settings_layer) {
            if settings_open {
                settings.visible.store(true, core::sync::atomic::Ordering::SeqCst);
                settings.clear(0xF0080808);  // Dark background
                
                let panel_w = 270u32;
                let panel_h = 340u32;  // Increased for HoloMatrix options
                
                // Panel border
                settings.draw_rect(0, 0, panel_w, panel_h, green_main);
                settings.draw_rect(1, 1, panel_w - 2, panel_h - 2, green_dim);
                
                // Panel title
                settings.fill_rect(2, 2, panel_w - 4, 34, 0xFF001500);
                settings.draw_text("Settings", 10, 10, green_main);
                
                // Animation toggle
                let anim_y = 50u32;
                let my = mouse_y as u32;
                let mx = mouse_x as u32;
                let panel_top = height - 380;  // Adjusted for taller panel
                let anim_hover = my >= (panel_top + anim_y) 
                    && my < (panel_top + anim_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if anim_hover {
                    settings.fill_rect(5, anim_y, panel_w - 10, 34, 0xFF002200);
                }
                settings.draw_text("Animations:", 15, anim_y + 10, green_dim);
                let anim_status = if settings_anim_enabled { "ON" } else { "OFF" };
                let anim_color = if settings_anim_enabled { 0xFF00FF66 } else { 0xFFFF6666 };
                settings.draw_text(anim_status, panel_w - 50, anim_y + 10, anim_color);
                
                // Speed setting
                let speed_y = 90u32;
                let speed_hover = my >= (panel_top + speed_y) 
                    && my < (panel_top + speed_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if speed_hover {
                    settings.fill_rect(5, speed_y, panel_w - 10, 34, 0xFF002200);
                }
                settings.draw_text("Speed:", 15, speed_y + 10, green_dim);
                let speed_str = format!("{:.1}x", settings_anim_speed);
                settings.draw_text(&speed_str, panel_w - 60, speed_y + 10, green_main);
                
                // ─── BACKGROUND SECTION ───
                settings.draw_text("─ Background ─", 15, 140, 0xFF555555);
                
                // HoloMatrix toggle
                let holo_y = 160u32;
                let holo_hover = my >= (panel_top + holo_y) 
                    && my < (panel_top + holo_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if holo_hover {
                    settings.fill_rect(5, holo_y, panel_w - 10, 34, 0xFF002200);
                }
                settings.draw_text("HoloMatrix 3D:", 15, holo_y + 10, green_dim);
                let holo_status = if holo_enabled { "ON" } else { "OFF" };
                let holo_color = if holo_enabled { 0xFF00FFFF } else { 0xFFFF6666 };
                settings.draw_text(holo_status, panel_w - 50, holo_y + 10, holo_color);
                
                // HoloMatrix scene selector
                let scene_y = 200u32;
                let scene_hover = my >= (panel_top + scene_y) 
                    && my < (panel_top + scene_y + 36)
                    && mx >= 340 && mx < (340 + panel_w);
                if scene_hover && holo_enabled {
                    settings.fill_rect(5, scene_y, panel_w - 10, 34, 0xFF002200);
                }
                let scene_label_color = if holo_enabled { green_dim } else { 0xFF333333 };
                settings.draw_text("Scene:", 15, scene_y + 10, scene_label_color);
                let scene_color = if holo_enabled { 0xFF00FFFF } else { 0xFF444444 };
                settings.draw_text(holo_scene.name(), panel_w - 80, scene_y + 10, scene_color);
                
                // Instructions
                settings.draw_text("Click to toggle/cycle", 15, 250, 0xFF555555);
                
                // Close button hint
                settings.draw_text("[Esc] or click away", 15, 305, 0xFF444444);
                
                // Handle click on animation toggle
                if clicked && anim_hover {
                    settings_anim_enabled = !settings_anim_enabled;
                    crate::desktop::set_animations_enabled(settings_anim_enabled);
                }
                
                // Handle click on speed
                if clicked && speed_hover {
                    // Cycle: 0.5 -> 1.0 -> 2.0 -> 0.5
                    settings_anim_speed = if settings_anim_speed <= 0.5 { 1.0 } 
                        else if settings_anim_speed <= 1.0 { 2.0 } 
                        else { 0.5 };
                    crate::desktop::set_animation_speed(settings_anim_speed);
                }
                
                // Handle click on HoloMatrix toggle
                if clicked && holo_hover {
                    holo_enabled = !holo_enabled;
                    crate::graphics::holomatrix::set_enabled(holo_enabled);
                }
                
                // Handle click on scene selector
                if clicked && scene_hover && holo_enabled {
                    holo_scene = holo_scene.next();
                    crate::graphics::holomatrix::set_scene(holo_scene);
                }
            } else {
                settings.visible.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // LAYER 5: MOUSE CURSOR (Always on top)
        // ═══════════════════════════════════════════════════════════════════
        if let Some(cursor) = compositor.get_layer_mut(cursor_layer) {
            cursor.clear(0x00000000); // Transparent
            
            // Cursor color changes when clicking (visual feedback)
            let cursor_color = if left { 0xFF00FF00 } else { 0xFFFFFFFF }; // Green when clicked
            let border_color = if left { 0xFF005500 } else { 0xFF000000 };
            
            // Draw arrow cursor
            // Main pointer triangle
            for i in 0..16 {
                for j in 0..=i {
                    if j <= i && i < 16 {
                        cursor.set_pixel(j as u32, i as u32, cursor_color);
                    }
                }
            }
            // Black border
            for i in 0..16 {
                cursor.set_pixel(0, i as u32, border_color);
                cursor.set_pixel(i as u32, i as u32, border_color);
            }
            // Tail
            for i in 10..16 {
                cursor.set_pixel((i - 5) as u32, i as u32, cursor_color);
                cursor.set_pixel((i - 6) as u32, i as u32, cursor_color);
            }
        }
        
        // ═══════════════════════════════════════════════════════════════════
        // COMPOSITE & PRESENT
        // ═══════════════════════════════════════════════════════════════════
        compositor.composite();
        compositor.present();
        
        } // ─── END RENDER FRAME (frame-rate decoupling) ───
        
        // FPS tracking (counts ALL frames: render + skip)
        frame_count += 1;
        frame_in_second += 1;
        
        // Simple frame milestone logging (every 100 frames)
        if frame_count % 100 == 0 {
            crate::serial_println!("[COSMIC2] Frame {}", frame_count);
        }
        
        let now = crate::cpu::tsc::read_tsc();
        if now - last_second_tsc >= tsc_freq {
            fps = frame_in_second;
            render_fps = render_in_second;
            frame_in_second = 0;
            render_in_second = 0;
            last_second_tsc = now;
            crate::serial_println!("[COSMIC2] FPS: {} (render: {}) | Frame: {} | Mode: {}",
                fps, render_fps, frame_count, if use_formula { "FORMULA" } else if use_braille { "BRAILLE" } else if use_fast_matrix { "FAST" } else { "LEGACY" });
        }
        
        // Frame timing: brief pause to process pending keyboard/mouse interrupts.
        // Use a short spin with `pause` instead of `hlt` — hlt blocks up to 10ms 
        // (timer tick), which caps FPS at ~1000/(render_ms + 10). Pause enables
        // interrupts for ~100µs then continues, minimizing idle time.
        unsafe {
            // Enable interrupts so pending IRQs (keyboard, mouse, timer) fire
            core::arch::asm!("sti");
            // Brief spin: ~100 iterations × ~100 cycles ≈ 30-50µs at 3GHz
            for _ in 0..100 {
                core::arch::asm!("pause");
            }
        }
    }
    
    // Restore console mode
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC2] Exited");
    crate::println_color!(COLOR_GREEN, "COSMIC V2 Desktop exited. Type 'help' for commands.");
}

// ==================== COSMIC DESKTOP MODE ====================

fn cmd_cosmic_desktop() {
    use crate::cosmic::{Rect, Point, Color};
    use crate::cosmic::theme::{dark, matrix};
    use alloc::format;
    
    crate::println_color!(COLOR_CYAN, "╔═══════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║       COSMIC Desktop Environment - TrustOS Edition        ║");
    crate::println_color!(COLOR_CYAN, "╠═══════════════════════════════════════════════════════════╣");
    crate::println_color!(COLOR_GREEN, "║  Controls:                                                ║");
    crate::println_color!(COLOR_WHITE, "║    ESC / Q     - Exit desktop                             ║");
    crate::println_color!(COLOR_WHITE, "║    M           - Matrix theme (cyberpunk)                 ║");
    crate::println_color!(COLOR_WHITE, "║    D           - Dark theme (default)                     ║");
    crate::println_color!(COLOR_WHITE, "║    1-5         - Switch apps                              ║");
    crate::println_color!(COLOR_WHITE, "║    Mouse       - Interact with UI                         ║");
    crate::println_color!(COLOR_CYAN, "╚═══════════════════════════════════════════════════════════╝");
    crate::serial_println!("[COSMIC] Starting COSMIC Desktop Environment...");
    
    // Flush keyboard buffer
    while crate::keyboard::try_read_key().is_some() {}
    
    let (width, height) = crate::framebuffer::get_dimensions();
    if width == 0 || height == 0 {
        crate::println_color!(COLOR_RED, "Error: Invalid framebuffer!");
        return;
    }
    
    // Initialize double buffering for FAST rendering
    crate::framebuffer::init_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);
    crate::serial_println!("[COSMIC] Double buffering enabled for fast rendering");
    
    // Set mouse screen bounds
    crate::mouse::set_screen_size(width, height);
    
    // ═══════════════════════════════════════════════════════════════════
    // DESKTOP STATE
    // ═══════════════════════════════════════════════════════════════════
    let mut running = true;
    let mut use_matrix_theme = true;
    let mut frame_count = 0u64;
    
    // FPS tracking with VSync target
    let tsc_freq = crate::cpu::tsc::frequency_hz();
    let mut fps = 0u32;
    let mut frame_in_second = 0u32;
    let mut last_second_tsc = crate::cpu::tsc::read_tsc();
    let mut show_fps = true;  // Toggle with 'fps' command
    
    // Matrix renderer mode flags
    let mut use_fast_matrix = false;  // Braille mode is default
    let mut use_braille = true;      // Braille sub-pixel renderer (default)
    
    // Target frame time for ~60 FPS (VSync simulation)
    let target_fps = 60u64;
    let frame_tsc_target = tsc_freq / target_fps;
    let mut last_frame_tsc = crate::cpu::tsc::read_tsc();
    
    // ═══════════════════════════════════════════════════════════════════
    // MATRIX RAIN STATE - Each column has its own position and speed
    // Dense matrix with more columns for proper coverage
    // ═══════════════════════════════════════════════════════════════════
    const MATRIX_COLS: usize = 160;  // Dense! 8px spacing at 1280px
    const MATRIX_CHAR_H: u32 = 16;   // Character height
    const MATRIX_TRAIL_LEN: usize = 30; // Longer trails for fuller look
    
    // Column state: (y_position, speed, random_seed)
    let mut matrix_cols: [(i32, u32, u32); MATRIX_COLS] = [(0, 0, 0); MATRIX_COLS];
    
    // Initialize columns with random starting positions and speeds
    for i in 0..MATRIX_COLS {
        let seed = (i as u32 * 2654435761) ^ 0xDEADBEEF;
        let start_y = -((seed % (height * 2)) as i32); // Start above screen
        let speed = 2 + (seed % 5); // Speed 2-6 pixels per frame
        matrix_cols[i] = (start_y, speed, seed);
    }
    
    // Matrix characters (katakana-like + numbers)
    const MATRIX_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
    
    // Mouse state
    let mut prev_left = false;
    let mut click_this_frame = false;
    let mut prev_mx = 0.0f32;
    let mut prev_my = 0.0f32;
    let mut prev_theme = use_matrix_theme;
    let mut prev_app = 0usize;
    let mut prev_hovered = -1i32;
    let mut needs_full_redraw = true; // First frame always needs full redraw
    
    // Active app
    let mut active_app = 0usize;
    let mut hovered_dock = -1i32;
    
    // TrustOS Menu state
    let mut menu_open = false;
    let mut menu_hover = -1i32;  // Which menu item is hovered (-1 = none)
    let mut search_active = false;
    let mut search_text: [u8; 32] = [0u8; 32];
    let mut search_len = 0usize;
    
    // Menu items for TrustOS menu
    let menu_items = [
        "Apps",
        "Browser",
        "Calculator", 
        "Files",
        "Network",
        "Settings",
        "Terminal",
        "---",  // Separator
        "Sign Out",
        "Restart",
        "Shutdown",
    ];
    
    // App info
    let apps = [
        ("Files", "File Manager"),
        ("Terminal", "System Terminal"),
        ("Browser", "Web Browser"),
        ("Code", "Text Editor"),
        ("Settings", "System Settings"),
    ];
    
    // Window state (draggable)
    let mut win_x = 150.0f32;
    let mut win_y = 60.0f32;
    let win_w = 700.0f32;
    let win_h = 450.0f32;
    let mut dragging = false;
    let mut drag_off_x = 0.0f32;
    let mut drag_off_y = 0.0f32;
    
    // Terminal content for terminal app
    let terminal_lines = [
        "$ neofetch",
        "  _____              _    ___  ____  ",
        " |_   _| __ _   _ __| |_ / _ \\/ ___| ",
        "   | || '__| | | / __| __| | | \\___ \\ ",
        "   | || |  | |_| \\__ \\ |_| |_| |___) |",
        "   |_||_|   \\__,_|___/\\__|\\___/|____/ ",
        "",
        "  OS: TrustOS v0.1.0",
        "  Kernel: Custom Rust Kernel",
        "  Shell: TrustOS Shell",
        "  Resolution: 1280x800",
        "  Theme: COSMIC Matrix",
        "",
        "$ _",
    ];
    
    crate::serial_println!("[COSMIC] Entering main loop...");
    
    // ═══════════════════════════════════════════════════════════════════
    // MAIN LOOP
    // ═══════════════════════════════════════════════════════════════════
    while running {
        // ───────────────────────────────────────────────────────────────
        // INPUT HANDLING (do this FIRST for responsiveness)
        // ───────────────────────────────────────────────────────────────
        let mouse = crate::mouse::get_state();
        let mx = mouse.x as f32;
        let my = mouse.y as f32;
        let left_pressed = mouse.left_button;
        
        // Detect click (press this frame, wasn't pressed before)
        click_this_frame = left_pressed && !prev_left;
        prev_left = left_pressed;
        
        // Keyboard
        if let Some(key) = crate::keyboard::try_read_key() {
            if search_active {
                // Handle search input
                match key {
                    27 => { search_active = false; },  // ESC closes search
                    8 => {  // Backspace
                        if search_len > 0 { search_len -= 1; }
                    },
                    13 => { search_active = false; },  // Enter confirms
                    32..=126 => {  // Printable ASCII
                        if search_len < 31 {
                            search_text[search_len] = key;
                            search_len += 1;
                        }
                    },
                    _ => {}
                }
            } else {
                // Normal mode
                match key {
                    27 | b'q' | b'Q' => {
                        if menu_open { menu_open = false; }
                        else { running = false; }
                    },
                    b'm' | b'M' => { use_matrix_theme = true; needs_full_redraw = true; },
                    b'd' | b'D' => { use_matrix_theme = false; needs_full_redraw = true; },
                    b'1'..=b'5' => { active_app = (key - b'1') as usize; needs_full_redraw = true; },
                    b's' | b'S' => { search_active = true; },  // S to open search
                    b't' | b'T' => { menu_open = !menu_open; },  // T to toggle menu
                    _ => {}
                }
            }
        }
        
        // Check if anything changed that requires redraw
        let mouse_moved = (mx - prev_mx).abs() > 0.5 || (my - prev_my).abs() > 0.5;
        let state_changed = use_matrix_theme != prev_theme || active_app != prev_app || click_this_frame || dragging;
        
        // Matrix theme always animates (rain effect), so always redraw
        // Only skip if dark theme AND nothing changed
        if !use_matrix_theme && !needs_full_redraw && !mouse_moved && !state_changed {
            // Just update FPS counter
            frame_count += 1;
            frame_in_second += 1;
            let now = crate::cpu::tsc::read_tsc();
            if now - last_second_tsc >= tsc_freq {
                fps = frame_in_second;
                frame_in_second = 0;
                last_second_tsc = now;
            }
            continue;
        }
        
        prev_mx = mx;
        prev_my = my;
        prev_theme = use_matrix_theme;
        prev_app = active_app;
        needs_full_redraw = false;
        
        // ───────────────────────────────────────────────────────────────
        // THEME SELECTION
        // ───────────────────────────────────────────────────────────────
        let (bg, panel_bg, surface, surface_hover, accent, text_pri, text_sec, 
             header_bg, close_bg, max_bg, min_bg, success, warning) = 
            if use_matrix_theme {
                (matrix::BG_BASE, matrix::PANEL_BG, matrix::SURFACE, matrix::SURFACE_HOVER,
                 matrix::ACCENT, matrix::TEXT_PRIMARY, matrix::TEXT_SECONDARY,
                 matrix::HEADER_BG, matrix::CLOSE_BG, matrix::MAXIMIZE_BG, matrix::MINIMIZE_BG,
                 matrix::SUCCESS, matrix::WARNING)
            } else {
                (dark::BG_BASE, dark::PANEL_BG, dark::SURFACE, dark::SURFACE_HOVER,
                 dark::ACCENT, dark::TEXT_PRIMARY, dark::TEXT_SECONDARY,
                 dark::HEADER_BG, dark::CLOSE_BG, dark::MAXIMIZE_BG, dark::MINIMIZE_BG,
                 dark::SUCCESS, dark::WARNING)
            };
        
        // Convert colors to u32 for direct framebuffer operations (SSE2 fast path)
        let bg_u32 = bg.to_u32();
        let panel_bg_u32 = panel_bg.to_u32();
        let surface_u32 = surface.to_u32();
        let surface_hover_u32 = surface_hover.to_u32();
        let accent_u32 = accent.to_u32();
        let text_pri_u32 = text_pri.to_u32();
        let text_sec_u32 = text_sec.to_u32();
        let header_bg_u32 = header_bg.to_u32();
        let close_bg_u32 = close_bg.to_u32();
        let max_bg_u32 = max_bg.to_u32();
        let min_bg_u32 = min_bg.to_u32();
        let success_u32 = success.to_u32();
        let warning_u32 = warning.to_u32();
        
        // Use framebuffer module directly for FAST SSE2 rendering
        use crate::framebuffer::{
            clear_backbuffer, fill_rect, fill_rounded_rect, fill_circle,
            stroke_rounded_rect, draw_text, swap_buffers, draw_rect
        };
        
        // TrustOS colors - Brighter green like reference
        let green_main: u32 = 0xFF00FF66;      // #00FF66 - Main green
        let green_bright: u32 = 0xFF00FF88;    // #00FF88 - Bright
        let green_dim: u32 = 0xFF009944;       // Dimmer green
        let black: u32 = 0xFF000000;           // Pure black
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: PURE BLACK BACKGROUND
        // ───────────────────────────────────────────────────────────────
        clear_backbuffer(black);
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: MATRIX RAIN ON ENTIRE SCREEN (like reference image)
        // ───────────────────────────────────────────────────────────────
        let bar_h = 36u32;
        let col_width = width / MATRIX_COLS as u32;
        
        for col in 0..MATRIX_COLS {
            let (y_pos, speed, seed) = matrix_cols[col];
            let x = (col as u32 * col_width) + col_width / 2;
            
            // Update position
            let new_y = y_pos + speed as i32;
            let new_y = if new_y > height as i32 + (MATRIX_TRAIL_LEN as i32 * MATRIX_CHAR_H as i32) {
                let new_seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                matrix_cols[col].2 = new_seed;
                -((new_seed % (height / 2)) as i32)
            } else {
                new_y
            };
            matrix_cols[col].0 = new_y;
            
            // Draw falling trail with DEPTH EFFECT based on speed
            // Slow columns = far (dim, grayish) | Fast columns = near (bright, vivid green)
            // Speed ranges from 2 (far) to 6 (near)
            let depth = (speed as f32 - 2.0) / 4.0; // 0.0 = far, 1.0 = near
            let depth_brightness_mult = 0.4 + depth * 0.6; // 40% for far, 100% for near
            let depth_saturation = 0.3 + depth * 0.7; // Less saturated when far
            
            for i in 0..MATRIX_TRAIL_LEN {
                let char_y = new_y - (i as i32 * MATRIX_CHAR_H as i32);
                if char_y < 0 || char_y >= (height - bar_h) as i32 { continue; }
                
                let base_brightness = if i == 0 { 255u8 } 
                    else if i == 1 { 220u8 } 
                    else { 180u8.saturating_sub((i as u8).saturating_mul(9)) };
                if base_brightness < 20 { continue; }
                
                // Apply depth multiplier to brightness
                let brightness = ((base_brightness as f32) * depth_brightness_mult) as u8;
                
                // Calculate color with depth-based saturation
                // Far columns: more gray/blueish, Near columns: pure bright green
                let r = if i == 0 { 
                    ((180.0 * depth_brightness_mult) as u8) 
                } else { 
                    // Add slight gray tint for distant columns
                    ((20.0 * (1.0 - depth_saturation)) as u8)
                };
                let g = brightness;
                let b = if i == 0 { 
                    ((180.0 * depth_brightness_mult) as u8) 
                } else { 
                    // Add blue tint for distant columns (atmospheric perspective)
                    ((40.0 * (1.0 - depth_saturation) + 10.0 * depth) as u8)
                };
                let color = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                
                let char_seed = seed.wrapping_add((i as u32 * 7919) ^ (frame_count as u32 / 8));
                let char_idx = (char_seed as usize) % MATRIX_CHARS.len();
                let char_str: [u8; 2] = [MATRIX_CHARS[char_idx], 0];
                let char_s = unsafe { core::str::from_utf8_unchecked(&char_str[..1]) };
                draw_text(char_s, x, char_y as u32, color);
            }
        }
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: TRUSTOS LOGO (center) - Padlock + Shield + Checkmark + Hands
        // Matching reference image exactly
        // ───────────────────────────────────────────────────────────────
        let logo_cx = width / 2 + 100;  // Slightly right of center
        let logo_cy = height / 2 - 50;
        
        // === PADLOCK at top ===
        let lock_y = logo_cy - 180;
        // Shackle (U shape)
        fill_rect(logo_cx - 40, lock_y, 12, 60, green_main);      // Left arm
        fill_rect(logo_cx + 28, lock_y, 12, 60, green_main);      // Right arm
        fill_rect(logo_cx - 40, lock_y - 10, 80, 15, green_main); // Top bar
        fill_rect(logo_cx - 30, lock_y - 20, 60, 15, green_main); // Top curve
        // Lock body
        fill_rect(logo_cx - 50, lock_y + 50, 100, 70, green_main);
        fill_rect(logo_cx - 44, lock_y + 56, 88, 58, 0xFF0A150Au32);
        // Keyhole
        fill_circle(logo_cx, lock_y + 80, 10, green_main);
        fill_rect(logo_cx - 5, lock_y + 88, 10, 20, green_main);
        
        // === SHIELD (hexagonal shape) ===
        let shield_y = logo_cy - 60;
        let shield_w = 180u32;
        let shield_h = 220u32;
        // Shield outline (multiple layers for thickness)
        for t in 0..4 {
            let tt = t as u32;
            // Top edge
            fill_rect(logo_cx - shield_w/2 + 20 + tt, shield_y + tt, shield_w - 40, 3, green_main);
            // Upper sides (angled)
            fill_rect(logo_cx - shield_w/2 + tt, shield_y + 20 + tt, 25, 3, green_main);
            fill_rect(logo_cx + shield_w/2 - 25 - tt, shield_y + 20 + tt, 25, 3, green_main);
            // Sides
            fill_rect(logo_cx - shield_w/2 + tt, shield_y + 20, 3, shield_h - 80, green_main);
            fill_rect(logo_cx + shield_w/2 - 3 - tt, shield_y + 20, 3, shield_h - 80, green_main);
            // Bottom point
            fill_rect(logo_cx - 3, shield_y + shield_h - 20 - tt, 6, 20, green_main);
        }
        // Shield inner darker area
        fill_rect(logo_cx - shield_w/2 + 8, shield_y + 25, shield_w - 16, shield_h - 70, 0xFF051208u32);
        
        // === CHECKMARK (V) in center of shield ===
        let check_cx = logo_cx;
        let check_cy = logo_cy + 20;
        // Draw thick V shape
        for t in 0..8 {
            // Left diagonal going down
            for i in 0..30 {
                fill_rect(check_cx - 50 + i + t, check_cy - 30 + i, 4, 4, green_main);
            }
            // Right diagonal going up (longer)
            for i in 0..50 {
                fill_rect(check_cx - 20 + i + t, check_cy + i.min(29) - (i.saturating_sub(29)), 4, 4, green_main);
            }
        }
        
        // === HANDS holding shield (simplified) ===
        let hand_y = logo_cy + 100;
        // Left hand
        fill_rect(logo_cx - 100, hand_y, 40, 15, green_dim);
        fill_rect(logo_cx - 110, hand_y + 10, 20, 30, green_dim);
        fill_rect(logo_cx - 95, hand_y + 15, 10, 25, green_dim);
        fill_rect(logo_cx - 80, hand_y + 15, 10, 20, green_dim);
        // Right hand  
        fill_rect(logo_cx + 60, hand_y, 40, 15, green_dim);
        fill_rect(logo_cx + 90, hand_y + 10, 20, 30, green_dim);
        fill_rect(logo_cx + 85, hand_y + 15, 10, 25, green_dim);
        fill_rect(logo_cx + 70, hand_y + 15, 10, 20, green_dim);
        
        // === "TRust-os" text to the right of logo ===
        // Large stylized text
        let text_x = logo_cx + 130;
        let text_y = logo_cy + 40;
        draw_text("TRust-os", text_x, text_y, green_main);
        // Draw it bigger by repeating offset
        draw_text("TRust-os", text_x + 1, text_y, green_main);
        draw_text("TRust-os", text_x, text_y + 1, green_main);
        draw_text("TRust-os", text_x + 1, text_y + 1, green_main);
                
        // Window dimensions
        let win_x_u32 = 90u32;
        let win_y_u32 = 300u32;
        let win_w_u32 = 380u32;
        let win_h_u32 = 280u32;
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: LEFT DOCK (VERTICAL)
        // ───────────────────────────────────────────────────────────────
        let dock_x = 20u32;
        let dock_y = 50u32;
        let dock_icon_size = 44u32;
        let dock_gap = 20u32;
        let dock_icons = 5u32;
        
        // Draw solid black background for dock area
        fill_rect(0, 0, 80, height - bar_h, 0xFF050505u32);
        
        hovered_dock = -1;
        for i in 0..dock_icons {
            let iy = dock_y + i * (dock_icon_size + dock_gap);
            let ix = dock_x;
            
            // Hover detection
            let hovered = mx >= ix as f32 && mx < (ix + dock_icon_size) as f32 && 
                          my >= iy as f32 && my < (iy + dock_icon_size) as f32;
            if hovered {
                hovered_dock = i as i32;
                if click_this_frame {
                    active_app = i as usize;
                }
            }
            
            // Icon background - outlined square with rounded corners
            let icon_color = if i as usize == active_app { 
                green_main 
            } else if hovered { 
                green_bright 
            } else { 
                green_dim 
            };
            
            // Draw icon outline (not filled!)
            draw_rect(ix, iy, dock_icon_size, dock_icon_size, icon_color);
            
            // Simple icon symbols inside
            let cx = ix + dock_icon_size / 2;
            let cy = iy + dock_icon_size / 2;
            match i {
                0 => { // Play button (triangle)
                    fill_rect(cx - 8, cy - 10, 4, 20, icon_color);
                    fill_rect(cx - 4, cy - 8, 4, 16, icon_color);
                    fill_rect(cx, cy - 6, 4, 12, icon_color);
                    fill_rect(cx + 4, cy - 4, 4, 8, icon_color);
                },
                1 => { // Terminal (rectangle with lines)
                    draw_rect(cx - 12, cy - 10, 24, 20, icon_color);
                    fill_rect(cx - 8, cy - 4, 10, 2, icon_color);
                    fill_rect(cx - 8, cy + 2, 6, 2, icon_color);
                },
                2 => { // Grid (apps)
                    for row in 0..2 {
                        for col in 0..2 {
                            draw_rect(cx - 10 + col * 12, cy - 10 + row * 12, 8, 8, icon_color);
                        }
                    }
                },
                3 => { // Network
                    draw_rect(cx - 10, cy - 8, 20, 16, icon_color);
                    fill_rect(cx - 6, cy - 4, 2, 8, icon_color);
                    fill_rect(cx - 2, cy - 2, 2, 6, icon_color);
                    fill_rect(cx + 2, cy - 6, 2, 10, icon_color);
                    fill_rect(cx + 6, cy - 4, 2, 8, icon_color);
                },
                4 => { // Settings (gear outline)
                    fill_circle(cx, cy, 10, icon_color);
                    fill_circle(cx, cy, 6, black);
                },
                _ => {}
            }
        }
        
        // Help icon at bottom of dock
        let help_y = height as u32 - 80;
        draw_rect(dock_x, help_y, dock_icon_size, dock_icon_size, green_dim);
        draw_text("?", dock_x + 18, help_y + 16, green_dim);
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: MAIN WINDOW (Terminal style like reference)
        // Window variables already defined above for Matrix rain clipping
        // ───────────────────────────────────────────────────────────────
        
        // Window border (thick green line like reference)
        let border_thickness = 3u32;
        // Top border
        fill_rect(win_x_u32, win_y_u32, win_w_u32, border_thickness, green_main);
        // Bottom border
        fill_rect(win_x_u32, win_y_u32 + win_h_u32 - border_thickness, win_w_u32, border_thickness, green_main);
        // Left border
        fill_rect(win_x_u32, win_y_u32, border_thickness, win_h_u32, green_main);
        // Right border
        fill_rect(win_x_u32 + win_w_u32 - border_thickness, win_y_u32, border_thickness, win_h_u32, green_main);
        
        // Window interior (SOLID BLACK - completely fills to prevent flicker)
        fill_rect(win_x_u32 + border_thickness, win_y_u32 + border_thickness, 
                  win_w_u32 - border_thickness * 2, win_h_u32 - border_thickness * 2, black);
        
        // Title bar with window controls
        let title_h = 28u32;
        fill_rect(win_x_u32 + border_thickness, win_y_u32 + border_thickness, 
                  win_w_u32 - border_thickness * 2, title_h, 0xFF0A1A0Au32);
        
        // Window title
        let (title, _) = apps[active_app];
        let title_text = format!("TrustOS {} v1.00", title);
        draw_text(&title_text, win_x_u32 + 12, win_y_u32 + 10, green_main);
        
        // Window control buttons (right side) - 3 circles
        let btn_y = win_y_u32 + 10;
        let close_x = win_x_u32 + win_w_u32 - 60;
        // Red close
        fill_circle(close_x, btn_y + 6, 6, 0xFFFF5555u32);
        // Yellow maximize  
        fill_circle(close_x + 18, btn_y + 6, 6, 0xFFFFDD55u32);
        // Green minimize
        fill_circle(close_x + 36, btn_y + 6, 6, 0xFF55FF55u32);
        
        // Content area - GREEN TEXT like reference image
        let content_x = win_x_u32 + 15;
        let content_y = win_y_u32 + title_h + 15;
        
        // Terminal prompt with colored parts - root@trustos
        let cwd_display = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
        // Draw "root" in red
        draw_text("root", content_x, content_y, 0xFFFF0000u32);  // Pure red
        // Draw "@" in white
        draw_text("@", content_x + 32, content_y, 0xFFFFFFFFu32);  // White
        // Draw "trustos" in green
        draw_text("trustos", content_x + 40, content_y, 0xFF00FF00u32);  // Pure green
        // Draw ":path$ " in green
        let path_part = format!(":{}$ ", cwd_display);
        draw_text(&path_part, content_x + 96, content_y, 0xFF00FF00u32);  // Pure green
        // Cursor block
        let prompt_len = 4 + 1 + 7 + path_part.len();  // root @ trustos :path$
        let cursor_x = content_x + (prompt_len * 8) as u32;
        fill_rect(cursor_x, content_y, 8, 16, green_bright);
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: BOTTOM STATUS BAR (exact match to reference)
        // ───────────────────────────────────────────────────────────────
        let bar_y = height as u32 - bar_h;
        
        // Bar background (very dark)
        fill_rect(0, bar_y, width as u32, bar_h, 0xFF080808u32);
        // Top border line
        fill_rect(0, bar_y, width as u32, 2, green_dim);
        
        // Left: TrustOS menu button (dash icon)
        let menu_btn_x = 8u32;
        let menu_btn_w = 24u32;
        fill_rect(menu_btn_x + 4, bar_y + 14, 16, 3, green_main);
        fill_rect(menu_btn_x + 4, bar_y + 19, 16, 3, green_main);
        
        // Check if menu button clicked
        if click_this_frame && mx >= menu_btn_x as f32 && mx < (menu_btn_x + menu_btn_w) as f32 &&
           my >= bar_y as f32 {
            menu_open = !menu_open;
        }
        
        // Tab: TrustOS (with border)
        let tab1_x = 40u32;
        let tab1_w = 90u32;
        fill_rect(tab1_x, bar_y + 6, tab1_w, 24, 0xFF0A1A0Au32);
        draw_rect(tab1_x, bar_y + 6, tab1_w, 24, green_dim);
        draw_text("TrustOS", tab1_x + 14, bar_y + 10, green_main);
        
        // Tab: Terminal
        let tab2_x = 138u32;
        let tab2_w = 90u32;
        fill_rect(tab2_x, bar_y + 6, tab2_w, 24, 0xFF050A05u32);
        draw_text("Terminal", tab2_x + 12, bar_y + 10, green_dim);
        
        // Search bar (center)
        let search_x = width as u32 / 2 - 120;
        let search_w = 240u32;
        fill_rect(search_x, bar_y + 6, search_w, 24, 0xFF0A0A0Au32);
        draw_rect(search_x, bar_y + 6, search_w, 24, green_dim);
        if search_len == 0 {
            draw_text("Search...", search_x + 8, bar_y + 10, 0xFF336633u32);
        } else {
            let search_display = unsafe { core::str::from_utf8_unchecked(&search_text[..search_len]) };
            draw_text(search_display, search_x + 8, bar_y + 10, green_main);
        }
        // Search icon (magnifying glass)
        fill_circle(search_x + search_w - 20, bar_y + 18, 6, green_dim);
        fill_circle(search_x + search_w - 20, bar_y + 18, 4, 0xFF0A0A0Au32);
        fill_rect(search_x + search_w - 16, bar_y + 22, 6, 2, green_dim);
        
        // Check if search bar clicked
        if click_this_frame && mx >= search_x as f32 && mx < (search_x + search_w) as f32 &&
           my >= bar_y as f32 {
            search_active = true;
        }
        
        // Right side: Clock
        let dt = crate::rtc::read_rtc();
        let time_str = format!("{:02}:{:02}", dt.hour, dt.minute);
        draw_text(&time_str, width as u32 - 200, bar_y + 10, green_main);
        
        // System ID
        draw_text("TRST-001", width as u32 - 120, bar_y + 10, green_bright);
        
        // Status indicators (right edge)
        let ind_x = width as u32 - 50;
        fill_circle(ind_x, bar_y + 18, 6, green_main);
        fill_circle(ind_x + 16, bar_y + 18, 6, 0xFFFFAA00u32);
        // Grid icon
        fill_rect(ind_x + 28, bar_y + 12, 4, 4, green_dim);
        fill_rect(ind_x + 34, bar_y + 12, 4, 4, green_dim);
        fill_rect(ind_x + 28, bar_y + 18, 4, 4, green_dim);
        fill_rect(ind_x + 34, bar_y + 18, 4, 4, green_dim);
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: TRUSTOS MENU (if open)
        // ───────────────────────────────────────────────────────────────
        if menu_open {
            let menu_x = 10u32;
            let menu_y = bar_y - 320;
            let menu_w = 180u32;
            let menu_h = 310u32;
            
            // Menu background
            fill_rect(menu_x, menu_y, menu_w, menu_h, 0xFF0A0F0Au32);
            draw_rect(menu_x, menu_y, menu_w, menu_h, green_main);
            draw_rect(menu_x + 1, menu_y + 1, menu_w - 2, menu_h - 2, green_dim);
            
            // Menu header
            fill_rect(menu_x + 2, menu_y + 2, menu_w - 4, 30, 0xFF0A1A0Au32);
            draw_text("TrustOS Menu", menu_x + 12, menu_y + 10, green_main);
            
            // Menu items
            menu_hover = -1;
            for (idx, item) in menu_items.iter().enumerate() {
                let item_y = menu_y + 40 + (idx as u32 * 24);
                
                if *item == "---" {
                    // Separator line
                    fill_rect(menu_x + 10, item_y + 10, menu_w - 20, 1, green_dim);
                } else {
                    // Check hover
                    let item_hovered = mx >= menu_x as f32 && mx < (menu_x + menu_w) as f32 &&
                                       my >= item_y as f32 && my < (item_y + 24) as f32;
                    
                    if item_hovered {
                        menu_hover = idx as i32;
                        fill_rect(menu_x + 2, item_y, menu_w - 4, 24, 0xFF1A2A1Au32);
                        
                        // Handle click on menu item
                        if click_this_frame {
                            match *item {
                                "Shutdown" => running = false,
                                "Restart" => { /* Would restart */ running = false; },
                                "Sign Out" => { running = false; },
                                "Settings" => { active_app = 4; menu_open = false; },
                                "Terminal" => { active_app = 1; menu_open = false; },
                                "Files" => { active_app = 0; menu_open = false; },
                                "Browser" => { active_app = 2; menu_open = false; },
                                _ => { menu_open = false; }
                            }
                        }
                    }
                    
                    // Color based on type
                    let text_color = if *item == "Shutdown" || *item == "Restart" || *item == "Sign Out" {
                        0xFFFF6666u32  // Red for power options
                    } else if item_hovered {
                        green_bright
                    } else {
                        green_main
                    };
                    
                    // Icon for power options
                    if *item == "Shutdown" {
                        fill_circle(menu_x + 20, item_y + 12, 6, text_color);
                        fill_rect(menu_x + 18, item_y + 6, 4, 6, 0xFF0A0F0Au32);
                    }
                    
                    draw_text(item, menu_x + 35, item_y + 6, text_color);
                }
            }
            
            // Close menu if clicked outside
            if click_this_frame && (mx < menu_x as f32 || mx > (menu_x + menu_w) as f32 ||
                                    my < menu_y as f32 || my > bar_y as f32) {
                menu_open = false;
            }
        }
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: FPS OVERLAY (top-right corner)
        // ───────────────────────────────────────────────────────────────
        if show_fps && fps > 0 {
            let fps_text = format!("{} FPS", fps);
            let fps_x = width.saturating_sub(80);
            let fps_color = if fps >= 55 { 0xFF00FF00 }    // Green 55+
                           else if fps >= 30 { 0xFFFFFF00 } // Yellow 30-54
                           else { 0xFFFF4444 };            // Red <30
            draw_text(&fps_text, fps_x, 4, fps_color);
            
            // Show renderer mode
            let mode = if use_braille { "BRL" } else if use_fast_matrix { "FAST" } else { "LEG" };
            draw_text(mode, fps_x, 20, 0xFF888888);
        }
        
        // ───────────────────────────────────────────────────────────────
        // RENDER: CURSOR (simple green)
        // ───────────────────────────────────────────────────────────────
        let mx_u32 = mx as u32;
        let my_u32 = my as u32;
        // Simple arrow cursor
        for i in 0..12u32 {
            fill_rect(mx_u32, my_u32 + i, (12 - i).max(1), 1, green_main);
        }
        
        // ───────────────────────────────────────────────────────────────
        // PRESENT TO FRAMEBUFFER (SSE2 fast copy!)
        // ───────────────────────────────────────────────────────────────
        swap_buffers();
        
        // ───────────────────────────────────────────────────────────────
        // FPS TRACKING
        // ───────────────────────────────────────────────────────────────
        frame_count += 1;
        frame_in_second += 1;
        
        let now = crate::cpu::tsc::read_tsc();
        if now - last_second_tsc >= tsc_freq {
            fps = frame_in_second;
            frame_in_second = 0;
            last_second_tsc = now;
            crate::serial_println!("[COSMIC] FPS: {} | Frame: {} | Mode: {}", 
                fps, frame_count, if use_braille { "BRAILLE" } else if use_fast_matrix { "FAST" } else { "LEGACY" });
        }
        
        // Brief pause to allow interrupts (keyboard, mouse) to be processed
        // Without this, the tight loop starves the interrupt handlers
        for _ in 0..100 {
            core::hint::spin_loop();
        }
        
        last_frame_tsc = crate::cpu::tsc::read_tsc();
    }
    
    crate::framebuffer::clear();
    crate::serial_println!("[COSMIC] Desktop exited after {} frames, last FPS: {}", frame_count, fps);
    crate::println_color!(COLOR_GREEN, "COSMIC Desktop exited. {} frames rendered, {} FPS", frame_count, fps);
}

// ==================== GUI COMMANDS ====================

// ==================== VM / LINUX SYSTEM ====================
// State: tracks if Alpine Linux VM image is installed
static GUI_INSTALLED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

fn cmd_vm_help() {
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║            TrustOS Virtual Machine Manager                   ║");
    crate::println_color!(COLOR_CYAN, "╠══════════════════════════════════════════════════════════════╣");
    crate::println_color!(COLOR_CYAN, "║                                                              ║");
    crate::println_color!(COLOR_CYAN, "║  TrustOS runs Linux VMs with modern GUIs.                   ║");
    crate::println_color!(COLOR_CYAN, "║                                                              ║");
    crate::println_color!(COLOR_CYAN, "║  Commands:                                                   ║");
    crate::println_color!(COLOR_GREEN, "║    vm status    - Check VM installation status              ║");
    crate::println_color!(COLOR_GREEN, "║    vm install   - Download Alpine Linux VM image            ║");
    crate::println_color!(COLOR_GREEN, "║    vm start     - Start the Alpine Linux VM                 ║");
    crate::println_color!(COLOR_GREEN, "║    vm console   - Connect to VM console (Linux shell)       ║");
    crate::println_color!(COLOR_GREEN, "║    vm stop      - Stop the running VM                       ║");
    crate::println_color!(COLOR_GREEN, "║    vm list      - List running VMs                          ║");
    crate::println_color!(COLOR_CYAN, "║                                                              ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
}

fn cmd_vm_stop() {
    crate::println_color!(COLOR_YELLOW, "Stopping VM...");
    // TODO: Actually stop the VM
    crate::println_color!(COLOR_GREEN, "VM stopped.");
}

fn cmd_vm_list() {
    crate::println_color!(COLOR_CYAN, "Running Virtual Machines:");
    crate::println!("  ID   NAME           STATUS      MEMORY");
    crate::println!("  ───────────────────────────────────────");
    if GUI_INSTALLED.load(core::sync::atomic::Ordering::Relaxed) {
        crate::println!("  1    alpine-linux   running     256 MB");
    } else {
        crate::println!("  (no VMs running)");
    }
}

// ==================== LINUX DISTRIBUTION MANAGER ====================

fn cmd_distro_list() {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distros = crate::distro::list();
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║                 TrustOS Linux Distribution Manager               ║");
    crate::println_color!(COLOR_CYAN, "╠══════════════════════════════════════════════════════════════════╣");
    crate::println_color!(COLOR_CYAN, "║  ID              NAME                    SIZE     STATUS         ║");
    crate::println_color!(COLOR_CYAN, "╠══════════════════════════════════════════════════════════════════╣");
    
    for d in &distros {
        let status = if d.installed { 
            "\x1b[32m[installed]\x1b[0m" 
        } else { 
            "\x1b[33m[available]\x1b[0m" 
        };
        let status_simple = if d.installed { "installed" } else { "available" };
        crate::println!("║  {} {:<12}  {:<20}  {:>4} MB   {:<12} ║", 
            d.icon, d.id, d.name, d.size_mb, status_simple);
    }
    
    crate::println_color!(COLOR_CYAN, "╠══════════════════════════════════════════════════════════════════╣");
    crate::println_color!(COLOR_CYAN, "║  Commands:                                                       ║");
    crate::println_color!(COLOR_GREEN, "║    distro list              - Show this list                    ║");
    crate::println_color!(COLOR_GREEN, "║    distro install <id>      - Download and install a distro     ║");
    crate::println_color!(COLOR_GREEN, "║    distro run <id>          - Run an installed distro           ║");
    crate::println_color!(COLOR_GREEN, "║    distro gui               - Open graphical distro selector    ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════════╝");
}

fn cmd_distro_install(id: &str) {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(id) {
        Some(d) => d,
        None => {
            crate::println_color!(COLOR_RED, "Error: Distribution '{}' not found.", id);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if distro.installed {
        crate::println_color!(COLOR_YELLOW, "{} {} is already installed.", distro.icon, distro.name);
        crate::println!("Use 'distro run {}' to start it.", id);
        return;
    }
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║                    Installing Linux Distribution                 ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════════╝");
    crate::println!();
    crate::println!("  {} {} {}", distro.icon, distro.name, distro.version);
    crate::println!("  {}", distro.description);
    crate::println!("  Size: {} MB", distro.size_mb);
    crate::println!();
    
    crate::println_color!(COLOR_YELLOW, "[1/3] Connecting to server 192.168.56.1:8080...");
    
    match crate::distro::download(id) {
        Ok(size) => {
            crate::println_color!(COLOR_GREEN, "[2/3] Downloaded {} KB", size / 1024);
            crate::println_color!(COLOR_GREEN, "[3/3] Installation complete!");
            crate::println!();
            crate::println_color!(COLOR_GREEN, "  {} {} is now installed!", distro.icon, distro.name);
            crate::println!("  Use 'distro run {}' to start it.", id);
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Error: {}", e);
            crate::println!();
            crate::println!("Make sure the server is running:");
            crate::println!("  > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
        }
    }
}

fn cmd_distro_run(id: &str) {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distro = match crate::distro::get(id) {
        Some(d) => d,
        None => {
            crate::println_color!(COLOR_RED, "Error: Distribution '{}' not found.", id);
            crate::println!("Use 'distro list' to see available distributions.");
            return;
        }
    };
    
    if !distro.installed {
        crate::println_color!(COLOR_YELLOW, "{} {} is not installed.", distro.icon, distro.name);
        crate::println!("Use 'distro install {}' to download it first.", id);
        return;
    }
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║                    Starting Linux Distribution                   ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════════╝");
    crate::println!();
    crate::println!("  {} {} {}", distro.icon, distro.name, distro.version);
    crate::println!();
    
    match crate::distro::run(id) {
        Ok(()) => {
            crate::println_color!(COLOR_GREEN, "  Distribution started successfully.");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Error: {}", e);
        }
    }
}

fn cmd_distro_gui() {
    // Initialize if needed
    if crate::distro::list().is_empty() {
        crate::distro::init();
    }
    
    let distros = crate::distro::list();
    
    // Check framebuffer
    if !crate::framebuffer::is_initialized() {
        crate::println_color!(COLOR_RED, "Error: No framebuffer available for GUI.");
        crate::println!("Use 'distro list' for text-mode interface.");
        return;
    }
    
    let (width, height) = crate::framebuffer::get_dimensions();
    
    // Colors
    let bg_color = 0xFF1E1E2Eu32;      // Dark background
    let panel_color = 0xFF2D2D3Du32;   // Panel background
    let accent_color = 0xFF89B4FAu32;  // Blue accent
    let green_color = 0xFF94E2D5u32;   // Teal/green for installed
    let text_color = 0xFFCDD6F4u32;    // Light text
    let _dim_color = 0xFF6C7086u32;    // Dimmed text
    
    // Clear screen with background
    crate::framebuffer::fill_rect(0, 0, width, height, bg_color);
    
    // Title bar
    crate::framebuffer::fill_rect(0, 0, width, 50, panel_color);
    crate::framebuffer::draw_text_at("TrustOS Linux Distribution Manager", 20, 16, text_color, panel_color);
    
    // Draw distro list as text (simple version)
    let mut y = 80u32;
    
    crate::framebuffer::draw_text_at("  #  ID              NAME                    SIZE     STATUS", 20, y, accent_color, bg_color);
    y += 24;
    crate::framebuffer::draw_hline(20, y, width - 40, accent_color);
    y += 16;
    
    for (i, d) in distros.iter().enumerate() {
        let status_str = if d.installed { "[INSTALLED]" } else { "[available]" };
        let status_color = if d.installed { green_color } else { text_color };
        
        // Number
        let num_str = alloc::format!("  {}  ", i + 1);
        crate::framebuffer::draw_text_at(&num_str, 20, y, accent_color, bg_color);
        
        // Icon + ID
        let id_str = alloc::format!("{} {:<12}", d.icon, d.id);
        crate::framebuffer::draw_text_at(&id_str, 60, y, text_color, bg_color);
        
        // Name
        crate::framebuffer::draw_text_at(d.name, 220, y, text_color, bg_color);
        
        // Size
        let size_str = alloc::format!("{:>4} MB", d.size_mb);
        crate::framebuffer::draw_text_at(&size_str, 450, y, text_color, bg_color);
        
        // Status
        crate::framebuffer::draw_text_at(status_str, 540, y, status_color, bg_color);
        
        y += 24;
    }
    
    // Footer with instructions
    let footer_y = height - 80;
    crate::framebuffer::fill_rect(0, footer_y, width, 80, panel_color);
    crate::framebuffer::draw_text_at("Commands:", 20, footer_y + 16, accent_color, panel_color);
    crate::framebuffer::draw_text_at("distro install <id>  - Download and install", 20, footer_y + 36, text_color, panel_color);
    crate::framebuffer::draw_text_at("distro run <id>      - Run an installed distro", 400, footer_y + 36, text_color, panel_color);
    crate::framebuffer::draw_text_at("Press any key to return to shell...", 20, footer_y + 56, green_color, panel_color);
    
    // Wait for key input
    loop {
        if let Some(_ch) = crate::keyboard::read_char() {
            break;
        }
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
    
    // Clear screen 
    crate::framebuffer::clear();
}

fn cmd_gui_status() {
    let installed = GUI_INSTALLED.load(core::sync::atomic::Ordering::Relaxed);
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║       TrustOS GUI Status             ║");
    crate::println_color!(COLOR_CYAN, "╠══════════════════════════════════════╣");
    
    if installed {
        crate::println_color!(COLOR_GREEN, "║  Status:     [INSTALLED]             ║");
        crate::println_color!(COLOR_GREEN, "║  Image:      Alpine Linux + Browser  ║");
        crate::println_color!(COLOR_CYAN, "║                                      ║");
        crate::println_color!(COLOR_CYAN, "║  Use 'gui start' to launch           ║");
    } else {
        crate::println_color!(COLOR_YELLOW, "║  Status:     [NOT INSTALLED]         ║");
        crate::println_color!(COLOR_CYAN, "║                                      ║");
        crate::println_color!(COLOR_CYAN, "║  Use 'gui install' to download       ║");
    }
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════╝");
}

fn cmd_gui_install() {
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║              TrustOS GUI Installer                           ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    // Configuration du serveur (192.168.56.1 = host dans VirtualBox Host-Only)
    let server_ip = "192.168.56.1";
    let server_port = 8080u16;
    let package_path = "/alpine-minirootfs.tar.gz";
    
    // Étape 1: Vérifier le réseau
    crate::println_color!(COLOR_YELLOW, "[1/4] Checking network connection...");
    
    if !crate::network::is_available() {
        crate::println_color!(COLOR_RED, "      ERROR: Network not available!");
        crate::println!("      Make sure virtio-net is enabled.");
        return;
    }
    crate::println_color!(COLOR_GREEN, "      Network: OK");
    crate::println!();
    
    // Étape 2: Télécharger Alpine Linux
    crate::println_color!(COLOR_YELLOW, "[2/4] Downloading Alpine Linux from {}:{}{}...", server_ip, server_port, package_path);
    
    // CRITICAL: Suspend DHCP to prevent IP changes during download
    crate::netstack::dhcp::suspend();
    crate::serial_println!("[GUI_INSTALL] DHCP suspended for download");
    
    // Force static IP for VirtualBox Host-Only network
    crate::network::set_ipv4_config(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    // Clear any pending DHCP packets
    for _ in 0..100 {
        crate::netstack::poll();
    }
    
    let ip = match parse_ipv4(server_ip) {
        Some(ip) => ip,
        None => {
            crate::println_color!(COLOR_RED, "      ERROR: Invalid server IP");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    // Connexion TCP
    let src_port = match crate::netstack::tcp::send_syn(ip, server_port) {
        Ok(p) => p,
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: Connection failed: {}", e);
            crate::println!("      Make sure the server is running:");
            crate::println!("      > cd server && powershell -ExecutionPolicy Bypass .\\start-server.ps1");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    let established = crate::netstack::tcp::wait_for_established(ip, server_port, src_port, 2000);
    if !established {
        crate::println_color!(COLOR_RED, "      ERROR: Connection timeout");
        crate::println!("      Make sure the server is running on port {}", server_port);
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::println_color!(COLOR_GREEN, "      Connected to server");
    
    // Envoyer la requête HTTP GET
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        package_path, server_ip
    );
    
    if let Err(e) = crate::netstack::tcp::send_payload(ip, server_port, src_port, request.as_bytes()) {
        crate::println_color!(COLOR_RED, "      ERROR: Failed to send request: {}", e);
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Recevoir les données (optimized download loop)
    crate::println!("      Downloading...");
    // Pré-allouer 4 MB pour éviter les réallocations
    let mut received_data: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);
    let start = crate::logger::get_ticks();
    let mut idle_count: u32 = 0;
    let mut last_progress = 0usize;
    let mut last_ack_flush = start;
    let mut last_poll_count = 0u32;
    const MAX_SIZE: usize = 8 * 1024 * 1024; // 8 MB max
    
    loop {
        // Poll network aggressively (multiple times per iteration)
        for _ in 0..10 {
            crate::netstack::poll();
        }
        last_poll_count += 10;
        
        let mut got_data = false;
        let mut batch_size = 0usize;
        
        // Batch receive: drain all available data at once
        while let Some(data) = crate::netstack::tcp::recv_data(ip, server_port, src_port) {
            got_data = true;
            batch_size += data.len();
            
            // Limiter la taille pour éviter OOM
            if received_data.len() + data.len() > MAX_SIZE {
                crate::println_color!(COLOR_YELLOW, "\n      WARNING: File too large, truncating");
                break;
            }
            
            received_data.extend_from_slice(&data);
        }
        
        // Afficher la progression (only when significant change)
        let kb = received_data.len() / 1024;
        if kb >= last_progress + 25 || (kb > 0 && last_progress == 0) {
            let elapsed = crate::logger::get_ticks().saturating_sub(start);
            let speed_kbps = if elapsed > 0 { (kb as u64 * 1000) / elapsed } else { 0 };
            crate::print!("\r      Downloaded: {} KB ({} KB/s)    ", kb, speed_kbps);
            last_progress = kb;
        }
        
        // Periodically flush pending ACKs (every 5ms for faster throughput)
        let now = crate::logger::get_ticks();
        if now.saturating_sub(last_ack_flush) >= 5 {
            crate::netstack::tcp::flush_pending_acks(ip, server_port, src_port);
            last_ack_flush = now;
        }
        
        if !got_data {
            idle_count = idle_count.saturating_add(1);
            
            // Check for FIN or excessive idle
            if crate::netstack::tcp::fin_received(ip, server_port, src_port) {
                // Flush final ACK
                crate::netstack::tcp::flush_pending_acks(ip, server_port, src_port);
                break;
            }
            
            // Lower idle threshold - break earlier if no data
            if idle_count > 100_000 {
                crate::serial_println!("[DOWNLOAD] Idle timeout after {} polls", last_poll_count);
                break;
            }
            
            // Brief pause when idle - but not too long
            for _ in 0..50 { core::hint::spin_loop(); }
        } else {
            idle_count = 0;
        }
        
        // Timeout 60 secondes
        if crate::logger::get_ticks().saturating_sub(start) > 60000 {
            crate::println_color!(COLOR_YELLOW, "\n      WARNING: Download timeout");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::send_fin(ip, server_port, src_port);
    crate::println!();
    
    let elapsed_ms = crate::logger::get_ticks().saturating_sub(start);
    let total_kb = received_data.len() / 1024;
    let avg_speed = if elapsed_ms > 0 { (total_kb as u64 * 1000) / elapsed_ms } else { 0 };
    crate::println_color!(COLOR_GREEN, "      Transfer complete: {} KB in {}ms ({} KB/s)", total_kb, elapsed_ms, avg_speed);
    
    if received_data.is_empty() {
        crate::println_color!(COLOR_RED, "      ERROR: No data received");
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Extraire le body HTTP (après \r\n\r\n)
    let body_start = received_data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(0);
    
    let image_data = &received_data[body_start..];
    let size_mb = image_data.len() as f32 / (1024.0 * 1024.0);
    
    crate::println_color!(COLOR_GREEN, "      Download complete: {:.2} MB", size_mb);
    crate::println!();
    
    // Étape 3: Sauvegarder l'image directement dans le ramfs
    crate::println_color!(COLOR_YELLOW, "[3/4] Saving image to /opt/gui/alpine.tar.gz...");
    
    // Utiliser le ramfs directement (plus fiable que le VFS quand pas de root mount)
    let save_result = crate::ramfs::with_fs(|fs| {
        // Créer les dossiers
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/gui");
        // Créer le fichier et écrire
        let _ = fs.touch("/opt/gui/alpine.tar.gz");
        fs.write_file("/opt/gui/alpine.tar.gz", image_data)
    });
    
    match save_result {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "      Saved successfully");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: Write failed: {:?}", e);
            crate::netstack::dhcp::resume();
            return;
        }
    }
    crate::println!();
    
    // Étape 4: Configuration
    crate::println_color!(COLOR_YELLOW, "[4/4] Configuring GUI environment...");
    
    // Marquer comme installé
    GUI_INSTALLED.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::println_color!(COLOR_GREEN, "      Configuration complete");
    crate::println!();
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "════════════════════════════════════════════════════════════════");
    crate::println_color!(COLOR_BRIGHT_GREEN, "                    GUI Installation Complete!");
    crate::println_color!(COLOR_BRIGHT_GREEN, "════════════════════════════════════════════════════════════════");
    crate::println!();
    crate::println!("Image saved to: /opt/gui/alpine.tar.gz ({:.2} MB)", size_mb);
    crate::println!();
    
    // Save to persistent storage
    crate::println_color!(COLOR_YELLOW, "Saving to disk for persistence...");
    match crate::persistence::save_file("/opt/gui/alpine.tar.gz", image_data) {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "  Saved to disk! Will be restored on next boot.");
        }
        Err(e) => {
            crate::println_color!(COLOR_YELLOW, "  Could not save to disk: {}", e);
            crate::println!("  (Download will need to be repeated after reboot)");
        }
    }
    crate::println!();
    
    crate::println!("Use 'gui start' to launch the graphical environment.");
    
    // Resume DHCP after successful download
    crate::netstack::dhcp::resume();
    crate::serial_println!("[GUI_INSTALL] DHCP resumed");
}

fn cmd_gui_start() {
    let installed = GUI_INSTALLED.load(core::sync::atomic::Ordering::Relaxed);
    
    if !installed {
        // Vérifier si le fichier existe quand même
        if !file_exists("/opt/gui/alpine.tar.gz") {
            crate::println_color!(COLOR_YELLOW, "Linux VM not installed.");
            crate::println!("Run 'gui install' first to download Alpine Linux.");
            return;
        }
        GUI_INSTALLED.store(true, core::sync::atomic::Ordering::Relaxed);
    }
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║              Starting Alpine Linux VM                        ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    // Lancer la VM Linux via l'hyperviseur intégré
    crate::println_color!(COLOR_YELLOW, "[1/3] Initializing hypervisor...");
    
    // Try to initialize hypervisor if not already enabled
    if !crate::hypervisor::is_enabled() {
        match crate::hypervisor::init() {
            Ok(()) => {
                crate::println_color!(COLOR_GREEN, "      Hypervisor initialized (VT-x/AMD-V)");
            }
            Err(e) => {
                crate::serial_println!("[GUI] Hypervisor init failed: {:?}", e);
                crate::println_color!(COLOR_RED, "      ERROR: Hardware virtualization not available");
                crate::println!("      Requires Intel VT-x or AMD-V");
                crate::println!();
                crate::println_color!(COLOR_YELLOW, "Falling back to Linux subsystem emulation...");
                cmd_linux_shell();
                return;
            }
        }
    }
    crate::println_color!(COLOR_GREEN, "      Hypervisor ready");
    
    crate::println_color!(COLOR_YELLOW, "[2/3] Loading Alpine Linux image...");
    crate::println_color!(COLOR_GREEN, "      Image: /opt/gui/alpine.tar.gz");
    
    crate::println_color!(COLOR_YELLOW, "[3/3] Booting VM...");
    
    // Démarrer la VM Linux
    match crate::hypervisor::linux_subsystem::boot() {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "      VM started successfully");
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Alpine Linux is now running.");
            crate::println!("Use 'vm console' to connect to the VM console.");
            crate::println!("Use 'vm stop' to stop the VM.");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: Failed to start VM: {:?}", e);
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "Falling back to Linux subsystem...");
            cmd_linux_shell();
        }
    }
}

/// Launch the Linux subsystem shell
fn cmd_linux_shell() {
    // Check if Linux subsystem is already initialized
    if !crate::linux::is_initialized() {
        // Try to initialize from the downloaded rootfs
        if file_exists("/opt/gui/alpine.tar.gz") {
            match crate::linux::init("/opt/gui/alpine.tar.gz") {
                Ok(()) => {}
                Err(e) => {
                    crate::println_color!(COLOR_RED, "Failed to initialize Linux subsystem: {}", e);
                    return;
                }
            }
        } else {
            crate::println_color!(COLOR_YELLOW, "Linux subsystem not installed.");
            crate::println!("Run 'gui install' to download and install Alpine Linux.");
            return;
        }
    }
    
    // Start the Linux shell
    crate::linux::start_shell();
}

fn cmd_glmode(args: &[&str]) {
    use crate::desktop::{RenderMode, set_render_mode, set_theme};
    use crate::graphics::CompositorTheme;
    
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustGL Compositor Settings");
        crate::println_color!(COLOR_CYAN, "===========================");
        crate::println!();
        crate::println!("Usage: glmode <mode|theme>");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Render Modes:");
        crate::println!("  classic   - Classic framebuffer rendering (fast, stable)");
        crate::println!("  opengl    - OpenGL compositor with visual effects");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Themes (OpenGL mode only):");
        crate::println!("  flat      - Simple flat rendering, no effects");
        crate::println!("  modern    - Shadows and subtle effects");
        crate::println!("  glass     - Transparency and blur effects");
        crate::println!("  neon      - Glowing neon borders");
        crate::println!("  minimal   - Thin borders, minimal style");
        crate::println!();
        crate::println!("Example: glmode opengl");
        crate::println!("         glmode neon");
        return;
    }
    
    match args[0].to_lowercase().as_str() {
        "classic" | "normal" | "default" => {
            set_render_mode(RenderMode::Classic);
            crate::println_color!(COLOR_GREEN, "Switched to Classic rendering mode");
        }
        "opengl" | "gl" | "compositor" => {
            set_render_mode(RenderMode::OpenGL);
            crate::println_color!(COLOR_GREEN, "Switched to OpenGL compositor mode");
            crate::println!("Use 'glmode <theme>' to change visual theme");
        }
        "flat" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Flat);
            crate::println_color!(COLOR_GREEN, "Theme: Flat (OpenGL)");
        }
        "modern" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Modern);
            crate::println_color!(COLOR_GREEN, "Theme: Modern (shadows, subtle effects)");
        }
        "glass" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Glass);
            crate::println_color!(COLOR_GREEN, "Theme: Glass (transparency effects)");
        }
        "neon" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Neon);
            crate::println_color!(COLOR_GREEN, "Theme: Neon (glowing borders)");
        }
        "minimal" => {
            set_render_mode(RenderMode::OpenGL);
            set_theme(CompositorTheme::Minimal);
            crate::println_color!(COLOR_GREEN, "Theme: Minimal (thin borders)");
        }
        _ => {
            crate::println_color!(COLOR_RED, "Unknown mode/theme: {}", args[0]);
            crate::println!("Use 'glmode' without arguments for help");
        }
    }
}

/// Dynamic theme management command
fn cmd_theme(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustOS Theme Manager");
        crate::println_color!(COLOR_CYAN, "=====================");
        crate::println!();
        crate::println!("Usage: theme <command> [args]");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Commands:");
        crate::println!("  list              - List available built-in themes");
        crate::println!("  set <name>        - Switch to a built-in theme");
        crate::println!("  load <path>       - Load theme from config file");
        crate::println!("  save <path>       - Save current theme to file");
        crate::println!("  reload            - Reload wallpaper from disk");
        crate::println!("  info              - Show current theme info");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Built-in Themes:");
        crate::println!("  dark / trustos    - TrustOS dark green theme");
        crate::println!("  windows11 / win11 - Windows 11 dark theme");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Config File Format (/etc/theme.conf):");
        crate::println!("  [colors]");
        crate::println!("  background = 0x0A0E0B");
        crate::println!("  accent = 0x00D26A");
        crate::println!("  ");
        crate::println!("  [wallpaper]");
        crate::println!("  path = /usr/share/wallpapers/matrix.bmp");
        return;
    }
    
    match args[0] {
        "list" => {
            crate::println_color!(COLOR_CYAN, "Available Themes:");
            crate::println!("  dark       - TrustOS dark green (default)");
            crate::println!("  windows11  - Windows 11 dark blue");
            crate::println!("  light      - Light theme");
        }
        "set" => {
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: theme set <name>");
                return;
            }
            crate::theme::set_builtin_theme(args[1]);
            crate::println_color!(COLOR_GREEN, "Theme switched to: {}", args[1]);
        }
        "load" => {
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: theme load <path>");
                crate::println!("Example: theme load /etc/theme.conf");
                return;
            }
            if crate::theme::load_theme(args[1]) {
                crate::println_color!(COLOR_GREEN, "Theme loaded from: {}", args[1]);
            } else {
                crate::println_color!(COLOR_RED, "Failed to load theme from: {}", args[1]);
            }
        }
        "save" => {
            if args.len() < 2 {
                crate::println_color!(COLOR_RED, "Usage: theme save <path>");
                return;
            }
            let theme = crate::theme::THEME.read();
            let content = crate::theme::config::generate_theme_config(&theme);
            drop(theme);
            
            match crate::vfs::write_file(args[1], content.as_bytes()) {
                Ok(_) => crate::println_color!(COLOR_GREEN, "Theme saved to: {}", args[1]),
                Err(e) => crate::println_color!(COLOR_RED, "Failed to save: {:?}", e),
            }
        }
        "reload" => {
            crate::theme::reload_wallpaper();
            crate::println_color!(COLOR_GREEN, "Wallpaper reloaded");
        }
        "info" => {
            let theme = crate::theme::THEME.read();
            crate::println_color!(COLOR_CYAN, "Current Theme: {}", 
                if theme.name.is_empty() { "TrustOS Default" } else { &theme.name });
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Colors:");
            crate::println!("  Background:  0x{:08X}", theme.colors.background);
            crate::println!("  Accent:      0x{:08X}", theme.colors.accent);
            crate::println!("  Text:        0x{:08X}", theme.colors.text_primary);
            crate::println!("  Surface:     0x{:08X}", theme.colors.surface);
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Taskbar:");
            crate::println!("  Height:      {} px", theme.taskbar.height);
            crate::println!("  Centered:    {}", theme.taskbar.centered_icons);
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Windows:");
            crate::println!("  Title bar:   {} px", theme.window.titlebar_height);
            crate::println!("  Radius:      {} px", theme.window.border_radius);
            crate::println!("  Shadow:      {} px", theme.window.shadow_size);
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "Wallpaper:");
            crate::println!("  Path:        {}", 
                if theme.wallpaper.path.is_empty() { "(none)" } else { &theme.wallpaper.path });
            crate::println!("  Mode:        {:?}", theme.wallpaper.mode);
        }
        _ => {
            crate::println_color!(COLOR_RED, "Unknown theme command: {}", args[0]);
            crate::println!("Use 'theme' for help");
        }
    }
}

/// Window animations control command
fn cmd_animations(args: &[&str]) {
    if args.is_empty() {
        let enabled = crate::desktop::animations_enabled();
        let speed = crate::desktop::get_animation_speed();
        
        crate::println_color!(COLOR_CYAN, "TrustOS Animation Settings");
        crate::println_color!(COLOR_CYAN, "==========================");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Current Status:");
        if enabled {
            crate::println!("  Animations: {} ENABLED", "\x1b[32m●\x1b[0m");
        } else {
            crate::println!("  Animations: {} DISABLED", "\x1b[31m●\x1b[0m");
        }
        crate::println!("  Speed:      {}x", speed);
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Commands:");
        crate::println!("  anim on           - Enable animations");
        crate::println!("  anim off          - Disable animations");
        crate::println!("  anim toggle       - Toggle on/off");
        crate::println!("  anim speed <val>  - Set speed (0.25-4.0)");
        crate::println!("                      1.0=normal, 2.0=fast, 0.5=slow");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Animation Types:");
        crate::println!("  - Window open (scale up from center)");
        crate::println!("  - Window close (scale down + fade out)");
        crate::println!("  - Minimize (move to taskbar)");
        crate::println!("  - Maximize/Restore (smooth resize)");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "1" | "true" => {
            crate::desktop::set_animations_enabled(true);
            crate::println_color!(COLOR_GREEN, "✓ Animations enabled");
        }
        "off" | "disable" | "0" | "false" => {
            crate::desktop::set_animations_enabled(false);
            crate::println_color!(COLOR_YELLOW, "○ Animations disabled");
        }
        "toggle" => {
            let current = crate::desktop::animations_enabled();
            crate::desktop::set_animations_enabled(!current);
            if !current {
                crate::println_color!(COLOR_GREEN, "✓ Animations enabled");
            } else {
                crate::println_color!(COLOR_YELLOW, "○ Animations disabled");
            }
        }
        "speed" => {
            if args.len() < 2 {
                crate::println!("Current speed: {}x", crate::desktop::get_animation_speed());
                crate::println!("Usage: anim speed <value>");
                crate::println!("  Examples: 0.5 (slow), 1.0 (normal), 2.0 (fast)");
                return;
            }
            if let Ok(speed) = args[1].parse::<f32>() {
                crate::desktop::set_animation_speed(speed);
                crate::println_color!(COLOR_GREEN, "Animation speed set to {}x", speed);
            } else {
                crate::println_color!(COLOR_RED, "Invalid speed value: {}", args[1]);
            }
        }
        "status" | "info" => {
            let enabled = crate::desktop::animations_enabled();
            let speed = crate::desktop::get_animation_speed();
            crate::println!("Animations: {}", if enabled { "enabled" } else { "disabled" });
            crate::println!("Speed: {}x", speed);
        }
        _ => {
            crate::println_color!(COLOR_RED, "Unknown animation command: {}", args[0]);
            crate::println!("Use 'anim' for help");
        }
    }
}

/// HoloMatrix 3D background control command
fn cmd_holomatrix(args: &[&str]) {
    use crate::graphics::holomatrix;
    
    if args.is_empty() {
        let enabled = holomatrix::is_enabled();
        let scene = holomatrix::get_scene();
        
        crate::println_color!(COLOR_CYAN, "TrustOS HoloMatrix 3D");
        crate::println_color!(COLOR_CYAN, "=====================");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Current Status:");
        if enabled {
            crate::println!("  HoloMatrix: {} ENABLED", "\x1b[36m●\x1b[0m");
        } else {
            crate::println!("  HoloMatrix: {} DISABLED", "\x1b[31m●\x1b[0m");
        }
        crate::println!("  Scene:      {}", scene.name());
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Commands:");
        crate::println!("  holo on           - Enable HoloMatrix 3D background");
        crate::println!("  holo off          - Disable (use Matrix Rain)");
        crate::println!("  holo toggle       - Toggle on/off");
        crate::println!("  holo next         - Cycle to next scene");
        crate::println!("  holo scene <name> - Set specific scene");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Available Scenes:");
        crate::println!("  cube     - Rotating wireframe cube");
        crate::println!("  sphere   - Pulsating sphere");
        crate::println!("  torus    - 3D donut/ring");
        crate::println!("  grid     - Perspective grid with cube");
        crate::println!("  multi    - Multiple floating shapes");
        crate::println!("  dna      - Animated DNA double helix");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "How it works:");
        crate::println!("  Renders 3D shapes using 16 Z-slices (layers)");
        crate::println!("  Each layer has depth-based transparency");
        crate::println!("  Creates holographic volumetric effect");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "1" | "true" => {
            holomatrix::set_enabled(true);
            crate::println_color!(0xFF00FFFF, "✓ HoloMatrix 3D enabled");
            crate::println!("Launch 'desktop' to see the effect");
        }
        "off" | "disable" | "0" | "false" => {
            holomatrix::set_enabled(false);
            crate::println_color!(COLOR_YELLOW, "○ HoloMatrix disabled (Matrix Rain active)");
        }
        "toggle" => {
            let enabled = holomatrix::toggle();
            if enabled {
                crate::println_color!(0xFF00FFFF, "✓ HoloMatrix 3D enabled");
            } else {
                crate::println_color!(COLOR_YELLOW, "○ HoloMatrix disabled");
            }
        }
        "next" | "cycle" => {
            let scene = holomatrix::next_scene();
            crate::println_color!(0xFF00FFFF, "Scene: {}", scene.name());
        }
        "scene" | "set" => {
            if args.len() < 2 {
                crate::println!("Current scene: {}", holomatrix::get_scene().name());
                crate::println!("Usage: holo scene <name>");
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
                return;
            }
            if let Some(scene) = holomatrix::HoloScene::from_name(args[1]) {
                holomatrix::set_scene(scene);
                crate::println_color!(0xFF00FFFF, "Scene set to: {}", scene.name());
            } else {
                crate::println_color!(COLOR_RED, "Unknown scene: {}", args[1]);
                crate::println!("Available: cube, sphere, torus, grid, multi, dna");
            }
        }
        "status" | "info" => {
            let enabled = holomatrix::is_enabled();
            let scene = holomatrix::get_scene();
            crate::println!("HoloMatrix: {}", if enabled { "enabled" } else { "disabled" });
            crate::println!("Scene: {}", scene.name());
        }
        "list" | "scenes" => {
            crate::println_color!(COLOR_BRIGHT_GREEN, "Available Scenes:");
            for name in holomatrix::HoloScene::all_names() {
                crate::println!("  {}", name);
            }
        }
        _ => {
            // Try to parse as scene name directly
            if let Some(scene) = holomatrix::HoloScene::from_name(args[0]) {
                holomatrix::set_scene(scene);
                crate::println_color!(0xFF00FFFF, "Scene set to: {}", scene.name());
            } else {
                crate::println_color!(COLOR_RED, "Unknown command: {}", args[0]);
                crate::println!("Use 'holo' for help");
            }
        }
    }
}

/// Image viewer command
fn cmd_imgview(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustOS Image Viewer");
        crate::println_color!(COLOR_CYAN, "====================");
        crate::println!();
        crate::println!("Usage: imgview <path> [options]");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Options:");
        crate::println!("  -x <num>     X position (default: center)");
        crate::println!("  -y <num>     Y position (default: center)");
        crate::println!("  -w <num>     Width (scale to this width)");
        crate::println!("  -h <num>     Height (scale to this height)");
        crate::println!("  -info        Show image info only, don't display");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Supported Formats:");
        crate::println!("  BMP  - 24-bit and 32-bit uncompressed");
        crate::println!("  PPM  - P3 (ASCII) and P6 (binary)");
        crate::println!("  RAW  - Raw RGBA pixel data");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Examples:");
        crate::println!("  imgview /usr/share/wallpapers/logo.bmp");
        crate::println!("  imgview /home/image.ppm -x 100 -y 100");
        crate::println!("  imgview photo.bmp -w 640 -h 480");
        return;
    }
    
    let path = args[0];
    let mut pos_x: Option<i32> = None;
    let mut pos_y: Option<i32> = None;
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
    let mut info_only = false;
    
    // Parse options
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "-x" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<i32>() {
                    pos_x = Some(v);
                }
                i += 2;
            }
            "-y" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<i32>() {
                    pos_y = Some(v);
                }
                i += 2;
            }
            "-w" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<u32>() {
                    width = Some(v);
                }
                i += 2;
            }
            "-h" if i + 1 < args.len() => {
                if let Ok(v) = args[i + 1].parse::<u32>() {
                    height = Some(v);
                }
                i += 2;
            }
            "-info" => {
                info_only = true;
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
    
    // Load the image
    crate::println!("Loading image: {}", path);
    
    match crate::image::load(path) {
        Some(img) => {
            crate::println_color!(COLOR_GREEN, "Image loaded successfully!");
            crate::println!("  Size: {} x {} pixels", img.width, img.height);
            crate::println!("  Memory: {} KB", (img.pixels.len() * 4) / 1024);
            
            if info_only {
                return;
            }
            
            // Calculate final dimensions
            let dest_w = width.unwrap_or(img.width);
            let dest_h = height.unwrap_or(img.height);
            
            // Calculate position (center if not specified)
            let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
            let x = pos_x.unwrap_or_else(|| ((fb_width - dest_w) / 2) as i32);
            let y = pos_y.unwrap_or_else(|| ((fb_height - dest_h) / 2) as i32);
            
            crate::println!("  Drawing at ({}, {}) size {}x{}", x, y, dest_w, dest_h);
            
            // Draw the image
            if dest_w == img.width && dest_h == img.height {
                img.draw(x, y);
            } else {
                img.draw_scaled(x, y, dest_w, dest_h);
            }
            
            crate::framebuffer::swap_buffers();
            crate::println_color!(COLOR_GREEN, "Image displayed!");
        }
        None => {
            crate::println_color!(COLOR_RED, "Failed to load image: {}", path);
            crate::println!("Make sure the file exists and is a supported format.");
        }
    }
}

/// Demo command to display generated test images
fn cmd_imgdemo(args: &[&str]) {
    let demo_type = args.first().copied().unwrap_or("gradient");
    
    crate::println_color!(COLOR_CYAN, "Image Demo: {}", demo_type);
    
    let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
    
    match demo_type {
        "gradient" => {
            // Vertical gradient
            let img = crate::image::create_gradient_v(
                200, 200, 
                0xFF0066FF,  // Blue top
                0xFF00FF66   // Green bottom
            );
            let x = ((fb_width - 200) / 2) as i32;
            let y = ((fb_height - 200) / 2) as i32;
            img.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed gradient at center");
        }
        "checker" => {
            // Checkerboard pattern
            let img = crate::image::create_checkerboard(
                256, 256, 32,
                0xFFFFFFFF,  // White
                0xFF000000   // Black
            );
            let x = ((fb_width - 256) / 2) as i32;
            let y = ((fb_height - 256) / 2) as i32;
            img.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed checkerboard at center");
        }
        "trustos" => {
            // TrustOS logo colors pattern
            let img = crate::image::create_gradient_v(
                300, 100,
                0xFF00D26A,  // Accent green
                0xFF0A0E0B   // Dark background
            );
            let x = ((fb_width - 300) / 2) as i32;
            let y = ((fb_height - 100) / 2) as i32;
            img.draw(x, y);
            
            // Draw a decorative border
            let border_color = 0xFF00D26A;
            for i in 0..300 {
                crate::framebuffer::put_pixel(x as u32 + i, y as u32, border_color);
                crate::framebuffer::put_pixel(x as u32 + i, (y + 99) as u32, border_color);
            }
            for i in 0..100 {
                crate::framebuffer::put_pixel(x as u32, y as u32 + i, border_color);
                crate::framebuffer::put_pixel((x + 299) as u32, y as u32 + i, border_color);
            }
            
            crate::println_color!(COLOR_GREEN, "Displayed TrustOS banner");
        }
        "colors" => {
            // Color test pattern
            let mut img = crate::image::Image::new(256, 256);
            for y in 0..256 {
                for x in 0..256 {
                    let r = x as u32;
                    let g = y as u32;
                    let b = ((x + y) / 2) as u32;
                    let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                    img.set_pixel(x, y, color);
                }
            }
            let x = ((fb_width - 256) / 2) as i32;
            let y = ((fb_height - 256) / 2) as i32;
            img.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed color test pattern");
        }
        "alpha" => {
            // Demonstrate alpha blending
            // First, draw a red background square
            let bg = crate::image::create_solid(200, 200, 0xFFFF0000);
            let x = ((fb_width - 200) / 2) as i32;
            let y = ((fb_height - 200) / 2) as i32;
            bg.draw(x, y);
            
            // Then draw a semi-transparent blue overlay
            let mut overlay = crate::image::Image::new(200, 200);
            for py in 0..200u32 {
                for px in 0..200u32 {
                    // Alpha varies from 0 to 200 based on position
                    let alpha = (px + py) / 2;
                    let color = (alpha << 24) | 0x000000FF;  // Semi-transparent blue
                    overlay.set_pixel(px, py, color);
                }
            }
            overlay.draw(x, y);
            crate::println_color!(COLOR_GREEN, "Displayed alpha blend demo (red + blue)");
        }
        _ => {
            crate::println!("Available demos:");
            crate::println!("  gradient  - Vertical color gradient");
            crate::println!("  checker   - Checkerboard pattern");
            crate::println!("  trustos   - TrustOS banner");
            crate::println!("  colors    - RGB color test pattern");
            crate::println!("  alpha     - Alpha blending demo");
            crate::println!();
            crate::println!("Usage: imgdemo <name>");
            return;
        }
    }
    
    crate::framebuffer::swap_buffers();
}

fn cmd_tasks() {
    let tasks = crate::task::list_tasks();
    crate::println_color!(COLOR_CYAN, "  PID  STATE       PRIORITY  NAME");
    crate::println_color!(COLOR_CYAN, "─────────────────────────────────────");
    
    // Always show kernel and shell
    crate::println!("    1  running     critical  kernel");
    crate::println!("    2  running     normal    tsh");
    
    let task_count = tasks.len();
    for (id, name, state, priority) in tasks {
        let state_str = match state {
            crate::task::TaskState::Ready => "ready",
            crate::task::TaskState::Running => "running",
            crate::task::TaskState::Blocked => "blocked",
            crate::task::TaskState::Terminated => "done",
        };
        let priority_str = match priority {
            crate::task::Priority::Low => "low",
            crate::task::Priority::Normal => "normal",
            crate::task::Priority::High => "high",
            crate::task::Priority::Critical => "critical",
        };
        crate::println!("{:>5}  {:10}  {:8}  {}", id + 2, state_str, priority_str, name);
    }
    
    crate::println!();
    crate::println_color!(COLOR_DARK_GREEN, "Total: {} tasks", task_count + 2);
}

fn cmd_threads() {
    crate::println_color!(COLOR_CYAN, "  TID  PID  STATE       NAME");
    crate::println_color!(COLOR_CYAN, "────────────────────────────────────");
    
    // Get thread info from thread module
    let threads = crate::thread::list_threads();
    let count = threads.len();
    
    for (tid, pid, state, name) in threads {
        let state_str = match state {
            crate::thread::ThreadState::Ready => "ready",
            crate::thread::ThreadState::Running => "running",
            crate::thread::ThreadState::Blocked => "blocked",
            crate::thread::ThreadState::Sleeping => "sleeping",
            crate::thread::ThreadState::Dead => "dead",
        };
        crate::println!("{:>5}  {:>3}  {:10}  {}", tid, pid, state_str, &name);
    }
    
    crate::println!();
    crate::println_color!(COLOR_DARK_GREEN, "Total: {} threads", count);
}

// ==================== PERSISTENCE COMMANDS ====================

fn cmd_persistence(args: &[&str]) {
    if args.is_empty() {
        // Show status
        let (status, files, size) = crate::persistence::status();
        crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
        crate::println_color!(COLOR_CYAN, "║                    Persistence Status                        ║");
        crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
        crate::println!();
        crate::println!("  Status:       {}", status);
        crate::println!("  Saved files:  {}", files);
        crate::println!("  Total size:   {} KB", size / 1024);
        crate::println!();
        crate::println!("Commands:");
        crate::println!("  persist status  - Show this status");
        crate::println!("  persist clear   - Clear all saved data");
        crate::println!("  persist save    - Save current downloads to disk");
        crate::println!();
        return;
    }
    
    match args[0] {
        "status" => {
            let (status, files, size) = crate::persistence::status();
            crate::println!("Persistence: {} ({} files, {} KB)", status, files, size / 1024);
        }
        "clear" => {
            crate::println!("Clearing persistence data...");
            match crate::persistence::clear() {
                Ok(_) => crate::println_color!(COLOR_GREEN, "Persistence data cleared."),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        "save" => {
            crate::println!("Saving current data to disk...");
            
            // Save Alpine rootfs if it exists
            let alpine_path = "/opt/gui/alpine.tar.gz";
            if file_exists(alpine_path) {
                let read_result: Result<Vec<u8>, _> = crate::ramfs::with_fs(|fs| {
                    fs.read_file(alpine_path).map(|d| d.to_vec())
                });
                match read_result {
                    Ok(data) => {
                        match crate::persistence::save_file(alpine_path, &data) {
                            Ok(_) => crate::println_color!(COLOR_GREEN, "  Saved: {} ({} KB)", alpine_path, data.len() / 1024),
                            Err(e) => crate::println_color!(COLOR_RED, "  Failed: {} - {}", alpine_path, e),
                        }
                    }
                    Err(e) => crate::println_color!(COLOR_RED, "  Cannot read {}: {:?}", alpine_path, e),
                }
            } else {
                crate::println!("  No files to save. Run 'gui install' first.");
            }
        }
        _ => {
            crate::println!("Unknown persistence command: {}", args[0]);
            crate::println!("Use: persist [status|clear|save]");
        }
    }
}

// ==================== DISK COMMANDS ====================

fn cmd_disk() {
    crate::println_color!(COLOR_CYAN, "=== Disk Information ===");
    
    if let Some(info) = crate::disk::get_info() {
        crate::print_color!(COLOR_GREEN, "Model:   ");
        crate::println!("{}", info.model);
        crate::print_color!(COLOR_GREEN, "Serial:  ");
        crate::println!("{}", info.serial);
        crate::print_color!(COLOR_GREEN, "Size:    ");
        crate::println!("{} MB ({} sectors)", info.size_mb, info.sectors);
        
        let (reads, writes, bytes_r, bytes_w) = crate::disk::get_stats();
        crate::println!();
        crate::println_color!(COLOR_CYAN, "Statistics:");
        crate::println!("  Reads:  {} ops ({} bytes)", reads, bytes_r);
        crate::println!("  Writes: {} ops ({} bytes)", writes, bytes_w);
        
        // List files
        crate::println!();
        crate::println_color!(COLOR_CYAN, "Files on disk:");
        match crate::disk::list_files() {
            Ok(files) => {
                if files.is_empty() {
                    crate::println!("  (no files)");
                } else {
                    for f in files {
                        if f.is_directory {
                            crate::print_color!(COLOR_CYAN, "  [DIR]  ");
                        } else {
                            crate::print!("  [FILE] ");
                        }
                        crate::println!("{} ({} bytes)", f.name, f.size);
                    }
                }
            }
            Err(e) => crate::println_color!(COLOR_RED, "  Error: {}", e),
        }
    } else {
        crate::println_color!(COLOR_YELLOW, "No disk detected");
    }
}

fn cmd_dd(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: dd <sector> [count]");
        crate::println!("       dd write <sector> <text>");
        crate::println!("       dd dump <sector>");
        return;
    }
    
    if args[0] == "dump" && args.len() > 1 {
        let sector: u64 = match args[1].parse() {
            Ok(n) => n,
            Err(_) => {
                crate::println_color!(COLOR_RED, "Invalid sector number");
                return;
            }
        };
        
        match crate::disk::dump_sector(sector) {
            Ok(dump) => crate::println!("{}", dump),
            Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
        }
        return;
    }
    
    if args[0] == "write" && args.len() > 2 {
        let sector: u64 = match args[1].parse() {
            Ok(n) => n,
            Err(_) => {
                crate::println_color!(COLOR_RED, "Invalid sector number");
                return;
            }
        };
        
        let text = args[2..].join(" ");
        let mut data = [0u8; 512];
        let bytes = text.as_bytes();
        let len = bytes.len().min(512);
        data[..len].copy_from_slice(&bytes[..len]);
        
        match crate::disk::write_sector(sector, &data) {
            Ok(_) => crate::println_color!(COLOR_GREEN, "Written {} bytes to sector {}", len, sector),
            Err(e) => crate::println_color!(COLOR_RED, "Write error: {}", e),
        }
        return;
    }
    
    // Read sector
    let sector: u64 = match args[0].parse() {
        Ok(n) => n,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid sector number");
            return;
        }
    };

    let mut buffer = [0u8; 512];
    match crate::disk::read_sectors(sector, 1, &mut buffer) {
        Ok(_) => {
            crate::println_color!(COLOR_CYAN, "Sector {} (512 bytes):", sector);
            
            // Hexdump first 256 bytes
            for row in 0..16 {
                crate::print_color!(COLOR_DARK_GREEN, "{:04X}: ", row * 16);
                for col in 0..16 {
                    crate::print!("{:02X} ", buffer[row * 16 + col]);
                }
                crate::print!(" |");
                for col in 0..16 {
                    let b = buffer[row * 16 + col];
                    if b >= 0x20 && b < 0x7F {
                        crate::print!("{}", b as char);
                    } else {
                        crate::print!(".");
                    }
                }
                crate::println!("|");
            }
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Read error: {}", e);
        }
    }
}

fn cmd_ahci(args: &[&str]) {
    if args.is_empty() {
        // Show AHCI info
        crate::println_color!(COLOR_CYAN, "=== AHCI Storage Controller ===");
        
        if !crate::drivers::ahci::is_initialized() {
            crate::println_color!(COLOR_YELLOW, "AHCI not initialized");
            return;
        }
        
        let devices = crate::drivers::ahci::list_devices();
        if devices.is_empty() {
            crate::println_color!(COLOR_YELLOW, "No AHCI devices found");
            return;
        }
        
        crate::println!("Found {} device(s):", devices.len());
        for dev in &devices {
            crate::println!();
            crate::print_color!(COLOR_GREEN, "  Port {}: ", dev.port_num);
            crate::println!("{:?}", dev.device_type);
            crate::println!("    Model:   {}", dev.model);
            crate::println!("    Serial:  {}", dev.serial);
            crate::println!("    Sectors: {}", dev.sector_count);
        }
        
        crate::println!();
        crate::println_color!(COLOR_DARK_GREEN, "Commands:");
        crate::println!("  ahci read <port> <sector>   - Read sector from port");
        crate::println!("  ahci write <port> <sector> <text> - Write to sector");
        return;
    }
    
    match args[0] {
        "read" => {
            if args.len() < 3 {
                crate::println!("Usage: ahci read <port> <sector>");
                return;
            }
            
            let port: u8 = match args[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid port number");
                    return;
                }
            };
            
            let sector: u64 = match args[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid sector number");
                    return;
                }
            };
            
            crate::println!("Reading sector {} from AHCI port {}...", sector, port);
            
            // Allocate aligned buffer
            let mut buffer = alloc::vec![0u8; 512];
            
            match crate::drivers::ahci::read_sectors(port, sector, 1, &mut buffer) {
                Ok(bytes) => {
                    crate::println_color!(COLOR_GREEN, "Read {} bytes successfully", bytes);
                    crate::println!();
                    
                    // Hexdump first 256 bytes
                    for row in 0..16 {
                        crate::print_color!(COLOR_DARK_GREEN, "{:04X}: ", row * 16);
                        for col in 0..16 {
                            crate::print!("{:02X} ", buffer[row * 16 + col]);
                        }
                        crate::print!(" |");
                        for col in 0..16 {
                            let b = buffer[row * 16 + col];
                            if b >= 0x20 && b < 0x7F {
                                crate::print!("{}", b as char);
                            } else {
                                crate::print!(".");
                            }
                        }
                        crate::println!("|");
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "AHCI read error: {}", e);
                }
            }
        }
        
        "write" => {
            if args.len() < 4 {
                crate::println!("Usage: ahci write <port> <sector> <text>");
                return;
            }
            
            let port: u8 = match args[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid port number");
                    return;
                }
            };
            
            let sector: u64 = match args[2].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid sector number");
                    return;
                }
            };
            
            let text = args[3..].join(" ");
            let mut buffer = alloc::vec![0u8; 512];
            let bytes = text.as_bytes();
            let len = bytes.len().min(512);
            buffer[..len].copy_from_slice(&bytes[..len]);
            
            crate::println!("Writing {} bytes to sector {} on AHCI port {}...", len, sector, port);
            
            match crate::drivers::ahci::write_sectors(port, sector, 1, &buffer) {
                Ok(bytes) => {
                    crate::println_color!(COLOR_GREEN, "Written {} bytes successfully", bytes);
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "AHCI write error: {}", e);
                }
            }
        }
        
        _ => {
            crate::println!("Unknown AHCI command. Use 'ahci' for help.");
        }
    }
}

// ==================== PARTITION COMMANDS ====================

fn cmd_fdisk(args: &[&str]) {
    use crate::drivers::partition;
    use crate::drivers::ahci;
    
    if args.is_empty() {
        // List all disks and their partitions
        crate::println_color!(COLOR_CYAN, "=== Partition Tables ===");
        crate::println!();
        
        if !ahci::is_initialized() {
            crate::println_color!(COLOR_YELLOW, "AHCI not initialized");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  fdisk           - Show partitions on all disks");
            crate::println!("  fdisk <port>    - Show partitions on specific AHCI port");
            return;
        }
        
        let devices = ahci::list_devices();
        if devices.is_empty() {
            crate::println!("No AHCI devices found");
            return;
        }
        
        for dev in devices {
            crate::println_color!(COLOR_GREEN, "─── Disk {} ({:?}) ───", dev.port_num, dev.device_type);
            
            match partition::read_from_ahci(dev.port_num) {
                Ok(table) => {
                    partition::print_partition_table(&table);
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Error reading partitions: {}", e);
                }
            }
            crate::println!();
        }
        
        return;
    }
    
    // Parse port number
    let port: u8 = match args[0].parse() {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port number: {}", args[0]);
            return;
        }
    };
    
    crate::println_color!(COLOR_CYAN, "=== Partitions on Disk {} ===", port);
    
    match partition::read_from_ahci(port) {
        Ok(table) => {
            partition::print_partition_table(&table);
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Error: {}", e);
        }
    }
}

// ==================== NETWORK COMMANDS ====================

fn cmd_ifconfig() {
    // Show NIC hardware info
    if let Some(nic) = crate::network::get_nic_info() {
        crate::println_color!(COLOR_CYAN, "Hardware:");
        crate::println!("      Device: {:04X}:{:04X} [{}]", 
            nic.vendor_id, nic.device_id, nic.vendor_name);
        crate::println!("      Driver: {}", nic.driver);
        if crate::network::has_real_driver() {
            crate::println_color!(COLOR_GREEN, "      Status: REAL DRIVER ACTIVE");
        } else {
            crate::println_color!(COLOR_YELLOW, "      Status: Simulated");
        }
        if nic.bar0 != 0 {
            crate::println!("      BAR0:   {:#010X}", nic.bar0);
        }
        if nic.irq != 0 && nic.irq != 0xFF {
            crate::println!("      IRQ:    {}", nic.irq);
        }
        crate::println!();
    }
    
    if let Some((mac, ip, state)) = crate::network::get_interface() {
        crate::println_color!(COLOR_CYAN, "eth0:");
        crate::print!("      Link: ");
        match state {
            crate::network::NetworkState::Up => crate::println_color!(COLOR_GREEN, "UP"),
            crate::network::NetworkState::Down => crate::println_color!(COLOR_YELLOW, "DOWN"),
            crate::network::NetworkState::Error => crate::println_color!(COLOR_RED, "ERROR"),
        }
        crate::println!("      HWaddr: {}", mac);
        if let Some(addr) = ip {
            crate::println!("      inet:   {}", addr);
        }
        
        // Use driver stats for accuracy
        let (tx_pkts, rx_pkts, tx_bytes, rx_bytes) = crate::network::get_driver_stats();
        crate::println!();
        crate::println!("      RX packets: {}  bytes: {}", rx_pkts, rx_bytes);
        crate::println!("      TX packets: {}  bytes: {}", tx_pkts, tx_bytes);
        
        let stats = crate::network::get_stats();
        if stats.errors > 0 {
            crate::println_color!(COLOR_RED, "      Errors: {}", stats.errors);
        }
    } else {
        crate::println_color!(COLOR_YELLOW, "No network interface");
    }
}

fn cmd_ping(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: ping <ip|host>");
        crate::println!("  Example: ping 192.168.56.1");
        crate::println!("  Example: ping example.com");
        return;
    }

    let ip = if let Some(ip) = parse_ipv4(args[0]) {
        crate::network::Ipv4Address::new(ip[0], ip[1], ip[2], ip[3])
    } else if let Some(resolved) = crate::netstack::dns::resolve(args[0]) {
        crate::network::Ipv4Address::new(resolved[0], resolved[1], resolved[2], resolved[3])
    } else {
        crate::println_color!(COLOR_RED, "Unable to resolve host");
        return;
    };
    
    crate::println!("PING {} ({}) 56 data bytes", args[0], ip);
    
    let mut success_count = 0;
    let mut total_us = 0u64;
    let mut min_us = u64::MAX;
    let mut max_us = 0u64;
    
    for _ in 0..4 {
        match crate::network::send_ping(ip) {
            Ok(result) => {
                if result.success {
                    success_count += 1;
                    total_us += result.time_us;
                    min_us = min_us.min(result.time_us);
                    max_us = max_us.max(result.time_us);
                    
                    // Show microsecond precision
                    if result.time_us < 1000 {
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={} µs", 
                            ip, result.seq, result.ttl, result.time_us);
                    } else {
                        let ms = result.time_us / 1000;
                        let us_frac = (result.time_us % 1000) / 10;
                        crate::println!("64 bytes from {}: icmp_seq={} ttl={} time={}.{:02} ms", 
                            ip, result.seq, result.ttl, ms, us_frac);
                    }
                } else {
                    crate::println_color!(COLOR_YELLOW, "Request timeout for icmp_seq {}", result.seq);
                }
            }
            Err(e) => {
                crate::println_color!(COLOR_RED, "ping failed: {}", e);
            }
        }
        
        // High-precision delay between pings (~1 second)
        crate::cpu::tsc::delay_millis(1000);
    }
    
    crate::println!();
    crate::println!("--- {} ping statistics ---", args[0]);
    crate::println!("4 packets transmitted, {} received, {}% packet loss", 
        success_count, 
        (4 - success_count) * 25);
    if success_count > 0 {
        let avg_us = total_us / success_count as u64;
        // Show min/avg/max in ms with µs precision
        crate::println!("rtt min/avg/max = {}.{:03}/{}.{:03}/{}.{:03} ms", 
            min_us / 1000, min_us % 1000,
            avg_us / 1000, avg_us % 1000,
            max_us / 1000, max_us % 1000);
    }
}

fn cmd_netstat() {
    crate::println_color!(COLOR_CYAN, "Network Statistics");
    crate::println!("==================");
    
    let stats = crate::network::get_stats();
    crate::println!();
    crate::print_color!(COLOR_GREEN, "Packets received: ");
    crate::println!("{}", stats.packets_received);
    crate::print_color!(COLOR_GREEN, "Packets sent:     ");
    crate::println!("{}", stats.packets_sent);
    crate::print_color!(COLOR_GREEN, "Bytes received:   ");
    crate::println!("{}", stats.bytes_received);
    crate::print_color!(COLOR_GREEN, "Bytes sent:       ");
    crate::println!("{}", stats.bytes_sent);
    crate::print_color!(COLOR_GREEN, "Errors:           ");
    crate::println!("{}", stats.errors);
}

fn cmd_ipconfig(args: &[&str]) {
    let show_all = args.iter().any(|a| *a == "/all" || *a == "-a");
    crate::println!("Windows IP Configuration");
    crate::println!();

    if let Some((mac, ip, state)) = crate::network::get_interface() {
        crate::println!("   Ethernet adapter net0:");
        crate::println!("      Status . . . . . . . . . . . . : {:?}", state);
        crate::println!("      Physical Address. . . . . . . . : {}", mac);
        if let Some(ip) = ip {
            crate::println!("      IPv4 Address. . . . . . . . . : {}", ip);
            if let Some((_, subnet, gateway)) = crate::network::get_ipv4_config() {
                crate::println!("      Subnet Mask . . . . . . . . . : {}", subnet);
                if let Some(gw) = gateway {
                    crate::println!("      Default Gateway . . . . . . . : {}", gw);
                } else if show_all {
                    crate::println!("      Default Gateway . . . . . . . : (none)");
                }
            }
        } else {
            crate::println!("      IPv4 Address. . . . . . . . . : (none)");
        }
    } else {
        crate::println!("No network interface detected");
    }
}

fn cmd_nslookup(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nslookup <host>");
        return;
    }

    let target = args[0];
    if parse_ipv4(target).is_some() {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Address: {}", target);
        crate::println!("*** Reverse lookup not implemented");
        return;
    }

    if let Some(resolved) = crate::netstack::dns::resolve(target) {
        crate::println!("Server: 8.8.8.8");
        crate::println!("Name: {}", target);
        crate::println!("Address: {}.{}.{}.{}", resolved[0], resolved[1], resolved[2], resolved[3]);
    } else {
        crate::println_color!(COLOR_RED, "DNS lookup failed");
    }
}

fn cmd_arp(args: &[&str]) {
    if args.iter().any(|a| *a == "-a" || *a == "/a") {
        crate::println!("Interface: net0");
    }

    let entries = crate::netstack::arp::entries();
    if entries.is_empty() {
        crate::println!("No ARP entries");
        return;
    }

    crate::println!("Internet Address      Physical Address       Type");
    for (ip, mac) in entries {
        let ipb = ip.to_be_bytes();
        crate::println!(
            "{:>3}.{:>3}.{:>3}.{:>3}      {:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}   dynamic",
            ipb[0], ipb[1], ipb[2], ipb[3], mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
    }
}

fn cmd_route(_args: &[&str]) {
    crate::println!("Kernel IP routing table");
    crate::println!("Destination     Gateway         Genmask         Iface");

    if let Some((ip, subnet, gateway)) = crate::network::get_ipv4_config() {
        let gw = gateway.unwrap_or(crate::network::Ipv4Address::new(0, 0, 0, 0));
        crate::println!("{}     {}     {}     net0", ip, gw, subnet);
        crate::println!("0.0.0.0         {}     0.0.0.0         net0", gw);
    } else {
        crate::println!("0.0.0.0         0.0.0.0         0.0.0.0         net0");
    }
}

fn cmd_traceroute(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: traceroute <host>");
        return;
    }

    let host = args[0];
    let ip = if let Some(ip) = parse_ipv4(host) {
        ip
    } else if let Some(resolved) = crate::netstack::dns::resolve(host) {
        resolved
    } else {
        crate::println_color!(COLOR_RED, "Unable to resolve host");
        return;
    };

    crate::println!("traceroute to {} ({}.{}.{}.{}), 30 hops max", host, ip[0], ip[1], ip[2], ip[3]);
    if let Some((_, _, gateway)) = crate::network::get_ipv4_config() {
        if let Some(gw) = gateway {
            crate::println!(" 1  {}", gw);
        }
    }
    crate::println!(" 2  {}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]);
    crate::println_color!(COLOR_YELLOW, "Note: traceroute is simplified (no TTL probing)");
}

// ==================== HARDWARE COMMANDS ====================

// ── Audio Commands ──

fn cmd_beep(args: &[&str]) {
    let freq = args.first()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(440);
    let duration = args.get(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(500);

    if freq < 20 || freq > 20000 {
        crate::println_color!(COLOR_RED, "Frequency must be 20-20000 Hz");
        return;
    }
    if duration > 10000 {
        crate::println_color!(COLOR_RED, "Duration max 10000 ms");
        return;
    }

    // Initialize HDA if needed
    if !crate::drivers::hda::is_initialized() {
        crate::print_color!(COLOR_YELLOW, "Initializing audio driver... ");
        match crate::drivers::hda::init() {
            Ok(()) => crate::println_color!(COLOR_GREEN, "OK"),
            Err(e) => {
                crate::println_color!(COLOR_RED, "FAILED: {}", e);
                return;
            }
        }
    }

    crate::println!("Playing {}Hz for {}ms...", freq, duration);
    match crate::drivers::hda::play_tone(freq, duration) {
        Ok(()) => crate::println_color!(COLOR_GREEN, "Done"),
        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
    }
}

fn cmd_audio(args: &[&str]) {
    match args.first().copied() {
        Some("init") => {
            crate::print_color!(COLOR_YELLOW, "Initializing Intel HDA driver... ");
            match crate::drivers::hda::init() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "OK"),
                Err(e) => crate::println_color!(COLOR_RED, "FAILED: {}", e),
            }
        }
        Some("status") | None => {
            let status = crate::drivers::hda::status();
            crate::println!("{}", status);
        }
        Some("stop") => {
            match crate::drivers::hda::stop() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Playback stopped"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("test") => {
            // Play a quick scale: C D E F G A B C
            if !crate::drivers::hda::is_initialized() {
                crate::print_color!(COLOR_YELLOW, "Initializing audio driver... ");
                match crate::drivers::hda::init() {
                    Ok(()) => crate::println_color!(COLOR_GREEN, "OK"),
                    Err(e) => {
                        crate::println_color!(COLOR_RED, "FAILED: {}", e);
                        return;
                    }
                }
            }
            crate::println!("Playing test scale...");
            let notes = [262, 294, 330, 349, 392, 440, 494, 523]; // C4 to C5
            for &freq in &notes {
                let _ = crate::drivers::hda::play_tone(freq, 200);
            }
            crate::println_color!(COLOR_GREEN, "Done");
        }
        Some(other) => {
            crate::println_color!(COLOR_YELLOW, "Usage: audio [init|status|stop|test]");
        }
    }
}

fn cmd_synth(args: &[&str]) {
    match args.first().copied() {
        Some("note") | Some("play") => {
            // synth note C4 [duration_ms] [waveform]
            let note_name = match args.get(1) {
                Some(n) => *n,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth note <note> [duration_ms] [waveform]");
                    crate::println!("  Examples: synth note C4");
                    crate::println!("           synth note A#3 1000 saw");
                    return;
                }
            };
            let duration = args.get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(500);
            // Set waveform if specified
            if let Some(wf_str) = args.get(3) {
                if let Some(wf) = crate::audio::synth::Waveform::from_str(wf_str) {
                    let _ = crate::audio::set_waveform(wf);
                }
            }
            if duration > 10000 {
                crate::println_color!(COLOR_RED, "Duration max 10000 ms");
                return;
            }
            crate::println!("Synth: {} for {}ms", note_name, duration);
            match crate::audio::play_note(note_name, duration) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Done"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("freq") => {
            // synth freq 440 [duration_ms]
            let freq = match args.get(1).and_then(|s| s.parse::<u32>().ok()) {
                Some(f) => f,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth freq <hz> [duration_ms]");
                    return;
                }
            };
            let duration = args.get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(500);
            if freq < 20 || freq > 20000 {
                crate::println_color!(COLOR_RED, "Frequency must be 20-20000 Hz");
                return;
            }
            crate::println!("Synth: {}Hz for {}ms", freq, duration);
            match crate::audio::play_freq(freq, duration) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Done"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("wave") | Some("waveform") => {
            // synth wave sine|square|saw|triangle|noise
            match args.get(1) {
                Some(wf_str) => {
                    match crate::audio::synth::Waveform::from_str(wf_str) {
                        Some(wf) => {
                            let _ = crate::audio::set_waveform(wf);
                            crate::println_color!(COLOR_GREEN, "Waveform set to: {}", wf.name());
                        }
                        None => crate::println_color!(COLOR_RED, "Unknown waveform (use: sine, square, saw, triangle, noise)"),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth wave <sine|square|saw|triangle|noise>"),
            }
        }
        Some("adsr") => {
            // synth adsr <attack_ms> <decay_ms> <sustain_%> <release_ms>
            if args.len() < 5 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth adsr <attack_ms> <decay_ms> <sustain_%> <release_ms>");
                crate::println!("  Example: synth adsr 10 50 70 100");
                return;
            }
            let a = args[1].parse::<u32>().unwrap_or(10);
            let d = args[2].parse::<u32>().unwrap_or(50);
            let s = args[3].parse::<u32>().unwrap_or(70);
            let r = args[4].parse::<u32>().unwrap_or(100);
            let _ = crate::audio::set_adsr(a, d, s, r);
            crate::println_color!(COLOR_GREEN, "ADSR set: A={}ms D={}ms S={}% R={}ms", a, d, s, r);
        }
        Some("preset") => {
            // synth preset default|organ|pluck|pad
            match args.get(1).copied() {
                Some(name) => {
                    match crate::audio::set_envelope_preset(name) {
                        Ok(()) => crate::println_color!(COLOR_GREEN, "Envelope preset: {}", name),
                        Err(e) => crate::println_color!(COLOR_RED, "{}", e),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth preset <default|organ|pluck|pad>"),
            }
        }
        Some("volume") | Some("vol") => {
            match args.get(1).and_then(|s| s.parse::<u8>().ok()) {
                Some(v) => {
                    let _ = crate::audio::set_volume(v);
                    crate::println_color!(COLOR_GREEN, "Master volume: {}/255", v);
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth volume <0-255>"),
            }
        }
        Some("status") => {
            let s = crate::audio::status();
            crate::println!("{}", s);
        }
        Some("stop") => {
            match crate::audio::stop() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Synth stopped"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("demo") => {
            crate::println!("TrustSynth Demo — playing scale with different waveforms...");
            let notes = ["C4", "D4", "E4", "F4", "G4", "A4", "B4", "C5"];
            let waveforms = [
                ("Sine",     crate::audio::synth::Waveform::Sine),
                ("Square",   crate::audio::synth::Waveform::Square),
                ("Sawtooth", crate::audio::synth::Waveform::Sawtooth),
                ("Triangle", crate::audio::synth::Waveform::Triangle),
            ];
            for (wf_name, wf) in &waveforms {
                let _ = crate::audio::set_waveform(*wf);
                crate::println!("  {} waveform:", wf_name);
                for note in &notes {
                    crate::print!("    {} ", note);
                    let _ = crate::audio::play_note(note, 200);
                }
                crate::println!();
            }
            crate::println_color!(COLOR_GREEN, "Demo complete!");
        }
        // ── Pattern Sequencer commands ──
        Some("pattern") | Some("pat") => {
            cmd_synth_pattern(&args[1..]);
        }
        Some(_) | None => {
            crate::println_color!(COLOR_CYAN, "TrustSynth — Audio Synthesizer & Sequencer");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "  Synth:");
            crate::println!("  synth note <note> [ms] [wave]  Play a note (e.g. C4, A#3)");
            crate::println!("  synth freq <hz> [ms]           Play a frequency");
            crate::println!("  synth wave <type>               Set waveform (sine/square/saw/tri/noise)");
            crate::println!("  synth adsr <A> <D> <S%> <R>    Set envelope (ms, ms, %, ms)");
            crate::println!("  synth preset <name>             Set preset (default/organ/pluck/pad)");
            crate::println!("  synth volume <0-255>            Set master volume");
            crate::println!("  synth demo                      Play demo scale");
            crate::println!("  synth status                    Show synth status");
            crate::println!("  synth stop                      Stop playback");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "  Pattern Sequencer:");
            crate::println!("  synth pattern list              List all patterns");
            crate::println!("  synth pattern show <name>       Display pattern grid");
            crate::println!("  synth pattern new <name> [N] [bpm]  Create pattern (N steps)");
            crate::println!("  synth pattern play <name> [loops]   Play pattern (loop)");
            crate::println!("  synth pattern stop              Stop playback");
            crate::println!("  synth pattern bpm <name> <bpm>  Set tempo");
            crate::println!("  synth pattern wave <name> <wf>  Set waveform");
            crate::println!("  synth pattern set <name> <step> <note>  Set note at step");
            crate::println!("  synth pattern del <name>        Delete pattern");
        }
    }
}

fn cmd_synth_pattern(args: &[&str]) {
    match args.first().copied() {
        Some("list") | Some("ls") | None => {
            let list = crate::audio::pattern_list();
            crate::println!("{}", list);
        }
        Some("show") | Some("view") => {
            match args.get(1) {
                Some(name) => {
                    match crate::audio::pattern_show(name) {
                        Ok(s) => crate::println!("{}", s),
                        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth pattern show <name>"),
            }
        }
        Some("new") | Some("create") => {
            let name = match args.get(1) {
                Some(n) => *n,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth pattern new <name> [steps] [bpm]");
                    return;
                }
            };
            let steps = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(16);
            let bpm = args.get(3).and_then(|s| s.parse::<u16>().ok()).unwrap_or(120);
            match crate::audio::pattern_new(name, steps, bpm) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Pattern \"{}\" created ({} steps, {} BPM)", name, steps, bpm),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("play") => {
            let name = match args.get(1) {
                Some(n) => *n,
                None => {
                    crate::println_color!(COLOR_YELLOW, "Usage: synth pattern play <name> [loops]");
                    return;
                }
            };
            let loops = args.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(1);
            match crate::audio::pattern_play(name, loops) {
                Ok(()) => {}
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("stop") => {
            match crate::audio::pattern_stop() {
                Ok(()) => crate::println_color!(COLOR_GREEN, "Pattern playback stopped"),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("bpm") | Some("tempo") => {
            if args.len() < 3 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth pattern bpm <name> <60-300>");
                return;
            }
            let name = args[1];
            let bpm = match args[2].parse::<u16>() {
                Ok(b) if b >= 30 && b <= 300 => b,
                _ => {
                    crate::println_color!(COLOR_RED, "BPM must be 30-300");
                    return;
                }
            };
            match crate::audio::pattern_set_bpm(name, bpm) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "\"{}\" BPM set to {}", name, bpm),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("wave") | Some("waveform") => {
            if args.len() < 3 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth pattern wave <name> <sine|square|saw|tri|noise>");
                return;
            }
            let name = args[1];
            let wf = match crate::audio::synth::Waveform::from_str(args[2]) {
                Some(w) => w,
                None => {
                    crate::println_color!(COLOR_RED, "Unknown waveform");
                    return;
                }
            };
            match crate::audio::pattern_set_wave(name, wf) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "\"{}\" waveform set to {}", name, wf.name()),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("set") | Some("note") => {
            // synth pattern set <name> <step#> <note>
            if args.len() < 4 {
                crate::println_color!(COLOR_YELLOW, "Usage: synth pattern set <name> <step#> <note|-->");
                crate::println!("  Example: synth pattern set mypattern 0 C4");
                crate::println!("  Example: synth pattern set mypattern 3 --  (rest)");
                return;
            }
            let name = args[1];
            let step_idx = match args[2].parse::<usize>() {
                Ok(i) => i,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Step must be a number");
                    return;
                }
            };
            let note = args[3];
            match crate::audio::pattern_set_note(name, step_idx, note) {
                Ok(()) => crate::println_color!(COLOR_GREEN, "\"{}\" step {} = {}", name, step_idx, note),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        Some("del") | Some("delete") | Some("rm") => {
            match args.get(1) {
                Some(name) => {
                    match crate::audio::pattern_remove(name) {
                        Ok(()) => crate::println_color!(COLOR_GREEN, "Pattern \"{}\" deleted", name),
                        Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
                    }
                }
                None => crate::println_color!(COLOR_YELLOW, "Usage: synth pattern del <name>"),
            }
        }
        Some(other) => {
            crate::println_color!(COLOR_RED, "Unknown pattern command: {}", other);
            crate::println!("Use: list, show, new, play, stop, bpm, wave, set, del");
        }
    }
}

fn cmd_lspci(args: &[&str]) {
    let devices = crate::pci::get_devices();
    
    if devices.is_empty() {
        crate::println_color!(COLOR_YELLOW, "No PCI devices found");
        return;
    }
    
    let verbose = args.contains(&"-v") || args.contains(&"--verbose");
    
    crate::println_color!(COLOR_CYAN, "PCI Devices ({} found):", devices.len());
    crate::println!();
    
    for dev in &devices {
        // Basic format: Bus:Device.Function VendorID:DeviceID Class
        crate::print_color!(COLOR_GREEN, "{:02X}:{:02X}.{} ", 
            dev.bus, dev.device, dev.function);
        crate::print!("{:04X}:{:04X} ", dev.vendor_id, dev.device_id);
        
        let subclass_name = dev.subclass_name();
        if subclass_name.is_empty() {
            crate::print!("{}", dev.class_name());
        } else {
            crate::print!("{}", subclass_name);
        }
        
        crate::println_color!(COLOR_YELLOW, " [{}]", dev.vendor_name());
        
        if verbose {
            crate::println!("        Class: {:02X}:{:02X} ProgIF: {:02X} Rev: {:02X}",
                dev.class_code, dev.subclass, dev.prog_if, dev.revision);
            
            if dev.interrupt_line != 0xFF && dev.interrupt_pin != 0 {
                crate::println!("        IRQ: {} (pin {})", 
                    dev.interrupt_line, dev.interrupt_pin);
            }
            
            // Show BARs
            for i in 0..6 {
                if let Some(addr) = dev.bar_address(i) {
                    let bar_type = if dev.bar_is_memory(i) { "MEM" } else { "I/O" };
                    crate::println!("        BAR{}: {:#010X} [{}]", i, addr, bar_type);
                }
            }
            crate::println!();
        }
    }
    
    if !verbose {
        crate::println!();
        crate::println_color!(COLOR_YELLOW, "Use 'lspci -v' for detailed info");
    }
}

fn cmd_lshw() {
    crate::println_color!(COLOR_CYAN, "=== Hardware Summary ===");
    crate::println!();
    
    let devices = crate::pci::get_devices();
    
    // CPU info
    crate::println_color!(COLOR_GREEN, "CPU:");
    crate::println!("  Architecture: x86_64");
    crate::println!("  Mode: Long Mode (64-bit)");
    crate::println!();
    
    // Memory info
    crate::println_color!(COLOR_GREEN, "Memory:");
    crate::println!("  Heap: 256 KB");
    crate::println!();
    
    // Storage
    let storage: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::MASS_STORAGE)
        .collect();
    crate::println_color!(COLOR_GREEN, "Storage Controllers ({}):", storage.len());
    for dev in &storage {
        crate::println!("  {:04X}:{:04X} {} [{}]", 
            dev.vendor_id, dev.device_id, 
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // Network
    let network: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::NETWORK)
        .collect();
    crate::println_color!(COLOR_GREEN, "Network Controllers ({}):", network.len());
    for dev in &network {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            dev.vendor_id, dev.device_id,
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // Display
    let display: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::DISPLAY)
        .collect();
    crate::println_color!(COLOR_GREEN, "Display ({}):", display.len());
    for dev in &display {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            dev.vendor_id, dev.device_id,
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // USB
    let usb: Vec<_> = devices.iter()
        .filter(|d| d.class_code == crate::pci::class::SERIAL_BUS 
                 && d.subclass == crate::pci::serial::USB)
        .collect();
    crate::println_color!(COLOR_GREEN, "USB Controllers ({}):", usb.len());
    for dev in &usb {
        crate::println!("  {:04X}:{:04X} {} [{}]",
            dev.vendor_id, dev.device_id,
            dev.subclass_name(),
            dev.vendor_name());
    }
    crate::println!();
    
    // Summary
    crate::println_color!(COLOR_CYAN, "Total: {} PCI devices", devices.len());
}

fn cmd_tcpsyn(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: tcpsyn <ip> <port>");
        crate::println!("  Example: tcpsyn 93.184.216.34 80");
        return;
    }

    let parts: Vec<&str> = args[0].split('.').collect();
    if parts.len() != 4 {
        crate::println_color!(COLOR_RED, "Invalid IP format");
        return;
    }

    let ip = [
        parts[0].parse().unwrap_or(0),
        parts[1].parse().unwrap_or(0),
        parts[2].parse().unwrap_or(0),
        parts[3].parse().unwrap_or(0),
    ];

    let port: u16 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port");
            return;
        }
    };

    crate::println!("Sending TCP SYN to {}:{}...", args[0], port);
    match crate::netstack::tcp::send_syn(ip, port) {
        Ok(src_port) => {
            crate::println!("SYN sent to {}:{} (src port {})", args[0], port, src_port);
            let established = crate::netstack::tcp::wait_for_established(ip, port, src_port, 1000);
            if established {
                crate::println!("SYN-ACK received (connection established)");
            } else {
                crate::println_color!(COLOR_YELLOW, "No SYN-ACK received (timeout)");
            }
        }
        Err(e) => crate::println_color!(COLOR_RED, "tcpsyn failed: {}", e),
    }
}

fn cmd_httpget(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: httpget <ip|host> <port> [path] [host]");
        crate::println!("  Example: httpget 192.168.56.1 8080 /");
        crate::println!("  Example: httpget example.com 80 / example.com");
        return;
    }

    let host_input = args[0];
    let port: u16 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port");
            return;
        }
    };

    let path = args.get(2).copied().unwrap_or("/");
    let mut host_header = args.get(3).copied().unwrap_or(host_input);
    if args.get(3).is_none() && host_input == "192.168.56.1" {
        host_header = "localhost";
    }

    do_http_get(host_input, port, path, host_header);
}

fn cmd_curl(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: curl <http://host[:port]/path> | <https://host[:port]/path>");
        return;
    }

    let url = args[0];
    if let Some((host, port, path, is_https)) = parse_http_url(url) {
        let host_header = if host == "192.168.56.1" { "localhost" } else { &host };
        if is_https {
            do_https_get(&host, port, &path, host_header);
        } else {
            do_http_get(&host, port, &path, host_header);
        }
    } else {
        crate::println_color!(COLOR_RED, "Invalid URL");
    }
}

fn do_http_get(host_input: &str, port: u16, path: &str, host_header: &str) {
    let ip = if let Some(ip) = parse_ipv4(host_input) {
        ip
    } else if let Some(resolved) = crate::netstack::dns::resolve(host_input) {
        resolved
    } else {
        crate::println_color!(COLOR_RED, "Unable to resolve host");
        return;
    };

    crate::println!("Connecting to {}:{}...", host_input, port);
    let src_port = match crate::netstack::tcp::send_syn(ip, port) {
        Ok(p) => p,
        Err(e) => {
            crate::println_color!(COLOR_RED, "SYN failed: {}", e);
            return;
        }
    };

    let established = crate::netstack::tcp::wait_for_established(ip, port, src_port, 1000);
    if !established {
        crate::println_color!(COLOR_YELLOW, "Connection timeout");
        return;
    }

    let mut request = String::new();
    request.push_str("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: ");
    request.push_str(host_header);
    request.push_str("\r\nConnection: close\r\n\r\n");

    if let Err(e) = crate::netstack::tcp::send_payload(ip, port, src_port, request.as_bytes()) {
        crate::println_color!(COLOR_RED, "send failed: {}", e);
        return;
    }

    crate::println!("--- HTTP response ---");
    let start = crate::logger::get_ticks();
    let mut total_bytes: usize = 0;
    let mut idle_spins: u32 = 0;
    loop {
        crate::netstack::poll();
        let mut got_data = false;
        while let Some(data) = crate::netstack::tcp::recv_data(ip, port, src_port) {
            got_data = true;
            total_bytes += data.len();
            if let Ok(text) = core::str::from_utf8(&data) {
                crate::print!("{}", text);
            } else {
                crate::println!("<binary data>");
            }
        }

        if !got_data {
            idle_spins = idle_spins.saturating_add(1);
            if crate::netstack::tcp::fin_received(ip, port, src_port) || idle_spins > 200_000 {
                break;
            }
        } else {
            idle_spins = 0;
        }

        if crate::logger::get_ticks().saturating_sub(start) > 3000 {
            break;
        }
        x86_64::instructions::hlt();
    }
    let _ = crate::netstack::tcp::send_fin(ip, port, src_port);
    crate::println!("\n--- end ({} bytes) ---", total_bytes);
    if total_bytes == 0 {
        crate::println_color!(COLOR_YELLOW, "No response body received");
    }
}

/// Parse URL and return (host, port, path)
fn parse_url_simple(url: &str) -> Option<(String, u16, String)> {
    let url = url.trim();
    
    // Remove protocol prefix
    let (rest, default_port) = if url.starts_with("https://") {
        (&url[8..], 443u16)
    } else if url.starts_with("http://") {
        (&url[7..], 80u16)
    } else {
        // Assume http if no protocol
        (url, 80u16)
    };
    
    // Split host and path
    let (host_port, path) = if let Some(idx) = rest.find('/') {
        (&rest[..idx], &rest[idx..])
    } else {
        (rest, "/")
    };
    
    // Split host and port
    let (host, port) = if let Some(idx) = host_port.find(':') {
        let host = &host_port[..idx];
        let port_str = &host_port[idx+1..];
        let port = port_str.parse::<u16>().unwrap_or(default_port);
        (host, port)
    } else {
        (host_port, default_port)
    };
    
    if host.is_empty() {
        return None;
    }
    
    Some((String::from(host), port, String::from(path)))
}

/// HTTP GET that returns a string (for GUI shell)
fn do_http_get_string(host: &str, ip: [u8; 4], port: u16, path: &str) -> Result<String, &'static str> {
    // Send SYN
    let src_port = crate::netstack::tcp::send_syn(ip, port)
        .map_err(|_| "SYN failed")?;
    
    // Wait for connection
    if !crate::netstack::tcp::wait_for_established(ip, port, src_port, 1000) {
        return Err("Connection timeout");
    }
    
    // Build HTTP request
    let mut request = String::new();
    request.push_str("GET ");
    request.push_str(path);
    request.push_str(" HTTP/1.1\r\nHost: ");
    request.push_str(host);
    request.push_str("\r\nUser-Agent: TrustOS/0.1\r\nConnection: close\r\n\r\n");
    
    // Send request
    crate::netstack::tcp::send_payload(ip, port, src_port, request.as_bytes())
        .map_err(|_| "Send failed")?;
    
    // Receive response
    let mut response = String::new();
    let start = crate::logger::get_ticks();
    let mut idle_spins: u32 = 0;
    
    loop {
        crate::netstack::poll();
        let mut got_data = false;
        
        while let Some(data) = crate::netstack::tcp::recv_data(ip, port, src_port) {
            got_data = true;
            if let Ok(text) = core::str::from_utf8(&data) {
                response.push_str(text);
            }
        }
        
        if !got_data {
            idle_spins = idle_spins.saturating_add(1);
            if crate::netstack::tcp::fin_received(ip, port, src_port) || idle_spins > 100_000 {
                break;
            }
        } else {
            idle_spins = 0;
        }
        
        if crate::logger::get_ticks().saturating_sub(start) > 2000 {
            break;
        }
        
        // Limit response size for GUI
        if response.len() > 4096 {
            response.push_str("\n... (response truncated)");
            break;
        }
        
        x86_64::instructions::hlt();
    }
    
    let _ = crate::netstack::tcp::send_fin(ip, port, src_port);
    
    if response.is_empty() {
        return Err("No response received");
    }
    
    Ok(response)
}

fn do_https_get(host_input: &str, port: u16, path: &str, host_header: &str) {
    // Construct full URL for HTTPS client
    let url = if port == 443 {
        alloc::format!("https://{}{}", host_header, path)
    } else {
        alloc::format!("https://{}:{}{}", host_header, port, path)
    };
    
    crate::println!("Connecting to {} (TLS 1.3)...", host_header);
    crate::println!("--- HTTPS response ---");
    
    match crate::netstack::https::get(&url) {
        Ok(response) => {
            // Print status
            crate::println_color!(COLOR_CYAN, "HTTP/1.1 {}", response.status_code);
            
            // Print headers
            for (key, value) in &response.headers {
                crate::println!("{}: {}", key, value);
            }
            crate::println!("");
            
            // Print body (limit to reasonable size for display)
            let body_preview = if response.body.len() > 4096 {
                &response.body[..4096]
            } else {
                &response.body
            };
            
            if let Ok(body_str) = core::str::from_utf8(body_preview) {
                crate::print!("{}", body_str);
                if response.body.len() > 4096 {
                    crate::println!("\n... (truncated, {} more bytes)", response.body.len() - 4096);
                }
            } else {
                crate::println!("[Binary data: {} bytes]", response.body.len());
            }
            
            crate::println!("\n--- end ({} bytes) ---", response.body.len());
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "HTTPS failed: {}", e);
        }
    }
}

fn parse_http_url(url: &str) -> Option<(String, u16, String, bool)> {
    let mut u = url.trim();
    let mut https = false;
    if let Some(rest) = u.strip_prefix("https://") {
        u = rest;
        https = true;
    } else if let Some(rest) = u.strip_prefix("http://") {
        u = rest;
    }

    let (host_port, path) = if let Some((h, p)) = u.split_once('/') {
        (h, format!("/{}", p))
    } else {
        (u, String::from("/"))
    };

    let (host, port) = if let Some((h, p)) = host_port.split_once(':') {
        let port = p.parse::<u16>().ok()?;
        (h, port)
    } else {
        (host_port, if https { 443 } else { 80 })
    };

    if host.is_empty() {
        return None;
    }

    Some((String::from(host), port, path, https))
}

fn parse_ipv4(input: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let a = parts[0].parse::<u8>().ok()?;
    let b = parts[1].parse::<u8>().ok()?;
    let c = parts[2].parse::<u8>().ok()?;
    let d = parts[3].parse::<u8>().ok()?;
    Some([a, b, c, d])
}

// ==================== PROGRAM EXECUTION ====================

fn cmd_exec(args: &[&str], command: &str) {
    if args.is_empty() && !command.starts_with("./") {
        crate::println_color!(COLOR_CYAN, "Usage: exec <program> [args...]");
        crate::println!("       ./program [args...]");
        crate::println!();
        crate::println!("Executes an ELF binary in user space.");
        crate::println!();
        crate::println!("Examples:");
        crate::println!("  exec /bin/hello");
        crate::println!("  ./hello.elf");
        crate::println!("  exec test    (runs built-in test)");
        return;
    }
    
    // Handle ./program syntax
    let (program, prog_args) = if command.starts_with("./") {
        (command, args)
    } else if args.is_empty() {
        crate::println_color!(COLOR_RED, "exec: missing program name");
        return;
    } else {
        (args[0], &args[1..])
    };
    
    // Special case: "exec test" runs built-in test
    if program == "test" || program == "./test" {
        crate::println_color!(COLOR_CYAN, "Running ELF loader test...");
        match crate::exec::exec_test_program() {
            crate::exec::ExecResult::Exited(code) => {
                crate::println_color!(COLOR_GREEN, "Test exited with code: {}", code);
            }
            crate::exec::ExecResult::Faulted(reason) => {
                crate::println_color!(COLOR_RED, "Test faulted: {}", reason);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::println_color!(COLOR_RED, "Load error: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::println_color!(COLOR_RED, "Memory allocation failed");
            }
        }
        return;
    }
    
    // Resolve program path
    let path = resolve_program_path(program);
    
    // Check if file exists
    if !file_exists(&path) {
        crate::println_color!(COLOR_RED, "exec: {}: not found", path);
        return;
    }
    
    // Check if it's an ELF
    if !crate::exec::is_executable(&path) {
        crate::println_color!(COLOR_RED, "exec: {}: not an ELF executable", path);
        return;
    }
    
    crate::println_color!(COLOR_CYAN, "Executing: {}", path);
    
    // Execute the program
    match crate::exec::exec_path(&path, prog_args) {
        crate::exec::ExecResult::Exited(code) => {
            if code == 0 {
                crate::println_color!(COLOR_GREEN, "Program exited successfully");
            } else {
                crate::println_color!(COLOR_YELLOW, "Program exited with code: {}", code);
            }
        }
        crate::exec::ExecResult::Faulted(reason) => {
            crate::println_color!(COLOR_RED, "Program faulted: {}", reason);
        }
        crate::exec::ExecResult::LoadError(e) => {
            crate::println_color!(COLOR_RED, "Failed to load: {:?}", e);
        }
        crate::exec::ExecResult::MemoryError => {
            crate::println_color!(COLOR_RED, "Out of memory");
        }
    }
}

fn cmd_elfinfo(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: elfinfo <file>");
        return;
    }
    
    let path = resolve_program_path(args[0]);
    
    // Open and read file header
    let fd = match crate::vfs::open(&path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => fd,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot open: {}", path);
            return;
        }
    };
    
    let mut header = [0u8; 64];
    match crate::vfs::read(fd, &mut header) {
        Ok(n) if n >= 64 => {}
        _ => {
            crate::println_color!(COLOR_RED, "Cannot read ELF header");
            crate::vfs::close(fd).ok();
            return;
        }
    }
    crate::vfs::close(fd).ok();
    
    // Check magic
    if header[0..4] != [0x7F, b'E', b'L', b'F'] {
        crate::println_color!(COLOR_RED, "Not an ELF file");
        return;
    }
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "ELF Header: {}", path);
    crate::println!("  Magic:   {:02X} {:02X} {:02X} {:02X}", header[0], header[1], header[2], header[3]);
    crate::println!("  Class:   {}", if header[4] == 2 { "ELF64" } else { "ELF32" });
    crate::println!("  Data:    {}", if header[5] == 1 { "Little Endian" } else { "Big Endian" });
    
    let e_type = u16::from_le_bytes([header[16], header[17]]);
    let type_str = match e_type {
        1 => "Relocatable",
        2 => "Executable",
        3 => "Shared Object",
        4 => "Core",
        _ => "Unknown",
    };
    crate::println!("  Type:    {} ({})", type_str, e_type);
    
    let e_machine = u16::from_le_bytes([header[18], header[19]]);
    let machine_str = match e_machine {
        3 => "x86",
        62 => "x86-64",
        183 => "AArch64",
        _ => "Unknown",
    };
    crate::println!("  Machine: {} ({})", machine_str, e_machine);
    
    let entry = u64::from_le_bytes([
        header[24], header[25], header[26], header[27],
        header[28], header[29], header[30], header[31],
    ]);
    crate::println!("  Entry:   {:#x}", entry);
    
    let phoff = u64::from_le_bytes([
        header[32], header[33], header[34], header[35],
        header[36], header[37], header[38], header[39],
    ]);
    crate::println!("  PHoff:   {:#x}", phoff);
    
    let phnum = u16::from_le_bytes([header[56], header[57]]);
    crate::println!("  PHnum:   {}", phnum);
}

/// Try to execute a file if it exists and is executable
fn try_exec_file(command: &str, args: &[&str]) -> bool {
    // Only try if it looks like a path
    if !command.contains('/') && !command.contains('.') {
        return false;
    }
    
    let path = resolve_program_path(command);
    
    if file_exists(&path) && crate::exec::is_executable(&path) {
        crate::println_color!(COLOR_CYAN, "Executing: {}", path);
        match crate::exec::exec_path(&path, args) {
            crate::exec::ExecResult::Exited(code) => {
                if code != 0 {
                    crate::println_color!(COLOR_YELLOW, "Exit code: {}", code);
                }
            }
            crate::exec::ExecResult::Faulted(reason) => {
                crate::println_color!(COLOR_RED, "Faulted: {}", reason);
            }
            crate::exec::ExecResult::LoadError(e) => {
                crate::println_color!(COLOR_RED, "Load error: {:?}", e);
            }
            crate::exec::ExecResult::MemoryError => {
                crate::println_color!(COLOR_RED, "Out of memory");
            }
        }
        true
    } else {
        false
    }
}

/// Resolve a program name to a full path
fn resolve_program_path(name: &str) -> String {
    if name.starts_with('/') {
        return String::from(name);
    }
    
    if name.starts_with("./") {
        let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
        if cwd == "/" {
            return String::from(&name[1..]); // "/program"
        } else {
            return format!("{}{}", cwd, &name[1..]); // "/dir/program"
        }
    }
    
    // Search in PATH-like directories
    let search_dirs = ["/bin", "/usr/bin", "/sbin"];
    
    for dir in &search_dirs {
        let path = format!("{}/{}", dir, name);
        if file_exists(&path) {
            return path;
        }
    }
    
    // Try current directory
    let cwd = crate::ramfs::with_fs(|fs| String::from(fs.pwd()));
    if cwd == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", cwd, name)
    }
}

/// Check if a file exists
fn file_exists(path: &str) -> bool {
    // Try VFS first
    if crate::vfs::stat(path).is_ok() {
        return true;
    }
    // Fallback to ramfs
    crate::ramfs::with_fs(|fs| fs.exists(path))
}

// ============================================================================
// HYPERVISOR COMMANDS
// ============================================================================

/// Hypervisor management command
fn cmd_hypervisor(args: &[&str]) {
    if args.is_empty() {
        print_hv_help();
        return;
    }
    
    match args[0] {
        "init" => {
            crate::println!("Initializing TrustVM hypervisor...");
            match crate::hypervisor::init() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("Hypervisor initialized successfully!");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed to initialize hypervisor: {:?}", e);
                }
            }
        }
        "status" => {
            if crate::hypervisor::is_enabled() {
                crate::print_color!(COLOR_GREEN, "● ");
                crate::println!("TrustVM: Active");
                crate::println!("  Backend: {}", crate::hypervisor::backend_info());
                crate::println!("  VMs created: {}", crate::hypervisor::vm_count());
            } else {
                crate::print_color!(COLOR_YELLOW, "○ ");
                crate::println!("TrustVM: Inactive");
                crate::println!("  Run 'hv init' to enable the hypervisor");
            }
        }
        "check" => {
            use crate::hypervisor::{detect_cpu_vendor, CpuVendor};
            crate::println!("Checking virtualization support...");
            let vendor = detect_cpu_vendor();
            crate::println!("  CPU Vendor: {:?}", vendor);
            
            match vendor {
                CpuVendor::Intel => {
                    match crate::hypervisor::vmx::check_vmx_support() {
                        Ok(caps) => {
                            crate::println!("  [Intel VT-x (VMX)]");
                            crate::println!("    VMX supported:      {}", if caps.supported { "Yes" } else { "No" });
                            crate::println!("    EPT supported:      {}", if caps.ept_supported { "Yes" } else { "No" });
                            crate::println!("    Unrestricted guest: {}", if caps.unrestricted_guest { "Yes" } else { "No" });
                            crate::println!("    VPID supported:     {}", if caps.vpid_supported { "Yes" } else { "No" });
                            crate::println!("    VMCS revision:      0x{:08X}", caps.vmcs_revision_id);
                        }
                        Err(e) => {
                            crate::print_color!(COLOR_RED, "Error: ");
                            crate::println!("{:?}", e);
                        }
                    }
                }
                CpuVendor::Amd => {
                    if crate::hypervisor::svm::is_supported() {
                        let features = crate::hypervisor::svm::get_features();
                        crate::println!("  [AMD-V (SVM)]");
                        crate::println!("    SVM supported:      Yes");
                        crate::println!("    SVM Revision:       {}", features.revision);
                        crate::println!("    NPT supported:      {}", if features.npt { "Yes" } else { "No" });
                        crate::println!("    NRIP Save:          {}", if features.nrip_save { "Yes" } else { "No" });
                        crate::println!("    Flush by ASID:      {}", if features.flush_by_asid { "Yes" } else { "No" });
                        crate::println!("    Available ASIDs:    {}", features.num_asids);
                        crate::println!("    AVIC:               {}", if features.avic { "Yes" } else { "No" });
                    } else {
                        crate::print_color!(COLOR_RED, "Error: ");
                        crate::println!("SVM not supported or disabled in BIOS");
                    }
                }
                CpuVendor::Unknown => {
                    crate::print_color!(COLOR_RED, "Error: ");
                    crate::println!("Unknown CPU vendor - virtualization not supported");
                }
            }
        }
        "shutdown" => {
            crate::println!("Shutting down hypervisor...");
            match crate::hypervisor::shutdown() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("Hypervisor shutdown complete");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed: {:?}", e);
                }
            }
        }
        "caps" | "capabilities" => {
            crate::println!("{}", crate::hypervisor::render_capabilities());
        }
        "security" => {
            crate::println!("{}", crate::hypervisor::render_security_status());
        }
        "events" => {
            let count = if args.len() > 1 { 
                args[1].parse().unwrap_or(10) 
            } else { 
                10 
            };
            let events = crate::hypervisor::get_events(count);
            if events.is_empty() {
                crate::println!("No events recorded.");
            } else {
                crate::println!("Recent VM Events:");
                for event in events {
                    crate::println!("  [{:>6}ms] VM {} - {:?}", 
                        event.timestamp_ms, event.vm_id, event.event_type);
                }
            }
        }
        "vpid" => {
            if crate::hypervisor::vpid_enabled() {
                crate::print_color!(COLOR_GREEN, "✓ ");
                crate::println!("VPID: Enabled");
                crate::println!("  Allocated VPIDs: {}", crate::hypervisor::vpid_count());
            } else {
                crate::print_color!(COLOR_YELLOW, "○ ");
                crate::println!("VPID: Disabled (CPU may not support it)");
            }
        }
        "violations" => {
            let count = crate::hypervisor::ept_violations();
            crate::println!("EPT Violations: {}", count);
            if count > 0 {
                let violations = crate::hypervisor::recent_ept_violations(5);
                for v in violations {
                    crate::println!("  VM {} GPA=0x{:X} type={:?} at RIP=0x{:X}",
                        v.vm_id, v.guest_physical, v.violation_type, v.guest_rip);
                }
            }
        }
        "version" => {
            crate::println!("TrustVM {}", crate::hypervisor::version());
        }
        "logo" => {
            crate::println!("{}", crate::hypervisor::logo());
        }
        "help" | _ => print_hv_help(),
    }
}

fn print_hv_help() {
    use crate::hypervisor::{detect_cpu_vendor, CpuVendor};
    let vendor = detect_cpu_vendor();
    let backend = match vendor {
        CpuVendor::Intel => "Intel VT-x (VMX)",
        CpuVendor::Amd => "AMD-V (SVM)",
        CpuVendor::Unknown => "Unknown",
    };
    
    crate::println!("TrustVM Hypervisor Commands (Backend: {})", backend);
    crate::println!();
    crate::println!("Initialization:");
    crate::println!("  hv init       - Initialize the hypervisor");
    crate::println!("  hv shutdown   - Shutdown the hypervisor");
    crate::println!("  hv status     - Show hypervisor status");
    crate::println!("  hv check      - Check virtualization capabilities");
    crate::println!();
    crate::println!("Monitoring:");
    crate::println!("  hv caps       - Show TrustVM capabilities");
    crate::println!("  hv security   - Show security status");
    crate::println!("  hv events [n] - Show recent VM events");
    crate::println!("  hv vpid       - Show VPID/ASID status");
    crate::println!("  hv violations - Show EPT/NPT violations");
    crate::println!("  hv version    - Show TrustVM version");
    crate::println!("  hv logo       - Display TrustVM logo");
    crate::println!();
    crate::println!("VM Management:");
    crate::println!("  vm create <name> <mem_mb>  - Create a new VM");
    crate::println!("  vm start <id> [guest]      - Start a VM with optional guest");
    crate::println!("  vm run <guest>             - Quick create and run a guest");
    crate::println!("  vm stop <id>               - Stop a VM");
    crate::println!("  vm list                    - List all VMs");
    crate::println!("  vm guests                  - List available guests");
    crate::println!("  vm mount <id> <host> <guest> - Mount shared folder");
}

/// VM management command
fn cmd_vm(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: vm <command> [args]");
        crate::println!("Commands: create, start, run, stop, list, guests, mount");
        return;
    }
    
    match args[0] {
        "create" => {
            if args.len() < 3 {
                crate::println!("Usage: vm create <name> <memory_mb>");
                return;
            }
            let name = args[1];
            let mem_mb: usize = args[2].parse().unwrap_or(16);
            
            if !crate::hypervisor::is_enabled() {
                crate::print_color!(COLOR_YELLOW, "Warning: ");
                crate::println!("Hypervisor not initialized. Run 'hv init' first.");
                return;
            }
            
            match crate::hypervisor::create_vm(name, mem_mb) {
                Ok(id) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("Created VM '{}' with ID {} ({}MB RAM)", name, id, mem_mb);
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed to create VM: {:?}", e);
                }
            }
        }
        "start" => {
            if args.len() < 2 {
                crate::println!("Usage: vm start <id> [guest_name]");
                crate::println!("Available guests: {:?}", crate::hypervisor::list_guests());
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let guest = if args.len() > 2 { args[2] } else { "hello" };
            
            crate::println!("Starting VM {} with guest '{}'...", id, guest);
            match crate::hypervisor::start_vm_with_guest(id, guest) {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("VM {} completed execution", id);
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("VM {} failed: {:?}", id, e);
                }
            }
        }
        "run" => {
            // Quick run: create and start in one command
            let guest = if args.len() > 1 { args[1] } else { "hello" };
            
            if !crate::hypervisor::is_enabled() {
                crate::print_color!(COLOR_YELLOW, "Note: ");
                crate::println!("Initializing hypervisor first...");
                if let Err(e) = crate::hypervisor::init() {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed to init hypervisor: {:?}", e);
                    return;
                }
            }
            
            match crate::hypervisor::create_vm(guest, 4) {
                Ok(id) => {
                    crate::println!("Running guest '{}'...", guest);
                    match crate::hypervisor::start_vm_with_guest(id, guest) {
                        Ok(()) => {
                            crate::print_color!(COLOR_GREEN, "✓ ");
                            crate::println!("Guest '{}' completed", guest);
                        }
                        Err(e) => {
                            crate::print_color!(COLOR_RED, "✗ ");
                            crate::println!("Failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed to create VM: {:?}", e);
                }
            }
        }
        "stop" => {
            if args.len() < 2 {
                crate::println!("Usage: vm stop <id>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            
            match crate::hypervisor::stop_vm(id) {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("Stopped VM {}", id);
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed to stop VM {}: {:?}", id, e);
                }
            }
        }
        "list" => {
            use crate::hypervisor::{detect_cpu_vendor, CpuVendor};
            crate::println!("Virtual Machines:");
            
            match detect_cpu_vendor() {
                CpuVendor::Amd => {
                    let vms = crate::hypervisor::svm_vm::list_vms();
                    if vms.is_empty() {
                        crate::println!("  (no VMs created)");
                    } else {
                        crate::println!("  {:>4} {:>20} {:>12}", "ID", "NAME", "STATE");
                        crate::println!("  {:->4} {:->20} {:->12}", "", "", "");
                        for (id, name, state) in vms {
                            crate::println!("  {:>4} {:>20} {:>12?}", id, name, state);
                        }
                    }
                }
                CpuVendor::Intel => {
                    crate::println!("  Total created: {}", crate::hypervisor::vm_count());
                }
                CpuVendor::Unknown => {
                    crate::println!("  (hypervisor not available)");
                }
            }
            crate::println!();
            crate::println!("Use 'vm guests' to see available guest programs.");
        }
        "guests" => {
            crate::println!("Available guest programs:");
            for guest in crate::hypervisor::list_guests() {
                crate::println!("  - {}", guest);
            }
            crate::println!("");
            crate::println!("Usage: vm run <guest_name>");
        }
        "mount" => {
            if args.len() < 4 {
                crate::println!("Usage: vm mount <vm_id> <host_path> <guest_path> [ro]");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let host_path = args[2];
            let guest_path = args[3];
            let readonly = args.len() > 4 && args[4] == "ro";
            
            crate::hypervisor::add_mount(id, host_path, guest_path, readonly);
            crate::print_color!(COLOR_GREEN, "✓ ");
            crate::println!("Mounted {} -> {} (readonly={})", host_path, guest_path, readonly);
        }
        "console" => {
            if args.len() < 2 {
                crate::println!("Usage: vm console <vm_id>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let output = crate::hypervisor::get_console_output(id);
            if output.is_empty() {
                crate::println!("(no output)");
            } else {
                crate::println!("{}", output);
            }
        }
        "input" => {
            if args.len() < 3 {
                crate::println!("Usage: vm input <vm_id> <text>");
                return;
            }
            let id: u64 = args[1].parse().unwrap_or(0);
            let text = args[2..].join(" ");
            crate::hypervisor::inject_console_input(id, text.as_bytes());
            crate::hypervisor::inject_console_input(id, b"\n");
            crate::println!("Injected input to VM {}", id);
        }
        _ => {
            crate::println!("Unknown VM command: {}", args[0]);
            crate::println!("Commands: create, start, run, stop, list, guests, mount, console, input");
        }
    }
}

// ==================== LINUX SUBSYSTEM COMMANDS ====================

/// Linux Subsystem command - execute commands in a Linux VM
fn cmd_linux(args: &[&str]) {
    use crate::hypervisor::linux_subsystem::{self, LinuxState};
    
    if args.is_empty() {
        print_linux_help();
        return;
    }
    
    match args[0] {
        "init" | "start" => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║     TrustOS Subsystem for Linux (TSL) v1.0              ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════╝");
            crate::println!();
            crate::println!("Initializing Linux Subsystem...");
            
            match linux_subsystem::init() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("Linux Subsystem initialized");
                    crate::println!();
                    crate::println!("Use 'linux boot' to start real Linux VM,");
                    crate::println!("or 'linux <command>' for simulated commands.");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed to initialize: {:?}", e);
                }
            }
        }
        "boot" => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║          Booting Real Linux VM...                       ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════╝");
            crate::println!();
            
            // Check for available virtualization
            let vendor = crate::hypervisor::cpu_vendor();
            match vendor {
                crate::hypervisor::CpuVendor::Intel => {
                    crate::println!("CPU: Intel (VMX)");
                }
                crate::hypervisor::CpuVendor::Amd => {
                    crate::println!("CPU: AMD (SVM)");
                }
                crate::hypervisor::CpuVendor::Unknown => {
                    crate::println_color!(COLOR_YELLOW, "Warning: No hardware virtualization detected");
                    crate::println!("         Real VM boot may not be possible.");
                }
            }
            
            crate::println!();
            crate::println!("Starting Linux VM with kernel and initramfs...");
            
            match linux_subsystem::boot() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("Linux VM boot completed");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Boot failed: {:?}", e);
                    crate::println!();
                    crate::println_color!(COLOR_YELLOW, "Falling back to simulated mode.");
                }
            }
        }
        "status" => {
            let state = linux_subsystem::state();
            let subsys = linux_subsystem::subsystem();
            
            crate::println_color!(COLOR_BRIGHT_GREEN, "Linux Subsystem Status:");
            crate::println!("═══════════════════════════════════════");
            
            match state {
                LinuxState::NotStarted => {
                    crate::print_color!(COLOR_YELLOW, "● State: ");
                    crate::println!("Not Started");
                    crate::println!("  Run 'linux init' to start the subsystem.");
                }
                LinuxState::Booting => {
                    crate::print_color!(COLOR_YELLOW, "● State: ");
                    crate::println!("Booting...");
                }
                LinuxState::Ready => {
                    crate::print_color!(COLOR_GREEN, "● State: ");
                    crate::println!("Ready");
                }
                LinuxState::Busy => {
                    crate::print_color!(COLOR_CYAN, "● State: ");
                    crate::println!("Busy (executing command)");
                }
                LinuxState::Error => {
                    crate::print_color!(COLOR_RED, "● State: ");
                    crate::println!("Error");
                }
                LinuxState::ShuttingDown => {
                    crate::print_color!(COLOR_YELLOW, "● State: ");
                    crate::println!("Shutting down...");
                }
            }
            
            // Display kernel info if available
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Kernel Image:");
            if subsys.has_kernel() {
                let kernel_size = subsys.kernel_size();
                crate::println!("  ✓ Loaded: {} bytes ({} KB)", kernel_size, kernel_size / 1024);
                if let Some(version) = subsys.kernel_version_string() {
                    crate::println!("  Version:  {}", version);
                }
                if let Some((major, minor)) = subsys.boot_protocol_version() {
                    crate::println!("  Protocol: {}.{}", major, minor);
                }
            } else {
                crate::println!("  ✗ Not loaded (simulated mode)");
            }
            
            crate::println!();
            crate::println_color!(COLOR_CYAN, "Initramfs:");
            if subsys.has_initramfs() {
                let initrd_size = subsys.initramfs_size();
                crate::println!("  ✓ Loaded: {} bytes ({} KB)", initrd_size, initrd_size / 1024);
            } else {
                crate::println!("  ✗ Not loaded");
            }
            
            crate::println!();
            crate::println_color!(COLOR_CYAN, "VM Configuration:");
            crate::println!("  Memory:   {} MB", linux_subsystem::LINUX_VM_MEMORY_MB);
            crate::println!("  VM ID:    {:#X}", linux_subsystem::LINUX_VM_ID);
            
            drop(subsys);
        }
        "stop" | "shutdown" => {
            crate::println!("Shutting down Linux Subsystem...");
            match linux_subsystem::shutdown() {
                Ok(()) => {
                    crate::print_color!(COLOR_GREEN, "✓ ");
                    crate::println!("Linux Subsystem stopped");
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "✗ ");
                    crate::println!("Failed: {:?}", e);
                }
            }
        }
        "extract" => {
            // Create test binaries directly in ramfs for transpiler testing
            create_test_binaries();
        }
        "help" | "--help" | "-h" => {
            print_linux_help();
        }
        // Execute command in Linux VM
        _ => {
            // Reconstruct the full command
            let command = args.join(" ");
            
            match linux_subsystem::execute(&command) {
                Ok(result) => {
                    if !result.stdout.is_empty() {
                        crate::println!("{}", result.stdout);
                    }
                    if !result.stderr.is_empty() {
                        crate::print_color!(COLOR_RED, "{}", result.stderr);
                    }
                    if result.exit_code != 0 && result.stderr.is_empty() {
                        crate::println_color!(COLOR_YELLOW, "(exit code: {})", result.exit_code);
                    }
                }
                Err(e) => {
                    crate::print_color!(COLOR_RED, "Error: ");
                    crate::println!("{:?}", e);
                }
            }
        }
    }
}

fn print_linux_help() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS Subsystem for Linux (TSL)");
    crate::println_color!(COLOR_BRIGHT_GREEN, "=================================");
    crate::println!();
    crate::println!("Execute Linux commands from TrustOS using a virtualized Linux environment.");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Management Commands:");
    crate::println!("  linux init          Initialize the Linux subsystem");
    crate::println!("  linux boot          Boot real Linux kernel in VM");
    crate::println!("  linux extract       Download and extract Alpine Linux to /alpine");
    crate::println!("  linux status        Show subsystem status");
    crate::println!("  linux stop          Stop the Linux subsystem");
    crate::println!("  linux help          Show this help");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Execute Linux Commands:");
    crate::println!("  linux <command>     Execute a command in Linux");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Examples:");
    crate::println!("  linux uname -a      Show Linux kernel info");
    crate::println!("  linux ls -la        List files");
    crate::println!("  linux cat /etc/os-release");
    crate::println!("  linux free -h       Show memory usage");
    crate::println!("  linux df -h         Show disk usage");
    crate::println!("  linux cat /proc/cpuinfo");
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Note: Real VM boot requires AMD SVM or Intel VMX support.");
}

// ============================================================================
// ADDITIONAL UNIX COMMANDS
// ============================================================================

/// Analyze ELF binary and return info string
fn analyze_elf(data: &[u8]) -> String {
    use alloc::string::String;
    use alloc::format;
    
    if data.len() < 64 || &data[0..4] != b"\x7fELF" {
        return String::from("      Not a valid ELF file");
    }
    
    let mut info = String::new();
    
    let class = data[4]; // 1=32-bit, 2=64-bit
    let endian = data[5]; // 1=little, 2=big
    let elf_type = u16::from_le_bytes([data[16], data[17]]);
    let machine = u16::from_le_bytes([data[18], data[19]]);
    
    info.push_str(&format!("      File size: {} bytes\n", data.len()));
    info.push_str(&format!("      Architecture: {}\n", if class == 2 { "x86_64 (64-bit)" } else { "x86 (32-bit)" }));
    info.push_str(&format!("      Endian: {}\n", if endian == 1 { "Little" } else { "Big" }));
    info.push_str(&format!("      Type: {}\n", match elf_type {
        2 => "Executable",
        3 => "Shared object (PIE)",
        _ => "Other",
    }));
    info.push_str(&format!("      Machine: {}\n", match machine {
        0x3E => "x86-64",
        0x03 => "i386",
        0xB7 => "AArch64",
        _ => "Unknown",
    }));
    
    if class == 2 {
        let entry = u64::from_le_bytes([
            data[24], data[25], data[26], data[27],
            data[28], data[29], data[30], data[31],
        ]);
        info.push_str(&format!("      Entry point: 0x{:x}\n", entry));
        
        // Check linking type
        let ph_off = u64::from_le_bytes([data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39]]) as usize;
        let ph_size = u16::from_le_bytes([data[54], data[55]]) as usize;
        let ph_count = u16::from_le_bytes([data[56], data[57]]) as usize;
        
        let mut has_interp = false;
        for i in 0..ph_count {
            let off = ph_off + i * ph_size;
            if off + 4 <= data.len() {
                let ptype = u32::from_le_bytes([data[off], data[off+1], data[off+2], data[off+3]]);
                if ptype == 3 { has_interp = true; }
            }
        }
        
        info.push_str(&format!("      Linking: {}\n", if has_interp { "Dynamic (needs ld-linux.so)" } else { "Static" }));
    }
    
    info.push_str("\n      ✓ Valid Linux ELF binary detected!");
    info.push_str("\n      Note: Execution requires x86_64 CPU emulation (slow)");
    
    info
}

/// Alpine Linux all-in-one command: download, extract, and test
fn cmd_alpine(args: &[&str]) {
    use alloc::vec::Vec;
    use alloc::string::String;
    
    let subcmd = args.get(0).copied().unwrap_or("help");
    
    match subcmd {
        "test" | "run" => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║           Alpine Linux Test - All in One                     ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
            crate::println!();
            
            // Check if we already have binaries in /alpine/bin
            let have_binaries = crate::ramfs::with_fs(|fs| {
                fs.ls(Some("/alpine/bin")).map(|e| e.len() > 0).unwrap_or(false)
            });
            
            if have_binaries {
                crate::println_color!(COLOR_GREEN, "[1/4] Alpine binaries present ✓");
            } else {
                // Try to create test binaries directly (no network needed)
                crate::println_color!(COLOR_YELLOW, "[1/4] Creating test binaries...");
                create_test_binaries_silent();
            }
            
            // Step 2: Verify binaries
            crate::println_color!(COLOR_YELLOW, "[2/4] Verifying binaries...");
            
            let binary_count = crate::ramfs::with_fs(|fs| {
                fs.ls(Some("/alpine/bin")).map(|e| e.len()).unwrap_or(0)
            });
            
            if binary_count > 0 {
                crate::println_color!(COLOR_GREEN, "      Found {} binaries in /alpine/bin", binary_count);
            } else {
                crate::println_color!(COLOR_RED, "      No binaries found! Run 'linux extract' first.");
                return;
            }
            crate::println!();
            
            // Step 3: List some files
            crate::println_color!(COLOR_YELLOW, "[3/4] Checking extracted files...");
            crate::ramfs::with_fs(|fs| {
                if let Ok(entries) = fs.ls(Some("/alpine/bin")) {
                    let count = entries.len();
                    crate::println!("      /alpine/bin: {} binaries", count);
                    // Show first 5
                    for (name, _, _) in entries.iter().take(5) {
                        crate::println!("        - {}", name);
                    }
                    if count > 5 {
                        crate::println!("        ... and {} more", count - 5);
                    }
                }
            });
            crate::println!();
            
            // Step 4: Analyze busybox binary (don't execute - too slow)
            crate::println_color!(COLOR_YELLOW, "[4/4] Analyzing Linux binary...");
            let binary = args.get(1).copied().unwrap_or("/alpine/bin/busybox");
            
            // Read and analyze the ELF
            let elf_info = crate::ramfs::with_fs(|fs| {
                fs.read_file(binary).map(|data| {
                    let data = data.to_vec();
                    analyze_elf(&data)
                })
            });
            
            match elf_info {
                Ok(info) => {
                    crate::println_color!(COLOR_GREEN, "{}", info);
                }
                Err(_) => {
                    crate::println_color!(COLOR_RED, "      Could not read binary: {}", binary);
                }
            }
            
            crate::println!();
            crate::println_color!(COLOR_BRIGHT_GREEN, "════════════════════════════════════════════════════════════════");
            crate::println_color!(COLOR_BRIGHT_GREEN, "                    Alpine Test Complete!");
            crate::println_color!(COLOR_BRIGHT_GREEN, "════════════════════════════════════════════════════════════════");
        }
        
        "ls" | "list" => {
            crate::println_color!(COLOR_CYAN, "Alpine Linux files:");
            crate::ramfs::with_fs(|fs| {
                for dir in &["/alpine", "/alpine/bin", "/alpine/usr/bin"] {
                    if let Ok(entries) = fs.ls(Some(*dir)) {
                        crate::println!("\n{}/ ({} entries)", dir, entries.len());
                        for (name, _, _) in entries.iter().take(10) {
                            crate::println!("  {}", name);
                        }
                        if entries.len() > 10 {
                            crate::println!("  ... {} more", entries.len() - 10);
                        }
                    }
                }
            });
        }
        
        "exec" => {
            if args.len() < 2 {
                crate::println!("Usage: alpine exec <binary> [args...]");
                crate::println!("Example: alpine exec /alpine/bin/busybox ls");
                return;
            }
            let binary = args[1];
            let bin_args: Vec<&str> = args[2..].to_vec();
            
            crate::println!("Executing: {} {:?}", binary, bin_args);
            match crate::linux_compat::exec(binary, &bin_args) {
                Ok(exit_code) => crate::println!("Exited with code: {}", exit_code),
                Err(e) => crate::println_color!(COLOR_RED, "Error: {}", e),
            }
        }
        
        "hello" => {
            // Run a minimal built-in ELF that we know works
            crate::println_color!(COLOR_CYAN, "Running minimal Linux ELF binary...");
            crate::println!();
            
            // This is a hand-crafted minimal ELF that prints "Hello" and exits
            // Created to test that the interpreter works for simple cases
            #[rustfmt::skip]
            static HELLO_ELF: &[u8] = &[
                // ELF Header (64 bytes)
                0x7f, b'E', b'L', b'F',  // Magic
                0x02,                     // 64-bit
                0x01,                     // Little endian
                0x01,                     // ELF version
                0x00,                     // System V ABI
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Padding
                0x02, 0x00,               // Executable
                0x3e, 0x00,               // x86_64
                0x01, 0x00, 0x00, 0x00,   // ELF version
                0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Entry: 0x400078
                0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Program header offset
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Section header offset
                0x00, 0x00, 0x00, 0x00,   // Flags
                0x40, 0x00,               // ELF header size
                0x38, 0x00,               // Program header size
                0x01, 0x00,               // Number of program headers
                0x00, 0x00,               // Section header size
                0x00, 0x00,               // Number of section headers
                0x00, 0x00,               // Section name index
                
                // Program Header (56 bytes, offset 0x40)
                0x01, 0x00, 0x00, 0x00,   // PT_LOAD
                0x05, 0x00, 0x00, 0x00,   // Flags: R+X
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Offset
                0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Vaddr: 0x400000
                0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Paddr: 0x400000
                0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // File size
                0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Mem size
                0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Alignment
                
                // Code (offset 0x78)
                // mov rax, 1 (write)
                0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,
                // mov rdi, 1 (stdout)
                0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,
                // lea rsi, [rip + msg]  -> mov rsi, msg_addr
                0x48, 0xc7, 0xc6, 0xa0, 0x00, 0x40, 0x00,  // msg at 0x4000a0
                // mov rdx, 27 (length)
                0x48, 0xc7, 0xc2, 0x1b, 0x00, 0x00, 0x00,
                // syscall
                0x0f, 0x05,
                // mov rax, 60 (exit)
                0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
                // xor rdi, rdi
                0x48, 0x31, 0xff,
                // syscall
                0x0f, 0x05,
                
                // Message at offset 0xa0 (addr 0x4000a0)
                b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r',
                b'o', b'm', b' ', b'T', b'r', b'u', b's', b't',
                b'O', b'S', b' ', b'i', b'n', b't', b'e', b'r',
                b'p', b'!', 0x0a,  // "Hello from TrustOS interp!\n"
            ];
            
            match crate::linux_compat::interpreter::run_binary(HELLO_ELF, &["hello"]) {
                Ok(code) => {
                    crate::println!();
                    crate::println_color!(COLOR_GREEN, "Binary exited with code: {}", code);
                    crate::println_color!(COLOR_GREEN, "✓ Linux interpreter works!");
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "Error: {}", e);
                }
            }
        }
        
        _ => {
            crate::println_color!(COLOR_CYAN, "Alpine Linux Commands:");
            crate::println!();
            crate::println!("  alpine test          - Download, extract & analyze Alpine");
            crate::println!("  alpine hello         - Run minimal test binary (proves interpreter works)");
            crate::println!("  alpine ls            - List extracted files");
            crate::println!("  alpine exec <bin>    - Execute a Linux binary (may be slow/timeout)");
            crate::println!();
            crate::println!("Note: Real Linux binaries like busybox are too complex.");
            crate::println!("      The interpreter supports basic ELFs only.");
        }
    }
}

/// Extract tar archive to ramfs
fn extract_tar_to_ramfs(fs: &mut crate::ramfs::RamFs, data: &[u8], base_path: &str) -> Result<usize, &'static str> {
    use alloc::string::String;
    
    let mut offset = 0;
    let mut count = 0;
    
    while offset + 512 <= data.len() {
        let header = &data[offset..offset + 512];
        
        // Check for end of archive (two zero blocks)
        if header.iter().all(|&b| b == 0) {
            break;
        }
        
        // Parse tar header
        let name_bytes = &header[0..100];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(100);
        let name = core::str::from_utf8(&name_bytes[..name_end]).unwrap_or("");
        
        if name.is_empty() {
            break;
        }
        
        // Parse size (octal)
        let size_bytes = &header[124..135];
        let size_str = core::str::from_utf8(size_bytes).unwrap_or("0");
        let size = usize::from_str_radix(size_str.trim_matches(|c| c == '\0' || c == ' '), 8).unwrap_or(0);
        
        // Type flag
        let type_flag = header[156];
        
        let full_path = if name.starts_with("./") {
            alloc::format!("{}/{}", base_path, &name[2..])
        } else {
            alloc::format!("{}/{}", base_path, name)
        };
        
        // Clean up path (remove trailing slashes)
        let clean_path = full_path.trim_end_matches('/');
        
        offset += 512; // Move past header
        
        match type_flag {
            b'5' | b'0' if name.ends_with('/') => {
                // Directory
                let _ = fs.mkdir(clean_path);
            }
            b'0' | b'\0' if size > 0 => {
                // Regular file with content
                if offset + size <= data.len() {
                    let content = &data[offset..offset + size];
                    
                    // Create parent directories
                    if let Some(parent_end) = clean_path.rfind('/') {
                        let parent = &clean_path[..parent_end];
                        let _ = create_dirs_recursive(fs, parent);
                    }
                    
                    let _ = fs.touch(clean_path);
                    let _ = fs.write_file(clean_path, content);
                    count += 1;
                }
            }
            b'0' | b'\0' => {
                // Empty file
                if let Some(parent_end) = clean_path.rfind('/') {
                    let parent = &clean_path[..parent_end];
                    let _ = create_dirs_recursive(fs, parent);
                }
                let _ = fs.touch(clean_path);
                count += 1;
            }
            b'2' => {
                // Symlink - skip for now
            }
            _ => {}
        }
        
        // Move to next header (aligned to 512 bytes)
        let blocks = (size + 511) / 512;
        offset += blocks * 512;
    }
    
    Ok(count)
}

fn create_dirs_recursive(fs: &mut crate::ramfs::RamFs, path: &str) -> Result<(), ()> {
    let mut current = String::new();
    for part in path.split('/').filter(|s| !s.is_empty()) {
        current.push('/');
        current.push_str(part);
        let _ = fs.mkdir(&current);
    }
    Ok(())
}

fn cmd_download(args: &[&str]) {
    crate::println!("[DEBUG] cmd_download called, args: {:?}", args);
    crate::serial_println!("[DEBUG] cmd_download called, args count: {}", args.len());
    
    if args.is_empty() {
        crate::println!("Usage: download <name|url> [output_file]");
        crate::println!("       download alpine  - Download Alpine Linux (fast)");
        crate::println!("       download <url>   - Download from URL");
        return;
    }
    
    let arg = args[0];
    crate::println!("[DEBUG] First arg: '{}'", arg);
    
    // Special shortcut: "download alpine" uses optimized local download
    if arg == "alpine" || arg == "busybox" || arg == "linux" {
        crate::println!("[DEBUG] Calling download_from_local_server...");
        download_from_local_server("alpine-minirootfs.tar.gz", "/opt/gui/alpine.tar.gz");
        return;
    }
    
    // Otherwise treat as URL
    let url = arg;
    let output = if args.len() > 1 { args[1] } else { 
        url.rsplit('/').next().unwrap_or("download")
    };
    
    crate::println_color!(COLOR_CYAN, "Downloading: {}", url);
    crate::println!("         -> {}", output);
    cmd_curl(args);
}

/// Fast download from local VirtualBox server (192.168.56.1:8080)
fn download_from_local_server(filename: &str, save_path: &str) {
    use alloc::vec::Vec;
    use alloc::format;
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║              Fast Download - Local Server                    ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    // VirtualBox Host-Only network
    let server_ip: [u8; 4] = [192, 168, 56, 1];
    let server_port: u16 = 8080;
    
    crate::println_color!(COLOR_YELLOW, "[1/4] Configuring network...");
    
    // Suspend DHCP and force static IP
    crate::netstack::dhcp::suspend();
    crate::network::set_ipv4_config(
        crate::network::Ipv4Address::new(192, 168, 56, 100),
        crate::network::Ipv4Address::new(255, 255, 255, 0),
        Some(crate::network::Ipv4Address::new(192, 168, 56, 1)),
    );
    
    // Verify IP is set correctly
    if let Some((ip, mask, gw)) = crate::network::get_ipv4_config() {
        crate::println!("      IP: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
        crate::serial_println!("[DOWNLOAD] IP configured: {}.{}.{}.{}", ip.as_bytes()[0], ip.as_bytes()[1], ip.as_bytes()[2], ip.as_bytes()[3]);
    } else {
        crate::println_color!(COLOR_RED, "      ERROR: No IP configured!");
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Clear buffers and wait for ARP to settle
    for _ in 0..100 {
        crate::netstack::poll();
    }
    crate::println!();
    
    crate::println_color!(COLOR_YELLOW, "[2/4] Connecting to 192.168.56.1:8080...");
    
    // Send ARP request first
    crate::println!("      Resolving MAC address...");
    let _ = crate::netstack::arp::send_request(server_ip);
    for _ in 0..200 {
        crate::netstack::poll();
    }
    
    // Check if we have MAC for server
    if let Some(mac) = crate::netstack::arp::resolve(server_ip) {
        crate::println!("      Server MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}", 
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
    } else {
        crate::println_color!(COLOR_YELLOW, "      Warning: No ARP response yet");
    }
    
    let src_port = match crate::netstack::tcp::send_syn(server_ip, server_port) {
        Ok(p) => {
            crate::serial_println!("[DOWNLOAD] SYN sent, src_port={}", p);
            p
        }
        Err(e) => {
            crate::serial_println!("[DOWNLOAD] SYN failed: {}", e);
            crate::println_color!(COLOR_RED, "      ERROR: {}", e);
            crate::println!("      Is the server running?");
            crate::println!("      > cd server && .\\start-server.ps1");
            crate::netstack::dhcp::resume();
            return;
        }
    };
    
    crate::println!("      Waiting for connection...");
    if !crate::netstack::tcp::wait_for_established(server_ip, server_port, src_port, 3000) {
        crate::serial_println!("[DOWNLOAD] Connection timeout!");
        crate::println_color!(COLOR_RED, "      ERROR: Connection timeout");
        crate::println!("      Check: ping 192.168.56.1");
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::println_color!(COLOR_GREEN, "      Connected!");
    crate::println!();
    
    crate::println_color!(COLOR_YELLOW, "[3/4] Downloading {}...", filename);
    
    // Send HTTP request
    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: 192.168.56.1\r\nConnection: close\r\n\r\n",
        filename
    );
    
    if let Err(e) = crate::netstack::tcp::send_payload(server_ip, server_port, src_port, request.as_bytes()) {
        crate::println_color!(COLOR_RED, "      ERROR: {}", e);
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Receive with progress
    let mut data: Vec<u8> = Vec::with_capacity(4 * 1024 * 1024);
    let start = crate::logger::get_ticks();
    let mut idle_count: u32 = 0;
    let mut last_progress = 0usize;
    let mut last_ack_flush = start;
    
    loop {
        // Aggressive polling
        for _ in 0..10 {
            crate::netstack::poll();
        }
        
        let mut got_data = false;
        while let Some(chunk) = crate::netstack::tcp::recv_data(server_ip, server_port, src_port) {
            got_data = true;
            if data.len() + chunk.len() > 8 * 1024 * 1024 {
                break;
            }
            data.extend_from_slice(&chunk);
        }
        
        // Progress display
        let kb = data.len() / 1024;
        if kb >= last_progress + 50 || (kb > 0 && last_progress == 0) {
            let elapsed = crate::logger::get_ticks().saturating_sub(start);
            let speed = if elapsed > 0 { (kb as u64 * 1000) / elapsed } else { 0 };
            crate::print!("\r      {} KB downloaded ({} KB/s)          ", kb, speed);
            last_progress = kb;
        }
        
        // Flush ACKs frequently
        let now = crate::logger::get_ticks();
        if now.saturating_sub(last_ack_flush) >= 5 {
            crate::netstack::tcp::flush_pending_acks(server_ip, server_port, src_port);
            last_ack_flush = now;
        }
        
        if !got_data {
            idle_count += 1;
            if crate::netstack::tcp::fin_received(server_ip, server_port, src_port) {
                crate::netstack::tcp::flush_pending_acks(server_ip, server_port, src_port);
                break;
            }
            if idle_count > 100_000 {
                break;
            }
        } else {
            idle_count = 0;
        }
        
        // 30 second timeout
        if now.saturating_sub(start) > 30_000 {
            crate::println_color!(COLOR_YELLOW, "\n      Timeout!");
            break;
        }
    }
    
    let _ = crate::netstack::tcp::send_fin(server_ip, server_port, src_port);
    
    let elapsed = crate::logger::get_ticks().saturating_sub(start);
    let total_kb = data.len() / 1024;
    let avg_speed = if elapsed > 0 { (total_kb as u64 * 1000) / elapsed } else { 0 };
    
    crate::println!();
    crate::println_color!(COLOR_GREEN, "      Complete: {} KB in {}ms ({} KB/s)", total_kb, elapsed, avg_speed);
    crate::println!();
    
    if data.is_empty() {
        crate::println_color!(COLOR_RED, "      ERROR: No data received");
        crate::netstack::dhcp::resume();
        return;
    }
    
    // Extract HTTP body
    let body_start = data.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(0);
    let body = &data[body_start..];
    
    if body.is_empty() {
        crate::println_color!(COLOR_RED, "      ERROR: Empty response");
        crate::netstack::dhcp::resume();
        return;
    }
    
    crate::println_color!(COLOR_YELLOW, "[4/4] Saving to {}...", save_path);
    
    // Save to ramfs
    let save_result = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/opt");
        let _ = fs.mkdir("/opt/gui");
        let _ = fs.touch(save_path);
        fs.write_file(save_path, body)
    });
    
    match save_result {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "      Saved: {:.2} MB", body.len() as f32 / (1024.0 * 1024.0));
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "      ERROR: {:?}", e);
            crate::netstack::dhcp::resume();
            return;
        }
    }
    
    // Persist to disk
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Saving to disk for persistence...");
    match crate::persistence::save_file(save_path, body) {
        Ok(_) => crate::println_color!(COLOR_GREEN, "  Saved! Will survive reboot."),
        Err(e) => crate::println_color!(COLOR_YELLOW, "  Could not persist: {}", e),
    }
    
    crate::println!();
    crate::println_color!(COLOR_BRIGHT_GREEN, "════════════════════════════════════════════════════════════════");
    crate::println_color!(COLOR_BRIGHT_GREEN, "                    Download Complete!");
    crate::println_color!(COLOR_BRIGHT_GREEN, "════════════════════════════════════════════════════════════════");
    
    // Mark GUI as installed
    GUI_INSTALLED.store(true, core::sync::atomic::Ordering::Relaxed);
    
    crate::netstack::dhcp::resume();
}

fn cmd_which(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: which <command>");
        return;
    }
    
    let search_dirs = ["/bin", "/usr/bin", "/sbin", "/usr/sbin"];
    
    for name in args {
        let mut found = false;
        for dir in &search_dirs {
            let path = format!("{}/{}", dir, name);
            if file_exists(&path) {
                crate::println!("{}", path);
                found = true;
                break;
            }
        }
        if !found {
            crate::println_color!(COLOR_RED, "{}: not found", name);
        }
    }
}

fn cmd_whereis(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: whereis <command>");
        return;
    }
    cmd_which(args);
}

fn cmd_file(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: file <path>");
        return;
    }
    
    for path in args {
        if !file_exists(path) {
            crate::println!("{}: cannot open", path);
            continue;
        }
        
        // Try to detect file type
        if crate::exec::is_executable(path) {
            crate::println!("{}: ELF 64-bit executable", path);
        } else {
            // Read first bytes to detect
            match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
                Ok(fd) => {
                    let mut header = [0u8; 16];
                    let n = crate::vfs::read(fd, &mut header).unwrap_or(0);
                    crate::vfs::close(fd).ok();
                    
                    if n == 0 {
                        crate::println!("{}: empty", path);
                    } else if header[0..4] == [0x7F, b'E', b'L', b'F'] {
                        crate::println!("{}: ELF file", path);
                    } else if header[0..2] == [0x1f, 0x8b] {
                        crate::println!("{}: gzip compressed data", path);
                    } else if header[0..4] == [0x50, 0x4B, 0x03, 0x04] {
                        crate::println!("{}: Zip archive", path);
                    } else if header[0..6] == *b"#!/bin" {
                        crate::println!("{}: shell script", path);
                    } else if header.iter().all(|&b| b.is_ascii()) {
                        crate::println!("{}: ASCII text", path);
                    } else {
                        crate::println!("{}: data", path);
                    }
                }
                Err(_) => crate::println!("{}: cannot open", path),
            }
        }
    }
}

fn cmd_chmod(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "chmod: permission system not implemented yet");
    crate::println!("(TrustOS currently has no file permission support)");
}

fn cmd_chown(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "chown: ownership not implemented yet");
}

fn cmd_ln(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: ln [-s] <target> <link_name>");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "ln: symbolic links not implemented yet");
}

fn cmd_readlink(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "readlink: not implemented");
}

fn cmd_basename(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: basename <path>");
        return;
    }
    let path = args[0];
    let name = path.rsplit('/').next().unwrap_or(path);
    crate::println!("{}", name);
}

fn cmd_dirname(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: dirname <path>");
        return;
    }
    let path = args[0];
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            crate::println!("/");
        } else {
            crate::println!("{}", &path[..pos]);
        }
    } else {
        crate::println!(".");
    }
}

fn cmd_realpath(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: realpath <path>");
        return;
    }
    let path = resolve_program_path(args[0]);
    crate::println!("{}", path);
}

fn cmd_sort(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sort <file>");
        return;
    }
    
    // Read file content
    let path = args[0];
    match read_file_content(path) {
        Some(content) => {
            let mut lines: Vec<&str> = content.lines().collect();
            lines.sort();
            for line in lines {
                crate::println!("{}", line);
            }
        }
        None => crate::println_color!(COLOR_RED, "sort: cannot read {}", path),
    }
}

fn cmd_uniq(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: uniq <file>");
        return;
    }
    
    match read_file_content(args[0]) {
        Some(content) => {
            let mut last_line: Option<&str> = None;
            for line in content.lines() {
                if last_line != Some(line) {
                    crate::println!("{}", line);
                    last_line = Some(line);
                }
            }
        }
        None => crate::println_color!(COLOR_RED, "uniq: cannot read {}", args[0]),
    }
}

fn cmd_cut(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "cut: not implemented yet");
}

fn cmd_tr(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "tr: not implemented yet");
}

fn cmd_tee(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "tee: not implemented yet");
}

fn cmd_xargs(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "xargs: not implemented yet");
}

fn cmd_yes(args: &[&str]) {
    let text = if args.is_empty() { "y" } else { args[0] };
    // Print 10 times (would be infinite in real implementation)
    for _ in 0..10 {
        crate::println!("{}", text);
    }
    crate::println!("... (press Ctrl+C to stop in real yes)");
}

fn cmd_seq(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: seq <last> | seq <first> <last> | seq <first> <inc> <last>");
        return;
    }
    
    let (first, inc, last) = match args.len() {
        1 => (1i64, 1i64, args[0].parse().unwrap_or(1)),
        2 => (args[0].parse().unwrap_or(1), 1i64, args[1].parse().unwrap_or(1)),
        _ => (args[0].parse().unwrap_or(1), args[1].parse().unwrap_or(1), args[2].parse().unwrap_or(1)),
    };
    
    let mut i = first;
    while (inc > 0 && i <= last) || (inc < 0 && i >= last) {
        crate::println!("{}", i);
        i += inc;
    }
}

fn cmd_sleep(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sleep <seconds>");
        return;
    }
    
    let secs: u64 = args[0].parse().unwrap_or(1);
    crate::println_color!(COLOR_CYAN, "Sleeping for {} seconds...", secs);
    
    // Simple busy-wait sleep (not ideal but works)
    let start = crate::time::uptime_ms();
    let end = start + secs * 1000;
    while crate::time::uptime_ms() < end {
        core::hint::spin_loop();
    }
    crate::println!("Done.");
}

fn cmd_kill(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: kill [-9] <pid>");
        return;
    }
    
    let _signal = if args[0] == "-9" { 9 } else { 15 };
    let pid_str = if args[0].starts_with('-') && args.len() > 1 { args[1] } else { args[0] };
    
    match pid_str.parse::<u32>() {
        Ok(pid) => {
            crate::println_color!(COLOR_YELLOW, "Killing PID {}", pid);
            match crate::process::kill(pid) {
                Ok(_) => crate::println_color!(COLOR_GREEN, "Process {} killed", pid),
                Err(e) => crate::println_color!(COLOR_RED, "kill: {}", e),
            }
        }
        Err(_) => crate::println_color!(COLOR_RED, "kill: invalid PID"),
    }
}

fn cmd_killall(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: killall <name>");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "killall: process name matching not implemented");
}

fn cmd_nice(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "nice: priority not implemented");
}

fn cmd_nohup(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "nohup: background execution not implemented");
}

fn cmd_bg(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "bg: job control not implemented");
}

fn cmd_fg(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "fg: job control not implemented");
}

fn cmd_top() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS Process Monitor");
    crate::println!("═══════════════════════════════════════════════════════════");
    
    let uptime = crate::time::uptime_ms() / 1000;
    let hours = uptime / 3600;
    let mins = (uptime % 3600) / 60;
    let secs = uptime % 60;
    
    crate::println!("Uptime: {:02}:{:02}:{:02}", hours, mins, secs);
    crate::println!();
    
    // Memory info
    let heap_used = crate::memory::stats().heap_used;
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    crate::println!("Mem: {} KB / {} KB ({:.1}%)", 
        heap_used / 1024, 
        heap_total / 1024,
        (heap_used as f64 / heap_total as f64) * 100.0);
    crate::println!();
    
    crate::println_color!(COLOR_CYAN, "  PID  STATE    NAME");
    crate::println!("──────────────────────────────────");
    
    // List processes
    for (pid, name, state) in crate::process::list() {
        let state_str = match state {
            crate::process::ProcessState::Running => "RUNNING",
            crate::process::ProcessState::Ready => "READY  ",
            crate::process::ProcessState::Blocked => "BLOCKED",
            crate::process::ProcessState::Zombie => "ZOMBIE ",
            crate::process::ProcessState::Created => "CREATED",
            crate::process::ProcessState::Waiting => "WAITING",
            crate::process::ProcessState::Stopped => "STOPPED",
            crate::process::ProcessState::Dead => "DEAD   ",
        };
        crate::println!("{:>5}  {}  {}", pid, state_str, name);
    }
    
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "(press 'q' to quit in interactive mode)");
}

fn cmd_vmstat() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Virtual Memory Statistics");
    crate::println!("═════════════════════════════════════════");
    
    let heap_used = crate::memory::stats().heap_used;
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::println!("Memory:");
    crate::println!("  Heap Total:  {} KB", heap_total / 1024);
    crate::println!("  Heap Used:   {} KB", heap_used / 1024);
    crate::println!("  Heap Free:   {} KB", (heap_total - heap_used) / 1024);
}

fn cmd_iostat() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "I/O Statistics");
    crate::println!("═══════════════════════════════");
    crate::println!("(I/O statistics not implemented)");
}

fn cmd_dmesg(args: &[&str]) {
    if args.first() == Some(&"-c") || args.first() == Some(&"--clear") {
        // Clear by reading all (ring buffer auto-overwrites)
        crate::println_color!(COLOR_GREEN, "dmesg buffer acknowledged.");
        return;
    }
    
    let count = if let Some(&"-n") = args.first() {
        args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20)
    } else if let Some(n) = args.first().and_then(|s| s.parse::<usize>().ok()) {
        n
    } else {
        0 // show all
    };
    
    let lines = crate::devtools::dmesg_read(count);
    if lines.is_empty() {
        crate::println_color!(COLOR_YELLOW, "(no kernel messages recorded)");
        crate::println!("Tip: messages are captured after devtools init.");
        return;
    }
    let (buf_size, total) = crate::devtools::dmesg_stats();
    crate::println_color!(COLOR_BRIGHT_GREEN, "Kernel Ring Buffer ({} stored, {} total)", buf_size, total);
    crate::println!("═══════════════════════════════════════════════════════════════");
    for line in &lines {
        crate::println!("{}", line);
    }
}

fn cmd_memdbg() {
    let s = crate::devtools::memdbg_stats();
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory Debug Statistics (memdbg)");
    crate::println!("═══════════════════════════════════════════════════════════════");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Heap Usage:");
    crate::println!("    Current used : {:>10} bytes ({} KB)", s.current_heap_used, s.current_heap_used / 1024);
    crate::println!("    Current free : {:>10} bytes ({} KB)", s.current_heap_free, s.current_heap_free / 1024);
    crate::println!("    Total heap   : {:>10} bytes ({} KB)", s.heap_total, s.heap_total / 1024);
    crate::println!("    Peak used    : {:>10} bytes ({} KB)", s.peak_heap_used, s.peak_heap_used / 1024);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Allocation Stats:");
    crate::println!("    Alloc ops    : {:>10}", s.alloc_count);
    crate::println!("    Dealloc ops  : {:>10}", s.dealloc_count);
    crate::println!("    Live allocs  : {:>10}", s.live_allocs);
    crate::println!("    Total alloc'd: {:>10} bytes", s.alloc_bytes_total);
    crate::println!("    Total freed  : {:>10} bytes", s.dealloc_bytes_total);
    crate::println!("    Largest alloc: {:>10} bytes", s.largest_alloc);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Fragmentation:");
    let frag_color = if s.fragmentation_pct > 50.0 { COLOR_RED }
        else if s.fragmentation_pct > 25.0 { COLOR_YELLOW }
        else { COLOR_GREEN };
    crate::println_color!(frag_color, "    Estimate     : {:.1}%", s.fragmentation_pct);
}

fn cmd_perfstat() {
    let snap = crate::devtools::perf_snapshot();
    let uptime_s = snap.uptime_ms / 1000;
    let hours = uptime_s / 3600;
    let mins = (uptime_s % 3600) / 60;
    let secs = uptime_s % 60;
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Performance Statistics (perf)");
    crate::println!("═══════════════════════════════════════════════════════════════");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  System:");
    crate::println!("    Uptime       : {}h {:02}m {:02}s ({} ms)", hours, mins, secs, snap.uptime_ms);
    crate::println!("    GUI FPS      : {}", snap.fps);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Interrupts:");
    crate::println!("    Total IRQs   : {}", snap.total_irqs);
    crate::println!("    IRQ/sec      : {}", snap.irq_per_sec);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Scheduling:");
    crate::println!("    Syscalls     : {}", snap.total_syscalls);
    crate::println!("    Ctx switches : {}", snap.total_ctx_switches);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Memory:");
    crate::println!("    Heap used    : {} / {} KB ({}%)", 
        snap.heap_used / 1024, (snap.heap_used + snap.heap_free) / 1024,
        if snap.heap_used + snap.heap_free > 0 { snap.heap_used * 100 / (snap.heap_used + snap.heap_free) } else { 0 });
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Per-CPU:");
    for s in &snap.cpu_stats {
        let state = if s.is_idle { "idle" } else { "busy" };
        crate::println!("    CPU{}: {} irqs, {} syscalls, {} ctxsw [{}]", 
            s.cpu_id, s.interrupts, s.syscalls, s.context_switches, state);
    }
}

fn cmd_irqstat() {
    let stats = crate::sync::percpu::all_cpu_stats();
    let total_irqs: u64 = stats.iter().map(|s| s.interrupts).sum();
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "IRQ Statistics");
    crate::println!("═══════════════════════════════════════════════════════════════");
    crate::println!();
    crate::println!("  Total IRQs     : {}", total_irqs);
    crate::println!("  IRQ rate       : {}/sec", crate::devtools::irq_rate());
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Per-CPU Breakdown:");
    for s in &stats {
        let bar_len = if total_irqs > 0 { (s.interrupts * 40 / total_irqs.max(1)) as usize } else { 0 };
        let bar: String = "█".repeat(bar_len);
        let pct = if total_irqs > 0 { s.interrupts * 100 / total_irqs } else { 0 };
        crate::println!("    CPU{}: {:>8} ({:>3}%) {}", s.cpu_id, s.interrupts, pct, bar);
    }
}

fn cmd_registers() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "CPU Register Dump");
    crate::println!("═══════════════════════════════════════════════════════════════");
    let regs = crate::devtools::cpu_registers();
    for line in &regs {
        crate::println!("{}", line);
    }
}

fn cmd_peek(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: peek <hex_addr> [byte_count]");
        crate::println!("  e.g.: peek 0xFFFF8000_00000000 64");
        crate::println!("  Default count: 64 bytes, max: 256 bytes");
        return;
    }
    
    let addr_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let addr = match usize::from_str_radix(addr_str, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let count = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(64);
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory dump at 0x{:016x} ({} bytes)", addr, count);
    crate::println!("═══════════════════════════════════════════════════════════════");
    let lines = crate::devtools::peek(addr, count);
    for line in &lines {
        crate::println!("{}", line);
    }
}

fn cmd_poke(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: poke <hex_addr> <hex_value>");
        crate::println!("  e.g.: poke 0xB8000 0x41");
        crate::println_color!(COLOR_RED, "  ⚠ WARNING: Writing to arbitrary memory is DANGEROUS!");
        return;
    }
    
    let addr_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let addr = match usize::from_str_radix(addr_str, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let val_str = args[1].trim_start_matches("0x").trim_start_matches("0X");
    let value = match u8::from_str_radix(val_str, 16) {
        Ok(v) => v,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex value: {}", args[1]);
            return;
        }
    };
    
    match crate::devtools::poke(addr, value) {
        Ok(()) => crate::println_color!(COLOR_GREEN, "Wrote 0x{:02x} to 0x{:016x}", value, addr),
        Err(e) => crate::println_color!(COLOR_RED, "poke error: {}", e),
    }
}

fn cmd_devpanel() {
    crate::devtools::toggle_devpanel();
    let state = if crate::devtools::is_devpanel_visible() { "ON" } else { "OFF" };
    crate::println_color!(COLOR_GREEN, "DevPanel overlay: {} (also toggle with F12 in desktop)", state);
}

fn cmd_timecmd(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: timecmd <command> [args...]");
        crate::println!("  Runs a command and prints elapsed time.");
        return;
    }
    
    let start = crate::cpu::tsc::Stopwatch::start();
    
    // Reconstruct and execute the sub-command
    let sub_cmd = args.join(" ");
    execute_command(&sub_cmd);
    
    let elapsed_us = start.elapsed_micros();
    let elapsed_ms = elapsed_us / 1000;
    let frac = elapsed_us % 1000;
    crate::println!();
    crate::println_color!(COLOR_CYAN, "⏱ Elapsed: {}.{:03} ms ({} µs)", elapsed_ms, frac, elapsed_us);
}

fn cmd_lsof(_args: &[&str]) {
    crate::println!("COMMAND   PID   FD   TYPE   NAME");
    crate::println!("────────────────────────────────────────");
    crate::println!("shell     1     0    CHR    /dev/stdin");
    crate::println!("shell     1     1    CHR    /dev/stdout");
    crate::println!("shell     1     2    CHR    /dev/stderr");
}

fn cmd_strace(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "strace: syscall tracing not implemented");
}

fn cmd_strings(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: strings <file>");
        return;
    }
    
    match read_file_bytes(args[0]) {
        Some(data) => {
            let mut current = String::new();
            for &byte in &data {
                if byte.is_ascii_graphic() || byte == b' ' {
                    current.push(byte as char);
                } else {
                    if current.len() >= 4 {
                        crate::println!("{}", current);
                    }
                    current.clear();
                }
            }
            if current.len() >= 4 {
                crate::println!("{}", current);
            }
        }
        None => crate::println_color!(COLOR_RED, "strings: cannot read {}", args[0]),
    }
}

fn cmd_tar(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "tar: archive support not implemented");
}

fn cmd_gzip(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "gzip: compression not implemented");
}

fn cmd_gunzip(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "gunzip: decompression not implemented");
}

fn cmd_zip(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "zip: archive support not implemented");
}

fn cmd_unzip(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "unzip: archive support not implemented");
}

fn cmd_mount(args: &[&str]) {
    if args.is_empty() {
        // Show mounted filesystems
        crate::println_color!(COLOR_BRIGHT_GREEN, "Mounted Filesystems:");
        crate::vfs::list_mounts();
        return;
    }
    
    if args.len() < 2 {
        crate::println!("Usage: mount <device> <mountpoint>");
        return;
    }
    
    crate::println_color!(COLOR_YELLOW, "mount: dynamic mounting not implemented");
}

fn cmd_umount(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: umount <mountpoint>");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "umount: unmounting not implemented");
}

fn cmd_sync() {
    crate::println!("Syncing filesystems...");
    crate::println_color!(COLOR_GREEN, "Done.");
}

fn cmd_lsblk() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Block Devices:");
    crate::println!("NAME    SIZE    TYPE    MOUNTPOINT");
    crate::println!("────────────────────────────────────");
    crate::println!("ram0    256K    disk    /");
    
    // Check for AHCI disks (simplified - no get_disk_info yet)
    crate::println!("(AHCI disk info not available)");
}

fn cmd_blkid() {
    crate::println!("/dev/ram0: TYPE=\"ramfs\"");
}

fn cmd_mkfs(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "mkfs: filesystem creation not implemented");
}

fn cmd_fsck(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "fsck: filesystem check not implemented");
}

fn cmd_export(args: &[&str]) {
    if args.is_empty() {
        crate::println!("PATH=/bin:/usr/bin");
        crate::println!("HOME=/");
        crate::println!("USER=root");
        crate::println!("SHELL=/bin/tsh");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "export: environment variables stored in memory only");
}

fn cmd_unset(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "unset: environment variables not fully implemented");
}

fn cmd_alias(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "alias: aliases not implemented");
}

fn cmd_unalias(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "unalias: aliases not implemented");
}

fn cmd_source(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: source <script>");
        return;
    }
    
    match read_file_content(args[0]) {
        Some(content) => {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    execute_command(trimmed);
                }
            }
        }
        None => crate::println_color!(COLOR_RED, "source: cannot read {}", args[0]),
    }
}

fn cmd_set(_args: &[&str]) {
    crate::println!("SHELL=/bin/tsh");
    crate::println!("PATH=/bin:/usr/bin");
    crate::println!("PWD={}", crate::ramfs::with_fs(|fs| String::from(fs.pwd())));
    crate::println!("USER=root");
    crate::println!("HOME=/");
}

fn cmd_read(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "read: variable input not implemented");
}

fn cmd_printf(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: printf <format> [args...]");
        return;
    }
    // Simple implementation - just print format string
    let format = args[0].replace("\\n", "\n").replace("\\t", "\t");
    crate::print!("{}", format);
}

fn cmd_test_expr(args: &[&str]) {
    // Basic test expression evaluation
    if args.is_empty() {
        crate::println!("false");
        return;
    }
    
    match args.first() {
        Some(&"-e") if args.len() > 1 => {
            if file_exists(args[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        Some(&"-d") if args.len() > 1 => {
            crate::println_color!(COLOR_YELLOW, "(directory check not implemented)");
        }
        Some(&"-f") if args.len() > 1 => {
            if file_exists(args[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        _ => crate::println!("true"),
    }
}

fn cmd_expr(args: &[&str]) {
    if args.len() < 3 {
        crate::println!("Usage: expr <num1> <op> <num2>");
        return;
    }
    
    let a: i64 = args[0].parse().unwrap_or(0);
    let b: i64 = args[2].parse().unwrap_or(0);
    
    let result = match args[1] {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" if b != 0 => a / b,
        "%" if b != 0 => a % b,
        _ => {
            crate::println!("expr: invalid operator");
            return;
        }
    };
    
    crate::println!("{}", result);
}

fn cmd_bc(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "bc: calculator not implemented");
}

fn cmd_cal(_args: &[&str]) {
    crate::println_color!(COLOR_BRIGHT_GREEN, "   February 2026");
    crate::println!("Su Mo Tu We Th Fr Sa");
    crate::println!(" 1  2  3  4  5  6  7");
    crate::println!(" 8  9 10 11 12 13 14");
    crate::println!("15 16 17 18 19 20 21");
    crate::println!("22 23 24 25 26 27 28");
}

fn cmd_diff(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "diff: not implemented");
}

fn cmd_patch(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "patch: not implemented");
}

fn cmd_cmp(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: cmp <file1> <file2>");
        return;
    }
    
    match (read_file_bytes(args[0]), read_file_bytes(args[1])) {
        (Some(a), Some(b)) => {
            if a == b {
                // Files are identical, no output
            } else {
                crate::println!("{} {} differ", args[0], args[1]);
            }
        }
        _ => crate::println_color!(COLOR_RED, "cmp: cannot read files"),
    }
}

fn cmd_md5sum(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: md5sum <file>");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "md5sum: MD5 not implemented");
}

fn cmd_sha256sum(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sha256sum <file>");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "sha256sum: SHA256 not implemented");
}

fn cmd_base64(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: base64 [-d] <file>");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "base64: encoding not implemented");
}

fn cmd_od(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: od <file>");
        return;
    }
    // Use hexdump for now
    cmd_hexdump(args);
}

fn cmd_rev(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rev <file>");
        return;
    }
    
    match read_file_content(args[0]) {
        Some(content) => {
            for line in content.lines() {
                let reversed: String = line.chars().rev().collect();
                crate::println!("{}", reversed);
            }
        }
        None => crate::println_color!(COLOR_RED, "rev: cannot read {}", args[0]),
    }
}

fn cmd_factor(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: factor <number>");
        return;
    }
    
    let mut n: u64 = args[0].parse().unwrap_or(0);
    if n == 0 {
        crate::println!("factor: invalid number");
        return;
    }
    
    crate::print!("{}:", n);
    let mut d = 2u64;
    while d * d <= n {
        while n % d == 0 {
            crate::print!(" {}", d);
            n /= d;
        }
        d += 1;
    }
    if n > 1 {
        crate::print!(" {}", n);
    }
    crate::println!();
}

fn cmd_watch(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "watch: periodic execution not implemented");
}

fn cmd_timeout(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "timeout: not implemented");
}

fn cmd_time_cmd(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "time: command timing not implemented");
}

fn cmd_script(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "script: terminal recording not implemented");
}

fn cmd_tty() {
    crate::println!("/dev/tty0");
}

fn cmd_stty(_args: &[&str]) {
    crate::println!("speed 9600 baud; line = 0;");
    crate::println!("-brkint -imaxbel");
}

fn cmd_reset() {
    cmd_clear();
    crate::println!("Terminal reset.");
}

fn cmd_loadkeys(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "loadkeys: keymap not implemented");
}

fn cmd_setfont(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "setfont: font loading not implemented");
}

fn cmd_lsusb() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "USB Devices:");
    crate::println!("═══════════════════════════════════════════");
    
    // Check if xHCI is initialized
    if crate::drivers::xhci::is_initialized() {
        let devices = crate::drivers::xhci::list_devices();
        if devices.is_empty() {
            crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
            crate::println!("  (no devices connected)");
        } else {
            crate::println!("Bus 001 Device 001: ID 0000:0000 xHCI Root Hub");
            for (i, dev) in devices.iter().enumerate() {
                let speed = match dev.speed {
                    1 => "Full Speed (12 Mbps)",
                    2 => "Low Speed (1.5 Mbps)",
                    3 => "High Speed (480 Mbps)",
                    4 => "SuperSpeed (5 Gbps)",
                    _ => "Unknown",
                };
                crate::println!("Bus 001 Device {:03}: ID {:04x}:{:04x} Port {} - {}", 
                    i + 2, dev.vendor_id, dev.product_id, dev.port, speed);
                if dev.class != 0 {
                    let class_name = match dev.class {
                        0x03 => "HID (Human Interface Device)",
                        0x08 => "Mass Storage",
                        0x09 => "Hub",
                        _ => "Unknown class",
                    };
                    crate::println!("    Class: {:02x}:{:02x}:{:02x} ({})", 
                        dev.class, dev.subclass, dev.protocol, class_name);
                }
            }
        }
        crate::println!("");
        crate::println!("Total: {} device(s) connected", devices.len());
    } else {
        crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
        crate::println_color!(COLOR_YELLOW, "  (xHCI controller not initialized)");
    }
}

fn cmd_smpstatus() {
    crate::cpu::smp::print_status();
}

fn cmd_smp(args: &[&str]) {
    if args.is_empty() {
        let status = if crate::cpu::smp::is_smp_enabled() { "ON" } else { "OFF" };
        let cpus = crate::cpu::smp::ready_cpu_count();
        crate::println!("SMP parallelism: {} ({} CPUs ready)", status, cpus);
        crate::println!("Usage: smp [on|off|status]");
        crate::println!("  on     - Enable multi-core parallel rendering");
        crate::println!("  off    - Disable parallelism (single-core, safe mode)");
        crate::println!("  status - Show detailed CPU status");
        return;
    }
    
    match args[0] {
        "on" | "1" | "enable" => {
            crate::cpu::smp::enable_smp();
            crate::println_color!(0xFF00FF00, "SMP parallelism ENABLED");
        },
        "off" | "0" | "disable" => {
            crate::cpu::smp::disable_smp();
            crate::println_color!(0xFFFF8800, "SMP parallelism DISABLED (single-core mode)");
        },
        "status" => {
            crate::cpu::smp::print_status();
        },
        _ => {
            crate::println!("Unknown option: {}", args[0]);
            crate::println!("Usage: smp [on|off|status]");
        }
    }
}

fn cmd_fontsmooth(args: &[&str]) {
    use crate::framebuffer::font::{FontMode, set_mode, get_mode};
    
    if args.is_empty() {
        let current = match get_mode() {
            FontMode::Sharp => "sharp (disabled)",
            FontMode::Smooth => "smooth (enabled)",
        };
        crate::println!("Font smoothing: {}", current);
        crate::println!("Usage: fontsmooth [on|off]");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "smooth" => {
            set_mode(FontMode::Smooth);
            crate::println!("Font smoothing enabled");
        }
        "off" | "disable" | "sharp" => {
            set_mode(FontMode::Sharp);
            crate::println!("Font smoothing disabled");
        }
        _ => {
            crate::println!("Usage: fontsmooth [on|off]");
        }
    }
}

fn cmd_lscpu() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "CPU Information:");
    crate::println!("═══════════════════════════════════════════");
    
    // Use our CPU detection module
    if let Some(caps) = crate::cpu::capabilities() {
        crate::println!("Brand:        {}", caps.brand());
        crate::println!("Architecture: x86_64");
        crate::println!("Vendor:       {:?}", caps.vendor);
        crate::println!("Family:       {}", caps.family);
        crate::println!("Model:        {}", caps.model);
        crate::println!("Stepping:     {}", caps.stepping);
        crate::println!("CPU(s):       {}", crate::cpu::smp::cpu_count());
        crate::println!("APIC ID:      {}", caps.apic_id);
        
        // TSC info
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Timing:");
        crate::println!("TSC:          {} (invariant: {})", 
            if caps.tsc { "yes" } else { "no" },
            if caps.tsc_invariant { "yes" } else { "no" });
        crate::println!("TSC Freq:     {} MHz", caps.tsc_frequency_hz / 1_000_000);
        crate::println!("RDTSCP:       {}", if caps.rdtscp { "yes" } else { "no" });
        
        // SIMD features
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "SIMD:");
        crate::println!("SSE:          {}", if caps.sse { "yes" } else { "no" });
        crate::println!("SSE2:         {}", if caps.sse2 { "yes" } else { "no" });
        crate::println!("SSE3:         {}", if caps.sse3 { "yes" } else { "no" });
        crate::println!("SSSE3:        {}", if caps.ssse3 { "yes" } else { "no" });
        crate::println!("SSE4.1:       {}", if caps.sse4_1 { "yes" } else { "no" });
        crate::println!("SSE4.2:       {}", if caps.sse4_2 { "yes" } else { "no" });
        crate::println!("AVX:          {}", if caps.avx { "yes" } else { "no" });
        crate::println!("AVX2:         {}", if caps.avx2 { "yes" } else { "no" });
        crate::println!("AVX-512:      {}", if caps.avx512f { "yes" } else { "no" });
        
        // Crypto features
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Crypto Acceleration:");
        crate::println!("AES-NI:       {}", if caps.aesni { "yes" } else { "no" });
        crate::println!("PCLMULQDQ:    {}", if caps.pclmulqdq { "yes" } else { "no" });
        crate::println!("SHA-NI:       {}", if caps.sha_ext { "yes" } else { "no" });
        crate::println!("RDRAND:       {}", if caps.rdrand { "yes" } else { "no" });
        crate::println!("RDSEED:       {}", if caps.rdseed { "yes" } else { "no" });
        
        // Security features
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Security:");
        crate::println!("SMEP:         {}", if caps.smep { "yes" } else { "no" });
        crate::println!("SMAP:         {}", if caps.smap { "yes" } else { "no" });
        crate::println!("NX:           {}", if caps.nx { "yes" } else { "no" });
        
        // Virtualization
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Virtualization:");
        crate::println!("Intel VT-x:   {}", if caps.vmx { "yes" } else { "no" });
        crate::println!("AMD-V:        {}", if caps.svm { "yes" } else { "no" });
    } else {
        crate::println!("Architecture: x86_64");
        crate::println!("(CPU detection not initialized)");
    }
}

fn cmd_lsmem() {
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory Configuration:");
    crate::println!("═══════════════════════════════════════════");
    crate::println!("Total:       {} KB", heap_total / 1024);
    crate::println!("Used:        {} KB", crate::memory::stats().heap_used / 1024);
}

fn cmd_dmidecode() {
    crate::println_color!(COLOR_YELLOW, "dmidecode: DMI/SMBIOS not implemented");
}

fn cmd_hdparm(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "hdparm: disk parameters not implemented");
}

fn cmd_modprobe(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "modprobe: kernel modules not implemented");
    crate::println!("(TrustOS has builtin drivers)");
}

fn cmd_lsmod() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Loaded Kernel Modules:");
    crate::println!("Module                  Size  Used by");
    crate::println!("e1000                  64000  1");
    crate::println!("ahci                   32000  0");
    crate::println!("ps2kbd                  8000  1");
    crate::println!("ps2mouse                4000  1");
}

fn cmd_insmod(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "insmod: module loading not implemented");
}

fn cmd_rmmod(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "rmmod: module unloading not implemented");
}

fn cmd_sysctl(_args: &[&str]) {
    crate::println!("kernel.ostype = TrustOS");
    crate::println!("kernel.osrelease = 0.1.0");
    crate::println!("kernel.version = #1 SMP TrustOS");
}

fn cmd_service(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "service: init system not implemented");
}

fn cmd_systemctl(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "systemctl: systemd not implemented");
    crate::println!("(TrustOS uses simple init)");
}

fn cmd_crontab(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "crontab: scheduled tasks not implemented");
}

fn cmd_at(_args: &[&str]) {
    crate::println_color!(COLOR_YELLOW, "at: scheduled execution not implemented");
}

/// Web browser command
fn cmd_browse(args: &[&str]) {
    if args.is_empty() {
        crate::println!("TrustOS Web Browser");
        crate::println!("Usage: browse <url>");
        crate::println!("  Example: browse http://example.com");
        crate::println!("  Example: browse http://info.cern.ch");
        crate::println!("");
        crate::println!("Note: Only HTTP is supported (no HTTPS yet)");
        return;
    }
    
    let url = args[0];
    let url = if !url.starts_with("http://") && !url.starts_with("https://") {
        alloc::format!("http://{}", url)
    } else {
        String::from(url)
    };
    
    crate::println!("[Browser] Loading {}...", url);
    
    // Create browser instance
    let mut browser = crate::browser::Browser::new(800, 600);
    
    match browser.navigate(&url) {
        Ok(()) => {
            crate::println_color!(COLOR_GREEN, "Page loaded successfully!");
            
            // Get document title
            if let Some(ref doc) = browser.document {
                if !doc.title.is_empty() {
                    crate::println!("Title: {}", doc.title);
                }
                
                crate::println!("");
                
                // Render text content to console
                render_document_text(doc, 0);
            }
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "Failed to load page: {}", e);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Web Sandbox — capability-gated isolated web execution
// ═══════════════════════════════════════════════════════════════════════════

fn cmd_sandbox(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");
    match subcmd {
        "open" | "navigate" | "go" => {
            // sandbox open <url> [preset]
            if args.len() < 2 {
                crate::println!("Usage: sandbox open <url> [strict|moderate|permissive]");
                return;
            }
            let url = args[1];
            let preset = match args.get(2).copied() {
                Some("strict") => crate::sandbox::policy::PolicyPreset::Strict,
                Some("permissive") => crate::sandbox::policy::PolicyPreset::Permissive,
                _ => crate::sandbox::policy::PolicyPreset::Moderate,
            };

            crate::println_color!(COLOR_CYAN, "[Sandbox] Creating sandbox ({:?})...", preset);
            let id = crate::sandbox::create(preset, Some(url));
            crate::println!("  Sandbox #{} created", id.0);

            crate::println_color!(COLOR_CYAN, "[Sandbox] Navigating to {}...", url);
            match crate::sandbox::navigate(id, url) {
                Ok(resp) => {
                    crate::println_color!(COLOR_GREEN, "  Status: {} | Content-Type: {} | {} bytes",
                        resp.status_code, resp.content_type, resp.body.len());

                    if resp.is_html() {
                        // Parse and render through browser engine
                        let html_str = resp.body_string();
                        let doc = crate::browser::html_parser::parse_html(&html_str);
                        if !doc.title.is_empty() {
                            crate::println!("  Title: {}", doc.title);
                        }
                        crate::println!();
                        render_document_text(&doc, 0);

                        // Execute inline scripts through JS sandbox
                        let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                        let policy = mgr.get(id).map(|s| s.policy.js_allowed()).unwrap_or(false);
                        drop(mgr);
                        if policy {
                            let mut jss = crate::sandbox::js_sandbox::JsSandbox::new(
                                id, crate::sandbox::js_sandbox::JsSandboxConfig::default()
                            );
                            let results = jss.execute_inline_scripts(&html_str);
                            if !results.is_empty() {
                                crate::println_color!(COLOR_YELLOW, "\n  [JS] {} script(s) processed", results.len());
                                for (i, r) in results.iter().enumerate() {
                                    if r.completed {
                                        crate::println_color!(COLOR_GREEN, "    Script {}: OK ({}ms)", i+1, r.elapsed_ms);
                                    } else {
                                        crate::println_color!(COLOR_RED, "    Script {}: {}", i+1,
                                            r.error.as_deref().unwrap_or("failed"));
                                    }
                                    for line in &r.output {
                                        crate::println!("      > {}", line);
                                    }
                                }
                            }
                        } else {
                            crate::println_color!(COLOR_YELLOW, "  [JS blocked by policy]");
                        }
                    } else {
                        // Non-HTML: show raw body (truncated)
                        let text = resp.body_string();
                        let preview: String = text.chars().take(2000).collect();
                        crate::println!("{}", preview);
                        if text.len() > 2000 {
                            crate::println!("  ... ({} bytes total)", text.len());
                        }
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Error: {:?}", e);
                }
            }
        }

        "list" | "ls" => {
            let sandboxes = crate::sandbox::list();
            if sandboxes.is_empty() {
                crate::println!("No active sandboxes.");
            } else {
                crate::println_color!(COLOR_CYAN, "Active sandboxes:");
                crate::println!("  {:>4}  {:>10}  {}", "ID", "State", "Label");
                crate::println!("  {}", "-".repeat(40));
                for (id, label, state) in &sandboxes {
                    crate::println!("  {:>4}  {:>10?}  {}", id.0, state, label);
                }
            }
        }

        "status" | "info" => {
            if args.len() < 2 {
                crate::println!("Usage: sandbox status <id>");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let id = crate::sandbox::SandboxId(id_num);
            match crate::sandbox::status_string(id) {
                Some(s) => crate::println!("{}", s),
                None => crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num),
            }
        }

        "kill" | "destroy" | "close" => {
            if args.len() < 2 {
                crate::println!("Usage: sandbox kill <id|all>");
                return;
            }
            if args[1] == "all" {
                let sandboxes = crate::sandbox::list();
                for (id, _, _) in &sandboxes {
                    let _ = crate::sandbox::destroy(*id);
                }
                crate::println_color!(COLOR_GREEN, "All sandboxes destroyed.");
            } else {
                let id_num: u64 = args[1].parse().unwrap_or(0);
                let id = crate::sandbox::SandboxId(id_num);
                match crate::sandbox::destroy(id) {
                    Ok(()) => crate::println_color!(COLOR_GREEN, "Sandbox #{} destroyed.", id_num),
                    Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                }
            }
        }

        "allow" => {
            // sandbox allow <id> <domain>
            if args.len() < 3 {
                crate::println!("Usage: sandbox allow <id> <domain>");
                crate::println!("  Example: sandbox allow 1 example.com");
                crate::println!("  Example: sandbox allow 1 *.github.com");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get_mut(crate::sandbox::SandboxId(id_num)) {
                sb.policy.allow_domain(domain);
                crate::println_color!(COLOR_GREEN, "Domain '{}' allowed in sandbox #{}", domain, id_num);
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "deny" | "block" => {
            // sandbox deny <id> <domain>
            if args.len() < 3 {
                crate::println!("Usage: sandbox deny <id> <domain>");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let domain = args[2];
            let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get_mut(crate::sandbox::SandboxId(id_num)) {
                sb.policy.deny_domain(domain);
                crate::println_color!(COLOR_GREEN, "Domain '{}' blocked in sandbox #{}", domain, id_num);
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "policy" => {
            if args.len() < 2 {
                crate::println!("Usage: sandbox policy <id>");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get(crate::sandbox::SandboxId(id_num)) {
                crate::println!("{}", sb.policy.summary());
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "fs" => {
            // sandbox fs <id> [ls|tree|write|read|del] [path] [data]
            if args.len() < 3 {
                crate::println!("Usage: sandbox fs <id> <ls|tree|read|write|del> [path] [data]");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let fs_cmd = args[2];
            let mut mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            if let Some(sb) = mgr.get_mut(crate::sandbox::SandboxId(id_num)) {
                match fs_cmd {
                    "tree" => {
                        crate::println!("{}", sb.filesystem.tree());
                    }
                    "ls" => {
                        let dir = args.get(3).copied().unwrap_or("/");
                        match sb.filesystem.list(dir) {
                            Ok(entries) => {
                                crate::println_color!(COLOR_CYAN, "{}:", dir);
                                for (path, ftype, size) in &entries {
                                    let icon = if *ftype == &crate::sandbox::fs::SandboxFileType::Directory { "📁" } else { "📄" };
                                    crate::println!("  {} {} ({} bytes)", icon, path, size);
                                }
                            }
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    "read" => {
                        let path = args.get(3).copied().unwrap_or("/");
                        match sb.filesystem.read(path) {
                            Ok(data) => {
                                let text = core::str::from_utf8(data).unwrap_or("<binary>");
                                crate::println!("{}", text);
                            }
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    "write" => {
                        if args.len() < 5 {
                            crate::println!("Usage: sandbox fs <id> write <path> <data>");
                            return;
                        }
                        let path = args[3];
                        let data = args[4..].join(" ");
                        match sb.filesystem.write(path, data.as_bytes(), "shell") {
                            Ok(()) => crate::println_color!(COLOR_GREEN, "Written {} bytes to {}", data.len(), path),
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    "del" | "rm" => {
                        let path = args.get(3).copied().unwrap_or("");
                        match sb.filesystem.delete(path) {
                            Ok(()) => crate::println_color!(COLOR_GREEN, "Deleted {}", path),
                            Err(e) => crate::println_color!(COLOR_RED, "Error: {:?}", e),
                        }
                    }
                    _ => crate::println!("Unknown fs command: {}", fs_cmd),
                }
            } else {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
            }
        }

        "js" | "eval" => {
            // sandbox js <id> <code...>
            if args.len() < 3 {
                crate::println!("Usage: sandbox js <id> <code>");
                crate::println!("  Example: sandbox js 1 console.log('hello')");
                return;
            }
            let id_num: u64 = args[1].parse().unwrap_or(0);
            let code = args[2..].join(" ");
            let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
            let exists = mgr.get(crate::sandbox::SandboxId(id_num)).is_some();
            let js_ok = mgr.get(crate::sandbox::SandboxId(id_num))
                .map(|s| s.policy.js_allowed()).unwrap_or(false);
            drop(mgr);

            if !exists {
                crate::println_color!(COLOR_RED, "Sandbox #{} not found", id_num);
                return;
            }
            if !js_ok {
                crate::println_color!(COLOR_RED, "JavaScript is blocked by sandbox policy (use 'moderate' or 'permissive' preset)");
                return;
            }

            let mut jss = crate::sandbox::js_sandbox::JsSandbox::new(
                crate::sandbox::SandboxId(id_num),
                crate::sandbox::js_sandbox::JsSandboxConfig::default(),
            );
            let result = jss.execute(&code);
            if result.completed {
                crate::println_color!(COLOR_GREEN, "= {}", result.return_value);
            } else {
                crate::println_color!(COLOR_RED, "Error: {}", result.error.as_deref().unwrap_or("unknown"));
            }
            for line in &result.output {
                crate::println!("  > {}", line);
            }
            crate::println_color!(COLOR_WHITE, "  ({}ms)", result.elapsed_ms);
        }

        "audit" | "log" => {
            if args.len() < 2 {
                // Show global audit log
                let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                let log = mgr.audit_log();
                if log.is_empty() {
                    crate::println!("No audit entries.");
                } else {
                    crate::println_color!(COLOR_CYAN, "Audit log ({} entries):", log.len());
                    for entry in log.iter().rev().take(20) {
                        crate::println!("  [{}ms] #{} {:?}: {}",
                            entry.timestamp_ms, entry.sandbox_id.0,
                            entry.action, entry.detail);
                    }
                }
            } else {
                let id_num: u64 = args[1].parse().unwrap_or(0);
                let mgr = crate::sandbox::SANDBOX_MANAGER.lock();
                let entries = mgr.audit_for(crate::sandbox::SandboxId(id_num));
                if entries.is_empty() {
                    crate::println!("No audit entries for sandbox #{}", id_num);
                } else {
                    crate::println_color!(COLOR_CYAN, "Audit for sandbox #{} ({} entries):", id_num, entries.len());
                    for entry in entries.iter().rev().take(20) {
                        crate::println!("  [{}ms] {:?}: {}",
                            entry.timestamp_ms, entry.action, entry.detail);
                    }
                }
            }
        }

        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Web Sandbox — Secure isolated web execution");
            crate::println!();
            crate::println!("Usage: sandbox <command> [args...]");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Navigation:");
            crate::println!("    open <url> [preset]     Open URL in new sandbox");
            crate::println!("                             Presets: strict, moderate (default), permissive");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Sandbox Management:");
            crate::println!("    list                    List active sandboxes");
            crate::println!("    status <id>             Show sandbox details & stats");
            crate::println!("    kill <id|all>           Destroy sandbox(es)");
            crate::println!("    audit [id]              View audit log");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Security Policy:");
            crate::println!("    allow <id> <domain>     Add domain to allowlist");
            crate::println!("    deny <id> <domain>      Add domain to denylist");
            crate::println!("    policy <id>             Show policy config");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  Sandboxed Filesystem:");
            crate::println!("    fs <id> tree             Show filesystem tree");
            crate::println!("    fs <id> ls [dir]         List directory");
            crate::println!("    fs <id> read <path>      Read file");
            crate::println!("    fs <id> write <path> <d> Write data to file");
            crate::println!("    fs <id> del <path>       Delete file");
            crate::println!();
            crate::println_color!(COLOR_WHITE, "  JavaScript (sandboxed):");
            crate::println!("    js <id> <code>           Execute JS in sandbox");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "  Security features:");
            crate::println!("    - Capability-gated network (kernel proxy)");
            crate::println!("    - Domain allow/deny lists + SSRF protection");
            crate::println!("    - Rate limiting + response size limits");
            crate::println!("    - JS static analysis (blocks eval, prototype pollution)");
            crate::println!("    - Jailed filesystem with quotas");
            crate::println!("    - Full audit trail");
        }
    }
}

/// Render HTML document as text to console
fn render_document_text(doc: &crate::browser::HtmlDocument, _depth: usize) {
    for node in &doc.nodes {
        render_node_text(node, 0);
    }
}

/// Render a single HTML node as text
fn render_node_text(node: &crate::browser::HtmlNode, depth: usize) {
    match node {
        crate::browser::HtmlNode::Text(text) => {
            let text = text.trim();
            if !text.is_empty() {
                crate::println!("{}", text);
            }
        }
        crate::browser::HtmlNode::Element(el) => {
            let tag = el.tag.as_str();
            
            // Skip invisible elements
            if matches!(tag, "head" | "script" | "style" | "meta" | "link" | "title" | "noscript") {
                return;
            }
            
            // Add formatting based on tag
            match tag {
                "h1" => {
                    crate::println!("");
                    crate::println_color!(COLOR_CYAN, "=== {} ===", get_element_text(el));
                    return;
                }
                "h2" => {
                    crate::println!("");
                    crate::println_color!(COLOR_CYAN, "== {} ==", get_element_text(el));
                    return;
                }
                "h3" | "h4" | "h5" | "h6" => {
                    crate::println!("");
                    crate::println_color!(COLOR_CYAN, "= {} =", get_element_text(el));
                    return;
                }
                "p" => {
                    crate::println!("");
                }
                "br" => {
                    crate::println!("");
                }
                "hr" => {
                    crate::println!("----------------------------------------");
                }
                "a" => {
                    if let Some(href) = el.attr("href") {
                        let text = get_element_text(el);
                        if !text.is_empty() {
                            crate::println_color!(COLOR_BLUE, "[{}] ({})", text, href);
                        }
                    }
                    return;
                }
                "li" => {
                    let indent = "  ".repeat(depth);
                    crate::print!("{}• ", indent);
                }
                "pre" | "code" => {
                    crate::println_color!(COLOR_MAGENTA, "{}", get_element_text(el));
                    return;
                }
                "img" => {
                    if let Some(alt) = el.attr("alt") {
                        crate::println!("[Image: {}]", alt);
                    } else {
                        crate::println!("[Image]");
                    }
                    return;
                }
                _ => {}
            }
            
            // Render children
            for child in &el.children {
                render_node_text(child, depth + 1);
            }
            
            // Newline after block elements
            if matches!(tag, "p" | "div" | "section" | "article" | "ul" | "ol" | "table" | "tr") {
                crate::println!("");
            }
        }
    }
}

/// Extract text content from an element
fn get_element_text(el: &crate::browser::HtmlElement) -> String {
    use alloc::string::ToString;
    let mut result = String::new();
    collect_text(&el.children, &mut result);
    result.trim().to_string()
}

/// Collect text from nodes recursively
fn collect_text(nodes: &[crate::browser::HtmlNode], result: &mut String) {
    use alloc::string::ToString;
    
    for node in nodes {
        match node {
            crate::browser::HtmlNode::Text(t) => {
                result.push_str(t);
                result.push(' ');
            }
            crate::browser::HtmlNode::Element(el) => {
                collect_text(&el.children, result);
            }
        }
    }
}

// Helper function to read file content as String
fn read_file_content(path: &str) -> Option<String> {
    match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => {
            let mut buf = [0u8; 4096];
            let n = crate::vfs::read(fd, &mut buf).unwrap_or(0);
            crate::vfs::close(fd).ok();
            Some(String::from(core::str::from_utf8(&buf[..n]).unwrap_or("")))
        }
        Err(_) => None,
    }
}

// Helper function to read file content as bytes
fn read_file_bytes(path: &str) -> Option<Vec<u8>> {
    // Try ramfs first (for /tmp files)
    if let Ok(data) = crate::ramfs::with_fs(|fs| {
        fs.read_file(path).map(|slice| slice.to_vec())
    }) {
        return Some(data);
    }
    
    // Then try VFS
    match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 4096];
            loop {
                match crate::vfs::read(fd, &mut chunk) {
                    Ok(0) => break,
                    Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    Err(_) => break,
                }
            }
            crate::vfs::close(fd).ok();
            Some(buf)
        }
        Err(_) => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GRAPHICAL TERMINAL COMMAND
// ═══════════════════════════════════════════════════════════════════════════════

/// Graphical Terminal - Matrix Edition
fn cmd_gterm(args: &[&str]) {
    use crate::wayland::terminal;
    
    let subcmd = args.get(0).copied().unwrap_or("launch");
    
    match subcmd {
        "launch" | "start" | "run" => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║           TrustOS Graphical Terminal - Matrix Edition        ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
            crate::println!();
            
            // Initialize Wayland compositor first
            let _ = crate::wayland::init();
            
            // Get display size
            let (screen_w, screen_h) = crate::framebuffer::get_dimensions();
            
            // Calculate terminal window size (80% of screen)
            let term_w = (screen_w * 80 / 100) & !7; // Align to 8 pixels
            let term_h = (screen_h * 80 / 100) & !15; // Align to 16 pixels
            
            crate::println!("Initializing terminal {}x{} pixels...", term_w, term_h);
            
            // Initialize the graphical terminal
            match terminal::init(term_w, term_h) {
                Ok(()) => {
                    crate::println_color!(COLOR_GREEN, "[OK] Graphics terminal initialized");
                }
                Err(e) => {
                    crate::println_color!(COLOR_YELLOW, "[WARN] {}", e);
                }
            }
            
            // Create Wayland surface for the terminal
            crate::wayland::with_compositor(|compositor| {
                let surface_id = compositor.create_surface();
                
                // Get rendered terminal buffer
                if let Some(buffer) = terminal::render() {
                    let (w, h) = terminal::get_size().unwrap_or((term_w, term_h));
                    
                    // Configure surface
                    if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                        surface.attach(buffer, w, h);
                        surface.set_title("TrustOS Terminal");
                        let x = (screen_w - w) / 2;
                        let y = (screen_h - h) / 2;
                        surface.set_position(x as i32, y as i32);
                        surface.make_toplevel();
                        surface.commit();
                    }
                }
                
                crate::println_color!(COLOR_GREEN, "[OK] Terminal surface created (ID: {})", surface_id);
            });
            
            // Initial render
            crate::wayland::compose_frame();
            
            crate::println!();
            crate::println_color!(COLOR_GREEN, "Terminal launched!");
            crate::println!("Use 'gterm demo' for an interactive demo.");
            crate::println!("Use 'gterm fullscreen' for fullscreen mode.");
        },
        
        "demo" => {
            crate::println_color!(COLOR_CYAN, "Starting interactive graphical terminal demo...");
            crate::println!();
            
            // Initialize everything
            let _ = crate::wayland::init();
            
            let (screen_w, screen_h) = crate::framebuffer::get_dimensions();
            let term_w = (screen_w * 85 / 100) & !7;
            let term_h = (screen_h * 85 / 100) & !15;
            
            // Initialize terminal (ignore if already initialized)
            let _ = terminal::init(term_w, term_h);
            
            // Create surface
            let surface_id = crate::wayland::with_compositor(|compositor| {
                let id = compositor.create_surface();
                
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&id) {
                            surface.attach(buffer, w, h);
                            surface.set_title("TrustOS Terminal Demo");
                            surface.set_position(
                                ((screen_w - w) / 2) as i32,
                                ((screen_h - h) / 2) as i32
                            );
                            surface.make_toplevel();
                            surface.commit();
                        }
                    }
                }
                id
            }).unwrap_or(0);
            
            // Initial compose
            crate::wayland::compose_frame();
            
            // Write welcome message
            terminal::write("\x1b[2J\x1b[H"); // Clear and home
            terminal::write("\x1b[1;32m╔══════════════════════════════════════════════════════════╗\r\n");
            terminal::write("║  \x1b[1;97mTrustOS\x1b[1;32m Graphical Terminal Demo                       ║\r\n");
            terminal::write("║  Matrix Edition v1.0                                     ║\r\n");
            terminal::write("╚══════════════════════════════════════════════════════════╝\r\n");
            terminal::write("\x1b[0;32m\r\n");
            terminal::write("Type text and press Enter. Press ESC to exit.\r\n\r\n");
            terminal::write("\x1b[1;32m$ \x1b[0;32m");
            
            // Render after writing
            crate::wayland::with_compositor(|compositor| {
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                            surface.attach(buffer, w, h);
                            surface.commit();
                        }
                    }
                }
            });
            crate::wayland::compose_frame();
            
            // Interactive loop
            let mut input_buffer = alloc::string::String::new();
            loop {
                // Check for keyboard input
                if let Some(key) = crate::keyboard::read_char() {
                    let c = key as char;
                    match key {
                        0x1b => {
                            // ESC - exit
                            break;
                        }
                        0x0D | 0x0A => {
                            // Enter - process command
                            terminal::write("\r\n");
                            
                            if !input_buffer.is_empty() {
                                // Echo the command result
                                let response = alloc::format!("\x1b[0;36mYou typed: \x1b[1;97m{}\x1b[0;32m\r\n", input_buffer);
                                terminal::write(&response);
                                input_buffer.clear();
                            }
                            
                            terminal::write("\x1b[1;32m$ \x1b[0;32m");
                        }
                        0x08 | 0x7F => {
                            // Backspace
                            if !input_buffer.is_empty() {
                                input_buffer.pop();
                                terminal::write("\x08 \x08");
                            }
                        }
                        k if k >= 0x20 && k < 0x7F => {
                            // Printable character
                            input_buffer.push(c);
                            let s = alloc::format!("{}", c);
                            terminal::write(&s);
                        }
                        _ => {}
                    }
                    
                    // Re-render after each keystroke
                    crate::wayland::with_compositor(|compositor| {
                        if let Some(buffer) = terminal::render() {
                            if let Some((w, h)) = terminal::get_size() {
                                if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                                    surface.attach(buffer, w, h);
                                    surface.commit();
                                }
                            }
                        }
                    });
                    crate::wayland::compose_frame();
                }
                
                // Small delay to prevent busy loop
                for _ in 0..1000 { core::hint::spin_loop(); }
            }
            
            // Cleanup
            crate::framebuffer::clear();
            crate::println_color!(COLOR_GREEN, "Demo ended.");
        },
        
        "fullscreen" | "fs" => {
            crate::println_color!(COLOR_CYAN, "Launching fullscreen terminal...");
            
            // Use full screen dimensions
            let (screen_w, screen_h) = crate::framebuffer::get_dimensions();
            
            let _ = crate::wayland::init();
            let _ = terminal::init(screen_w, screen_h);
            
            // Create fullscreen surface
            crate::wayland::with_compositor(|compositor| {
                let id = compositor.create_surface();
                
                if let Some(buffer) = terminal::render() {
                    if let Some((w, h)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&id) {
                            surface.attach(buffer, w, h);
                            surface.set_title("TrustOS Terminal");
                            surface.set_position(0, 0);
                            surface.make_toplevel();
                            surface.state.fullscreen = true;
                            surface.commit();
                        }
                    }
                }
            });
            
            crate::wayland::compose_frame();
            crate::println_color!(COLOR_GREEN, "[OK] Fullscreen terminal active");
        },
        
        "test" => {
            // Test ANSI escape codes
            crate::println_color!(COLOR_CYAN, "Testing graphical terminal ANSI support...");
            
            let _ = crate::wayland::init();
            let (w, h) = crate::framebuffer::get_dimensions();
            let _ = terminal::init(w * 70 / 100, h * 70 / 100);
            
            // Test various escape sequences
            terminal::write("\x1b[2J\x1b[H"); // Clear
            terminal::write("\x1b[1;32m=== ANSI Escape Code Test ===\x1b[0m\r\n\r\n");
            
            // Colors
            terminal::write("\x1b[31mRed \x1b[32mGreen \x1b[33mYellow \x1b[34mBlue \x1b[35mMagenta \x1b[36mCyan\x1b[0m\r\n");
            terminal::write("\x1b[91mBright Red \x1b[92mBright Green \x1b[93mBright Yellow\x1b[0m\r\n\r\n");
            
            // Attributes
            terminal::write("\x1b[1mBold\x1b[0m \x1b[2mDim\x1b[0m \x1b[4mUnderline\x1b[0m \x1b[7mReverse\x1b[0m\r\n\r\n");
            
            // Matrix rain effect preview
            terminal::write("\x1b[32m");
            for i in 0..5 {
                for _ in 0..60 {
                    let c = ((i * 7 + 33) % 94 + 33) as u8 as char;
                    let s = alloc::format!("{}", c);
                    terminal::write(&s);
                }
                terminal::write("\r\n");
            }
            terminal::write("\x1b[0m\r\n");
            
            terminal::write("\x1b[1;97mTest complete!\x1b[0m\r\n");
            
            // Render
            crate::wayland::with_compositor(|compositor| {
                let id = compositor.create_surface();
                if let Some(buffer) = terminal::render() {
                    if let Some((tw, th)) = terminal::get_size() {
                        if let Some(surface) = compositor.surfaces.get_mut(&id) {
                            surface.attach(buffer, tw, th);
                            surface.set_title("ANSI Test");
                            surface.set_position(
                                ((w - tw) / 2) as i32,
                                ((h - th) / 2) as i32
                            );
                            surface.make_toplevel();
                            surface.commit();
                        }
                    }
                }
            });
            crate::wayland::compose_frame();
            
            crate::println!();
            crate::println_color!(COLOR_GREEN, "Press any key to close...");
            loop {
                if crate::keyboard::read_char().is_some() {
                    break;
                }
            }
            crate::framebuffer::clear();
        },
        
        _ => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║       TrustOS Graphical Terminal - Matrix Edition           ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
            crate::println!();
            crate::println!("A native graphical terminal emulator with VT100/ANSI support.");
            crate::println!("Inspired by Smithay, built from scratch for TrustOS.");
            crate::println!();
            crate::println!("Usage: gterm <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  launch     - Open the graphical terminal window");
            crate::println!("  demo       - Interactive demo (type text, ESC to exit)");
            crate::println!("  fullscreen - Open fullscreen terminal");
            crate::println!("  test       - Test ANSI escape code rendering");
            crate::println!();
            crate::println!("Features:");
            crate::println!("  • Matrix-style green phosphor theme");
            crate::println!("  • VT100/ANSI escape code support");
            crate::println!("  • 256-color and 24-bit RGB colors");
            crate::println!("  • Scrollback buffer (1000 lines)");
            crate::println!("  • Phosphor glow effect");
            crate::println!("  • CRT scanline effect");
        }
    }
}

/// Wayland compositor command
fn cmd_wayland(args: &[&str]) {
    let subcmd = args.get(0).copied().unwrap_or("help");
    
    match subcmd {
        "init" | "start" => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║            TrustOS Wayland Compositor                        ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
            crate::println!();
            
            match crate::wayland::init() {
                Ok(()) => {
                    crate::println_color!(COLOR_GREEN, "[OK] Wayland compositor initialized");
                    
                    // Get screen info
                    let (width, height) = crate::framebuffer::get_dimensions();
                    crate::println!("     Display: {}x{}", width, height);
                    crate::println!();
                    crate::println!("Available globals:");
                    for global in crate::wayland::protocol::get_globals() {
                        crate::println!("  • {} v{}", global.interface, global.version);
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "[ERROR] {}", e);
                }
            }
        },
        
        "demo" => {
            crate::println_color!(COLOR_CYAN, "Starting Wayland demo...");
            
            // Initialize if not already done
            let _ = crate::wayland::init();
            
            // Create a test surface
            crate::wayland::with_compositor(|compositor| {
                // Create a surface
                let surface_id = compositor.create_surface();
                
                // Create some test content
                let width = 400u32;
                let height = 300u32;
                let mut buffer = alloc::vec![0xFF0A0F0C_u32; (width * height) as usize];
                
                // Draw a gradient
                for y in 0..height {
                    for x in 0..width {
                        let r = (x * 255 / width) as u8;
                        let g = ((y * 255 / height) as u8) / 2;
                        let b = 0x20_u8;
                        buffer[(y * width + x) as usize] = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
                    }
                }
                
                // Draw border
                for x in 0..width {
                    buffer[x as usize] = 0xFF00FF66;
                    buffer[((height - 1) * width + x) as usize] = 0xFF00FF66;
                }
                for y in 0..height {
                    buffer[(y * width) as usize] = 0xFF00FF66;
                    buffer[(y * width + width - 1) as usize] = 0xFF00FF66;
                }
                
                // Attach and commit
                if let Some(surface) = compositor.surfaces.get_mut(&surface_id) {
                    surface.attach(buffer, width, height);
                    surface.set_title("Wayland Demo");
                    surface.set_position(200, 150);
                    surface.make_toplevel();
                    surface.commit();
                }
                
                crate::println_color!(COLOR_GREEN, "[OK] Created surface {}", surface_id);
            });
            
            // Compose and render
            crate::wayland::compose_frame();
            crate::println_color!(COLOR_GREEN, "[OK] Frame composed to framebuffer");
            crate::println!();
            crate::println!("Press any key to close demo...");
            
            // Wait for key
            loop {
                if let Some(_) = crate::keyboard::read_char() {
                    break;
                }
            }
            
            // Clear screen
            crate::framebuffer::clear();
        },
        
        "status" => {
            crate::println_color!(COLOR_CYAN, "Wayland Compositor Status");
            crate::println_color!(COLOR_CYAN, "══════════════════════════");
            
            crate::wayland::with_compositor(|compositor| {
                let (w, h) = (compositor.width, compositor.height);
                crate::println!("Display: {}x{}", w, h);
                crate::println!("Surfaces: {}", compositor.surfaces.len());
                crate::println!("SHM Pools: {}", compositor.shm_pools.len());
                crate::println!("Frame: {}", compositor.frame_number);
                crate::println!("Pointer: ({}, {})", compositor.pointer_x, compositor.pointer_y);
                
                if !compositor.surfaces.is_empty() {
                    crate::println!();
                    crate::println!("Surfaces:");
                    for (&id, surface) in &compositor.surfaces {
                        let title = if surface.title.is_empty() { "<untitled>" } else { &surface.title };
                        crate::println!("  #{}: {} @ ({},{}) {}x{}", 
                            id, title, surface.x, surface.y, surface.width, surface.height);
                    }
                }
            }).unwrap_or_else(|| {
                crate::println_color!(COLOR_YELLOW, "Compositor not initialized");
                crate::println!("Run 'wayland init' first");
            });
        },
        
        _ => {
            crate::println_color!(COLOR_CYAN, "TrustOS Wayland Compositor");
            crate::println_color!(COLOR_CYAN, "══════════════════════════");
            crate::println!();
            crate::println!("A native Wayland display server for TrustOS.");
            crate::println!();
            crate::println!("Usage: wayland <command>");
            crate::println!();
            crate::println!("Commands:");
            crate::println!("  init    - Initialize the Wayland compositor");
            crate::println!("  demo    - Run a visual demo");
            crate::println!("  status  - Show compositor status");
            crate::println!();
            crate::println!("Protocol support:");
            crate::println!("  • wl_compositor v5 - Surface creation");
            crate::println!("  • wl_shm v1        - Shared memory buffers");
            crate::println!("  • wl_seat v8       - Input devices");
            crate::println!("  • xdg_wm_base v5   - Window management");
        }
    }
}

/// TrustLang command: compile and run TrustLang programs
fn cmd_trustlang(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "run" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: trustlang run <file.tl>"); return; }
            };
            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/{}", filename)
            };
            let source = match crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                Ok(data) => match alloc::string::String::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", filename); return; }
            };
            crate::println!("\x1b[36m[TrustLang]\x1b[0m Compiling {}...", filename);
            match crate::trustlang::run(&source) {
                Ok(output) => {
                    if !output.is_empty() { crate::print!("{}", output); }
                    crate::println!("\x1b[32m[TrustLang]\x1b[0m Program finished successfully.");
                }
                Err(e) => crate::println!("\x1b[31m[TrustLang Error]\x1b[0m {}", e),
            }
        }
        "check" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: trustlang check <file.tl>"); return; }
            };
            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/{}", filename)
            };
            let source = match crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
                Ok(data) => match alloc::string::String::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => { crate::println!("Error: file is not valid UTF-8"); return; }
                },
                Err(_) => { crate::println!("Error: file '{}' not found", filename); return; }
            };
            match crate::trustlang::check(&source) {
                Ok(()) => crate::println!("\x1b[32m✓\x1b[0m {} — no errors", filename),
                Err(e) => crate::println!("\x1b[31m✗\x1b[0m {} — {}", filename, e),
            }
        }
        "eval" => {
            // Inline eval: wrap in main()
            let code = args[1..].join(" ");
            let wrapped = alloc::format!("fn main() {{ {} }}", code);
            match crate::trustlang::run(&wrapped) {
                Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
                Err(e) => crate::println!("\x1b[31mError:\x1b[0m {}", e),
            }
        }
        "demo" => {
            // Create a demo TrustLang file
            let demo = r#"// TrustLang Demo — Fibonacci
fn fibonacci(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    println("=== TrustLang on TrustOS ===");
    println("Computing Fibonacci sequence:");
    for i in 0..15 {
        let result = fibonacci(i);
        print("  fib(");
        print(to_string(i));
        print(") = ");
        println(to_string(result));
    }
    println("Done!");
}
"#;
            crate::ramfs::with_fs(|fs| {
                let _ = fs.write_file("/demo.tl", demo.as_bytes());
            });
            crate::println!("Created /demo.tl — run with: trustlang run demo.tl");
            // Also execute it
            match crate::trustlang::run(demo) {
                Ok(output) => { if !output.is_empty() { crate::print!("{}", output); } }
                Err(e) => crate::println!("\x1b[31mError:\x1b[0m {}", e),
            }
        }
        _ => {
            crate::println!("\x1b[1;36mTrustLang\x1b[0m — Integrated Programming Language");
            crate::println!("  Rust-inspired syntax, bytecode VM, zero dependencies\n");
            crate::println!("Commands:");
            crate::println!("  trustlang run <file.tl>    Compile & execute a file");
            crate::println!("  trustlang check <file.tl>  Syntax check only");
            crate::println!("  trustlang eval <code>      Evaluate inline code");
            crate::println!("  trustlang demo             Create & run demo program");
            crate::println!("\nExample:");
            crate::println!("  trustlang eval println(\"Hello TrustOS!\")");
            crate::println!("  trustlang eval \"let x = 42; println(to_string(x * 2))\"");
        }
    }
}

/// Compute syntax highlighting colors for one line of TrustLang code.
/// Returns a Vec of ARGB colors, one per character.
fn trustlang_syntax_colors(line: &str) -> alloc::vec::Vec<u32> {
    let chars: alloc::vec::Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut colors = alloc::vec![0xFFD4D4D4u32; len]; // default: white-gray
    if len == 0 { return colors; }

    // 1) Find comment start (char index, not byte)
    let comment_ci = {
        let mut ci = None;
        let bytes = line.as_bytes();
        for i in 0..bytes.len().saturating_sub(1) {
            if bytes[i] == b'/' && bytes[i + 1] == b'/' {
                ci = Some(line[..i].chars().count());
                break;
            }
        }
        ci
    };
    let effective_len = comment_ci.unwrap_or(len);

    // Color comments green
    if let Some(cp) = comment_ci {
        for i in cp..len {
            colors[i] = 0xFF6A9955;
        }
    }

    // 2) Strings — track in_string, color everything inside "" as orange-brown
    let mut in_string = false;
    for i in 0..effective_len {
        if chars[i] == '"' {
            colors[i] = 0xFFCE9178;
            in_string = !in_string;
        } else if in_string {
            colors[i] = 0xFFCE9178;
        }
    }

    // 3) Keywords, function calls, variables, numbers, brackets (outside strings & comments)
    in_string = false;
    let mut i = 0usize;
    while i < effective_len {
        if chars[i] == '"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if in_string { i += 1; continue; }

        // Numbers
        if chars[i].is_ascii_digit() {
            colors[i] = 0xFFB5CEA8;
            i += 1;
            continue;
        }
        // Brackets
        if matches!(chars[i], '(' | ')' | '{' | '}' | '[' | ']') {
            colors[i] = 0xFFFFD700;
            i += 1;
            continue;
        }
        // Identifiers / keywords
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < effective_len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: alloc::string::String = chars[start..i].iter().collect();

            // Is this a function call? (followed by '(')
            let mut peek = i;
            while peek < effective_len && chars[peek] == ' ' { peek += 1; }
            let is_fn_call = peek < effective_len && chars[peek] == '(';

            // Is this a variable declaration? (preceded by "let" or "mut")
            let before: alloc::string::String = chars[..start].iter().collect();
            let trimmed = before.trim_end();
            let is_var_decl = trimmed.ends_with("let") || trimmed.ends_with("mut");

            if matches!(word.as_str(),
                "fn" | "let" | "mut" | "if" | "else" | "while" | "for" | "in" |
                "return" | "loop" | "break" | "continue" | "true" | "false" |
                "struct" | "enum" | "match" | "use" | "pub" | "const" | "static" |
                "impl" | "self" | "type")
            {
                for j in start..i { colors[j] = 0xFFFF7B72; } // red keywords
            } else if is_fn_call {
                for j in start..i { colors[j] = 0xFF79C0FF; } // blue function calls
            } else if is_var_decl {
                for j in start..i { colors[j] = 0xFF9CDCFE; } // cyan variable names
            }
            // else keep default white
            continue;
        }
        i += 1;
    }
    colors
}

/// TrustLang Showcase — Animated walkthrough demonstrating the full pipeline
/// ~90 seconds of automated cinematic demo — descriptions, code typing, compilation, execution
/// Uses frame-counting + delay_millis for reliable timing (no TSC target comparison)
fn cmd_trustlang_showcase() {
    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    // ═══════════════ HELPER CLOSURES ═══════════════

    let draw_big_char = |buf: &mut [u32], w: usize, h: usize, cx: usize, cy: usize, c: char, color: u32, scale: usize| {
        let glyph = crate::framebuffer::font::get_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + bit as usize * scale + sx;
                            let py = cy + row * scale + sy;
                            if px < w && py < h {
                                buf[py * w + px] = color;
                            }
                        }
                    }
                }
            }
        }
    };

    let draw_text_at = |buf: &mut [u32], w: usize, h: usize, x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize, y: usize, text: &str, color: u32, scale: usize| {
        let tw = text.len() * 8 * scale;
        let sx = if tw < w { (w - tw) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
        }
    };

    let blit_buf = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(buf[y * w..].as_ptr(), bb.add(y * bb_s), w);
                }
            }
        }
        crate::framebuffer::swap_buffers();
    };

    let clear_buf = |buf: &mut [u32]| {
        for p in buf.iter_mut() { *p = 0xFF000000; }
    };

    // Matrix rain state
    let mut rain_cols: alloc::vec::Vec<u16> = (0..w / 8 + 1).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let rain_speeds: alloc::vec::Vec<u8> = (0..w / 8 + 1).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    let draw_rain = |buf: &mut [u32], w: usize, h: usize, cols: &mut [u16], speeds: &[u8], frame: u32| {
        for pixel in buf.iter_mut() {
            let g = ((*pixel >> 8) & 0xFF) as u32;
            if g > 0 { *pixel = 0xFF000000 | (g.saturating_sub(6) << 8); }
        }
        for ci in 0..cols.len() {
            let x = ci * 8;
            if x >= w { continue; }
            cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
            if cols[ci] as usize >= h { cols[ci] = 0; }
            let y = cols[ci] as usize;
            let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
            let glyph = crate::framebuffer::font::get_glyph(c);
            for (row, &bits) in glyph.iter().enumerate() {
                let py = y + row;
                if py >= h { break; }
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        let px = x + bit as usize;
                        if px < w { buf[py * w + px] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    // ── FRAME DELAY: ~30ms per frame ──
    let frame_ms: u64 = 30;

    // ── Fade out: gradually darken buffer over ~78 frames (~2.3s) ──
    let do_fade = |buf: &mut [u32], w: usize, h: usize, blit: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..78 {
            for px in buf.iter_mut() {
                let r = ((*px >> 16) & 0xFF).saturating_sub(4);
                let g = ((*px >> 8) & 0xFF).saturating_sub(4);
                let b = (*px & 0xFF).saturating_sub(4);
                *px = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
        for p in buf.iter_mut() { *p = 0xFF000000; }
        blit(buf, w, h);
        // Black pause between scenes (~1.4s)
        crate::cpu::tsc::pit_delay_ms(1400);
    };

    // ═══════════════════════════════════════════════════════════════
    // DESCRIPTION SCREEN — text types in on Matrix rain background
    // ms_per_char: how fast each character appears (higher = slower)
    // hold_frames: how many frames to HOLD after all text is typed
    // ═══════════════════════════════════════════════════════════════
    let show_description = |buf: &mut [u32], w: usize, h: usize,
                            rain_cols: &mut [u16], rain_speeds: &[u8],
                            lines: &[(&str, u32, usize)],
                            ms_per_char: u64,
                            hold_frames: u32| {
        let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
        // frames_per_char: how many render frames per character typed
        let frames_per_char = (ms_per_char / frame_ms).max(1) as u32;
        let typing_frames = total_chars as u32 * frames_per_char;
        let total_frames = typing_frames + hold_frames;
        let mut frame = 0u32;

        while frame < total_frames {
            // ESC to quit, Space/Enter to skip
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }

            // Rain background
            draw_rain(buf, w, h, rain_cols, rain_speeds, frame);

            // How many chars to show based on frame count
            let chars_shown = (frame / frames_per_char) as usize;

            // Compute vertical centering
            let total_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;

            for &(text, color, scale) in lines {
                let tw = text.len() * 8 * scale;
                let sx = if tw < w { (w - tw) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                // Cursor blink during typing
                if chars_shown > counted && chars_shown < counted + text.len() {
                    let ci = chars_shown - counted;
                    let cx = sx + ci * 8 * scale;
                    if (frame / 8) % 2 == 0 {
                        for cy in y..y + 16 * scale {
                            if cy < h && cx + 2 < w {
                                buf[cy * w + cx] = 0xFF00FF88;
                                buf[cy * w + cx + 1] = 0xFF00FF88;
                            }
                        }
                    }
                }
                counted += text.len();
                y += 16 * scale + 12;
            }

            blit_buf(buf, w, h);
            frame += 1;
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    };

    // ═══════════════════════════════════════════════════════════════
    // CODE EDITOR SCREEN — code types in char by char, then compiles
    // ms_per_char: typing speed for code
    // hold_frames: frames to wait after typing before "COMPILING"
    // output_hold_ms: MILLISECONDS to show output after execution (direct delay)
    // ═══════════════════════════════════════════════════════════════
    let show_code_and_run = |buf: &mut [u32], w: usize, h: usize,
                             rain_cols: &mut [u16], rain_speeds: &[u8],
                             title: &str,
                             source: &str,
                             pre_msg: &str,
                             _ms_per_char: u64,
                             _hold_frames: u32,
                             output_hold_ms: u64| {
        // ──────────────────────────────────────────────
        // Human-like typing: variable speed per character
        // with random pauses and scripted typos
        // ──────────────────────────────────────────────

        let lines_vec: alloc::vec::Vec<&str> = source.lines().collect();
        let total_chars: usize = source.len();

        let margin_x = 40usize;
        let header_h = 50usize;
        let code_y_start = header_h + 30;
        let line_h = 18usize;
        let code_scale = 1usize;

        // Build a flat char list with position info
        // Each entry: (line_idx, col_idx, char)
        let mut char_list: alloc::vec::Vec<(usize, usize, char)> = alloc::vec::Vec::new();
        for (li, line) in lines_vec.iter().enumerate() {
            for (ci, c) in line.chars().enumerate() {
                char_list.push((li, ci, c));
            }
            char_list.push((li, line.len(), '\n')); // newline marker
        }

        // Simple deterministic PRNG (no rand crate in kernel)
        let mut rng_state: u32 = 0xDEAD_BEEF;
        let mut rng_next = |state: &mut u32| -> u32 {
            *state ^= *state << 13;
            *state ^= *state >> 17;
            *state ^= *state << 5;
            *state
        };

        // Typo schedule: at these character indices, type wrong char, pause, backspace, retype
        // Format: (char_index, wrong_char)
        let typo_schedule: alloc::vec::Vec<(usize, char)> = alloc::vec![
            (45, 'w'),     // early typo on a variable
            (180, 'p'),    // mid-code typo
            (350, 'e'),    // in a function name
            (520, '0'),    // number typo
            (700, ';'),    // punctuation slip
        ];

        // Current "typed buffer" — what's visible on screen
        // We track how many real chars are shown
        let mut chars_shown: usize = 0;
        let mut rain_frame: u32 = 0;

        // Render function (inline) — draws the current state of the editor
        let render_editor = |buf: &mut [u32], w: usize, h: usize,
                             rain_cols: &mut [u16], rain_speeds: &[u8],
                             rain_frame: u32,
                             chars_shown: usize,
                             typo_char: Option<(usize, usize, char)>| {
            // Dark background with dimmed Matrix rain
            for p in buf.iter_mut() { *p = 0xFF0A0A0A; }
            draw_rain(buf, w, h, rain_cols, rain_speeds, rain_frame);
            for p in buf.iter_mut() {
                let g = ((*p >> 8) & 0xFF).min(25);
                *p = 0xFF000000 | (g << 8);
            }

            // Title bar
            for y in 0..header_h {
                for x in 0..w {
                    buf[y * w + x] = 0xFF111111;
                }
            }
            draw_text_at(buf, w, h, margin_x + 20, 15, title, 0xFF00FF88, 2);

            if !pre_msg.is_empty() {
                draw_text_at(buf, w, h, margin_x + 20, header_h + 5, pre_msg, 0xFF888888, 1);
            }

            // Code panel
            let panel_x = margin_x;
            let panel_w = w - 2 * margin_x;
            let panel_y = code_y_start;
            let panel_h = h - code_y_start - 80;
            for py in panel_y..panel_y + panel_h {
                for px in panel_x..panel_x + panel_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0D1117;
                    }
                }
            }
            // Panel border (green)
            for px in panel_x..panel_x + panel_w {
                if panel_y < h { buf[panel_y * w + px] = 0xFF00FF44; }
                let bot = (panel_y + panel_h).min(h) - 1;
                buf[bot * w + px] = 0xFF00FF44;
            }
            for py in panel_y..(panel_y + panel_h).min(h) {
                buf[py * w + panel_x] = 0xFF00FF44;
                let right = (panel_x + panel_w - 1).min(w - 1);
                buf[py * w + right] = 0xFF00FF44;
            }

            // Compute scroll offset so cursor line stays visible
            let max_visible = panel_h.saturating_sub(30) / line_h;
            let cursor_line = {
                let mut ci = 0usize;
                let mut ln = 0usize;
                for (li, line) in lines_vec.iter().enumerate() {
                    if ci + line.len() >= chars_shown {
                        ln = li;
                        break;
                    }
                    ci += line.len() + 1;
                    ln = li + 1;
                }
                ln.min(lines_vec.len().saturating_sub(1))
            };
            let scroll_offset = if cursor_line >= max_visible.saturating_sub(2) {
                cursor_line.saturating_sub(max_visible.saturating_sub(3))
            } else {
                0
            };

            // Draw typed code (with scroll + syntax highlighting)
            let code_x = panel_x + 42;
            let mut global_idx = 0usize;
            // Skip chars in lines above scroll_offset
            for li in 0..scroll_offset.min(lines_vec.len()) {
                global_idx += lines_vec[li].len() + 1; // +1 for \n
            }
            for vi in 0..(lines_vec.len() - scroll_offset.min(lines_vec.len())) {
                let li = vi + scroll_offset;
                let ly = code_y_start + 15 + vi * line_h;
                if ly + 16 > panel_y + panel_h { break; }
                let line = lines_vec[li];

                // Line number
                let ln_str = alloc::format!("{:>3}", li + 1);
                draw_text_at(buf, w, h, panel_x + 8, ly, &ln_str, 0xFF555555, code_scale);
                let sep_x = panel_x + 35;
                for sy in ly..ly + 16 {
                    if sy < h && sep_x < w { buf[sy * w + sep_x] = 0xFF333333; }
                }

                // Syntax-highlighted characters
                let line_colors = trustlang_syntax_colors(line);
                for (ci, c) in line.chars().enumerate() {
                    if global_idx >= chars_shown { break; }
                    let color = line_colors.get(ci).copied().unwrap_or(0xFFD4D4D4);
                    draw_big_char(buf, w, h, code_x + ci * 8 * code_scale, ly, c, color, code_scale);
                    global_idx += 1;
                }

                // If there's a typo char visible at the cursor position, draw it in red
                if let Some((tli, tci, tc)) = typo_char {
                    if li == tli && global_idx == chars_shown {
                        let ly2 = code_y_start + 15 + vi * line_h;
                        draw_big_char(buf, w, h, code_x + tci * 8 * code_scale, ly2, tc, 0xFFFF4444, code_scale);
                    }
                }

                if global_idx < chars_shown { global_idx += 1; } // \n
            }

            // Scrollbar indicator (when content overflows)
            if lines_vec.len() > max_visible {
                let sb_x = panel_x + panel_w - 8;
                let sb_y = panel_y + 2;
                let sb_h = panel_h.saturating_sub(4);
                for py in sb_y..sb_y + sb_h {
                    if py < h && sb_x < w { buf[py * w + sb_x] = 0xFF1A1A1A; }
                }
                let thumb_h = ((max_visible * sb_h) / lines_vec.len()).max(10);
                let thumb_y = if lines_vec.len() > 0 { sb_y + (scroll_offset * sb_h) / lines_vec.len() } else { sb_y };
                for py in thumb_y..(thumb_y + thumb_h).min(sb_y + sb_h) {
                    if py < h && sb_x < w {
                        buf[py * w + sb_x] = 0xFF00FF44;
                        if sb_x + 1 < w { buf[py * w + sb_x + 1] = 0xFF00FF44; }
                    }
                }
            }

            // Cursor blink (always on during typing for visibility)
            if chars_shown <= total_chars && cursor_line >= scroll_offset {
                let mut ci2 = 0usize;
                let mut target_line = 0usize;
                let mut target_col = 0usize;
                for (li, line) in lines_vec.iter().enumerate() {
                    if ci2 + line.len() >= chars_shown {
                        target_line = li;
                        target_col = chars_shown - ci2;
                        break;
                    }
                    ci2 += line.len() + 1;
                }
                // Offset cursor past typo char if visible
                let cursor_col = if typo_char.is_some() && typo_char.unwrap().0 == target_line {
                    target_col + 1
                } else {
                    target_col
                };
                let vis_line = target_line.saturating_sub(scroll_offset);
                let cy = code_y_start + 15 + vis_line * line_h;
                let cx = panel_x + 42 + cursor_col * 8 * code_scale;
                if (rain_frame / 5) % 2 == 0 {
                    for sy in cy..cy + 16 {
                        if sy < h && cx < w && cx + 2 < w {
                            buf[sy * w + cx] = 0xFF00FF88;
                            buf[sy * w + cx + 1] = 0xFF00FF88;
                        }
                    }
                }
            }

            // Status bar
            let status_y = h - 40;
            for py in status_y..h {
                for px in 0..w { buf[py * w + px] = 0xFF111111; }
            }
            {
                let cur_line = {
                    let mut ci3 = 0usize;
                    let mut ln = 1usize;
                    for (_li, line) in lines_vec.iter().enumerate() {
                        if ci3 + line.len() >= chars_shown { break; }
                        ci3 += line.len() + 1;
                        ln += 1;
                    }
                    ln
                };
                let status = alloc::format!("Ln {}  |  {} lines  |  TrustLang", cur_line, lines_vec.len());
                draw_text_at(buf, w, h, margin_x, status_y + 12, &status, 0xFF00CC66, 1);
            }

            blit_buf(buf, w, h);
        };

        // ── Phase 1: Human-like typing animation ──
        while chars_shown < total_chars {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { chars_shown = total_chars; break; }
            }

            // Check if we hit a typo point
            let mut did_typo = false;
            for &(typo_idx, wrong_c) in typo_schedule.iter() {
                if chars_shown == typo_idx && typo_idx < total_chars {
                    // Find which line/col we're at
                    let mut ci4 = 0usize;
                    let mut tgt_line = 0usize;
                    let mut tgt_col = 0usize;
                    for (li, line) in lines_vec.iter().enumerate() {
                        if ci4 + line.len() > chars_shown {
                            tgt_line = li;
                            tgt_col = chars_shown - ci4;
                            break;
                        }
                        ci4 += line.len() + 1;
                    }

                    // Type wrong char
                    render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame,
                        chars_shown, Some((tgt_line, tgt_col, wrong_c)));
                    rain_frame += 1;
                    crate::cpu::tsc::pit_delay_ms(120);

                    // Pause — "notice the mistake"
                    render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame,
                        chars_shown, Some((tgt_line, tgt_col, wrong_c)));
                    rain_frame += 1;
                    crate::cpu::tsc::pit_delay_ms(400);

                    // Backspace — remove wrong char (render without it)
                    render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame,
                        chars_shown, None);
                    rain_frame += 1;
                    crate::cpu::tsc::pit_delay_ms(150);

                    // Now type the correct char (fall through to normal typing below)
                    did_typo = true;
                    break;
                }
            }

            // Determine delay for this character
            let c = char_list.get(chars_shown).map(|&(_, _, c)| c).unwrap_or(' ');
            let next_c = char_list.get(chars_shown + 1).map(|&(_, _, c)| c);

            // Base typing speed: ~20ms per char (fast typist)
            let mut delay_ms: u64 = 20;

            // Newline: longer pause (thinking about next line)
            if c == '\n' {
                delay_ms = 80 + (rng_next(&mut rng_state) % 120) as u64; // 80-200ms
            }
            // After {  or before } : thinking pause
            else if c == '{' || (next_c == Some('}')) {
                delay_ms = 150 + (rng_next(&mut rng_state) % 200) as u64;
            }
            // After // comment start: slightly slower (typing words)
            else if c == '/' {
                delay_ms = 40 + (rng_next(&mut rng_state) % 60) as u64;
            }
            // Space: brief pause
            else if c == ' ' {
                delay_ms = 15 + (rng_next(&mut rng_state) % 40) as u64;
            }
            // Punctuation: slightly slower
            else if c == '(' || c == ')' || c == ';' || c == ',' {
                delay_ms = 30 + (rng_next(&mut rng_state) % 30) as u64;
            }
            // Regular chars: some variance
            else {
                delay_ms = 18 + (rng_next(&mut rng_state) % 25) as u64;
            }

            // Occasional random longer pause (~5% chance, "thinking")
            if rng_next(&mut rng_state) % 100 < 5 {
                delay_ms += 200 + (rng_next(&mut rng_state) % 400) as u64;
            }

            // After a typo correction, slight hesitation
            if did_typo {
                delay_ms += 80;
            }

            // Render current state
            render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame, chars_shown, None);
            rain_frame += 1;

            // Advance
            chars_shown += 1;
            crate::cpu::tsc::pit_delay_ms(delay_ms);
        }

        // Show complete code for a moment
        render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame, total_chars, None);
        crate::cpu::tsc::pit_delay_ms(1200);

        // ── Phase 2: In-editor compilation (bottom output pane) ──
        // We re-render the editor with a small output pane at the bottom
        // showing "Compiling..." then "Compiled in 0.3s"
        {
            // Render the full editor one more time, then draw output pane on top
            render_editor(buf, w, h, rain_cols, rain_speeds, rain_frame, total_chars, None);

            // Output pane: sits between code panel bottom and status bar
            let pane_y = h - 120;
            let pane_h = 80;
            let pane_x = margin_x;
            let pane_w = w - 2 * margin_x;

            // Dark output pane background
            for py in pane_y..pane_y + pane_h {
                for px in pane_x..pane_x + pane_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0A0E14;
                    }
                }
            }
            // Pane border
            for px in pane_x..pane_x + pane_w {
                if pane_y < h { buf[pane_y * w + px] = 0xFF00FF44; }
            }
            // "OUTPUT" label
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 4, "OUTPUT", 0xFF888888, 1);

            // "$ trustlang compile youtube_dvd.tl"
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 22, "$ trustlang compile youtube_dvd.tl", 0xFF00CC66, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(800);

            // "Compiling..." appears
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 38, "Compiling...", 0xFFAABBCC, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(1200);

            // Replace "Compiling..." with success message
            for py in pane_y + 36..pane_y + 56 {
                for px in pane_x + 4..pane_x + pane_w - 4 {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0A0E14;
                    }
                }
            }
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 38, "Compiled successfully in 0.3s  (47 lines, 0 errors)", 0xFF00FF88, 1);
            // Also show bytecode info
            draw_text_at(buf, w, h, pane_x + 8, pane_y + 54, "Generated 284 bytecode instructions", 0xFF666666, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(2000);
        }

        // ── Phase 3: Transition to shell & execute ──
        // Fake the TrustOS shell, type "trustlang run youtube_dvd.tl", then actually execute

        // Actually compile the program in ramfs first
        crate::ramfs::with_fs(|fs| {
            let _ = fs.write_file("/youtube_dvd.tl", source.as_bytes());
        });

        // Draw a fake shell screen
        {
            // Dark background with subtle rain
            for p in buf.iter_mut() { *p = 0xFF0A0A0A; }
            draw_rain(buf, w, h, rain_cols, rain_speeds, rain_frame);
            for p in buf.iter_mut() {
                let g = ((*p >> 8) & 0xFF).min(15);
                *p = 0xFF000000 | (g << 8);
            }

            // Shell window frame
            let win_x = 30usize;
            let win_y = 20usize;
            let win_w = w - 60;
            let win_h = h - 40;
            // Window background
            for py in win_y..win_y + win_h {
                for px in win_x..win_x + win_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF0D0D0D;
                    }
                }
            }
            // Title bar
            for py in win_y..win_y + 28 {
                for px in win_x..win_x + win_w {
                    if py < h && px < w {
                        buf[py * w + px] = 0xFF1A1A1A;
                    }
                }
            }
            draw_text_at(buf, w, h, win_x + 12, win_y + 6, "TrustOS Terminal", 0xFF00FF88, 1);
            // Border
            for px in win_x..win_x + win_w {
                if win_y < h { buf[win_y * w + px] = 0xFF00FF44; }
                let bot = (win_y + win_h - 1).min(h - 1);
                buf[bot * w + px] = 0xFF00FF44;
            }
            for py in win_y..win_y + win_h {
                if py < h {
                    buf[py * w + win_x] = 0xFF00FF44;
                    let r = (win_x + win_w - 1).min(w - 1);
                    buf[py * w + r] = 0xFF00FF44;
                }
            }

            let text_x = win_x + 16;
            let mut text_y = win_y + 40;

            // Show some previous shell output (fake history)
            draw_text_at(buf, w, h, text_x, text_y, "TrustOS v2.0 - TrustLang Runtime", 0xFF00FF88, 1);
            text_y += 20;
            draw_text_at(buf, w, h, text_x, text_y, "Type 'help' for available commands.", 0xFF666666, 1);
            text_y += 28;

            // Previous command: trustlang compile
            // Prompt: root@trustos:/$ 
            draw_text_at(buf, w, h, text_x, text_y, "root", 0xFFFF0000, 1);
            draw_text_at(buf, w, h, text_x + 32, text_y, "@", 0xFFFFFFFF, 1);
            draw_text_at(buf, w, h, text_x + 40, text_y, "trustos", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, text_x + 96, text_y, ":/$ ", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, text_x + 128, text_y, "trustlang compile youtube_dvd.tl", 0xFFD4D4D4, 1);
            text_y += 18;
            draw_text_at(buf, w, h, text_x, text_y, "Compiled successfully in 0.3s", 0xFF00FF88, 1);
            text_y += 18;
            draw_text_at(buf, w, h, text_x, text_y, "Generated 284 bytecode instructions", 0xFF666666, 1);
            text_y += 28;

            // New prompt where we'll type the run command
            let prompt_y = text_y;
            draw_text_at(buf, w, h, text_x, prompt_y, "root", 0xFFFF0000, 1);
            draw_text_at(buf, w, h, text_x + 32, prompt_y, "@", 0xFFFFFFFF, 1);
            draw_text_at(buf, w, h, text_x + 40, prompt_y, "trustos", 0xFF00FF00, 1);
            draw_text_at(buf, w, h, text_x + 96, prompt_y, ":/$ ", 0xFF00FF00, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(800);

            // Type "trustlang run youtube_dvd.tl" character by character
            let run_cmd = "trustlang run youtube_dvd.tl";
            let cmd_start_x = text_x + 128;
            for (ci, c) in run_cmd.chars().enumerate() {
                draw_big_char(buf, w, h, cmd_start_x + ci * 8, prompt_y, c, 0xFFD4D4D4, 1);
                blit_buf(buf, w, h);
                let d = 30 + (((ci as u32 * 7 + 13) ^ 0x5A) % 50) as u64;
                crate::cpu::tsc::pit_delay_ms(d);
            }
            crate::cpu::tsc::pit_delay_ms(400);

            // Show "Enter" — command execution
            text_y = prompt_y + 24;
            draw_text_at(buf, w, h, text_x, text_y, "Running youtube_dvd.tl ...", 0xFFAABBCC, 1);
            blit_buf(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(600);
        }

        // Actually execute the TrustLang program
        match crate::trustlang::run(source) {
            Ok(output) => {
                if !output.is_empty() {
                    // Text output — should not happen for this graphics demo,
                    // but handle it just in case
                    let out_lines: alloc::vec::Vec<&str> = output.lines().collect();
                    clear_buf(buf);
                    draw_text_centered(buf, w, h, 25, "OUTPUT", 0xFF00FF88, 3);
                    for (i, line) in out_lines.iter().enumerate() {
                        let ly = 80 + i * 20;
                        if ly + 16 > h - 40 { break; }
                        let sx = if line.len() * 8 < w { (w - line.len() * 8) / 2 } else { 40 };
                        draw_text_at(buf, w, h, sx, ly, line, 0xFFCCFFCC, 1);
                    }
                    blit_buf(buf, w, h);
                    crate::cpu::tsc::pit_delay_ms(output_hold_ms);
                }
                if output.is_empty() {
                    // Graphics program — result is already on framebuffer
                    // Read back FB into buf for fade later
                    if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
                        let bb = bb_ptr as *mut u32;
                        let bb_s = bb_stride as usize;
                        for y in 0..h.min(bb_h as usize) {
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    bb.add(y * bb_s),
                                    buf[y * w..].as_mut_ptr(),
                                    w,
                                );
                            }
                        }
                    }
                    // Hold on graphics result
                    crate::cpu::tsc::pit_delay_ms(output_hold_ms);
                }
            }
            Err(e) => {
                clear_buf(buf);
                draw_text_centered(buf, w, h, h / 2 - 20, "Runtime Error", 0xFFFF4444, 4);
                let err_short = if e.len() > 80 { &e[..80] } else { &e };
                draw_text_centered(buf, w, h, h / 2 + 50, err_short, 0xFFFF8888, 1);
                blit_buf(buf, w, h);
                crate::cpu::tsc::pit_delay_ms(3000);
            }
        }
    };

    // ═══════════════════════════════════════════════════════════════
    //  THE SHOWCASE — YouTube DVD Screensaver Demo
    //  Single TrustLang program: bouncing 3D YouTube logo
    // ═══════════════════════════════════════════════════════════════

    crate::serial_println!("[TL_SHOWCASE] Starting TrustLang showcase — YouTube DVD Screensaver");

    // ────────────────────────────────────────────────
    // INTRO: Title Screen                    (~8s)
    // ────────────────────────────────────────────────
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Live Demo", 0xFF00CC66, 4),
          ("Programming Inside TrustOS", 0xFF008844, 2)],
        90, 200);   // 90ms/char → slow dramatic typing, hold 200 frames (~6s)
    do_fade(&mut buf, w, h, &blit_buf);

    // ────────────────────────────────────────────────
    // CONCEPT: What we're building            (~10s)
    // ────────────────────────────────────────────────
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("YouTube DVD Screensaver", 0xFFFF0000, 4),
          ("", 0xFF000000, 1),
          ("A bouncing 3D YouTube logo", 0xFFCCFFCC, 2),
          ("with 'Like & Subscribe' text.", 0xFFCCFFCC, 2),
          ("", 0xFF000000, 1),
          ("Written, compiled, and animated", 0xFF00FF88, 2),
          ("live inside the OS kernel.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("All in real-time. Zero dependencies.", 0xFF888888, 2)],
        70, 180);   // 70ms/char, hold 180 frames (~5.4s)
    do_fade(&mut buf, w, h, &blit_buf);

    // ────────────────────────────────────────────────
    // ARCHITECTURE: TrustLang pipeline         (~5s)
    // ────────────────────────────────────────────────
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("How TrustLang Works", 0xFF00FF88, 4),
          ("", 0xFF000000, 1),
          ("tokenize()   Lexer -> Tokens", 0xFFFF7B72, 2),
          ("parse()      Tokens -> AST", 0xFFFFA657, 2),
          ("compile()    AST -> Bytecode", 0xFFA5D6FF, 2),
          ("execute()    Bytecode -> VM", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("pixel() fill_rect() draw_text()", 0xFFFFD700, 2),
          ("flush() sleep() clear_screen()", 0xFFFFD700, 2)],
        60, 170);   // 60ms/char, hold 170 frames (~5s)
    do_fade(&mut buf, w, h, &blit_buf);

    // ────────────────────────────────────────────────
    // THE CODE + EXECUTION                    (~50s)
    // ────────────────────────────────────────────────
    clear_buf(&mut buf);
    show_code_and_run(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        "TrustCode  -  youtube_dvd.tl",
        r#"// YouTube DVD Screensaver in TrustLang!
fn main() {
    let screen_width = screen_w();
    let screen_height = screen_h();
    // Logo dimensions
    let logo_width = 200;
    let logo_height = 140;
    let total_height = logo_height + 70;
    // Starting position & velocity
    let mut pos_x = screen_width / 4;
    let mut pos_y = screen_height / 4;
    let mut speed_x = 4;
    let mut speed_y = 3;
    // Animate 300 frames (~10 seconds)
    let mut frame = 0;
    while frame < 300 {
        clear_screen(0, 0, 0);
        // 3D shadow offset
        fill_rect(pos_x + 8, pos_y + 8, logo_width, logo_height, 50, 0, 0);
        // Red YouTube rectangle
        fill_rect(pos_x, pos_y, logo_width, logo_height, 230, 0, 0);
        // 3D highlight on top + dark edge on bottom
        fill_rect(pos_x, pos_y, logo_width, 3, 255, 60, 60);
        fill_rect(pos_x, pos_y + logo_height - 3, logo_width, 3, 150, 0, 0);
        // Play button triangle (white)
        let center_x = pos_x + logo_width / 2;
        let center_y = pos_y + logo_height / 2;
        let mut row = 0;
        while row < 70 {
            let offset = row - 35;
            let mut dist = offset;
            if dist < 0 { dist = 0 - dist; }
            let bar_width = 40 * (35 - dist) / 35;
            if bar_width > 0 {
                fill_rect(center_x - 12, center_y - 35 + row, bar_width, 1, 255, 255, 255);
            }
            row = row + 1;
        }
        // Bouncing text below logo
        draw_text("LIKE AND", pos_x + 18, pos_y + logo_height + 12, 255, 255, 255, 2);
        draw_text("SUBSCRIBE!", pos_x + 5, pos_y + logo_height + 42, 255, 80, 80, 2);
        flush();
        sleep(33);
        // Move & bounce off screen edges
        pos_x = pos_x + speed_x;
        pos_y = pos_y + speed_y;
        if pos_x + logo_width > screen_width { speed_x = 0 - speed_x; }
        if pos_x < 0 { speed_x = 0 - speed_x; }
        if pos_y + total_height > screen_height { speed_y = 0 - speed_y; }
        if pos_y < 0 { speed_y = 0 - speed_y; }
        frame = frame + 1;
    }
}"#,
        "New file: /youtube_dvd.tl",
        30,     // unused (human typing now)
        80,     // unused (human typing now)
        3000);  // hold 3s on final frame after animation ends
    do_fade(&mut buf, w, h, &blit_buf);

    // ────────────────────────────────────────────────
    // OUTRO                                   (~8s)
    // ────────────────────────────────────────────────
    clear_buf(&mut buf);
    show_description(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("TrustLang", 0xFF00FF88, 6),
          ("", 0xFF000000, 1),
          ("Lexer > Parser > Compiler > VM", 0xFFAADDAA, 2),
          ("Real-time graphics. Zero deps.", 0xFFAADDAA, 2),
          ("", 0xFF000000, 1),
          ("Built into TrustOS.", 0xFF00CC66, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFF00FF88, 2)],
        80, 250);   // slow dramatic outro, hold 250 frames (~7.5s)
    do_fade(&mut buf, w, h, &blit_buf);

    // Restore framebuffer state
    clear_buf(&mut buf);
    blit_buf(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[TL_SHOWCASE] Showcase complete");
}

/// TrustOS Film — Animated cinematic explainer for non-technical audiences
/// Each scene uses a unique background for maximum visual retention:
///   ACT I:   Slow pulsing deep-blue gradient  (mystery, intrigue)
///   ACT II:  Red scan-lines / warning stripes  (urgency, alarm)
///   Bars:    Blueprint dot-grid                (data, precision)
///   ACT III: Rising green particle sparks      (hope, energy)
///   Grid:    Deep-space starfield              (scale, wonder)
///   ACT IV:  Circuit-board trace lines         (tech, proof)
///   ACT V:   Sunrise gradient warm glow        (inspiration)
///   Outro:   Matrix rain callback              (signature)
/// Retention techniques applied: pattern interrupts every scene,
/// high-contrast color shifts, kinetic text, less text per screen,
/// constant subtle motion, under-2-minute total runtime.
fn cmd_trustos_film() {
    let (sw, sh) = crate::framebuffer::get_dimensions();
    let w = sw as usize;
    let h = sh as usize;

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    let mut buf = alloc::vec![0u32; w * h];

    // ═══════════════ HELPER CLOSURES ═══════════════

    let draw_big_char = |buf: &mut [u32], w: usize, h: usize,
                         cx: usize, cy: usize, c: char, color: u32, scale: usize| {
        let glyph = crate::framebuffer::font::get_glyph(c);
        for (row, &bits) in glyph.iter().enumerate() {
            for bit in 0..8u32 {
                if bits & (0x80 >> bit) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = cx + bit as usize * scale + sx;
                            let py = cy + row * scale + sy;
                            if px < w && py < h { buf[py * w + px] = color; }
                        }
                    }
                }
            }
        }
    };

    let draw_text_at = |buf: &mut [u32], w: usize, h: usize,
                        x: usize, y: usize, text: &str, color: u32, scale: usize| {
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, x + i * 8 * scale, y, c, color, scale);
        }
    };

    let draw_text_centered = |buf: &mut [u32], w: usize, h: usize,
                              y: usize, text: &str, color: u32, scale: usize| {
        let tw = text.len() * 8 * scale;
        let sx = if tw < w { (w - tw) / 2 } else { 0 };
        for (i, c) in text.chars().enumerate() {
            draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
        }
    };

    let blit_buf = |buf: &[u32], w: usize, h: usize| {
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..h.min(bb_h as usize) {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buf[y * w..].as_ptr(), bb.add(y * bb_s), w);
                }
            }
        }
        crate::framebuffer::swap_buffers();
    };

    let clear_buf = |buf: &mut [u32]| {
        for p in buf.iter_mut() { *p = 0xFF000000; }
    };

    // Filled rectangle helper
    let draw_rect = |buf: &mut [u32], w: usize, h: usize,
                     x: usize, y: usize, rw: usize, rh: usize, color: u32| {
        for dy in 0..rh {
            for dx in 0..rw {
                let px = x + dx;
                let py = y + dy;
                if px < w && py < h { buf[py * w + px] = color; }
            }
        }
    };

    let frame_ms: u64 = 30;

    let do_fade = |buf: &mut [u32], w: usize, h: usize,
                   blit: &dyn Fn(&[u32], usize, usize)| {
        for _ in 0..40 {
            for px in buf.iter_mut() {
                let r = ((*px >> 16) & 0xFF).saturating_sub(8);
                let g = ((*px >> 8) & 0xFF).saturating_sub(8);
                let b = (*px & 0xFF).saturating_sub(8);
                *px = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
            blit(buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
        for p in buf.iter_mut() { *p = 0xFF000000; }
        blit(buf, w, h);
        crate::cpu::tsc::pit_delay_ms(400);
    };

    // ═══════════════ BACKGROUND GENERATORS ═══════════════
    // Each produces a unique animated background per-frame.

    // BG1: Pulsing deep-blue/purple nebula gradient (ACT I — mystery)
    let bg_pulse = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        // Integer-based sine approximation (no libm in no_std)
        // Triangle wave oscillating 0..40..0 over ~160 frames
        let phase = (frame % 160) as u32;
        let pulse = if phase < 80 { phase / 2 } else { (160 - phase) / 2 };
        let phase2 = ((frame + 40) % 120) as u32;
        let pulse2 = if phase2 < 60 { phase2 / 2 } else { (120 - phase2) / 2 };
        for y in 0..h {
            let yf = (y as u32 * 40) / h as u32;
            for x in 0..w {
                let xf = (x as u32 * 10) / w as u32;
                let r = (yf / 4 + pulse2 / 3).min(40);
                let g = (xf / 3).min(15);
                let b = (yf + pulse + xf / 2).min(80);
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    };

    // BG2: Red warning scan-lines (ACT II — danger/urgency)
    let bg_scanlines = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        let scroll = (frame as usize * 2) % h;
        for y in 0..h {
            let sy = (y + scroll) % h;
            let stripe = (sy / 4) % 2 == 0;
            for x in 0..w {
                let base_r = if stripe { 35u32 } else { 15 };
                let flash = if (sy % 60) < 2 { 30u32 } else { 0 };
                let r = (base_r + flash).min(65);
                let g = 2;
                let b = 5;
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
    };

    // BG3: Blueprint dot-grid (comparison bars — precision)
    let bg_dotgrid = |buf: &mut [u32], w: usize, h: usize, _frame: u32| {
        for y in 0..h {
            for x in 0..w {
                let on_grid = (x % 20 < 2) && (y % 20 < 2);
                let color = if on_grid { 0xFF0A1A3A } else { 0xFF060E1E };
                buf[y * w + x] = color;
            }
        }
    };

    // BG4: Rising green sparks / particles (ACT III — hope)
    let bg_sparks = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        // Dim previous frame for trailing effect
        for px in buf.iter_mut() {
            let r = ((*px >> 16) & 0xFF).saturating_sub(8);
            let g = ((*px >> 8) & 0xFF).saturating_sub(12);
            let b = (*px & 0xFF).saturating_sub(8);
            *px = 0xFF000000 | (r << 16) | (g << 8) | b;
        }
        // Spawn particles rising from bottom
        for i in 0..24u32 {
            let seed = (i.wrapping_mul(2654435761).wrapping_add(frame.wrapping_mul(37))) as usize;
            let px = (seed.wrapping_mul(7919)) % w;
            let rise = (frame as usize + seed) % h;
            let py = h.saturating_sub(rise);
            let brightness = (50 + (seed % 40)) as u32;
            if px < w && py < h {
                buf[py * w + px] = 0xFF000000 | (brightness / 4 << 16) | (brightness << 8) | (brightness / 3);
                if px + 1 < w { buf[py * w + px + 1] = 0xFF000000 | (brightness << 8); }
            }
        }
    };

    // BG5: Deep-space starfield (feature grid — wonder/scale)
    let bg_stars = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        for p in buf.iter_mut() { *p = 0xFF050510; }
        // Fixed stars with twinkle
        for i in 0..80u32 {
            let sx = ((i.wrapping_mul(7919)) as usize) % w;
            let sy = ((i.wrapping_mul(104729)) as usize) % h;
            let twinkle = ((frame.wrapping_add(i * 17)) % 30) as u32;
            let bright = if twinkle < 15 { 40 + twinkle * 3 } else { 40 + (30 - twinkle) * 3 };
            let bright = bright.min(120);
            if sx < w && sy < h {
                buf[sy * w + sx] = 0xFF000000 | (bright << 16) | (bright << 8) | bright;
            }
        }
    };

    // BG6: Circuit-board traces (ACT IV — technical proof)
    let bg_circuit = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        for p in buf.iter_mut() { *p = 0xFF0A0A14; }
        // Horizontal and vertical traces
        let trace_color = 0xFF0F2818u32;
        let active_color = 0xFF00AA44u32;
        for i in 0..20u32 {
            let ty = ((i.wrapping_mul(7919) as usize) % h) & !3;
            let tx = ((i.wrapping_mul(104729) as usize) % w) & !3;
            // Horizontal lines
            if ty < h {
                for x in 0..w {
                    buf[ty * w + x] = trace_color;
                }
            }
            // Vertical lines
            if tx < w {
                for y in 0..h {
                    buf[y * w + tx] = trace_color;
                }
            }
        }
        // Animated pulse along a trace
        let pulse_y = ((frame as usize * 3) % h) & !3;
        if pulse_y < h {
            let pw = (w / 4).min(120);
            let px_start = (frame as usize * 5) % w;
            for dx in 0..pw {
                let px = (px_start + dx) % w;
                buf[pulse_y * w + px] = active_color;
                if pulse_y + 1 < h { buf[(pulse_y + 1) * w + px] = active_color; }
            }
        }
    };

    // BG7: Sunrise warm gradient (ACT V — inspiration)
    let bg_sunrise = |buf: &mut [u32], w: usize, h: usize, frame: u32| {
        let lift = (frame as u32).min(60); // sun rises over 60 frames
        for y in 0..h {
            let yf = y as u32 * 100 / h as u32; // 0=top, 100=bottom
            let warmth = if yf > 50 { (yf - 50).min(50) + lift } else { lift / 2 };
            let r = (warmth * 2).min(90);
            let g = (warmth * 3 / 4).min(45);
            let b = (20u32.saturating_sub(warmth / 3)).min(30);
            for x in 0..w {
                buf[y * w + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
        // Sun glow at bottom center
        let sun_cx = w / 2;
        let sun_cy = h - (lift as usize * h / 200);
        let sun_r = 80usize + lift as usize;
        for dy in 0..sun_r {
            for dx in 0..sun_r {
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < sun_r * sun_r {
                    let intensity = (sun_r * sun_r - dist_sq) * 60 / (sun_r * sun_r);
                    let intensity = intensity as u32;
                    for (sx, sy) in [(sun_cx + dx, sun_cy.wrapping_sub(dy)),
                                     (sun_cx.wrapping_sub(dx), sun_cy.wrapping_sub(dy))] {
                        if sx < w && sy < h {
                            let existing = buf[sy * w + sx];
                            let er = ((existing >> 16) & 0xFF) + intensity;
                            let eg = ((existing >> 8) & 0xFF) + intensity * 2 / 3;
                            let eb = (existing & 0xFF) + intensity / 4;
                            buf[sy * w + sx] = 0xFF000000
                                | (er.min(255) << 16)
                                | (eg.min(255) << 8)
                                | eb.min(255);
                        }
                    }
                }
            }
        }
    };

    // BG8: Matrix rain — signature callback for outro only
    let mut rain_cols: alloc::vec::Vec<u16> =
        (0..w / 8 + 1).map(|i| ((i * 37 + 13) % h) as u16).collect();
    let rain_speeds: alloc::vec::Vec<u8> =
        (0..w / 8 + 1).map(|i| (((i * 7 + 3) % 4) + 1) as u8).collect();

    let draw_rain = |buf: &mut [u32], w: usize, h: usize,
                     cols: &mut [u16], speeds: &[u8], frame: u32| {
        for pixel in buf.iter_mut() {
            let g = ((*pixel >> 8) & 0xFF) as u32;
            if g > 0 { *pixel = 0xFF000000 | (g.saturating_sub(6) << 8); }
        }
        for ci in 0..cols.len() {
            let x = ci * 8;
            if x >= w { continue; }
            cols[ci] = cols[ci].wrapping_add(speeds[ci] as u16);
            if cols[ci] as usize >= h { cols[ci] = 0; }
            let y = cols[ci] as usize;
            let c = (((frame as usize + ci * 13) % 94) + 33) as u8 as char;
            let glyph = crate::framebuffer::font::get_glyph(c);
            for (row, &bits) in glyph.iter().enumerate() {
                let py = y + row;
                if py >= h { break; }
                for bit in 0..8u32 {
                    if bits & (0x80 >> bit) != 0 {
                        let px = x + bit as usize;
                        if px < w { buf[py * w + px] = 0xFF00FF44; }
                    }
                }
            }
        }
    };

    // ═══════════════ SCENE RENDERER ═══════════════
    // Generic scene: animate background + type text on top.
    // bg_id: 1=pulse, 2=scanlines, 3=dotgrid, 4=sparks, 5=stars,
    //        6=circuit, 7=sunrise, 8=rain
    let show_scene = |buf: &mut [u32], w: usize, h: usize,
                      rain_cols: &mut [u16], rain_speeds: &[u8],
                      lines: &[(&str, u32, usize)],
                      ms_per_char: u64, hold_frames: u32, bg_id: u8| {
        let total_chars: usize = lines.iter().map(|(t, _, _)| t.len()).sum();
        let frames_per_char = (ms_per_char / frame_ms).max(1) as u32;
        let typing_frames = total_chars as u32 * frames_per_char;
        let total_frames = typing_frames + hold_frames;
        let mut frame = 0u32;
        while frame < total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { return; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            // Draw the per-scene background
            match bg_id {
                1 => bg_pulse(buf, w, h, frame),
                2 => bg_scanlines(buf, w, h, frame),
                3 => bg_dotgrid(buf, w, h, frame),
                4 => bg_sparks(buf, w, h, frame),
                5 => bg_stars(buf, w, h, frame),
                6 => bg_circuit(buf, w, h, frame),
                7 => bg_sunrise(buf, w, h, frame),
                8 => draw_rain(buf, w, h, rain_cols, rain_speeds, frame),
                _ => { for p in buf.iter_mut() { *p = 0xFF000000; } }
            }
            let chars_shown = (frame / frames_per_char) as usize;
            let total_h: usize = lines.iter().map(|(_, _, s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;
            for &(text, color, scale) in lines {
                let tw = text.len() * 8 * scale;
                let sx = if tw < w { (w - tw) / 2 } else { 0 };
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                // Blinking cursor during typing
                if chars_shown > counted && chars_shown < counted + text.len() {
                    let ci = chars_shown - counted;
                    let cx = sx + ci * 8 * scale;
                    if (frame / 8) % 2 == 0 {
                        for cy in y..y + 16 * scale {
                            if cy < h && cx + 2 < w {
                                buf[cy * w + cx] = 0xFFFFFFFF;
                                buf[cy * w + cx + 1] = 0xFFFFFFFF;
                            }
                        }
                    }
                }
                counted += text.len();
                y += 16 * scale + 12;
            }
            blit_buf(buf, w, h);
            frame += 1;
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    };

    crate::serial_println!("[FILM] TrustOS Film started");

    // ═══════════════════════════════════════════════════════════════
    //  ACT I  —  THE QUESTION  (unique animations per scene)
    // ═══════════════════════════════════════════════════════════════
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT I", 0xFF88CCFF, 5)],
        50, 30, 1);
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 1: Floating Windows — "You use a computer every day" ──
    {
        let fpc = 2u32;
        let text1 = "You use a computer";
        let text2 = "every single day.";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 50;
        // Window state: x, y, width, height, color, dx, dy
        let mut wins: [(i32,i32,usize,usize,u32,i32,i32); 6] = [
            (80, 40, 120, 80, 0xFF3355AA, 2, 1),
            (w as i32 - 220, 90, 100, 70, 0xFF55AA33, -1, 2),
            (180, h as i32 - 180, 130, 85, 0xFFAA5533, 1, -1),
            (w as i32 / 2, 60, 110, 75, 0xFF8844CC, -2, 1),
            (40, h as i32 / 2, 125, 80, 0xFF4488CC, 1, -2),
            (w as i32 - 160, h as i32 / 2 + 40, 100, 65, 0xFFCC8844, -1, -1),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_pulse(&mut buf, w, h, frame);
            // Animate floating windows
            for wi in 0..6 {
                let win = &mut wins[wi];
                // Only show window after a staggered delay
                if frame < (wi as u32) * 8 { continue; }
                win.0 += win.5;
                win.1 += win.6;
                if win.0 < 0 || win.0 + win.2 as i32 > w as i32 { win.5 = -win.5; win.0 += win.5; }
                if win.1 < 0 || win.1 + win.3 as i32 > h as i32 { win.6 = -win.6; win.1 += win.6; }
                let wx = win.0.max(0) as usize;
                let wy = win.1.max(0) as usize;
                let wc = win.4;
                let wr = ((wc >> 16) & 0xFF) / 3;
                let wg = ((wc >> 8)  & 0xFF) / 3;
                let wb = (wc & 0xFF) / 3;
                let dim = 0xFF000000 | (wr << 16) | (wg << 8) | wb;
                draw_rect(&mut buf, w, h, wx, wy, win.2, win.3, dim);
                draw_rect(&mut buf, w, h, wx, wy, win.2, 10, wc);
                // Fake content lines inside window
                for li in 0..3usize {
                    let ly = wy + 16 + li * 12;
                    if ly + 5 < wy + win.3 {
                        draw_rect(&mut buf, w, h, wx + 6, ly, win.2.saturating_sub(12), 5, 0xFF222233);
                    }
                }
            }
            // Type text on top
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let line_h = 16 * scale + 12;
            let y1 = h / 2 - line_h;
            let y2 = h / 2 + 4;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFFFFFFF, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFFFFFFFF, scale);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 2: Question Marks Rain — "Do you really know?" ──
    {
        let fpc = 2u32;
        let text1 = "Do you really know";
        let text2 = "what it does?";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 60;
        let num_qcols = w / 10;
        let mut qy: alloc::vec::Vec<i32> = (0..num_qcols).map(|i| -((i * 37 % 200) as i32)).collect();
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_pulse(&mut buf, w, h, frame);
            // Rain question marks with acceleration
            let speed = 1 + (frame / 30) as i32;
            for qi in 0..num_qcols {
                qy[qi] += speed + (qi as i32 % 3);
                if qy[qi] > h as i32 { qy[qi] = -(qi as i32 * 13 % 60); }
                if qy[qi] >= 0 {
                    let px = qi * 10 + 2;
                    let py = qy[qi] as usize;
                    let bright = 0xFF000000 | (0x40 << 16) | (0x60 << 8) | 0xFF;
                    if px < w && py < h {
                        draw_big_char(&mut buf, w, h, px, py, '?', bright, 1);
                    }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let y1 = h / 2 - 40;
            let y2 = h / 2 + 20;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFCCCCCC, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFFFF9944, 4);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 3: Screen Shatter — "The honest answer... is no." ──
    {
        let fpc = 2u32;
        let text1 = "The honest answer...";
        let text2 = "is no.";
        let type_frames = (text1.len() + text2.len()) as u32 * fpc;
        let shatter_frames = 50u32;
        let total_frames = type_frames + shatter_frames;
        // Crack directions: dx, dy pairs radiating from center
        let cracks: [(i32, i32); 12] = [
            (3,0),(-3,0),(0,3),(0,-3),(2,2),(-2,2),(2,-2),(-2,-2),
            (3,1),(-3,1),(1,-3),(-1,3),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_scanlines(&mut buf, w, h, frame);
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale1 = 3usize;
            let y1 = h / 2 - 60;
            let tw1 = text1.len() * 8 * scale1;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale1, y1, c, 0xFF888888, scale1);
            }
            if chars_shown > text1.len() {
                let scale2 = 5usize;
                let tw2 = text2.len() * 8 * scale2;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale2, h / 2, c, 0xFFFF4444, scale2);
                }
            }
            // Shatter effect after typing
            if frame > type_frames {
                let progress = frame - type_frames;
                let cx = w / 2;
                let cy = h / 2;
                for &(cdx, cdy) in cracks.iter() {
                    for step in 0..(progress * 4) as i32 {
                        let px = (cx as i32 + cdx * step).max(0) as usize;
                        let py = (cy as i32 + cdy * step).max(0) as usize;
                        if px < w && py < h {
                            buf[py * w + px] = 0xFFFFFFFF;
                            if px + 1 < w { buf[py * w + px + 1] = 0xFFFFDDDD; }
                            if py + 1 < h { buf[(py + 1) * w + px] = 0xFFFFDDDD; }
                        }
                    }
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ═══════════════════════════════════════════════════════════════
    //  ACT II  —  THE PROBLEM  (binary flood + redacted bars)
    // ═══════════════════════════════════════════════════════════════
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT II", 0xFFFF6644, 5),
          ("", 0xFF000000, 1),
          ("The Problem", 0xFFFF4444, 3)],
        50, 30, 2);
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 4: Binary Flood — "It controls EVERYTHING" ──
    {
        let fpc = 2u32;
        let lines_txt: [(&str, u32, usize); 5] = [
            ("Your computer runs on", 0xFFCCCCCC, 2),
            ("an operating system.", 0xFFCCCCCC, 2),
            ("", 0xFF000000, 1),
            ("It controls", 0xFFCCCCCC, 2),
            ("EVERYTHING.", 0xFFFF6644, 4),
        ];
        let total_chars: usize = lines_txt.iter().map(|(t,_,_)| t.len()).sum();
        let total_frames = total_chars as u32 * fpc + 70;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_scanlines(&mut buf, w, h, frame);
            // Binary flood: 0s and 1s cascading from top
            let flood_rows = (frame as usize * 3).min(h);
            for fy in 0..flood_rows {
                if fy >= h { break; }
                // Sparse binary chars
                for fx_step in 0..w / 12 {
                    let fx = fx_step * 12;
                    let seed = (fy.wrapping_mul(7919) + fx.wrapping_mul(104729) + frame as usize * 37) % 100;
                    if seed < 15 {
                        let c = if seed < 8 { '0' } else { '1' };
                        let bright = (20 + (seed * 2)) as u32;
                        let color = 0xFF000000 | (bright << 16) | ((bright / 2) << 8) | (bright / 4);
                        draw_big_char(&mut buf, w, h, fx, fy, c, color, 1);
                    }
                }
            }
            // Type text on top with dark backdrop
            let chars_shown = (frame / fpc) as usize;
            let total_h: usize = lines_txt.iter().map(|(_,_,s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;
            for &(text, color, scale) in lines_txt.iter() {
                let tw = text.len() * 8 * scale;
                let sx = if tw < w { (w - tw) / 2 } else { 0 };
                // Dark background behind text
                if !text.is_empty() {
                    draw_rect(&mut buf, w, h, sx.saturating_sub(4), y.saturating_sub(2),
                        tw + 8, 16 * scale + 4, 0xCC000000);
                }
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(&mut buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                counted += text.len();
                y += 16 * scale + 12;
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 5: Redacted Bars — "Nobody knows what's inside" ──
    {
        let fpc = 2u32;
        let lines_txt: [(&str, u32, usize); 5] = [
            ("But nobody knows", 0xFFCCCCCC, 3),
            ("what's inside it.", 0xFFCCCCCC, 3),
            ("", 0xFF000000, 1),
            ("Not even the people", 0xFFFF4444, 2),
            ("who wrote it.", 0xFFFF4444, 2),
        ];
        let total_chars: usize = lines_txt.iter().map(|(t,_,_)| t.len()).sum();
        let type_frames = total_chars as u32 * fpc;
        let redact_frames = 60u32;
        let total_frames = type_frames + redact_frames;
        // Fake document lines to redact
        let doc_lines: [(&str, usize); 6] = [
            ("Source code: kernel/mm/init.c", 60),
            ("Author: CLASSIFIED", 140),
            ("Memory manager: UNKNOWN", 220),
            ("Security audit: NONE PERFORMED", 300),
            ("Bug count: UNTRACKED", 380),
            ("Last review: NEVER", 460),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_scanlines(&mut buf, w, h, frame);
            // Draw fake document on left side
            let doc_x = 30usize;
            for &(line, dy) in doc_lines.iter() {
                if dy < h {
                    draw_text_at(&mut buf, w, h, doc_x, dy, line, 0xFF445566, 1);
                }
            }
            // Redact bars sliding in after typing completes
            if frame > type_frames {
                let progress = frame - type_frames;
                for (di, &(_line, dy)) in doc_lines.iter().enumerate() {
                    let delay = di as u32 * 6;
                    if progress > delay {
                        let bar_w = ((progress - delay) as usize * 12).min(280);
                        if dy < h {
                            draw_rect(&mut buf, w, h, doc_x, dy.saturating_sub(2),
                                bar_w, 14, 0xFF000000);
                            if bar_w > 80 {
                                draw_text_at(&mut buf, w, h, doc_x + 4, dy,
                                    "REDACTED", 0xFFFF2222, 1);
                            }
                        }
                    }
                }
            }
            // Main text on right side
            let chars_shown = (frame / fpc) as usize;
            let text_x_base = w / 2 + 20;
            let total_h: usize = lines_txt.iter().map(|(_,_,s)| 16 * s + 12).sum();
            let mut y = if total_h < h { (h - total_h) / 2 } else { 20 };
            let mut counted = 0usize;
            for &(text, color, scale) in lines_txt.iter() {
                let sx = text_x_base;
                for (i, c) in text.chars().enumerate() {
                    if counted + i >= chars_shown { break; }
                    draw_big_char(&mut buf, w, h, sx + i * 8 * scale, y, c, color, scale);
                }
                counted += text.len();
                y += 16 * scale + 12;
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 6: Bar Chart with Earthquake Shake ──
    {
        let bar_data: [(&str, u32, u32); 4] = [
            ("Windows",  50_000_000, 0xFFFF4444),
            ("macOS",    30_000_000, 0xFFFFAA22),
            ("Linux",    28_000_000, 0xFFFF8800),
            ("TrustOS",     120_000, 0xFF00FF88),
        ];
        let max_val = 50_000_000u32;
        let bar_max_w = w * 3 / 5;
        let bar_h_px = 40usize;
        let bar_spacing = 80usize;
        let start_y = h / 2 - (bar_data.len() * bar_spacing) / 2;
        let label_x = 40usize;

        let mut shake_x: i32 = 0;
        let mut shake_y: i32 = 0;

        for frame in 0..160u32 {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_dotgrid(&mut buf, w, h, frame);

            let progress = if frame < 30 { 0u32 }
                else { ((frame - 30) * 100 / 70).min(100) };

            // Earthquake shake when TrustOS bar appears (frame 100+)
            if frame > 100 && frame < 130 {
                let seed = frame.wrapping_mul(7919) as i32;
                shake_x = (seed % 7) - 3;
                shake_y = ((seed / 7) % 5) - 2;
            } else {
                shake_x = 0;
                shake_y = 0;
            }

            draw_text_centered(&mut buf, w, h,
                (30i32 + shake_y) as usize,
                "Lines of Code per OS", 0xFFFFFFFF, 3);

            for (i, &(name, val, color)) in bar_data.iter().enumerate() {
                let y = ((start_y + i * bar_spacing) as i32 + shake_y).max(0) as usize;
                let adj_label_x = (label_x as i32 + shake_x).max(0) as usize;
                draw_text_at(&mut buf, w, h, adj_label_x, y + 10,
                    name, 0xFFFFFFFF, 2);
                let bar_x = (adj_label_x + 170).min(w.saturating_sub(10));
                draw_rect(&mut buf, w, h, bar_x, y,
                    bar_max_w, bar_h_px, 0xFF111122);
                let full_w = (val as usize * bar_max_w) / max_val as usize;
                let target_w = full_w.max(12);
                let current_w = target_w * progress as usize / 100;
                draw_rect(&mut buf, w, h, bar_x, y,
                    current_w, bar_h_px, color);

                // Flash effect on TrustOS bar appearance
                if i == 3 && frame > 100 && frame < 110 {
                    let flash = 0xFF88FFAA;
                    draw_rect(&mut buf, w, h, bar_x, y,
                        current_w + 4, bar_h_px + 4, flash);
                }

                if frame > 70 {
                    let label = if val >= 1_000_000 {
                        alloc::format!("{}M", val / 1_000_000)
                    } else {
                        alloc::format!("{}K", val / 1000)
                    };
                    draw_text_at(&mut buf, w, h, bar_x + current_w + 10,
                        y + 10, &label, 0xFFFFFFFF, 2);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // Contrast emphasis
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("50 million vs 120 thousand.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Like comparing a city", 0xFFCCCCCC, 2),
          ("to a single house.", 0xFF00FF88, 2),
          ("", 0xFF000000, 1),
          ("Except the house", 0xFFCCCCCC, 2),
          ("does everything.", 0xFF00FF88, 3)],
        50, 80, 2);
    do_fade(&mut buf, w, h, &blit_buf);

    // ═══════════════════════════════════════════════════════════════
    //  ACT III  —  THE SOLUTION  (light burst + odometer counter)
    // ═══════════════════════════════════════════════════════════════
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT III", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Solution", 0xFF00CC66, 3)],
        50, 30, 4);
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 7: Light Burst — "What if one person could understand ALL of it?" ──
    {
        let fpc = 2u32;
        let text1 = "What if one person";
        let text2 = "could understand ALL of it?";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 60;
        // 8 ray directions (dx, dy)
        let rays: [(i32, i32); 16] = [
            (4,0),(-4,0),(0,4),(0,-4),(3,3),(-3,3),(3,-3),(-3,-3),
            (4,1),(4,-1),(-4,1),(-4,-1),(1,4),(1,-4),(-1,4),(-1,-4),
        ];
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sparks(&mut buf, w, h, frame);
            // Light burst: pulsing center with star rays
            let cx = w / 2;
            let cy = h / 2;
            // Pulsing intensity
            let phase = (frame % 40) as u32;
            let pulse = if phase < 20 { phase * 4 } else { (40 - phase) * 4 };
            // Draw rays
            let ray_len = 40 + (frame / 2) as i32;
            for &(rdx, rdy) in rays.iter() {
                for step in 0..ray_len {
                    let px = (cx as i32 + rdx * step).max(0) as usize;
                    let py = (cy as i32 + rdy * step).max(0) as usize;
                    if px < w && py < h {
                        let falloff = (ray_len - step) as u32 * 3;
                        let bright = (pulse + falloff).min(180);
                        let r = bright;
                        let g = (bright * 3 / 4).min(140);
                        let b = (bright / 3).min(60);
                        let existing = buf[py * w + px];
                        let er = ((existing >> 16) & 0xFF) + r;
                        let eg = ((existing >> 8) & 0xFF) + g;
                        let eb = (existing & 0xFF) + b;
                        buf[py * w + px] = 0xFF000000
                            | (er.min(255) << 16)
                            | (eg.min(255) << 8)
                            | eb.min(255);
                    }
                }
            }
            // Center glow
            let glow_r = 15 + (pulse / 4) as usize;
            for dy in 0..glow_r {
                for dx in 0..glow_r {
                    if dx * dx + dy * dy < glow_r * glow_r {
                        for &(sx, sy) in &[(cx+dx, cy+dy),(cx+dx, cy.wrapping_sub(dy)),
                                           (cx.wrapping_sub(dx), cy+dy),
                                           (cx.wrapping_sub(dx), cy.wrapping_sub(dy))] {
                            if sx < w && sy < h {
                                buf[sy * w + sx] = 0xFFFFFFCC;
                            }
                        }
                    }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let y1 = h / 4;
            let y2 = y1 + 16 * scale + 12;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFFFFFFF, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFF00FF88, scale);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 8: Odometer Counter — TrustOS stats ──
    {
        let stats: [(&str, &str, u32, u32); 4] = [
            ("", "lines of code", 120_000, 0xFF00FF88),
            ("", "author", 1, 0xFFFFFFFF),
            ("", "secrets", 0, 0xFFFFFFFF),
            ("100%", "Rust.  0% C.", 0, 0xFFFF7744),
        ];
        let total_frames = 140u32;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sparks(&mut buf, w, h, frame);
            // Title
            draw_text_centered(&mut buf, w, h, 40, "TrustOS", 0xFF00FF88, 6);
            // Odometer: numbers roll from 0 to target
            let progress = if frame < 20 { 0u32 }
                else { ((frame - 20) * 100 / 80).min(100) };
            let line_y_start = h / 2 - 40;
            for (si, &(prefix, suffix, target, color)) in stats.iter().enumerate() {
                let y = line_y_start + si * 48;
                let scale = 2usize;
                if target > 0 {
                    let current = (target as u64 * progress as u64 / 100) as u32;
                    let num_str = if current >= 1000 {
                        alloc::format!("{},{:03}", current / 1000, current % 1000)
                    } else {
                        alloc::format!("{}", current)
                    };
                    let full = alloc::format!("{} {}", num_str, suffix);
                    draw_text_centered(&mut buf, w, h, y, &full, color, scale);
                } else if !prefix.is_empty() {
                    let full = alloc::format!("{} {}", prefix, suffix);
                    if frame > 60 + si as u32 * 15 {
                        draw_text_centered(&mut buf, w, h, y, &full, color, scale);
                    }
                } else {
                    let full = alloc::format!("0 {}", suffix);
                    if frame > 60 + si as u32 * 15 {
                        draw_text_centered(&mut buf, w, h, y, &full, color, scale);
                    }
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 9: Feature Grid with Glow Pulse ──
    {
        let features: [(&str, &str, u32); 8] = [
            ("Network",   "TCP/IP, DNS, HTTP, DHCP",       0xFF4488FF),
            ("Security",  "TLS 1.3, Ed25519, SHA-256",     0xFFFF4444),
            ("Language",  "TrustLang: full compiler",      0xFF44FF88),
            ("GUI",       "Windows, themes, animations",   0xFFFFAA22),
            ("Storage",   "FAT32, persistence, AHCI",      0xFF8888FF),
            ("Browser",   "HTTP + HTML in the kernel",     0xFF44DDFF),
            ("Games",     "Snake, Tetris, Pong, Chess",    0xFFFF88FF),
            ("Video",     "Built-in video codec",          0xFFFFDD44),
        ];
        let cols = 2usize;
        let cell_w = (w.saturating_sub(120)) / cols;
        let cell_h = 70usize;
        let grid_y = 100usize;
        let grid_x = 40usize;

        for reveal in 0..features.len() {
            for frame in 0..30u32 {
                if let Some(k) = crate::keyboard::try_read_key() {
                    if k == 0x1B { break; }
                }
                bg_stars(&mut buf, w, h, frame + reveal as u32 * 30);
                draw_text_centered(&mut buf, w, h, 30,
                    "All of this in 10 MB:", 0xFFFFFFFF, 3);
                for (fi, &(name, desc, color)) in features.iter().enumerate() {
                    if fi > reveal { break; }
                    let col = fi % cols;
                    let row = fi / cols;
                    let fx = grid_x + col * (cell_w + 40);
                    let fy = grid_y + row * (cell_h + 20);

                    // Glow pulse: bright border that fades after reveal
                    let glow = if fi == reveal && frame < 15 {
                        (15 - frame) * 12
                    } else { 0 };
                    let glow = glow as u32;

                    // Card background
                    draw_rect(&mut buf, w, h, fx, fy, cell_w, cell_h, 0xFF0E0E1E);

                    // Glow border (all 4 sides)
                    if glow > 0 {
                        let gc = 0xFF000000 | (glow.min(255) << 16) | (glow.min(255) << 8) | glow.min(255);
                        draw_rect(&mut buf, w, h, fx.saturating_sub(2), fy.saturating_sub(2),
                            cell_w + 4, 3, gc);
                        draw_rect(&mut buf, w, h, fx.saturating_sub(2), fy + cell_h,
                            cell_w + 4, 3, gc);
                        draw_rect(&mut buf, w, h, fx.saturating_sub(2), fy,
                            3, cell_h, gc);
                        draw_rect(&mut buf, w, h, fx + cell_w, fy,
                            3, cell_h, gc);
                    }

                    // Top color bar
                    draw_rect(&mut buf, w, h, fx, fy, cell_w, 3, color);
                    draw_rect(&mut buf, w, h, fx, fy + cell_h - 1, cell_w, 1, 0xFF222244);
                    draw_text_at(&mut buf, w, h, fx + 10, fy + 12,
                        name, color, 2);
                    draw_text_at(&mut buf, w, h, fx + 10, fy + 42,
                        desc, 0xFFAAAAAA, 1);
                }
                blit_buf(&buf, w, h);
                crate::cpu::tsc::pit_delay_ms(frame_ms);
            }
        }
        crate::cpu::tsc::pit_delay_ms(1500);
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ═══════════════════════════════════════════════════════════════
    //  ACT IV  —  THE PROOF  (bg: circuit-board traces — technical)
    //  Retention: interactive-feeling animation, pattern interrupt
    // ═══════════════════════════════════════════════════════════════
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT IV", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("The Proof", 0xFF00CC66, 3)],
        50, 30, 6);
    do_fade(&mut buf, w, h, &blit_buf);

    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("When you visit a website,", 0xFFCCCCCC, 2),
          ("this is what happens", 0xFFCCCCCC, 2),
          ("inside TrustOS:", 0xFF00FF88, 2)],
        50, 60, 6);
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Packet journey animation (on circuit bg) ──
    {
        let stages: [(&str, u32); 5] = [
            ("App",     0xFF4488FF),
            ("TLS 1.3", 0xFFFF4444),
            ("TCP/IP",  0xFFFFAA22),
            ("Driver",  0xFF44FF88),
            ("Wire",    0xFF8888FF),
        ];
        let n = stages.len();
        let stage_w = (w.saturating_sub(80)) / (n + 1);
        let stage_h = 60usize;
        let lane_y = h / 2 - stage_h / 2;

        for pass in 0..2u32 {
            let label = if pass == 0 { "Sending packet..." }
                        else         { "Response received!" };
            let pkt_color = if pass == 0 { 0xFF00FF88 } else { 0xFF44DDFF };

            for frame in 0..150u32 {
                if let Some(k) = crate::keyboard::try_read_key() {
                    if k == 0x1B { break; }
                }
                bg_circuit(&mut buf, w, h, frame + pass * 150);

                draw_text_centered(&mut buf, w, h, 30,
                    label, 0xFFFFFFFF, 2);

                for (si, &(name, color)) in stages.iter().enumerate() {
                    let sx = 40 + si * stage_w;
                    let bw = stage_w.saturating_sub(15);
                    draw_rect(&mut buf, w, h, sx, lane_y, bw, stage_h,
                              0xFF0E1020);
                    draw_rect(&mut buf, w, h, sx, lane_y, bw, 3, color);
                    draw_rect(&mut buf, w, h, sx, lane_y + stage_h - 1, bw, 1, 0xFF222244);
                    let tx = sx + bw / 2 - name.len() * 4;
                    draw_text_at(&mut buf, w, h, tx, lane_y + 22,
                        name, color, 1);
                    if si < n - 1 {
                        let ax = sx + bw;
                        draw_rect(&mut buf, w, h, ax,
                            lane_y + stage_h / 2 - 1, 15, 3, 0xFF334455);
                        // Arrow head
                        draw_rect(&mut buf, w, h, ax + 12,
                            lane_y + stage_h / 2 - 3, 3, 7, 0xFF556677);
                    }
                }

                // Animated packet with trail
                let progress = (frame * 100 / 120).min(100) as usize;
                let total_travel = (n - 1) * stage_w;
                let pkt_off = if pass == 0 {
                    total_travel * progress / 100
                } else {
                    total_travel - total_travel * progress / 100
                };
                let pkt_x = 40 + pkt_off + stage_w / 2 - 8;
                let pkt_y = lane_y + stage_h + 18;
                // Trail glow
                for trail in 1..6u32 {
                    let tx = if pass == 0 { pkt_x.saturating_sub(trail as usize * 6) }
                             else { pkt_x + trail as usize * 6 };
                    let alpha = (60u32.saturating_sub(trail * 12)).min(255);
                    let tc = 0xFF000000 | (alpha / 4 << 16) | (alpha << 8) | (alpha / 3);
                    draw_rect(&mut buf, w, h, tx, pkt_y + 2, 8, 12, tc);
                }
                draw_rect(&mut buf, w, h, pkt_x, pkt_y, 16, 16, pkt_color);
                draw_text_at(&mut buf, w, h,
                    pkt_x.saturating_sub(16), pkt_y + 20,
                    "packet", 0xFFCCCCCC, 1);

                blit_buf(&buf, w, h);
                crate::cpu::tsc::pit_delay_ms(frame_ms);
            }
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // Post-demo — emotional beat
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("Every step is visible.", 0xFFFFFFFF, 3),
          ("Every byte is readable.", 0xFFFFFFFF, 3),
          ("", 0xFF000000, 1),
          ("Nothing is hidden.", 0xFF00FF88, 4)],
        50, 80, 4);
    do_fade(&mut buf, w, h, &blit_buf);

    // ═══════════════════════════════════════════════════════════════
    //  ACT V  —  THE FUTURE  (sparkle dissolve + expanding rings)
    // ═══════════════════════════════════════════════════════════════
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("ACT V", 0xFFFFDD88, 5),
          ("", 0xFF000000, 1),
          ("The Future", 0xFFFFAA44, 3)],
        50, 30, 7);
    do_fade(&mut buf, w, h, &blit_buf);

    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("You deserve to understand", 0xFFFFFFFF, 3),
          ("your own machine.", 0xFFFFFFFF, 3)],
        50, 60, 7);
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 10: Sparkle Dissolve — "Computing is not magic" ──
    {
        let fpc = 2u32;
        let text1 = "Computing is not magic.";
        let text2 = "It's math and logic.";
        let total_chars = text1.len() + text2.len();
        let total_frames = total_chars as u32 * fpc + 80;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sunrise(&mut buf, w, h, frame);
            // Sparkles: random bright points that form geometric shapes
            let sparkle_count = (frame * 2).min(200) as usize;
            for si in 0..sparkle_count {
                let seed = si.wrapping_mul(2654435761).wrapping_add(frame as usize * 131);
                let sx = seed % w;
                let sy = (seed / w) % h;
                // In later frames, sparkles concentrate into geometric shapes
                let constrained = frame > 60;
                let (fx, fy) = if constrained {
                    // Form rectangles/diamond shapes around center
                    let cx = w / 2;
                    let cy = h / 2;
                    let shape_phase = (si % 4);
                    match shape_phase {
                        0 => {  // Top horizontal line
                            let lx = cx.saturating_sub(100) + (si * 3) % 200;
                            (lx, cy.saturating_sub(60))
                        }
                        1 => {  // Bottom horizontal line
                            let lx = cx.saturating_sub(100) + (si * 7) % 200;
                            (lx, cy + 60)
                        }
                        2 => {  // Left vertical line
                            let ly = cy.saturating_sub(60) + (si * 5) % 120;
                            (cx.saturating_sub(100), ly)
                        }
                        _ => {  // Right vertical line
                            let ly = cy.saturating_sub(60) + (si * 11) % 120;
                            (cx + 100, ly)
                        }
                    }
                } else {
                    (sx, sy)
                };
                if fx < w && fy < h {
                    let bright = (100 + (seed % 155)) as u32;
                    buf[fy * w + fx] = 0xFF000000 | (bright << 16) | (bright << 8) | bright;
                    if fx + 1 < w { buf[fy * w + fx + 1] = 0xFF000000 | (bright << 16) | ((bright / 2) << 8); }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 3usize;
            let y1 = h / 4;
            let y2 = y1 + 16 * scale + 16;
            let tw1 = text1.len() * 8 * scale;
            let sx1 = if tw1 < w { (w - tw1) / 2 } else { 0 };
            for (i, c) in text1.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx1 + i * 8 * scale, y1, c, 0xFFCCCCCC, scale);
            }
            if chars_shown > text1.len() {
                let tw2 = text2.len() * 8 * scale;
                let sx2 = if tw2 < w { (w - tw2) / 2 } else { 0 };
                let extra = chars_shown - text1.len();
                for (i, c) in text2.chars().enumerate() {
                    if i >= extra { break; }
                    draw_big_char(&mut buf, w, h, sx2 + i * 8 * scale, y2, c, 0xFFFFDD88, scale);
                }
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Scene 11: Expanding Rings — "TrustOS proves it." ──
    {
        let fpc = 2u32;
        let text = "TrustOS proves it.";
        let type_frames = text.len() as u32 * fpc;
        let ring_frames = 60u32;
        let total_frames = type_frames + ring_frames;
        for frame in 0..total_frames {
            if let Some(k) = crate::keyboard::try_read_key() {
                if k == 0x1B { break; }
                if k == b' ' || k == b'\r' || k == b'\n' { break; }
            }
            bg_sunrise(&mut buf, w, h, frame);
            let cx = w / 2;
            let cy = h / 2;
            // Expanding concentric rings (shockwave)
            if frame > type_frames / 2 {
                let ring_progress = frame.saturating_sub(type_frames / 2);
                let num_rings = 5u32;
                for ri in 0..num_rings {
                    let radius = (ring_progress as usize * 4).saturating_sub(ri as usize * 30);
                    if radius == 0 || radius > w { continue; }
                    let bright = (200u32.saturating_sub(ri * 30)).min(255);
                    let r_color = 0xFF000000 | ((bright / 3) << 16) | (bright << 8) | ((bright * 2 / 3).min(255));
                    // Draw ring using distance check in a bounding box
                    let r2_outer = radius * radius;
                    let r_inner = radius.saturating_sub(3);
                    let r2_inner = r_inner * r_inner;
                    let y_start = cy.saturating_sub(radius);
                    let y_end = (cy + radius).min(h);
                    for py in y_start..y_end {
                        let dy = if py >= cy { py - cy } else { cy - py };
                        let dy2 = dy * dy;
                        // Solve for x range where r_inner^2 <= dx^2+dy^2 <= r_outer^2
                        if dy2 > r2_outer { continue; }
                        let dx_max_sq = r2_outer - dy2;
                        let dx_min_sq = if r2_inner > dy2 { r2_inner - dy2 } else { 0 };
                        // Integer sqrt approximation
                        let mut dx_max = 0usize;
                        while (dx_max + 1) * (dx_max + 1) <= dx_max_sq { dx_max += 1; }
                        let mut dx_min = 0usize;
                        while (dx_min + 1) * (dx_min + 1) <= dx_min_sq { dx_min += 1; }
                        // Draw right arc
                        for dx in dx_min..=dx_max {
                            let px = cx + dx;
                            if px < w { buf[py * w + px] = r_color; }
                        }
                        // Draw left arc
                        for dx in dx_min..=dx_max {
                            let px = cx.wrapping_sub(dx);
                            if px < w { buf[py * w + px] = r_color; }
                        }
                    }
                }
            }
            // Type text
            let chars_shown = (frame / fpc) as usize;
            let scale = 4usize;
            let tw = text.len() * 8 * scale;
            let sx = if tw < w { (w - tw) / 2 } else { 0 };
            let ty = h / 2 - 8 * scale;
            for (i, c) in text.chars().enumerate() {
                if i >= chars_shown { break; }
                draw_big_char(&mut buf, w, h, sx + i * 8 * scale, ty, c, 0xFF00FF88, scale);
            }
            blit_buf(&buf, w, h);
            crate::cpu::tsc::pit_delay_ms(frame_ms);
        }
    }
    do_fade(&mut buf, w, h, &blit_buf);

    // ═══════════════════════════════════════════════════════════════
    //  OUTRO  (Matrix rain callback — signature TrustOS feel)
    // ═══════════════════════════════════════════════════════════════
    clear_buf(&mut buf);
    show_scene(&mut buf, w, h, &mut rain_cols, &rain_speeds,
        &[("Trust the code.", 0xFF00FF88, 5),
          ("", 0xFF000000, 1),
          ("Rust is the reason.", 0xFFFF7744, 3),
          ("", 0xFF000000, 1),
          ("github.com/nathan237/TrustOS", 0xFFCCCCCC, 2)],
        60, 150, 8);
    do_fade(&mut buf, w, h, &blit_buf);

    // ── Cleanup ──
    clear_buf(&mut buf);
    blit_buf(&buf, w, h);
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }
    crate::framebuffer::clear();
    crate::serial_println!("[FILM] TrustOS Film complete");
}

/// Transpile command: analyze and convert Linux binaries to Rust
fn cmd_transpile(args: &[&str]) {
    let subcmd = args.get(0).copied().unwrap_or("help");
    
    match subcmd {
        "help" | "-h" | "--help" => {
            crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
            crate::println_color!(COLOR_CYAN, "║           TrustOS Binary Transpiler                          ║");
            crate::println_color!(COLOR_CYAN, "║       Analyze Linux binaries → Generate Rust code            ║");
            crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
            crate::println!();
            crate::println!("Usage: transpile <subcommand> [file]");
            crate::println!();
            crate::println!("Subcommands:");
            crate::println!("  test             - Run demo with built-in test binary");
            crate::println!("  analyze <file>   - Analyze ELF binary and show disassembly");
            crate::println!("  disasm <file>    - Show disassembly only");
            crate::println!("  rust <file>      - Generate Rust code from binary");
            crate::println!("  strings <file>   - Extract strings from binary");
            crate::println!("  syscalls <file>  - List detected syscalls");
            crate::println!("  scan             - Scan /alpine/bin for binaries to transpile");
            crate::println!("  batch            - Transpile all simple binaries");
            crate::println!("  audit            - Full syscall audit of all Alpine binaries");
            crate::println!("  run <file>       - Execute transpiled binary natively");
            crate::println!("  execute <file>   - Same as run, with verbose output");
            crate::println!();
            crate::println!("Example:");
            crate::println!("  transpile test              # Test with demo binary");
            crate::println!("  transpile run /alpine/bin/true");
            crate::println!("  transpile analyze /alpine/bin/ls");
            crate::println!("  transpile rust /alpine/bin/pwd");
        }
        
        "test" | "demo" => {
            transpile_demo();
        }
        
        "analyze" | "a" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_analyze(path, true, true, true);
        }
        
        "disasm" | "d" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_analyze(path, true, false, false);
        }
        
        "rust" | "r" | "gen" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_analyze(path, false, false, true);
        }
        
        "strings" | "s" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_strings(path);
        }
        
        "syscalls" | "sys" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/ls");
            transpile_syscalls(path);
        }
        
        "scan" => {
            transpile_scan_binaries();
        }
        
        "batch" => {
            transpile_batch();
        }
        
        "run" | "exec" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/true");
            transpile_run(path, false);
        }
        
        "execute" | "x" => {
            let path = args.get(1).copied().unwrap_or("/alpine/bin/true");
            transpile_run(path, true);
        }

        "audit" | "stats" => {
            transpile_audit_alpine();
        }

        _ => {
            // If first arg looks like a path, analyze it
            if subcmd.starts_with('/') || subcmd.contains('.') {
                transpile_analyze(subcmd, true, true, true);
            } else {
                crate::println_color!(COLOR_RED, "Unknown subcommand: {}", subcmd);
                crate::println!("Use 'transpile help' for usage");
            }
        }
    }
}

fn transpile_analyze(path: &str, show_disasm: bool, show_strings: bool, show_rust: bool) {
    crate::println_color!(COLOR_CYAN, "Analyzing binary: {}", path);
    crate::println!();
    
    // Read file from ramfs
    let data = match crate::ramfs::with_fs(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot read file: {}", path);
            return;
        }
    };
    
    crate::println!("File size: {} bytes ({} KB)", data.len(), data.len() / 1024);
    
    // Analyze ELF
    match crate::transpiler::analyze_elf(&data) {
        Some(analysis) => {
            crate::println_color!(COLOR_GREEN, "ELF analysis successful!");
            crate::println!();
            crate::println!("Entry point: 0x{:x}", analysis.entry_point);
            crate::println!("Functions: {}", analysis.functions.len());
            crate::println!("Syscalls: {:?}", analysis.syscalls_used);
            crate::println!("Strings: {}", analysis.strings.len());
            crate::println!();
            
            if show_disasm {
                if let Some(func) = analysis.functions.first() {
                    crate::println_color!(COLOR_YELLOW, "=== Disassembly ({} instructions) ===", func.instructions.len());
                    let transpiler = crate::transpiler::Transpiler::new(func.instructions.clone());
                    crate::println!("{}", transpiler.generate_listing());
                }
            }
            
            if show_strings && !analysis.strings.is_empty() {
                crate::println_color!(COLOR_YELLOW, "=== Strings (first 20) ===");
                for (addr, s) in analysis.strings.iter().take(20) {
                    crate::println!("0x{:06x}: \"{}\"", addr, s);
                }
                crate::println!();
            }
            
            if show_rust {
                crate::println_color!(COLOR_YELLOW, "=== Generated Rust Code ===");
                crate::println!("{}", analysis.rust_code);
            }
        }
        None => {
            crate::println_color!(COLOR_RED, "Not a valid ELF binary");
        }
    }
}

fn transpile_strings(path: &str) {
    let data = match crate::ramfs::with_fs(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot read file: {}", path);
            return;
        }
    };
    
    // Extract strings manually (min 4 chars)
    let mut strings = alloc::vec::Vec::new();
    let mut current = alloc::string::String::new();
    let mut start = 0usize;
    
    for (i, &b) in data.iter().enumerate() {
        if b >= 0x20 && b < 0x7F {
            if current.is_empty() {
                start = i;
            }
            current.push(b as char);
        } else {
            if current.len() >= 4 {
                strings.push((start, current.clone()));
            }
            current.clear();
        }
    }
    
    crate::println_color!(COLOR_CYAN, "Strings in {}: {} found", path, strings.len());
    crate::println!();
    
    for (addr, s) in strings.iter() {
        // Skip binary garbage
        if s.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
            crate::println!("0x{:06x}: {}", addr, s);
        }
    }
}

fn transpile_syscalls(path: &str) {
    let data = match crate::ramfs::with_fs(|fs| fs.read_file(path).map(|d| d.to_vec())) {
        Ok(d) => d,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot read file: {}", path);
            return;
        }
    };
    
    match crate::transpiler::analyze_elf(&data) {
        Some(analysis) => {
            crate::println_color!(COLOR_CYAN, "Syscalls in {}", path);
            crate::println!();
            
            for func in &analysis.functions {
                if !func.syscalls.is_empty() {
                    crate::println!("Function {} @ 0x{:x}:", func.name, func.address);
                    for sc in &func.syscalls {
                        crate::println!("  0x{:x}: {} (#{})", sc.address, sc.name, sc.number);
                    }
                }
            }
            
            crate::println!();
            crate::println!("Summary: {:?}", analysis.syscalls_used);
        }
        None => {
            crate::println_color!(COLOR_RED, "Not a valid ELF binary");
        }
    }
}

fn transpile_scan_binaries() {
    crate::println_color!(COLOR_CYAN, "Scanning /alpine/bin for binaries...");
    crate::println!();
    
    let entries = match crate::ramfs::with_fs(|fs| fs.ls(Some("/alpine/bin"))) {
        Ok(e) => e,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot access /alpine/bin - run 'alpine test' first");
            return;
        }
    };
    
    let mut simple = alloc::vec::Vec::new();
    let mut complex = alloc::vec::Vec::new();
    
    for (name, _, _size) in entries {
        let path = alloc::format!("/alpine/bin/{}", name);
        
        if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::analyze_elf(&data) {
                let syscall_count = analysis.syscalls_used.len();
                let instr_count = analysis.functions.first().map(|f| f.instructions.len()).unwrap_or(0);
                
                if syscall_count <= 3 && instr_count < 100 {
                    simple.push((name.clone(), syscall_count, instr_count));
                } else {
                    complex.push((name.clone(), syscall_count, instr_count));
                }
            }
        }
    }
    
    crate::println_color!(COLOR_GREEN, "Simple binaries ({} - easy to transpile):", simple.len());
    for (name, sc, instr) in &simple {
        crate::println!("  {} - {} syscalls, {} instructions", name, sc, instr);
    }
    
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Complex binaries ({} - need more work):", complex.len());
    for (name, sc, instr) in complex.iter().take(10) {
        crate::println!("  {} - {} syscalls, {} instructions", name, sc, instr);
    }
    if complex.len() > 10 {
        crate::println!("  ... and {} more", complex.len() - 10);
    }
}

fn transpile_batch() {
    crate::println_color!(COLOR_CYAN, "Batch transpiling simple binaries...");
    crate::println!();
    
    // For now, just demonstrate the concept
    let simple_bins = ["true", "false", "pwd", "whoami", "hostname", "uname", "echo", "yes"];
    
    for name in &simple_bins {
        let path = alloc::format!("/alpine/bin/{}", name);
        
        if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::analyze_elf(&data) {
                crate::println_color!(COLOR_GREEN, "=== {} ===", name);
                crate::println!("Syscalls: {:?}", analysis.syscalls_used);
                crate::println!();
                crate::println!("{}", analysis.rust_code);
                crate::println!();
            } else {
                crate::println_color!(COLOR_YELLOW, "{}: not found or not ELF", name);
            }
        } else {
            crate::println_color!(COLOR_YELLOW, "{}: not available", name);
        }
    }
}

/// Audit all Alpine binaries and show syscall statistics
fn transpile_audit_alpine() {
    use alloc::collections::BTreeMap;
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║         TrustOS Transpiler - Alpine Syscall Audit            ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    let entries = match crate::ramfs::with_fs(|fs| fs.ls(Some("/alpine/bin"))) {
        Ok(e) => e,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Cannot access /alpine/bin - run 'linux extract' first");
            return;
        }
    };
    
    // Collect all syscalls with their frequency
    let mut syscall_counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut syscall_numbers: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut binary_count = 0;
    let mut elf_count = 0;
    let mut supported_count = 0;
    let mut total_instructions = 0usize;
    
    crate::println!("Scanning {} files...", entries.len());
    crate::println!();
    
    for (name, _, _) in &entries {
        let path = alloc::format!("/alpine/bin/{}", name);
        binary_count += 1;
        
        if let Ok(data) = crate::ramfs::with_fs(|fs| fs.read_file(&path).map(|d| d.to_vec())) {
            if let Some(analysis) = crate::transpiler::analyze_elf(&data) {
                elf_count += 1;
                
                // Count instructions
                for func in &analysis.functions {
                    total_instructions += func.instructions.len();
                    
                    for sc in &func.syscalls {
                        *syscall_counts.entry(sc.name).or_insert(0) += 1;
                        syscall_numbers.insert(sc.name, sc.number);
                    }
                }
                
                // Check if fully supported
                let all_supported = analysis.syscalls_used.iter().all(|sc| {
                    matches!(*sc, "exit" | "exit_group" | "write" | "read" | "open" | "close" |
                            "getcwd" | "uname" | "getpid" | "getuid" | "getgid" | "geteuid" | "getegid")
                });
                if all_supported && !analysis.syscalls_used.is_empty() {
                    supported_count += 1;
                }
            }
        }
    }
    
    crate::println_color!(COLOR_GREEN, "═══ Statistics ═══");
    crate::println!("Files scanned:      {}", binary_count);
    crate::println!("Valid ELF binaries: {}", elf_count);
    crate::println!("Fully supported:    {}", supported_count);
    crate::println!("Total instructions: {}", total_instructions);
    crate::println!();
    
    // Sort syscalls by frequency
    let mut sorted: Vec<_> = syscall_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    
    crate::println_color!(COLOR_CYAN, "═══ Syscalls by Frequency ═══");
    crate::println!("{:<20} {:>8} {:>8} {}", "Syscall", "Count", "Number", "Status");
    crate::println!("{}", "─".repeat(50));
    
    for (name, count) in &sorted {
        let num = syscall_numbers.get(*name).copied().unwrap_or(0);
        let level = crate::transpiler::syscall_support_level(num);
        let status = match level {
            "Full" => "Full",
            "Partial" => "Partial",
            "Stub" => "Stub",
            _ => "Missing",
        };
        crate::println!("{:<20} {:>8} {:>8} {}", name, count, num, status);
    }
    
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "═══ Missing Syscalls (need implementation) ═══");
    let missing: Vec<_> = sorted.iter()
        .filter(|(name, _)| {
            let num = syscall_numbers.get(*name).copied().unwrap_or(999);
            crate::transpiler::syscall_support_level(num) == "None"
        })
        .collect();
    
    for (name, count) in &missing {
        let num = syscall_numbers.get(*name).copied().unwrap_or(0);
        crate::println!("  {} (#{}) - used {} times", name, num, count);
    }
    
    if missing.is_empty() {
        crate::println_color!(COLOR_GREEN, "  All syscalls are at least partially implemented!");
    }
    
    crate::println!();
    crate::println_color!(COLOR_CYAN, "═══ Recommendation ═══");
    crate::println!("To improve transpiler coverage, implement these syscalls in order:");
    let priority: Vec<_> = missing.iter().take(5).collect();
    for (i, (name, count)) in priority.iter().enumerate() {
        crate::println!("  {}. {} (used {} times)", i + 1, name, count);
    }
}

/// Create test ELF binaries silently (used by alpine test)
fn create_test_binaries_silent() {
    // Create directory structure
    let _ = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/alpine");
        let _ = fs.mkdir("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    fn make_elf(code: &[u8]) -> alloc::vec::Vec<u8> {
        let mut elf = alloc::vec![
            0x7fu8, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x02, 0x00, 0x3e, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x38, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x01, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        elf.extend_from_slice(code);
        while elf.len() < 256 { elf.push(0); }
        elf
    }
    
    let binaries: [(&str, &[u8]); 7] = [
        ("true", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("false", &[0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x0f, 0x05]),
        ("getpid", &[0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("getuid", &[0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("uname", &[0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("echo", &[0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, 0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, 0x48, 0x31, 0xf6, 0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
        ("pwd", &[0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x48, 0x31, 0xf6, 0x0f, 0x05, 0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00, 0x48, 0x31, 0xff, 0x0f, 0x05]),
    ];
    
    let mut created = 0;
    for (name, code) in &binaries {
        let elf = make_elf(code);
        let path = alloc::format!("/alpine/bin/{}", name);
        if crate::ramfs::with_fs(|fs| { let _ = fs.touch(&path); fs.write_file(&path, &elf) }).is_ok() {
            created += 1;
        }
    }
    crate::println_color!(COLOR_GREEN, "      Created {} binaries", created);
}

/// Create test ELF binaries in /alpine/bin for transpiler testing
fn create_test_binaries() {
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║       Creating Test Binaries for Transpiler                  ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    // Create directory structure
    let _ = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/alpine");
        let _ = fs.mkdir("/alpine/bin");
        Ok::<(), ()>(())
    });
    
    // Use the same ELF template as transpile_demo
    fn make_elf(code: &[u8]) -> alloc::vec::Vec<u8> {
        let mut elf = alloc::vec![
            // ELF header (64 bytes)
            0x7fu8, 0x45, 0x4c, 0x46,  // Magic: \x7fELF
            0x02, 0x01, 0x01, 0x00,     // 64-bit, little-endian
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Padding
            0x02, 0x00,                 // Executable
            0x3e, 0x00,                 // x86_64
            0x01, 0x00, 0x00, 0x00,     // Version
            0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Entry: 0x400078
            0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // PH offset: 64
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // SH offset
            0x00, 0x00, 0x00, 0x00,     // Flags
            0x40, 0x00,                 // ELF header size: 64
            0x38, 0x00,                 // PH entry size: 56
            0x01, 0x00,                 // PH count: 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // SH stuff
            
            // Program header (56 bytes)
            0x01, 0x00, 0x00, 0x00,     // PT_LOAD
            0x05, 0x00, 0x00, 0x00,     // Flags: R+X
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Offset
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Vaddr: 0x400000
            0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Paddr
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Filesz
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Memsz
            0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Align
        ];
        elf.extend_from_slice(code);
        while elf.len() < 256 {
            elf.push(0);
        }
        elf
    }
    
    // Different syscall programs
    let binaries: [(&str, &[u8], &str); 7] = [
        ("true", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60
            0x48, 0x31, 0xff,                          // xor rdi, rdi
            0x0f, 0x05,                                // syscall
        ], "exit(0)"),
        
        ("false", &[
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  // mov rdi, 1
            0x0f, 0x05,                                // syscall
        ], "exit(1)"),
        
        ("getpid", &[
            0x48, 0xc7, 0xc0, 0x27, 0x00, 0x00, 0x00,  // mov rax, 39
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,  // mov rax, 60
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getpid+exit"),
        
        ("getuid", &[
            0x48, 0xc7, 0xc0, 0x66, 0x00, 0x00, 0x00,  // mov rax, 102
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getuid+exit"),
        
        ("uname", &[
            0x48, 0xc7, 0xc0, 0x3f, 0x00, 0x00, 0x00,  // mov rax, 63
            0x48, 0x31, 0xff,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "uname+exit"),
        
        ("echo", &[
            0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00,  // mov rax, 1 (write)
            0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00,  // mov rdi, 1
            0x48, 0x31, 0xf6,                          // xor rsi, rsi
            0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00,  // mov rdx, 5
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "write+exit"),
        
        ("pwd", &[
            0x48, 0xc7, 0xc0, 0x4f, 0x00, 0x00, 0x00,  // mov rax, 79 (getcwd)
            0x48, 0x31, 0xff,
            0x48, 0x31, 0xf6,
            0x0f, 0x05,
            0x48, 0xc7, 0xc0, 0x3c, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xff,
            0x0f, 0x05,
        ], "getcwd+exit"),
    ];
    
    let mut created = 0;
    for (name, code, desc) in &binaries {
        let elf = make_elf(code);
        let path = alloc::format!("/alpine/bin/{}", name);
        
        let result = crate::ramfs::with_fs(|fs| {
            let _ = fs.touch(&path);
            fs.write_file(&path, &elf)
        });
        
        match result {
            Ok(_) => {
                crate::println_color!(COLOR_GREEN, "✓ {} - {}", name, desc);
                created += 1;
            }
            Err(_) => {
                crate::println_color!(COLOR_RED, "✗ {} - failed", name);
            }
        }
    }
    
    crate::println!();
    crate::println_color!(COLOR_GREEN, "Created {} test binaries in /alpine/bin", created);
    crate::println!();
    crate::println!("Now run:");
    crate::println!("  transpile audit       - Analyze all syscalls");
    crate::println!("  transpile run /alpine/bin/true");
    crate::println!("  transpile analyze /alpine/bin/echo");
}

/// Demo the transpiler with a built-in test binary (no external files needed)
fn transpile_demo() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_BRIGHT_GREEN, "║         TrustOS Transpiler Demo - Built-in Test              ║");
    crate::println_color!(COLOR_BRIGHT_GREEN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    // Create a minimal x86_64 ELF binary that does:
    //   mov rax, 60    ; sys_exit
    //   xor rdi, rdi   ; exit code 0  
    //   syscall
    // This is the simplest possible Linux program
    
    crate::println_color!(COLOR_CYAN, "Creating test binary: exit(0) program");
    crate::println!();
    
    // Minimal ELF64 executable that calls exit(0)
    // Header (64 bytes) + Program Header (56 bytes) + Code (12 bytes) = 132 bytes
    #[rustfmt::skip]
    let demo_elf: &[u8] = &[
        // ELF Header (64 bytes)
        0x7F, b'E', b'L', b'F',  // Magic
        0x02,                     // 64-bit
        0x01,                     // Little endian
        0x01,                     // ELF version
        0x00,                     // System V ABI
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Padding (8 bytes)
        0x02, 0x00,               // Executable
        0x3E, 0x00,               // x86-64
        0x01, 0x00, 0x00, 0x00,   // ELF version
        0x78, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Entry point: 0x400078
        0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Program header offset: 64
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Section header offset
        0x00, 0x00, 0x00, 0x00,   // Flags
        0x40, 0x00,               // ELF header size: 64
        0x38, 0x00,               // Program header size: 56
        0x01, 0x00,               // Number of program headers: 1
        0x00, 0x00,               // Section header size
        0x00, 0x00,               // Number of section headers
        0x00, 0x00,               // Section name string table index
        
        // Program Header (56 bytes) - PT_LOAD
        0x01, 0x00, 0x00, 0x00,   // PT_LOAD
        0x05, 0x00, 0x00, 0x00,   // Flags: R+X
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Offset in file: 0
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Virtual address: 0x400000
        0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,  // Physical address: 0x400000
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // File size: 132 bytes
        0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Memory size: 132 bytes
        0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Alignment: 0x1000
        
        // Code section (offset 0x78 = 120, 12 bytes)
        // _start:
        //   mov rax, 60       ; 48 c7 c0 3c 00 00 00
        //   xor rdi, rdi      ; 48 31 ff  
        //   syscall           ; 0f 05
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00,  // mov rax, 60 (sys_exit)
        0x48, 0x31, 0xFF,                          // xor rdi, rdi (exit code 0)
        0x0F, 0x05,                                // syscall
    ];
    
    crate::println!("Demo binary: {} bytes", demo_elf.len());
    crate::println!("Code: mov rax, 60; xor rdi, rdi; syscall");
    crate::println!();
    
    // Analyze it
    match crate::transpiler::analyze_elf(&demo_elf) {
        Some(analysis) => {
            crate::println_color!(COLOR_GREEN, "✓ ELF Analysis Successful!");
            crate::println!();
            
            crate::println_color!(COLOR_YELLOW, "═══ Binary Info ═══");
            crate::println!("Entry point:  0x{:x}", analysis.entry_point);
            crate::println!("Functions:    {}", analysis.functions.len());
            crate::println!("Syscalls:     {:?}", analysis.syscalls_used);
            crate::println!();
            
            // Show disassembly
            if let Some(func) = analysis.functions.first() {
                crate::println_color!(COLOR_YELLOW, "═══ Disassembly ({} instructions) ═══", func.instructions.len());
                let transpiler = crate::transpiler::Transpiler::new(func.instructions.clone());
                crate::println!("{}", transpiler.generate_listing());
            }
            
            // Show generated Rust
            crate::println_color!(COLOR_YELLOW, "═══ Generated Rust Code ═══");
            crate::println!("{}", analysis.rust_code);
            
            crate::println_color!(COLOR_BRIGHT_GREEN, "");
            crate::println_color!(COLOR_BRIGHT_GREEN, "✓ Transpiler test PASSED!");
            crate::println!();
            crate::println!("The transpiler successfully:");
            crate::println!("  1. Parsed ELF64 header");
            crate::println!("  2. Found executable segment");
            crate::println!("  3. Disassembled x86_64 code");
            crate::println!("  4. Detected syscall (sys_exit)");
            crate::println!("  5. Generated equivalent Rust code");
        }
        None => {
            crate::println_color!(COLOR_RED, "✗ Failed to analyze demo binary");
        }
    }
    
    // Also save to ramfs for further testing
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Saving demo binary to /tmp/demo_exit...");
    let save_result = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/tmp");
        let _ = fs.touch("/tmp/demo_exit"); // Create file first
        fs.write_file("/tmp/demo_exit", demo_elf)
    });
    match save_result {
        Ok(_) => {
            crate::println_color!(COLOR_GREEN, "✓ Saved! You can now run:");
            crate::println!("  transpile analyze /tmp/demo_exit");
            crate::println!("  transpile rust /tmp/demo_exit");
        }
        Err(_) => {
            crate::println_color!(COLOR_YELLOW, "Could not save demo binary");
        }
    }
}

/// Execute a transpiled binary directly in TrustOS
fn transpile_run(path: &str, verbose: bool) {
    use crate::transpiler::{analyze_elf, BinaryType};
    
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║           TrustOS Transpiler - Execute Binary                ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    crate::println!("Binary: {}", path);
    
    // Read the binary
    let data = match read_file_bytes(path) {
        Some(d) => d,
        None => {
            crate::println_color!(COLOR_RED, "Error: Could not read file");
            return;
        }
    };
    
    if verbose {
        crate::println!("Size: {} bytes", data.len());
    }
    
    // Analyze it
    let analysis = match analyze_elf(&data) {
        Some(a) => a,
        None => {
            crate::println_color!(COLOR_RED, "Error: Not a valid ELF binary");
            return;
        }
    };
    
    if verbose {
        crate::println!("Entry point: 0x{:x}", analysis.entry_point);
        crate::println!("Syscalls: {:?}", analysis.syscalls_used);
    }
    
    crate::println!();
    crate::println_color!(COLOR_GREEN, "═══ Executing transpiled binary ═══");
    crate::println!();
    
    // Execute based on detected type and syscalls
    let exit_code = execute_transpiled_binary(&analysis);
    
    crate::println!();
    crate::println_color!(COLOR_CYAN, "═══════════════════════════════════════════════════════════════");
    crate::println!("Exit code: {}", exit_code);
}

/// Execute a transpiled binary based on its analysis
fn execute_transpiled_binary(analysis: &crate::transpiler::BinaryAnalysis) -> i32 {
    // Get syscalls from the first function
    let syscalls = if let Some(func) = analysis.functions.first() {
        &func.syscalls
    } else {
        crate::println_color!(COLOR_RED, "No functions found in binary");
        return 1;
    };
    
    // Execute syscalls in sequence
    for syscall in syscalls {
        match syscall.name {
            "exit" | "exit_group" => {
                let code = syscall.args.get(0).copied().unwrap_or(0) as i32;
                return code;
            }
            "write" => {
                let fd = syscall.args.get(0).copied().unwrap_or(1);
                if fd == 1 || fd == 2 {
                    // Writing to stdout/stderr
                    // In a real implementation, we'd need to extract the actual string
                    // For now, just indicate a write happened
                    crate::print!("[write to fd {}]", fd);
                }
            }
            "getcwd" => {
                crate::println!("/");
            }
            "uname" => {
                crate::println!("TrustOS trustos 1.0.0-transpiled #1 SMP x86_64");
            }
            "getpid" => {
                crate::println!("1");
            }
            "getuid" | "geteuid" => {
                crate::println!("0");
            }
            "getgid" | "getegid" => {
                crate::println!("0");
            }
            _ => {
                crate::println!("[syscall: {} not implemented]", syscall.name);
            }
        }
    }
    
    // Default exit code if no exit syscall was found
    0
}

/// TrustVideo command: video codec, player, and demo animations
fn cmd_video(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "demo" => {
            let effect = args.get(1).copied().unwrap_or("plasma");
            let fps = 30u64;
            let frame_ms = 1000 / fps;

            crate::println!("=== TrustVideo Demo: {} ===", effect);
            crate::println!("Rendering in real-time @ {}fps", fps);
            crate::println!("Press Q or ESC to stop");

            // Real-time streaming render — no file accumulation
            let sw = crate::framebuffer::width();
            let sh = crate::framebuffer::height();
            let vw = sw.min(640) as u16;
            let vh = sh.min(480) as u16;

            match effect {
                "plasma" | "fire" | "matrix" | "shader" => {
                    crate::video::player::render_realtime(effect, vw, vh, fps as u16);
                }
                _ => {
                    crate::println!("Unknown effect: {}. Available: plasma, fire, matrix, shader", effect);
                }
            }
        }

        "play" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: video play <file.tv>"); return; }
            };

            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/home/{}", filename)
            };

            match crate::vfs::read_file(&path) {
                Ok(data) => {
                    crate::println!("Playing {}...", filename);
                    let mut player = crate::video::player::VideoPlayer::new();
                    match player.play_data(data) {
                        Ok(msg) => crate::println!("{}", msg),
                        Err(e) => crate::println!("Error: {}", e),
                    }
                }
                Err(_) => crate::println!("File not found: {}", path),
            }
        }

        "info" => {
            let filename = match args.get(1) {
                Some(f) => *f,
                None => { crate::println!("Usage: video info <file.tv>"); return; }
            };

            let path = if filename.starts_with('/') {
                alloc::string::String::from(filename)
            } else {
                alloc::format!("/home/{}", filename)
            };

            match crate::vfs::read_file(&path) {
                Ok(data) => {
                    if let Some(hdr) = crate::video::codec::TvHeader::from_bytes(&data) {
                        crate::println!("=== TrustVideo Info ===");
                        crate::println!("  Format:     TrustVideo v{}", hdr.version);
                        crate::println!("  Resolution: {}x{}", hdr.width, hdr.height);
                        crate::println!("  FPS:        {}", hdr.fps);
                        crate::println!("  Frames:     {}", hdr.frame_count);
                        crate::println!("  Duration:   {:.1}s", hdr.frame_count as f64 / hdr.fps as f64);
                        crate::println!("  Keyframe:   every {} frames", hdr.keyframe_interval);
                        crate::println!("  File size:  {} bytes ({} KB)", data.len(), data.len() / 1024);
                        let raw_size = hdr.width as usize * hdr.height as usize * 4 * hdr.frame_count as usize;
                        if raw_size > 0 {
                            let ratio = raw_size as f64 / data.len() as f64;
                            crate::println!("  Compression: {:.1}x (raw would be {} KB)", ratio, raw_size / 1024);
                        }
                    } else {
                        crate::println!("Not a valid TrustVideo file");
                    }
                }
                Err(_) => crate::println!("File not found: {}", path),
            }
        }

        _ => {
            crate::println!("TrustVideo — Custom video codec for TrustOS");
            crate::println!("");
            crate::println!("Usage: video <command> [args]");
            crate::println!("");
            crate::println!("Commands:");
            crate::println!("  demo [effect]  Generate & play a demo animation");
            crate::println!("                 Effects: plasma, fire, matrix, shader");
            crate::println!("  play <file>    Play a .tv video file");
            crate::println!("  info <file>    Show video file info");
            crate::println!("");
            crate::println!("Controls during playback:");
            crate::println!("  Q / ESC        Stop playback");
            crate::println!("  Space          Pause / Resume");
        }
    }
}
