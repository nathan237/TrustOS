//! Kernel Shell (Bootstrap Mode)
//! 
//! A full-featured shell running in kernel mode with standard commands.
//! This is temporary until Ring 3 userland is implemented.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex as SpinMutex;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};
use crate::ramfs::FileType;

/// Ctrl+C interrupt flag — set by shell when user presses Ctrl+C
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Check if Ctrl+C was pressed (non-destructive read)
pub fn is_interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Clear the interrupt flag (call before starting a long-running command)
pub fn clear_interrupted() {
    INTERRUPTED.store(false, Ordering::SeqCst);
}

/// Set the interrupt flag (called from keyboard handler or shell)
pub fn set_interrupted() {
    INTERRUPTED.store(true, Ordering::SeqCst);
}

/// Capture mode: when true, print output goes to CAPTURE_BUF instead of screen
pub(crate) // Variable atomique — accès thread-safe sans verrou.
static CAPTURE_MODE: AtomicBool = AtomicBool::new(false);
/// Buffer for captured output during pipe execution
static CAPTURE_BUFFER: SpinMutex<String> = SpinMutex::new(String::new());

/// Append text to the capture buffer (called from print macros when capture mode is on)
pub fn capture_write(s: &str) {
    if CAPTURE_MODE.load(core::sync::atomic::Ordering::Relaxed) {
        let mut buffer = CAPTURE_BUFFER.lock();
        buffer.push_str(s);
    }
}

/// Check if we are in capture mode (for print macros to redirect output)
pub fn is_capturing() -> bool {
    CAPTURE_MODE.load(core::sync::atomic::Ordering::Relaxed)
}

/// Clear the capture buffer and return its contents.
/// Used by the desktop terminal to retrieve command output.
pub fn take_captured() -> String {
    let mut buffer = CAPTURE_BUFFER.lock();
    let s = buffer.clone();
    buffer.clear();
    s
}

/// Draw a solid block cursor at the current console position (no cursor movement).
/// Uses fill_rect directly to avoid non-ASCII character encoding issues.
#[inline]
fn show_cursor_block() {
    let (column, row) = crate::framebuffer::get_cursor();
    crate::framebuffer::fill_rect(
        (column * 8) as u32, (row * 16) as u32, 8, 16, COLOR_GREEN,
    );
}

// ===============================================================================
// PARALLEL MATRIX RENDERING
// ===============================================================================

/// Parameters for parallel Matrix rendering
#[repr(C)]
// Structure publique — visible à l'extérieur de ce module.
pub struct MatrixRenderParams {
    pub buffer_pointer: *mut u32,
    pub buffer_length: usize,
    pub width: u32,
    pub height: u32,
    pub matrix_chars: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8,              // Pointer to flat character grid (col * 68 + row)
    pub matrix_heads: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const i32,              // Pointer to head positions array
    pub holo_intensity: *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8,             // Pointer to flat holo intensity map (col * 68 + row)
    pub matrix_rows: usize,                    // Number of rows (68)
}

// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Send for MatrixRenderParams {}
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Sync for MatrixRenderParams {}

/// Render Matrix columns in parallel - called by each core
/// start/end are column indices
pub(super) fn render_matrix_columns_parallel(start: usize, end: usize, data: *mut u8) {
    let params = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*(data as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MatrixRenderParams) };
    let buffer_pointer = params.buffer_pointer;
    let buffer_length = params.buffer_length;
    let width = params.width;
    let height = params.height;
    let holo_pointer = params.holo_intensity;
    let has_holo = !holo_pointer.is_null();
    let matrix_rows = params.matrix_rows;
    
    let column_width = 8u32;
    
    for column in start..end {
        if column >= 240 { break; }
        
        let x = column as u32 * column_width;
        if x >= width { continue; }
        
        let head = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *params.matrix_heads.add(column) };
        let start_row = (head - 5).maximum(0) as usize;
        let end_row = if head + 30 < 0 { 0 } else { ((head + 30) as usize).minimum(matrix_rows) };
        
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
                let factor = ((dist - 12) as u32).minimum(15) * 16;
                let fade = 255u32.saturating_sub(factor);
                (160 * fade) / 255
            } else {
                continue;
            };
            
            // Flat index for matrix access
            let flat_index = column * matrix_rows + row;
            
            // Apply holo intensity modifier if present
            // Character to use (may be overridden by holo)
            let (final_char, color) = if has_holo {
                let holo_value = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *holo_pointer.add(flat_index) };
                if holo_value >= 1 {
                    // INSIDE SHAPE (edge or interior) - use fixed character '#' and pure green
                    ('#', 0xFF00FF00)
                } else {
                    // OUTSIDE - normal character and normal color
                    let c = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *params.matrix_chars.add(flat_index) as char };
                    (c, 0xFF000000 | (base_green << 8))
                }
            } else {
                // No holo - normal rendering
                let c = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { *params.matrix_chars.add(flat_index) as char };
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
                        if bits & 0x80 != 0 { let index = row_offset + x_usize; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                        if bits & 0x40 != 0 { let index = row_offset + x_usize + 1; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                        if bits & 0x20 != 0 { let index = row_offset + x_usize + 2; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                        if bits & 0x10 != 0 { let index = row_offset + x_usize + 3; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                        if bits & 0x08 != 0 { let index = row_offset + x_usize + 4; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                        if bits & 0x04 != 0 { let index = row_offset + x_usize + 5; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                        if bits & 0x02 != 0 { let index = row_offset + x_usize + 6; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                        if bits & 0x01 != 0 { let index = row_offset + x_usize + 7; if index < buffer_length { *buffer_pointer.add(index) = color; } }
                    }
                }
            }
        }
    }
}

/// List of all shell commands for autocomplete
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SHELL_COMMANDS: &[&str] = &[
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
    "hwtest", "keytest", "hexdump", "xxd", "panic",
    // Hardware debug toolkit
    "hwdiag", "cpudump", "stacktrace", "backtrace", "bootlog", "postcode",
    "ioport", "rdmsr", "wrmsr", "cpuid", "memmap", "watchdog",
    // Desktop GUI (multi-layer compositor)
    "desktop", "gui", "mobile", "cosmic", "open", "trustedit",
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
    "lspci", "lshw", "hwinfo", "gpu", "gpuexec", "sdma", "neural", "gpufw", "a11y",
    // USB / checkm8
    "lsusb", "checkm8",
    // Audio
    "beep", "audio", "synth", "play", "vizfx",
    // ThinkPad EC / Power
    "fan", "temp", "sensors", "cpufreq", "speedstep",
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
    // TrustProbe
    "hwscan", "trustprobe", "probe",
    // Fun
    "neofetch", "matrix", "cowsay", "rain",
    // Showcase
    "showcase",
    "showcase3d",
    "filled3d",
    // Tutorial
    "demo", "tutorial", "tour",
    // Hypervisor
    "hv", "hypervisor",
    // Trailer
    "trailer", "trustos_trailer",
    // Text editor
    "nano", "edit", "vi",
    // Aliases and utilities
    "alias", "unalias", "bc", "diff", "md5sum", "sha256sum", "base64",
    "cut", "tr", "tee", "xargs", "chmod", "chown", "ln", "readlink",
    "watch", "timeout", "tar", "gzip", "zip", "unzip",
    "service", "systemctl", "crontab", "at", "read",
];

/// Run the kernel shell
pub fn run() -> ! {
    // Initialize theme and clear screen
    crate::framebuffer::set_matrix_theme();
    crate::framebuffer::clear();

    // Initialize shell scripting engine (variables, etc.)
    scripting::init();

    // Note: ramfs is already initialized in main.rs before persistence restore

    // Bootstrapping complete: allow timer handler to run
    crate::interrupts::set_bootstrap_ready(true);

    print_banner();

    // Run startup script (.trustrc)
    unix::run_trustrc();
    
    let mut command_buffer = [0u8; 512];
    
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        print_prompt();
        
        // Read command with autocomplete support
        let len = read_line_with_autocomplete(&mut command_buffer);
        
        // Parse and execute
        let command_str = core::str::from_utf8(&command_buffer[..len]).unwrap_or("");
        clear_interrupted(); // Reset Ctrl+C flag before each command
        execute_command(command_str.trim());
    }
}

/// Read a line with dynamic autocomplete and arrow navigation for suggestions
fn read_line_with_autocomplete(buffer: &mut [u8]) -> usize {
    use crate::keyboard::{read_char, add_to_history, history_previous, history_next, history_reset,
                          KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_HOME, KEY_END, KEY_DELETE,
                          KEY_PGUP, KEY_PGDOWN};
    
    let mut position: usize = 0;
    let mut cursor: usize = 0;
    let mut suggestions: Vec<&str> = Vec::new();
    let mut suggestion_index: i32 = -1; // -1 means no suggestion selected
    let mut showing_suggestions = false;
    
    // Save the row where input starts (for suggestion display)
    // If we're at the bottom of the screen, scroll up to make room for suggestions
    let (_scr_w, scr_h) = crate::framebuffer::get_dimensions();
    let maximum_rows = (scr_h as usize) / 16; // CHAR_HEIGHT = 16
    let mut input_row = crate::framebuffer::get_cursor().1;
    let input_column_start = crate::framebuffer::get_cursor().0;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SUGGESTION_ROOM: usize = 4; // Reserve space for up to 4 visible suggestions
    if maximum_rows > SUGGESTION_ROOM && input_row + SUGGESTION_ROOM >= maximum_rows {
        let lines_needed = input_row + SUGGESTION_ROOM - maximum_rows + 1;
        for _ in 0..lines_needed {
            crate::framebuffer::scroll_up();
        }
        input_row = input_row.saturating_sub(lines_needed);
        crate::framebuffer::set_cursor(input_column_start, input_row);
    }
    
    // Cursor blinking state
    let mut cursor_visible = true;
    let mut blink_counter: u32 = 0;
        // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BLINK_INTERVAL: u32 = 500000;
    
    history_reset();
    
    // Show initial cursor
    show_cursor_block();
    
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        if let Some(c) = read_char() {
            // Auto-snap to live view on any non-scroll keypress
            if c != KEY_PGUP && c != KEY_PGDOWN && crate::framebuffer::is_scrolled_back() {
                let (_column, row) = crate::framebuffer::restore_live_view();
                input_row = row;
                // current_line already contains prompt + typed text (scrollback tracks it).
                // Just position the cursor at the right spot within the input.
                crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
            }
            
            // Hide cursor before processing
            let under_cursor = if cursor < position { buffer[cursor] as char } else { ' ' };
            crate::print_framebuffer_only!("{}", under_cursor);
            crate::print_framebuffer_only!("\x08");
            cursor_visible = true;
            blink_counter = 0;
            
                        // Correspondance de motifs — branchement exhaustif de Rust.
match c {
                b'\n' | b'\r' => {
                    // Reset scroll to bottom when pressing Enter
                    crate::framebuffer::scroll_to_bottom();
                    
                    // If a suggestion is selected, use it
                    if suggestion_index >= 0 && (suggestion_index as usize) < suggestions.len() {
                        let selected = suggestions[suggestion_index as usize];
                        clear_suggestions_at_row(input_row, suggestions.len());
                        clear_line_display(input_column_start, input_row, position);
                        let bytes = selected.as_bytes();
                        let len = bytes.len().minimum(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        position = len;
                        cursor = len;
                        for i in 0..position {
                            crate::print!("{}", buffer[i] as char);
                        }
                    } else if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                    }
                    crate::println!();
                    let cmd = core::str::from_utf8(&buffer[..position]).unwrap_or("");
                    if !cmd.trim().is_empty() {
                        add_to_history(cmd);
                    }
                    break;
                }
                b'\t' => {
                    // Tab: complete with first/selected suggestion
                    if !suggestions.is_empty() {
                        let index = if suggestion_index >= 0 { suggestion_index as usize } else { 0 };
                        let selected = suggestions[index];
                        clear_suggestions_at_row(input_row, suggestions.len());
                        clear_line_display(input_column_start, input_row, position);
                        let bytes = selected.as_bytes();
                        let len = bytes.len().minimum(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        position = len;
                        cursor = len;
                        for i in 0..position {
                            crate::print!("{}", buffer[i] as char);
                        }
                        suggestion_index = -1;
                        showing_suggestions = false;
                        suggestions.clear();
                        if position < buffer.len() - 1 {
                            buffer[position] = b' ';
                            position += 1;
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
                        suggestion_index = -1;
                        showing_suggestions = false;
                    }
                }
                0x08 => {
                    // Backspace
                    if cursor > 0 {
                        // Clear suggestions first and restore cursor position
                        if showing_suggestions {
                            clear_suggestions_at_row(input_row, suggestions.len());
                            crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                        }
                        for i in cursor..position {
                            buffer[i - 1] = buffer[i];
                        }
                        position = position.saturating_sub(1);
                        cursor = cursor.saturating_sub(1);
                        crate::print_framebuffer_only!("\x08");
                        for i in cursor..position {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print_framebuffer_only!(" ");
                        for _ in cursor..=position {
                            crate::print_framebuffer_only!("\x08");
                        }
                        // Update and show suggestions
                        update_suggestions(buffer, position, &mut suggestions);
                        if !suggestions.is_empty() && position > 0 {
                            show_suggestions_at_row(input_row, &suggestions, suggestion_index);
                            showing_suggestions = true;
                            crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                        } else {
                            showing_suggestions = false;
                            suggestion_index = -1;
                        }
                    }
                }
                KEY_UP => {
                    if position == 0 {
                        // No input: navigate history
                        if let Some(previous) = history_previous() {
                            let bytes = previous.as_bytes();
                            let len = bytes.len().minimum(buffer.len() - 1);
                            buffer[..len].copy_from_slice(&bytes[..len]);
                            position = len;
                            cursor = len;
                            crate::print!("{}", &previous[..len]);
                        }
                    } else {
                        // Has input: show/navigate suggestions
                        if !showing_suggestions {
                            // First UP press: show suggestions
                            update_suggestions(buffer, position, &mut suggestions);
                            if !suggestions.is_empty() {
                                suggestion_index = 0;
                                show_suggestions_at_row(input_row, &suggestions, suggestion_index);
                                showing_suggestions = true;
                                // Return cursor to input position
                                crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                            }
                        } else if !suggestions.is_empty() {
                            // Already showing: navigate up
                            clear_suggestions_at_row(input_row, suggestions.len());
                            if suggestion_index <= 0 {
                                suggestion_index = suggestions.len() as i32 - 1;
                            } else {
                                suggestion_index -= 1;
                            }
                            show_suggestions_at_row(input_row, &suggestions, suggestion_index);
                            crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                        }
                    }
                }
                KEY_DOWN => {
                    if position == 0 {
                        // No input: navigate history
                        if let Some(next) = history_next() {
                            let bytes = next.as_bytes();
                            let len = bytes.len().minimum(buffer.len() - 1);
                            buffer[..len].copy_from_slice(&bytes[..len]);
                            position = len;
                            cursor = len;
                            crate::print!("{}", &next[..len]);
                        } else {
                            clear_line_display(input_column_start, input_row, position);
                            position = 0;
                            cursor = 0;
                        }
                    } else if showing_suggestions && !suggestions.is_empty() {
                        // Navigate suggestions down
                        clear_suggestions_at_row(input_row, suggestions.len());
                        suggestion_index += 1;
                        if suggestion_index >= suggestions.len() as i32 {
                            suggestion_index = 0;
                        }
                        show_suggestions_at_row(input_row, &suggestions, suggestion_index);
                        crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                    }
                }
                KEY_LEFT => {
                    if cursor > 0 {
                        cursor -= 1;
                        crate::print_framebuffer_only!("\x08");
                    }
                }
                KEY_RIGHT => {
                    if cursor < position {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                KEY_HOME => {
                    while cursor > 0 {
                        crate::print_framebuffer_only!("\x08");
                        cursor -= 1;
                    }
                }
                KEY_END => {
                    while cursor < position {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                KEY_DELETE => {
                    if cursor < position {
                        if showing_suggestions {
                            clear_suggestions_at_row(input_row, suggestions.len());
                            crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                        }
                        for i in cursor..position.saturating_sub(1) {
                            buffer[i] = buffer[i + 1];
                        }
                        position = position.saturating_sub(1);
                        for i in cursor..position {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::print_framebuffer_only!(" ");
                        for _ in cursor..=position {
                            crate::print_framebuffer_only!("\x08");
                        }
                        update_suggestions(buffer, position, &mut suggestions);
                        if !suggestions.is_empty() && position > 0 {
                            show_suggestions_at_row(input_row, &suggestions, suggestion_index);
                            showing_suggestions = true;
                            crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                        } else {
                            showing_suggestions = false;
                            suggestion_index = -1;
                        }
                    }
                }
                KEY_PGUP => {
                    crate::framebuffer::scroll_up_lines(10);
                }
                KEY_PGDOWN => {
                    crate::framebuffer::scroll_down(10);
                    // If we scrolled all the way back to live view, update input_row
                    // so the cursor stays on the correct row.
                    if !crate::framebuffer::is_scrolled_back() {
                        let (_, row) = crate::framebuffer::get_cursor();
                        input_row = row;
                        crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                    }
                }
                27 => {
                    // Escape - reset scroll to bottom (live view)
                    if crate::framebuffer::is_scrolled_back() {
                        let (_column, row) = crate::framebuffer::restore_live_view();
                        input_row = row;
                        // current_line already has the content; just position cursor
                        crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                    }
                }
                3 => {
                    // Ctrl+C — abort current input line
                    if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                        showing_suggestions = false;
                    }
                    crate::print_color!(COLOR_RED, "^C");
                    crate::println!();
                    set_interrupted();
                    position = 0;
                    break;
                }
                12 => {
                    // Ctrl+L - clear screen
                    if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                        showing_suggestions = false;
                    }
                    crate::framebuffer::clear();
                    print_prompt();
                    for i in 0..position {
                        crate::print!("{}", buffer[i] as char);
                    }
                    for _ in cursor..position {
                        crate::print_framebuffer_only!("\x08");
                    }
                }
                c if c >= 0x20 && c < 0x7F && position < buffer.len() - 1 => {
                    // Printable character
                    // First, restore cursor to input line if suggestions were showing
                    if showing_suggestions {
                        clear_suggestions_at_row(input_row, suggestions.len());
                        crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                    }
                    
                    if cursor < position {
                        for i in (cursor..position).rev() {
                            buffer[i + 1] = buffer[i];
                        }
                    }
                    buffer[cursor] = c;
                    position += 1;
                    cursor += 1;
                    for i in cursor - 1..position {
                        crate::print!("{}", buffer[i] as char);
                    }
                    for _ in cursor..position {
                        crate::print_framebuffer_only!("\x08");
                    }
                    
                    // Update and show suggestions if we have matches
                    update_suggestions(buffer, position, &mut suggestions);
                    if !suggestions.is_empty() {
                        show_suggestions_at_row(input_row, &suggestions, suggestion_index);
                        showing_suggestions = true;
                        // Restore cursor to input line
                        crate::framebuffer::set_cursor(input_column_start + cursor, input_row);
                    } else {
                        showing_suggestions = false;
                        suggestion_index = -1;
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
                    let under_cursor = if cursor < position { buffer[cursor] as char } else { ' ' };
                    crate::print_framebuffer_only!("{}", under_cursor);
                    crate::print_framebuffer_only!("\x08");
                }
            }
            // Poll network stack while idle (DHCP, packet RX, etc.)
            {
                                // Variable atomique — accès thread-safe sans verrou.
static POLL_DIVISOR: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                let count = POLL_DIVISOR.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                if count % 5000 == 0 {
                    crate::netstack::poll();
                }
                // Poll mesh network more frequently for RPC responsiveness
                if count % 100 == 0 && crate::jarvis::mesh::is_active() {
                    crate::jarvis::mesh_poll();
                }
                // Poll Jarvis mentor serial commands (learn from external AI)
                if count % 10000 == 0 {
                    crate::jarvis::mentor::poll_serial();
                }
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }
    }
    
    // Hide cursor before returning
    let under_cursor = if cursor < position { buffer[cursor] as char } else { ' ' };
    crate::print_framebuffer_only!("{}", under_cursor);
    if cursor < position { crate::print_framebuffer_only!("\x08"); }
    
    buffer[position] = 0;
    position
}

/// Update autocomplete suggestions based on current input
fn update_suggestions(buffer: &[u8], position: usize, suggestions: &mut Vec<&'static str>) {
    suggestions.clear();
    if position == 0 {
        return;
    }
    
    let input = // Correspondance de motifs — branchement exhaustif de Rust.
match core::str::from_utf8(&buffer[..position]) {
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

/// Display suggestions starting from the row below input.
/// Uses raw drawing to avoid corrupting the scrollback buffer.
fn show_suggestions_at_row(input_row: usize, suggestions: &[&str], selected_index: i32) {
    if suggestions.is_empty() {
        return;
    }
    
    // Screen bounds: don't render suggestions beyond the visible area
    let (_width, height) = crate::framebuffer::get_dimensions();
    let maximum_row = (height as usize) / 16; // CHAR_HEIGHT = 16
    let fg = crate::framebuffer::get_fg_color();
    let bg = 0xFF000000u32; // black background
    
    for (i, cmd) in suggestions.iter().enumerate() {
        let row = input_row + 1 + i;
        if row >= maximum_row { break; } // Stop rendering off-screen
        crate::framebuffer::clear_char_row(row);
        if i as i32 == selected_index {
            let prefix = alloc::format!(" > {}", cmd);
            crate::framebuffer::draw_text_raw(0, row, &prefix, fg, bg);
        } else {
            let prefix = alloc::format!("   {}", cmd);
            crate::framebuffer::draw_text_raw(0, row, &prefix, fg, bg);
        }
    }
}

/// Clear the suggestions display at given row.
/// Uses raw pixel clearing to avoid corrupting the scrollback buffer.
fn clear_suggestions_at_row(input_row: usize, count: usize) {
    let (_width, height) = crate::framebuffer::get_dimensions();
    let maximum_row = (height as usize) / 16; // CHAR_HEIGHT = 16
    for i in 0..count {
        let row = input_row + 1 + i;
        if row >= maximum_row { break; }
        crate::framebuffer::clear_char_row(row);
    }
}

/// Clear the current input line display (from input_col_start to end of input).
/// Uses raw pixel clearing for the character cells, then repositions the console cursor.
fn clear_line_display(input_column_start: usize, input_row: usize, position: usize) {
    let (width, _) = crate::framebuffer::get_dimensions();
    let cols = (width as usize) / 8; // CHAR_WIDTH = 8
    // Clear from input_col_start to end of typed text (plus some margin)
    let clear_length = (position + 2).minimum(cols.saturating_sub(input_column_start));
    // Draw spaces directly on the framebuffer (bypasses Writer/scrollback)
    let spaces: alloc::string::String = core::iter::repeat(' ').take(clear_length).collect();
    crate::framebuffer::draw_text_raw(input_column_start, input_row, &spaces, 0xFF000000, 0xFF000000);
    // Position cursor at start of input area
    crate::framebuffer::set_cursor(input_column_start, input_row);
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
        crate::ramfs::with_filesystem(|fs| String::from(fs.pwd()))
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
    let mut buffer = [0u8; 512];
    let len = crate::keyboard::read_line(&mut buffer);
    core::str::from_utf8(&buffer[..len]).unwrap_or("").into()
}

/// Execute a shell command (handles pipes and redirection)
pub fn execute_command(cmd: &str) {
    if cmd.is_empty() {
        return;
    }

    // ── Variable expansion ─────────────────────────────────────────
    let expanded = scripting::expand_variables(cmd);
    let cmd = expanded.as_str();
    
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
        let mut buffer = CAPTURE_BUFFER.lock();
        buffer.clear();
    }
    
    // Run the command (output goes to capture buffer via print macros)
    execute_single(cmd, piped_input);
    
    // Disable capture mode and return captured output
    CAPTURE_MODE.store(false, core::sync::atomic::Ordering::SeqCst);
    let buffer = CAPTURE_BUFFER.lock();
    buffer.clone()
}

/// Execute a single command, possibly with piped stdin
fn execute_single(cmd: &str, piped_input: Option<String>) {
    
    // Handle output redirection (skip > inside parentheses or quotes)
    let (command_part, redirect) = {
        let mut redir_position: Option<usize> = None;
        let mut paren_depth: i32 = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let bytes = cmd.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let character = bytes[i] as char;
                        // Correspondance de motifs — branchement exhaustif de Rust.
match character {
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                '(' if !in_single_quote && !in_double_quote => paren_depth += 1,
                ')' if !in_single_quote && !in_double_quote => {
                    if paren_depth > 0 { paren_depth -= 1; }
                }
                '>' if !in_single_quote && !in_double_quote && paren_depth == 0 => {
                    redir_position = Some(i);
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        if let Some(position) = redir_position {
            let append = cmd[position..].starts_with(">>");
            let file = if append {
                cmd[position + 2..].trim()
            } else {
                cmd[position + 1..].trim()
            };
            (cmd[..position].trim(), Some((file, append)))
        } else {
            (cmd, None)
        }
    };
    
    // Split into command and arguments
    let parts: Vec<&str> = command_part.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    
    // Check for alias expansion
    let command = parts[0];
    if let Some(alias_value) = unix::get_alias(command) {
        // Re-execute with expanded alias
        let new_command = if parts.len() > 1 {
            alloc::format!("{} {}", alias_value, parts[1..].join(" "))
        } else {
            alias_value
        };
        execute_command(&new_command);
        return;
    }
    let command = parts[0];
    let args = &parts[1..];
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match command {
        // -- commands module: Help, FS, System, Auth, Debug, Exit, Easter eggs --
        "help" => commands::command_help(args),
        "man" => commands::command_man(args),
        "info" => commands::command_information(),
        "version" => commands::command_version(),
        "uname" => commands::command_uname(args),
        "ls" | "dir" => commands::command_ls(args),
        "cd" => commands::command_cd(args),
        "pwd" => commands::command_pwd(),
        "mkdir" => commands::command_mkdir(args),
        "rmdir" => commands::command_rmdir(args),
        "touch" => commands::command_touch(args),
        "rm" | "del" => commands::command_rm(args),
        "cp" | "copy" => commands::command_cp(args),
        "mv" | "move" | "rename" => commands::command_mv(args),
        "cat" | "type" => commands::command_cat(args, redirect, piped_input.as_deref()),
        "head" => commands::command_head(args, piped_input.as_deref()),
        "tail" => commands::command_tail(args, piped_input.as_deref()),
        "wc" => commands::command_wc(args, piped_input.as_deref()),
        "stat" => commands::command_status(args),
        "tree" => commands::command_tree(args),
        "find" => commands::command_find(args),
        "echo" => commands::command_echo(args, redirect),
        "grep" => commands::command_grep(args, piped_input.as_deref()),
        "clear" | "cls" => commands::command_clear(),
        "time" | "uptime" => commands::command_time(),
        "date" => commands::command_date(),
        "whoami" => commands::command_whoami(),
        "hostname" => commands::command_hostname(),
        "id" => commands::command_id(),
        "env" | "printenv" => commands::command_env(),
        "history" => commands::command_history(),
        "ps" => commands::command_ps(),
        "free" => commands::command_free(),
        "df" => commands::command_df(),
        "login" => commands::command_login(),
        "su" => commands::command_su(args),
        "passwd" => commands::command_passwd(args),
        "adduser" | "useradd" => commands::command_adduser(args),
        "deluser" | "userdel" => commands::command_deluser(args),
        "users" => commands::command_users(),
        "hwtest" => commands::command_test(),
        "memtest" => commands::command_memtest(),
        "restest" => commands::command_restest(),
        "inttest" => commands::command_inttest(),
        "debugnew" => commands::command_debugnew(),
        "nvme" => commands::command_nvme(),
        "keytest" => commands::command_keytest(),
        "hexdump" | "xxd" => commands::command_hexdump(args),
        "panic" => commands::command_panic(),
        "exit" | "logout" => commands::command_logout(),
        "reboot" => commands::command_reboot(),
        "shutdown" | "halt" | "poweroff" => commands::command_halt(),
        "suspend" | "s3" => commands::command_sleep(),
        "neofetch" => commands::command_neofetch(),
        "matrix" => commands::command_matrix(),
        "rain" => {
            // rain [slow|mid|fast] — set matrix rain speed preset
            if args.is_empty() {
                let d = crate::desktop::DESKTOP.lock();
                let name = // Correspondance de motifs — branchement exhaustif de Rust.
match d.matrix_rain_preset { 0 => "slow", 2 => "fast", _ => "mid" };
                drop(d);
                crate::println!("Current rain preset: {}", name);
                crate::println!("Usage: rain <slow|mid|fast>");
            } else {
                let preset: u8 = // Correspondance de motifs — branchement exhaustif de Rust.
match args[0] {
                    "slow" | "s" | "0" => 0,
                    "mid" | "m" | "1" | "medium" => 1,
                    "fast" | "f" | "2" => 2,
                    _ => {
                        crate::println!("Unknown preset '{}'. Use: slow, mid, fast", args[0]);
                        return;
                    }
                };
                crate::desktop::DESKTOP.lock().set_rain_preset(preset);
                let name = // Correspondance de motifs — branchement exhaustif de Rust.
match preset { 0 => "slow", 2 => "fast", _ => "mid" };
                crate::println!("Rain speed set to: {}", name);
            }
        },
        "cowsay" => commands::command_cowsay(args),

        // -- desktop module: COSMIC, Showcase, Benchmark, Signature, Security --
        "benchmark" | "bench" => desktop::command_benchmark(args),
        "showcase" => desktop::command_showcase(args),
        "showcase-jarvis" | "jarvis-showcase" | "jdemo" => desktop::command_showcase_jarvis(args),
        "showcase3d" | "demo3d" => desktop::command_showcase3d(),
        "demo" | "tutorial" | "tour" => desktop::command_demo(args),
        "filled3d" => desktop::command_filled3d(),
        "desktop" | "gui" => desktop::launch_desktop_env(None),
        "mobile" => desktop::launch_mobile_env(),
        "cosmic" => desktop::command_cosmic_v2(),
        "open" => desktop::command_open(args),
        "trustedit" | "edit3d" | "3dedit" => desktop::launch_desktop_env(Some(("TrustEdit 3D", crate::desktop::WindowType::ModelEditor, 100, 60, 700, 500))),
        "calculator" | "calc" => desktop::launch_desktop_env(Some(("Calculator", crate::desktop::WindowType::Calculator, 300, 200, 320, 420))),
        "snake" => desktop::launch_desktop_env(Some(("Snake", crate::desktop::WindowType::Game, 200, 100, 400, 400))),
        "signature" | "sig" => desktop::command_signature(args),
        "security" | "sec" | "caps" => desktop::command_security(args),

        // -- vm module: VM, Linux, Distro, Alpine, Disk, Hardware, Network core --
        "vm" | "linux" => {
            if args.is_empty() {
                vm::command_linux_shell();
            } else {
                                // Correspondance de motifs — branchement exhaustif de Rust.
match args[0] {
                    // Real hypervisor VM commands
                    "create" | "run" | "start" | "guests" | "inspect" | "mount" | "input"
                    | "debug" | "stack" | "regs" | "dump" | "linux" => vm::command_vm(args),
                    "status" => vm::command_gui_status(),
                    "install" => vm::command_gui_install(),
                    "console" | "shell" => vm::command_linux_shell(),
                    "stop" => vm::command_vm_stop(),
                    "list" => vm::command_vm_list(),
                    "extract" => apps::create_test_binaries(),
                    "exec" => {
                        if args.len() > 1 {
                            let binary = args[1];
                            let bin_args: Vec<&str> = args[2..].to_vec();
                                                        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::linux_compat::exec(binary, &bin_args) {
                                Ok(code) => crate::println!("[Exited with code {}]", code),
                                Err(e) => crate::println_color!(0xFF0000, "Error: {}", e),
                            }
                        } else {
                            crate::println!("Usage: linux exec <binary> [args...]");
                            crate::println!("Example: linux exec /bin/busybox ls");
                        }
                    },
                    "help" | "--help" | "-h" => vm::command_vm_help(),
                    _ => vm::command_vm_help(),
                }
            }
        },
        "distro" | "distros" => {
            if args.is_empty() {
                vm::command_distro_list();
            } else {
                                // Correspondance de motifs — branchement exhaustif de Rust.
match args[0] {
                    "list" => vm::command_distro_list(),
                    "install" | "download" => {
                        if args.len() > 1 { vm::command_distro_install(args[1]); }
                        else { vm::command_distro_gui(); }
                    },
                    "run" | "start" => {
                        if args.len() > 1 { vm::command_distro_run(args[1]); }
                        else { crate::println!("Usage: distro run <id>"); }
                    },
                    "pick" | "select" => vm::command_distro_gui(),
                    _ => vm::command_distro_list(),
                }
            }
        },
        "glmode" | "compositor" => vm::command_glmode(args),
        "theme" => vm::command_theme(args),
        "anim" | "animations" => vm::command_animations(args),
        "holo" | "holomatrix" => vm::command_holomatrix(args),
        "imgview" | "imageview" | "view" => vm::command_imgview(args),
        "imgdemo" | "imagedemo" => vm::command_imgdemo(args),
        "tasks" | "jobs" => vm::command_tasks(),
        "threads" => vm::command_threads(),
        "alpine" => vm::command_alpine(args),
        "apt-get" | "apt" | "apk" | "dpkg" => vm::command_pkg(command, args),
        "persist" | "persistence" => vm::command_persistence(args),
        "disk" => vm::command_disk(),
        "dd" => vm::command_dd(args),
        "ahci" => vm::command_ahci(args),
        "fdisk" | "partitions" => vm::command_fdisk(args),
        "lspci" => vm::command_lspci(args),
        "lshw" | "hwinfo" => vm::command_lshw(),
        "gpu" => vm::command_gpu(args),
        "gpuexec" | "gpurun" | "gpuagent" => commands::command_gpuexec(args),
        "sdma" | "dma" => commands::command_sdma(args),
        "neural" | "nn" | "gemm" => commands::command_neural(args),
        "gpufw" | "firmware" => commands::command_gpufw(args),
        "a11y" | "accessibility" => vm::command_a11y(args),
        "beep" => vm::command_beep(args),
        "audio" => vm::command_audio(args),
        "synth" => vm::command_synth(args),
        "play" => vm::command_play(args),
        "vizfx" | "liveviz" => vm::command_vizfx(args),
        "daw" | "trustdaw" => vm::command_daw(args),
        "ifconfig" | "ip" => vm::command_ifconfig(),
        "ipconfig" => vm::command_ipconfig(args),
        "ping" => vm::command_ping(args),
        "tcpsyn" => vm::command_tcpsyn(args),
        "httpget" => vm::command_httpget(args),
        "curl" | "wget" => vm::command_curl(args),
        "download" => vm::command_download(args),
        "nslookup" | "dig" => vm::command_nslookup(args),
        "arp" => vm::command_arp(args),
        "route" => vm::command_route(args),
        "traceroute" | "tracert" => vm::command_traceroute_real(args),
        "netstat" => vm::command_netstat(),
        "exec" | "run" | "./" => vm::command_execute(args, command),
        "elfinfo" => vm::command_elfinfo(args),
        "lsusb" => unix::command_lsusb(),
        "checkm8" => {
            let argument_str = args.join(" ");
            let result = crate::drivers::checkm8::run_exploit(&argument_str);
            crate::println!("{}", result);
        }
        "lscpu" => unix::command_lscpu(),
        "smpstatus" => unix::command_smpstatus(),
        "smp" => unix::command_smp(args),
        "fontsmooth" => unix::command_fontsmooth(args),
        "hv" | "hypervisor" => vm::command_hypervisor(args),

        // -- Security Toolkit: TrustScan --
        "nmap" | "portscan" | "scan" => vm::command_nmap(args),
        "discover" | "hostscan" | "arpscan" => vm::command_discover(args),
        "banner" | "grabber" => vm::command_banner(args),
        "sniff" | "capture" | "tcpdump" => vm::command_sniff(args),
        "vulnscan" | "vuln" => vm::command_vulnscan(args),
        "scantest" | "netscantest" => vm::command_netscan_test(args),

        // -- HTTP Server --
        "httpd" | "httpserv" | "webserv" => commands::command_httpd(args),

        // -- Package Manager --
        "trustpkg" | "pkg" => commands::command_trustpkg(args),

        // -- network module: Browser, Sandbox, Container --
        "browse" | "www" | "web" => network::command_browse(args),
        "sandbox" | "websandbox" => network::command_sandbox(args),
        "container" | "webcontainer" | "wc" => network::command_container(args),

        // -- unix module: Unix utilities and stubs --
        "which" => unix::command_which(args),
        "whereis" => unix::command_whereis(args),
        "file" => unix::command_file(args),
        "basename" => unix::command_basename(args),
        "dirname" => unix::command_dirname(args),
        "realpath" => unix::command_realpath(args),
        "sort" => unix::command_sort(args, piped_input.as_deref()),
        "uniq" => unix::command_uniq(args, piped_input.as_deref()),

        // -- Text Editor --
        "nano" | "vi" | "edit" => editor::command_nano(args),

        // -- Newly implemented commands (formerly stubs) --
        "alias" => unix::command_alias(args),
        "unalias" => unix::command_unalias(args),
        "bc" => unix::command_bc(args),
        "diff" => unix::command_diff(args),
        "md5sum" => unix::command_md5sum(args),
        "sha256sum" => unix::command_sha256sum(args),
        "base64" => unix::command_base64(args, piped_input.as_deref()),
        "cut" => unix::command_cut(args, piped_input.as_deref()),
        "tr" => unix::command_tr(args, piped_input.as_deref()),
        "tee" => unix::command_tee(args, piped_input.as_deref()),
        "xargs" => unix::command_xargs(args, piped_input.as_deref()),
        "chmod" => unix::command_chmod(args),
        "chown" => unix::command_chown(args),
        "ln" => unix::command_line(args),
        "readlink" => unix::command_readlink(args),
        "watch" => unix::command_watch(args),
        "timeout" => unix::command_timeout(args),
        "tar" => unix::command_tar(args),
        "gzip" => unix::command_gzip(args),
        "zip" => unix::command_zip(args),
        "unzip" => unix::command_unzip(args),
        "service" => unix::command_service(args),
        "systemctl" => unix::command_systemctl(args),
        "crontab" => unix::command_crontab(args),
        "at" => unix::command_at(args),
        "unset" => unix::command_unset(args),
        "read" => unix::command_read(args),

        "yes" => unix::command_yes(args),
        "seq" => unix::command_sequence(args),
        "sleep" => unix::command_sleep(args),
        "kill" => unix::command_kill(args),
        "killall" => unix::command_killall(args),
        "nice" => unix::command_nice(args),

        "top" => unix::command_top(),
        "htop" => unix::command_top(),
        "vmstat" => unix::command_vmstat(),
        "iostat" => unix::command_iostat(),
        "strace" => unix::command_strace(args),
        "dmidecode" => unix::command_dmidecode(),
        "hdparm" => unix::command_hdparm(args),
        "screenshot" | "scrot" => unix::command_screenshot(args),
        "httpd" | "serve" => unix::command_httpd(args),
        "benchmark" | "bench" => unix::command_benchmark(),
        "uptime" => unix::command_uptime_full(),

        "lsof" => unix::command_lsof(args),

        "strings" => unix::command_strings(args),

        "mount" => unix::command_mount(args),
        "umount" => unix::command_umount(args),
        "fsck" => unix::command_fsck(args),

        "sync" => unix::command_sync(),
        "lsblk" => unix::command_lsblk(),
        "blkid" => unix::command_blkid(),

        "export" => unix::command_export(args),

        "source" | "." => unix::command_source(args),
        "set" => unix::command_set(args),

        "printf" => unix::command_printf(args),
        "test" | "[" => unix::command_test_expr(args),
        "expr" => unix::command_expr(args),

        "cal" => unix::command_cal(args),

        "cmp" => unix::command_cmp(args),

        "od" => unix::command_od(args),
        "rev" => unix::command_rev(args),
        "factor" => unix::command_factor(args),

        "tty" => unix::command_tty(),
        "stty" => unix::command_stty(args),
        "reset" => unix::command_reset(),

        "lsmem" => unix::command_lsmem(),

        "lsmod" => unix::command_lsmod(),

        "sysctl" => unix::command_sysctl(args),
        "firewall" | "iptables" | "fw" => unix::command_firewall(args),
        "du" => unix::command_du(args),

        "dmesg" => unix::command_dmesg(args),
        "memdbg" | "heapdbg" => unix::command_memdbg(),
        "perf" | "perfstat" => unix::command_perfstat(),
        "irqstat" | "irqs" => unix::command_irqstat(),
        "regs" | "registers" | "cpuregs" => unix::command_registers(),
        "peek" | "memdump" => unix::command_peek(args),
        "poke" | "memwrite" => unix::command_poke(args),
        "devpanel" => unix::command_devpanel(),
        "timecmd" => unix::command_timecmd(args),

        // -- Hardware Debug Toolkit --
        "hwdiag" | "diagnostic" | "diag" => unix::command_hwdiag(),
        "cpudump" | "fullregs" => unix::command_cpudump(),
        "stacktrace" | "backtrace" | "bt" => unix::command_stacktrace(args),
        "bootlog" | "checkpoints" => unix::command_bootlog(),
        "postcode" => unix::command_postcode(args),
        "ioport" => unix::command_ioport(args),
        "rdmsr" => unix::command_rdmsr(args),
        "wrmsr" => unix::command_wrmsr(args),
        "cpuid" => unix::command_cpuid(args),
        "memmap" => unix::command_memmap(),
        "watchdog" | "wdt" => unix::command_watchdog(args),

        // -- ThinkPad EC: Fan, Thermal, CPU Frequency --
        "fan" => crate::drivers::thinkpad_ec::command_fan(args),
        "temp" | "sensors" => crate::drivers::thinkpad_ec::command_temporary(args),
        "cpufreq" | "speedstep" => crate::drivers::thinkpad_ec::command_cpufreq(args),

        // -- apps module: TrustLang, Film, Transpile, Video, Lab, Gterm, Wayland --
        "wayland" | "wl" => apps::command_wayland(args),
        "gterm" | "graphterm" => apps::command_gterm(args),
        "transpile" | "disasm" | "analyze" => apps::command_transpile(args),
        "rv-xlat" | "rvxlat" | "xlat" => apps::command_rv_xlat(args),
        "rv-disasm" | "rvdisasm" => apps::command_rv_disasm(args),
        "trustview" | "tv" => apps::command_trustview(args),
        "lab" | "trustlab" => apps::command_lab(args),
        "hwscan" | "trustprobe" | "probe" => apps::command_hwscan(args),
        "trustlang" | "tl" => apps::command_trustlang(args),
        "trustlang_showcase" | "tl_showcase" => apps::command_trustlang_showcase(),
        "film" | "trustos_film" => apps::command_trustos_film(),
        "trailer" | "trustos_trailer" => trailer::command_trustos_trailer(),
        "video" => apps::command_video(args),

        // -- Jarvis AI assistant --
        "jarvis" | "j" | "ai" | "assistant" => jarvis::command_jarvis(args),
        "mesh" | "jarvis-mesh" | "jmesh" => commands::command_mesh(args),
        "pxe" | "pxeboot" | "replicate" => commands::command_pxe(args),
        "guardian" | "pact" | "gardien" => commands::command_guardian(args),

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
mod unix;           // Unix utility stubs and POSIX commands             (~2400 lines)
mod apps;           // TrustLang, Film, Transpile, Video, Lab, Gterm    (~3930 lines)
mod trailer;        // TrustOS Trailer -- 2-min cinematic showcase         (~900 lines)
mod jarvis;         // Jarvis AI assistant — NLU + planner + executor     (~600 lines)
pub(crate) mod scripting;  // Shell scripting engine — variables, if/for/while    (~640 lines)
mod editor;         // TrustEdit — nano-like terminal text editor          (~520 lines)
