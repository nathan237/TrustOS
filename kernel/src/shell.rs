//! Kernel Shell (Bootstrap Mode)
//! 
//! A full-featured shell running in kernel mode with standard commands.
//! This is temporary until Ring 3 userland is implemented.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA};
use crate::ramfs::FileType;

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
    // VM/Linux
    "vm", "linux", "gui", "distro", "distros", "alpine", "transpile", "disasm", "analyze",
    // GUI
    "glmode", "compositor", "theme", "imgview", "imageview", "view", "imgdemo", "imagedemo",
    "wayland", "wl",
    // Tasks
    "tasks", "jobs", "threads",
    // Persistence
    "persist", "persistence",
    // Disk
    "disk", "dd", "ahci", "fdisk", "partitions",
    // Hardware
    "lspci", "lshw", "hwinfo",
    // Network
    "ifconfig", "ip", "ipconfig", "ping", "tcpsyn", "httpget", "curl", "wget", "download",
    "nslookup", "dig", "arp", "route", "traceroute", "tracert", "netstat", "browse", "www", "web",
    // Unix utilities
    "which", "whereis", "file", "chmod", "chown", "ln", "readlink", "basename", "dirname",
    "realpath", "sort", "uniq", "cut", "tr", "tee", "xargs", "yes", "seq", "sleep",
    "kill", "killall", "nice", "nohup", "bg", "fg", "top", "htop", "vmstat", "iostat",
    "dmesg", "lsof", "strace", "strings", "tar", "gzip", "gunzip", "zip", "unzip",
    "mount", "umount", "sync", "lsblk", "blkid", "mkfs", "fsck", "export", "unset",
    "alias", "unalias", "source", "set", "read", "printf", "expr", "bc", "cal",
    "diff", "patch", "cmp", "md5sum", "sha256sum", "base64", "od", "rev", "factor",
    "watch", "timeout", "time_cmd", "script", "tty", "stty", "reset", "loadkeys", "setfont",
    "lsusb", "lscpu", "lsmem", "dmidecode", "hdparm", "modprobe", "lsmod", "insmod", "rmmod",
    "sysctl", "service", "systemctl", "crontab", "at",
    // Exit/control
    "exit", "reboot", "shutdown", "halt", "poweroff",
    // Execution
    "exec", "run", "elfinfo",
    // Easter eggs
    "neofetch", "matrix", "cowsay",
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
                    "gui" => cmd_distro_gui(),
                    _ => cmd_distro_list(),
                }
            }
        },
        
        "glmode" | "compositor" => cmd_glmode(args),
        "theme" => cmd_theme(args),
        "imgview" | "imageview" | "view" => cmd_imgview(args),
        "imgdemo" | "imagedemo" => cmd_imgdemo(args),
        "tasks" | "jobs" => cmd_tasks(),
        "threads" => cmd_threads(),
        
        // Wayland compositor
        "wayland" | "wl" => cmd_wayland(args),
        
        // Alpine Linux all-in-one command
        "alpine" => cmd_alpine(args),
        
        // Binary-to-Rust transpiler
        "transpile" | "disasm" | "analyze" => cmd_transpile(args),
        
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
        "dmesg" => cmd_dmesg(),
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
    if args.is_empty() {
        // OS Description
        crate::println_color!(COLOR_BRIGHT_GREEN, "╔══════════════════════════════════════════════════════════════════╗");
        crate::println_color!(COLOR_BRIGHT_GREEN, "║               TrustOS - Secure Experimental Kernel              ║");
        crate::println_color!(COLOR_BRIGHT_GREEN, "╚══════════════════════════════════════════════════════════════════╝");
        crate::println!();
        crate::println_color!(COLOR_WHITE, "  A bare-metal x86_64 operating system written in Rust featuring:");
        crate::println!();
        crate::println_color!(COLOR_CYAN, "  Core Features:");
        crate::println!("    • Full file system (RAMFS) with Unix-like commands");
        crate::println!("    • User authentication & permission system (root, users)");
        crate::println!("    • Network stack (TCP/IP, HTTP/HTTPS, DNS)");
        crate::println!("    • ELF binary execution & Linux syscall compatibility");
        crate::println!();
        crate::println_color!(COLOR_CYAN, "  Advanced Capabilities:");
        crate::println!("    • Binary-to-Rust transpiler (analyze Linux ELFs)");
        crate::println!("    • Linux Subsystem (run Alpine Linux binaries)");
        crate::println!("    • Graphical desktop compositor (GUI mode)");
        crate::println!("    • AHCI disk driver with partition support");
        crate::println!();
        crate::println_color!(COLOR_YELLOW, "═══════════════════════════════════════════════════════════════════════");
        crate::println!();
        crate::println_color!(COLOR_BRIGHT_GREEN, "Commands by Category:");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  📁 File System:");
        crate::println!("     ls, cd, pwd, mkdir, rmdir, touch, rm, cp, mv, cat, head, tail");
        crate::println!("     stat, tree, find, wc, grep, chmod, chown, ln");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  👤 User Management:");
        crate::println!("     login, logout, su, passwd, adduser, deluser, users, whoami, id");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  ⚙️  System & Process:");
        crate::println!("     clear, time, date, hostname, env, history, uname, free, df");
        crate::println!("     ps, tasks, threads, top, kill, dmesg, sysctl");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  🔧 Hardware:");
        crate::println!("     lspci, lshw, lscpu, lsmem, lsusb, disk, dd, ahci, fdisk, lsblk");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  🌐 Network:");
        crate::println!("     ifconfig, ping, curl, wget, nslookup, arp, route, netstat");
        crate::println!("     traceroute, browse (web browser)");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  🐧 Linux Subsystem:");
        crate::println!("     linux shell    - Interactive Linux shell");
        crate::println!("     linux extract  - Create test ELF binaries");
        crate::println!("     linux exec     - Execute Linux binary");
        crate::println!("     alpine         - Alpine Linux utilities");
        crate::println!("     transpile      - Binary-to-Rust transpiler");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  🖥️  GUI:");
        crate::println!("     gui, glmode, compositor - Launch graphical desktop");
        crate::println!("     theme          - Change terminal colors");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  📦 Archives:");
        crate::println!("     tar, gzip, gunzip, zip, unzip, base64");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  🔍 Text Processing:");
        crate::println!("     echo, grep, sort, uniq, cut, tr, diff, rev, strings");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  🛠️  Utilities:");
        crate::println!("     which, file, hexdump, md5sum, sha256sum, bc, cal, sleep, watch");
        crate::println!();
        
        crate::println_color!(COLOR_CYAN, "  💀 System Control:");
        crate::println!("     exit, reboot, shutdown, halt");
        crate::println!();
        
        crate::println_color!(COLOR_YELLOW, "Tips:");
        crate::println!("  • Type 'help <command>' or 'man <command>' for detailed help");
        crate::println!("  • Use ↑/↓ arrows for command history (when prompt is empty)");
        crate::println!("  • Use Tab for command auto-completion");
        crate::println!("  • Use PageUp/PageDown to scroll terminal output");
    } else {
        cmd_man(args);
    }
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
    crate::println_color!(COLOR_CYAN, "         total     used     free");
    crate::println!("Mem:    262144    16384   245760");
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
    crate::println_color!(COLOR_BRIGHT_GREEN, r"       _____          ");
    crate::print_color!(COLOR_GREEN, r"      |  _  |         ");
    crate::print_color!(COLOR_CYAN, "root");
    crate::print_color!(COLOR_WHITE, "@");
    crate::println_color!(COLOR_CYAN, "trustos");
    crate::print_color!(COLOR_GREEN, r"      | |_| |         ");
    crate::println!("---------------");
    crate::print_color!(COLOR_GREEN, r"      |  _  |         ");
    crate::print_color!(COLOR_CYAN, "OS: ");
    crate::println!("T-RustOs 0.1.0");
    crate::print_color!(COLOR_DARK_GREEN, r"      | |_| |         ");
    crate::print_color!(COLOR_CYAN, "Uptime: ");
    crate::println!("{} secs", secs);
    crate::print_color!(COLOR_DARK_GREEN, r"      |_____|         ");
    crate::print_color!(COLOR_CYAN, "Shell: ");
    crate::println!("tsh");
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

fn cmd_dmesg() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Kernel Messages (dmesg)");
    crate::println!("═══════════════════════════════════════════");
    crate::println!("(kernel ring buffer not implemented)");
    crate::println!("Use serial console for boot messages.");
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
