//! Kernel Shell (Bootstrap Mode)
//! 
//! A full-featured shell running in kernel mode with standard commands.
//! This is temporary until Ring 3 userland is implemented.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::AtomicBool;
use spin::Mutex as SpinMutex;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};
use crate::ramfs::FileType;

/// Capture mode: when true, print output goes to CAPTURE_BUF instead of screen
pub(crate) static CAPTURE_MODE: AtomicBool = AtomicBool::new(false);
/// Buffer for captured output during pipe execution
static CAPTURE_BUF: SpinMutex<String> = SpinMutex::new(String::new());

/// Append text to the capture buffer (called from print macros when capture mode is on)
pub fn capture_write(s: &str) {
    if CAPTURE_MODE.load(core::sync::atomic::Ordering::Relaxed) {
        let mut buf = CAPTURE_BUF.lock();
        buf.push_str(s);
    }
}

/// Check if we are in capture mode (for print macros to redirect output)
pub fn is_capturing() -> bool {
    CAPTURE_MODE.load(core::sync::atomic::Ordering::Relaxed)
}

/// Draw a solid block cursor at the current console position (no cursor movement).
/// Uses fill_rect directly to avoid non-ASCII character encoding issues.
#[inline]
fn show_cursor_block() {
    let (col, row) = crate::framebuffer::get_cursor();
    crate::framebuffer::fill_rect(
        (col * 8) as u32, (row * 16) as u32, 8, 16, COLOR_GREEN,
    );
}

// ===============================================================================
// PARALLEL MATRIX RENDERING
// ===============================================================================

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
pub(super) fn render_matrix_columns_parallel(start: usize, end: usize, data: *mut u8) {
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
    // Binary analysis
    "trustview", "tv",
    // TrustLab
    "lab", "trustlab",
    // Fun
    "neofetch", "matrix", "cowsay",
    // Showcase
    "showcase",
    "showcase3d",
    "filled3d",
    // Hypervisor
    "hv", "hypervisor",
    // Trailer
    "trailer", "trustos_trailer",
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
    show_cursor_block();
    
    loop {
        if let Some(c) = read_char() {
            // Hide cursor before processing
            let under_cursor = if cursor < pos { buffer[cursor] as char } else { ' ' };
            crate::print_fb_only!("{}", under_cursor);
            crate::print_fb_only!("\x08");
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
                        crate::print_fb_only!("\x08");
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print_fb_only!(" ");
                        for _ in cursor..=pos {
                            crate::print_fb_only!("\x08");
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
                        crate::print_fb_only!("\x08");
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
                        crate::print_fb_only!("\x08");
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
                        crate::print_fb_only!(" ");
                        for _ in cursor..=pos {
                            crate::print_fb_only!("\x08");
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
                            crate::print_fb_only!("\x08");
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
                        crate::print_fb_only!("\x08");
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
                        crate::print_fb_only!("\x08");
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
            show_cursor_block();
        } else {
            // No input - handle cursor blinking
            blink_counter += 1;
            if blink_counter >= BLINK_INTERVAL {
                blink_counter = 0;
                cursor_visible = !cursor_visible;
                
                if cursor_visible {
                    show_cursor_block();
                } else {
                    let under_cursor = if cursor < pos { buffer[cursor] as char } else { ' ' };
                    crate::print_fb_only!("{}", under_cursor);
                    crate::print_fb_only!("\x08");
                }
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }
    }
    
    // Hide cursor before returning
    let under_cursor = if cursor < pos { buffer[cursor] as char } else { ' ' };
    crate::print_fb_only!("{}", under_cursor);
    if cursor < pos { crate::print_fb_only!("\x08"); }
    
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
    
    // Use direct Writer access to guarantee no serial output
    use core::fmt::Write;
    for (i, cmd) in suggestions.iter().enumerate() {
        crate::framebuffer::set_cursor(0, input_row + 1 + i);
        if i as i32 == selected_idx {
            let _ = write!(crate::framebuffer::Writer, " > {}", cmd);
        } else {
            let _ = write!(crate::framebuffer::Writer, "   {}", cmd);
        }
    }
}

/// Clear the suggestions display at given row
fn clear_suggestions_at_row(input_row: usize, count: usize) {
    use core::fmt::Write;
    for i in 0..count {
        crate::framebuffer::set_cursor(0, input_row + 1 + i);
        for _ in 0..40 {
            let _ = write!(crate::framebuffer::Writer, " ");
        }
    }
}

/// Clear the current input line display
fn clear_line_display(cursor: usize, pos: usize) {
    use core::fmt::Write;
    // Move cursor to start of input
    for _ in 0..cursor {
        let _ = write!(crate::framebuffer::Writer, "\x08");
    }
    // Clear all characters
    for _ in 0..pos {
        let _ = write!(crate::framebuffer::Writer, " ");
    }
    // Move back to start
    for _ in 0..pos {
        let _ = write!(crate::framebuffer::Writer, "\x08");
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
    crate::println_color!(COLOR_CYAN, "  T-RustOs v0.2.0 - Type 'help' for commands");
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

/// Read a line from keyboard input (public for REPL use)
pub fn read_line() -> alloc::string::String {
    let mut buf = [0u8; 512];
    let len = crate::keyboard::read_line(&mut buf);
    core::str::from_utf8(&buf[..len]).unwrap_or("").into()
}

/// Execute a shell command (handles pipes and redirection)
pub(super) fn execute_command(cmd: &str) {
    if cmd.is_empty() {
        return;
    }
    
    // ── Pipeline: split on | (outside quotes) ──────────────────────────
    let pipe_segments = split_pipes(cmd);
    if pipe_segments.len() > 1 {
        execute_pipeline(&pipe_segments);
        return;
    }
    
    // ── Single command (no pipes) ──────────────────────────────────────
    execute_single(cmd, None);
}

/// Split command on unquoted pipe characters
fn split_pipes(cmd: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut in_sq = false;
    let mut in_dq = false;
    let bytes = cmd.as_bytes();
    
    for i in 0..bytes.len() {
        match bytes[i] {
            b'\'' if !in_dq => in_sq = !in_sq,
            b'"' if !in_sq => in_dq = !in_dq,
            b'|' if !in_sq && !in_dq => {
                // Make sure this isn't || (logical OR — not supported, but don't break on it)
                if i + 1 < bytes.len() && bytes[i + 1] == b'|' {
                    continue; // skip ||
                }
                segments.push(cmd[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    segments.push(cmd[start..].trim());
    segments
}

/// Execute a pipeline: cmd1 | cmd2 | cmd3
fn execute_pipeline(segments: &[&str]) {
    let mut piped_input: Option<String> = None;
    
    for (i, segment) in segments.iter().enumerate() {
        let is_last = i == segments.len() - 1;
        let input = piped_input.take();
        
        if is_last {
            // Last stage: print output normally (or redirect)
            execute_single(segment, input);
        } else {
            // Intermediate stage: capture output
            piped_input = Some(capture_command(segment, input));
        }
    }
}

/// Capture a command's output as a string (for pipe intermediates)
fn capture_command(cmd: &str, piped_input: Option<String>) -> String {
    // Enable capture mode
    CAPTURE_MODE.store(true, core::sync::atomic::Ordering::SeqCst);
    {
        let mut buf = CAPTURE_BUF.lock();
        buf.clear();
    }
    
    // Run the command (output goes to capture buffer via print macros)
    execute_single(cmd, piped_input);
    
    // Disable capture mode and return captured output
    CAPTURE_MODE.store(false, core::sync::atomic::Ordering::SeqCst);
    let buf = CAPTURE_BUF.lock();
    buf.clone()
}

/// Execute a single command, possibly with piped stdin
fn execute_single(cmd: &str, piped_input: Option<String>) {
    
    // Handle output redirection (skip > inside parentheses or quotes)
    let (cmd_part, redirect) = {
        let mut redir_pos: Option<usize> = None;
        let mut paren_depth: i32 = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let bytes = cmd.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let ch = bytes[i] as char;
            match ch {
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                '(' if !in_single_quote && !in_double_quote => paren_depth += 1,
                ')' if !in_single_quote && !in_double_quote => {
                    if paren_depth > 0 { paren_depth -= 1; }
                }
                '>' if !in_single_quote && !in_double_quote && paren_depth == 0 => {
                    redir_pos = Some(i);
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        if let Some(pos) = redir_pos {
            let append = cmd[pos..].starts_with(">>");
            let file = if append {
                cmd[pos + 2..].trim()
            } else {
                cmd[pos + 1..].trim()
            };
            (cmd[..pos].trim(), Some((file, append)))
        } else {
            (cmd, None)
        }
    };
    
    // Split into command and arguments
    let parts: Vec<&str> = cmd_part.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    
    let command = parts[0];
    let args = &parts[1..];
    
    match command {
        // -- commands module: Help, FS, System, Auth, Debug, Exit, Easter eggs --
        "help" => commands::cmd_help(args),
        "man" => commands::cmd_man(args),
        "info" => commands::cmd_info(),
        "version" => commands::cmd_version(),
        "uname" => commands::cmd_uname(args),
        "ls" | "dir" => commands::cmd_ls(args),
        "cd" => commands::cmd_cd(args),
        "pwd" => commands::cmd_pwd(),
        "mkdir" => commands::cmd_mkdir(args),
        "rmdir" => commands::cmd_rmdir(args),
        "touch" => commands::cmd_touch(args),
        "rm" | "del" => commands::cmd_rm(args),
        "cp" | "copy" => commands::cmd_cp(args),
        "mv" | "move" | "rename" => commands::cmd_mv(args),
        "cat" | "type" => commands::cmd_cat(args, redirect, piped_input.as_deref()),
        "head" => commands::cmd_head(args, piped_input.as_deref()),
        "tail" => commands::cmd_tail(args, piped_input.as_deref()),
        "wc" => commands::cmd_wc(args, piped_input.as_deref()),
        "stat" => commands::cmd_stat(args),
        "tree" => commands::cmd_tree(args),
        "find" => commands::cmd_find(args),
        "echo" => commands::cmd_echo(args, redirect),
        "grep" => commands::cmd_grep(args, piped_input.as_deref()),
        "clear" | "cls" => commands::cmd_clear(),
        "time" | "uptime" => commands::cmd_time(),
        "date" => commands::cmd_date(),
        "whoami" => commands::cmd_whoami(),
        "hostname" => commands::cmd_hostname(),
        "id" => commands::cmd_id(),
        "env" | "printenv" => commands::cmd_env(),
        "history" => commands::cmd_history(),
        "ps" => commands::cmd_ps(),
        "free" => commands::cmd_free(),
        "df" => commands::cmd_df(),
        "login" => commands::cmd_login(),
        "su" => commands::cmd_su(args),
        "passwd" => commands::cmd_passwd(args),
        "adduser" | "useradd" => commands::cmd_adduser(args),
        "deluser" | "userdel" => commands::cmd_deluser(args),
        "users" => commands::cmd_users(),
        "test" => commands::cmd_test(),
        "memtest" => commands::cmd_memtest(),
        "inttest" => commands::cmd_inttest(),
        "debugnew" => commands::cmd_debugnew(),
        "nvme" => commands::cmd_nvme(),
        "keytest" => commands::cmd_keytest(),
        "hexdump" | "xxd" => commands::cmd_hexdump(args),
        "panic" => commands::cmd_panic(),
        "exit" | "logout" => commands::cmd_logout(),
        "reboot" => commands::cmd_reboot(),
        "shutdown" | "halt" | "poweroff" => commands::cmd_halt(),
        "neofetch" => commands::cmd_neofetch(),
        "matrix" => commands::cmd_matrix(),
        "cowsay" => commands::cmd_cowsay(args),

        // -- desktop module: COSMIC, Showcase, Benchmark, Signature, Security --
        "benchmark" | "bench" => desktop::cmd_benchmark(args),
        "showcase" => desktop::cmd_showcase(args),
        "showcase3d" | "demo3d" => desktop::cmd_showcase3d(),
        "filled3d" => desktop::cmd_filled3d(),
        "desktop" | "gui" => desktop::launch_desktop_env(None),
        "cosmic" => desktop::cmd_cosmic_v2(),
        "open" => desktop::cmd_open(args),
        "trustedit" | "edit3d" | "3dedit" => desktop::launch_desktop_env(Some(("TrustEdit 3D", crate::desktop::WindowType::ModelEditor, 100, 60, 700, 500))),
        "calculator" | "calc" => desktop::launch_desktop_env(Some(("Calculator", crate::desktop::WindowType::Calculator, 300, 200, 320, 420))),
        "snake" => desktop::launch_desktop_env(Some(("Snake", crate::desktop::WindowType::Game, 200, 100, 400, 400))),
        "signature" | "sig" => desktop::cmd_signature(args),
        "security" | "sec" | "caps" => desktop::cmd_security(args),

        // -- vm module: VM, Linux, Distro, Alpine, Disk, Hardware, Network core --
        "vm" | "linux" | "gui" => {
            if args.is_empty() {
                vm::cmd_linux_shell();
            } else {
                match args[0] {
                    // Real hypervisor VM commands
                    "create" | "run" | "start" | "guests" | "inspect" | "mount" | "input" => vm::cmd_vm(args),
                    "status" => vm::cmd_gui_status(),
                    "install" => vm::cmd_gui_install(),
                    "console" | "shell" => vm::cmd_linux_shell(),
                    "stop" => vm::cmd_vm_stop(),
                    "list" => vm::cmd_vm_list(),
                    "extract" => apps::create_test_binaries(),
                    "exec" => {
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
                    "help" | "--help" | "-h" => vm::cmd_vm_help(),
                    _ => vm::cmd_vm_help(),
                }
            }
        },
        "distro" | "distros" => {
            if args.is_empty() {
                vm::cmd_distro_list();
            } else {
                match args[0] {
                    "list" => vm::cmd_distro_list(),
                    "install" | "download" => {
                        if args.len() > 1 { vm::cmd_distro_install(args[1]); }
                        else { vm::cmd_distro_gui(); }
                    },
                    "run" | "start" => {
                        if args.len() > 1 { vm::cmd_distro_run(args[1]); }
                        else { crate::println!("Usage: distro run <id>"); }
                    },
                    "pick" | "select" => vm::cmd_distro_gui(),
                    _ => vm::cmd_distro_list(),
                }
            }
        },
        "glmode" | "compositor" => vm::cmd_glmode(args),
        "theme" => vm::cmd_theme(args),
        "anim" | "animations" => vm::cmd_animations(args),
        "holo" | "holomatrix" => vm::cmd_holomatrix(args),
        "imgview" | "imageview" | "view" => vm::cmd_imgview(args),
        "imgdemo" | "imagedemo" => vm::cmd_imgdemo(args),
        "tasks" | "jobs" => vm::cmd_tasks(),
        "threads" => vm::cmd_threads(),
        "alpine" => vm::cmd_alpine(args),
        "apt-get" | "apt" | "apk" | "dpkg" => vm::cmd_pkg(command, args),
        "persist" | "persistence" => vm::cmd_persistence(args),
        "disk" => vm::cmd_disk(),
        "dd" => vm::cmd_dd(args),
        "ahci" => vm::cmd_ahci(args),
        "fdisk" | "partitions" => vm::cmd_fdisk(args),
        "lspci" => vm::cmd_lspci(args),
        "lshw" | "hwinfo" => vm::cmd_lshw(),
        "beep" => vm::cmd_beep(args),
        "audio" => vm::cmd_audio(args),
        "synth" => vm::cmd_synth(args),
        "ifconfig" | "ip" => vm::cmd_ifconfig(),
        "ipconfig" => vm::cmd_ipconfig(args),
        "ping" => vm::cmd_ping(args),
        "tcpsyn" => vm::cmd_tcpsyn(args),
        "httpget" => vm::cmd_httpget(args),
        "curl" | "wget" => vm::cmd_curl(args),
        "download" => vm::cmd_download(args),
        "nslookup" | "dig" => vm::cmd_nslookup(args),
        "arp" => vm::cmd_arp(args),
        "route" => vm::cmd_route(args),
        "traceroute" | "tracert" => vm::cmd_traceroute(args),
        "netstat" => vm::cmd_netstat(),
        "exec" | "run" | "./" => vm::cmd_exec(args, command),
        "elfinfo" => vm::cmd_elfinfo(args),
        "lsusb" => unix::cmd_lsusb(),
        "lscpu" => unix::cmd_lscpu(),
        "smpstatus" => unix::cmd_smpstatus(),
        "smp" => unix::cmd_smp(args),
        "fontsmooth" => unix::cmd_fontsmooth(args),
        "hv" | "hypervisor" => vm::cmd_hypervisor(args),

        // -- network module: Browser, Sandbox, Container --
        "browse" | "www" | "web" => network::cmd_browse(args),
        "sandbox" | "websandbox" => network::cmd_sandbox(args),
        "container" | "webcontainer" | "wc" => network::cmd_container(args),

        // -- unix module: Unix utilities and stubs --
        "which" => unix::cmd_which(args),
        "whereis" => unix::cmd_whereis(args),
        "file" => unix::cmd_file(args),
        "basename" => unix::cmd_basename(args),
        "dirname" => unix::cmd_dirname(args),
        "realpath" => unix::cmd_realpath(args),
        "sort" => unix::cmd_sort(args, piped_input.as_deref()),
        "uniq" => unix::cmd_uniq(args, piped_input.as_deref()),

        "yes" => unix::cmd_yes(args),
        "seq" => unix::cmd_seq(args),
        "sleep" => unix::cmd_sleep(args),
        "kill" => unix::cmd_kill(args),

        "top" => unix::cmd_top(),
        "htop" => unix::cmd_top(),
        "vmstat" => unix::cmd_vmstat(),

        "lsof" => unix::cmd_lsof(args),

        "strings" => unix::cmd_strings(args),

        "mount" => unix::cmd_mount(args),

        "sync" => unix::cmd_sync(),
        "lsblk" => unix::cmd_lsblk(),
        "blkid" => unix::cmd_blkid(),

        "export" => unix::cmd_export(args),

        "source" | "." => unix::cmd_source(args),
        "set" => unix::cmd_set(args),

        "printf" => unix::cmd_printf(args),
        "test" | "[" => unix::cmd_test_expr(args),
        "expr" => unix::cmd_expr(args),

        "cal" => unix::cmd_cal(args),

        "cmp" => unix::cmd_cmp(args),

        "od" => unix::cmd_od(args),
        "rev" => unix::cmd_rev(args),
        "factor" => unix::cmd_factor(args),

        "tty" => unix::cmd_tty(),
        "stty" => unix::cmd_stty(args),
        "reset" => unix::cmd_reset(),

        "lsmem" => unix::cmd_lsmem(),

        "lsmod" => unix::cmd_lsmod(),

        "sysctl" => unix::cmd_sysctl(args),

        "dmesg" => unix::cmd_dmesg(args),
        "memdbg" | "heapdbg" => unix::cmd_memdbg(),
        "perf" | "perfstat" => unix::cmd_perfstat(),
        "irqstat" | "irqs" => unix::cmd_irqstat(),
        "regs" | "registers" | "cpuregs" => unix::cmd_registers(),
        "peek" | "memdump" => unix::cmd_peek(args),
        "poke" | "memwrite" => unix::cmd_poke(args),
        "devpanel" => unix::cmd_devpanel(),
        "timecmd" => unix::cmd_timecmd(args),

        // -- apps module: TrustLang, Film, Transpile, Video, Lab, Gterm, Wayland --
        "wayland" | "wl" => apps::cmd_wayland(args),
        "gterm" | "graphterm" => apps::cmd_gterm(args),
        "transpile" | "disasm" | "analyze" => apps::cmd_transpile(args),
        "trustview" | "tv" => apps::cmd_trustview(args),
        "lab" | "trustlab" => apps::cmd_lab(args),
        "trustlang" | "tl" => apps::cmd_trustlang(args),
        "trustlang_showcase" | "tl_showcase" => apps::cmd_trustlang_showcase(),
        "film" | "trustos_film" => apps::cmd_trustos_film(),
        "trailer" | "trustos_trailer" => trailer::cmd_trustos_trailer(),
        "video" => apps::cmd_video(args),

        // -- Jarvis AI assistant --
        "jarvis" | "j" | "ai" | "assistant" => jarvis::cmd_jarvis(args),

        "" => {}
        _ if unix::try_stub(command) => {}
        _ => {
            // Check if it's an executable file
            if vm::try_exec_file(command, args) {
                return;
            }
            crate::print_color!(COLOR_RED, "tsh: ");
            crate::print!("{}", command);
            crate::println_color!(COLOR_RED, ": command not found");
        }
    }
}


// 
// SUBMODULES  split from the original 19K-line monolith for clarity
// 

mod commands;       // Help, FS, System, Auth, Debug, Exit, Easter eggs  (~1530 lines)
pub(crate) mod desktop;    // COSMIC UI, Showcase, Benchmark, Signature         (~6870 lines)
mod vm;             // VM, Linux, Distro, Alpine, Hypervisor, Disk       (~4220 lines)
mod network;        // Browser, Sandbox, Container, HTML rendering       (~830 lines)
mod unix;           // Unix utility stubs and POSIX commands             (~1160 lines)
mod apps;           // TrustLang, Film, Transpile, Video, Lab, Gterm    (~3930 lines)
mod trailer;        // TrustOS Trailer -- 2-min cinematic showcase         (~900 lines)
mod jarvis;         // Jarvis AI assistant — NLU + planner + executor     (~600 lines)
