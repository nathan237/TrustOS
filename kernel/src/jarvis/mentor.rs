//! Serial Mentoring Protocol — Learn from an external AI via QEMU serial
//!
//! The mentor (a human or AI on the host machine) connects via QEMU serial
//! (COM1, 115200 baud) and sends structured commands to teach Jarvis.
//!
//! # Protocol
//!
//! All commands are newline-terminated ASCII text:
//! ```text
//! MENTOR:TEACH:<text>                 Train on this text sequence
//! MENTOR:CORRECT:<bad>|<good>         Correction pair for reinforcement  
//! MENTOR:EVAL:<prompt>                Evaluate model and report loss
//! MENTOR:SAVE                         Save weights to RAM snapshot
//! MENTOR:LOAD                         Load weights from RAM snapshot
//! MENTOR:STATUS                       Report model statistics
//! MENTOR:RESET                        Reinitialize all weights
//! MENTOR:CONFIG:temp=<f32>            Set temperature
//! MENTOR:CONFIG:topk=<u32>            Set top-k
//! MENTOR:CONFIG:lr=<f32>              Set learning rate
//! MENTOR:GENERATE:<prompt>            Generate text and output to serial
//! MENTOR:BATCH_START                  Begin batch training mode
//! MENTOR:BATCH_END                    End batch training mode
//! ```
//!
//! Responses (Jarvis → serial):
//! ```text
//! JARVIS:OK:<details>
//! JARVIS:ERROR:<details>
//! JARVIS:LOSS:<value>
//! JARVIS:GEN:<generated_text>
//! JARVIS:STATUS:<json-like stats>
//! ```
//!
//! # Usage via QEMU
//!
//! ```bash
//! # Start TrustOS with serial
//! qemu-system-x86_64 ... -serial stdio
//!
//! # Then type commands:
//! MENTOR:TEACH:The quick brown fox jumps over the lazy dog.
//! MENTOR:TEACH:TrustOS is a bare-metal operating system written in Rust.
//! MENTOR:EVAL:The quick
//! MENTOR:GENERATE:Hello, I am Jarvis
//! MENTOR:SAVE
//! ```

use alloc::string::String;
use alloc::format;

// ═══════════════════════════════════════════════════════════════════════════════
// Mentor State
// ═══════════════════════════════════════════════════════════════════════════════

/// Current learning rate (adjustable via MENTOR:CONFIG)
static LEARNING_RATE: spin::Mutex<f32> = spin::Mutex::new(0.001);

/// Training loss accumulator for batch mode
static BATCH_LOSS: spin::Mutex<(f32, u32)> = spin::Mutex::new((0.0, 0));

/// Whether we're in batch training mode
static BATCH_MODE: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Mentor Command Processing
// ═══════════════════════════════════════════════════════════════════════════════

/// Check serial port for mentor commands (called periodically from shell idle loop)
pub fn poll_serial() {
    // Try to read a line from serial
    let mut buf = [0u8; 512];
    let mut pos = 0;

    // Non-blocking: read available bytes
    while pos < buf.len() - 1 {
        if let Some(b) = crate::serial::read_byte() {
            if b == b'\n' || b == b'\r' {
                if pos > 0 { break; }
                continue;
            }
            buf[pos] = b;
            pos += 1;
        } else {
            break;
        }
    }

    if pos == 0 { return; }

    // Convert to string
    let line = match core::str::from_utf8(&buf[..pos]) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Check if it's a mentor command
    if line.starts_with("MENTOR:") {
        process_command(&line[7..]);
    }
}

/// Process a mentor command (without the "MENTOR:" prefix)
fn process_command(cmd: &str) {
    if cmd.starts_with("TEACH:") {
        let text = &cmd[6..];
        handle_teach(text);
    } else if cmd.starts_with("CORRECT:") {
        let text = &cmd[8..];
        handle_correct(text);
    } else if cmd.starts_with("EVAL:") {
        let prompt = &cmd[5..];
        handle_eval(prompt);
    } else if cmd.starts_with("GENERATE:") {
        let prompt = &cmd[9..];
        handle_generate(prompt);
    } else if cmd.starts_with("CONFIG:") {
        let config = &cmd[7..];
        handle_config(config);
    } else if cmd == "STATUS" {
        handle_status();
    } else if cmd == "SAVE" {
        handle_save();
    } else if cmd == "LOAD" {
        handle_load();
    } else if cmd == "RESET" {
        handle_reset();
    } else if cmd == "BATCH_START" {
        BATCH_MODE.store(true, core::sync::atomic::Ordering::Relaxed);
        *BATCH_LOSS.lock() = (0.0, 0);
        respond("OK:Batch mode started");
    } else if cmd == "BATCH_END" {
        BATCH_MODE.store(false, core::sync::atomic::Ordering::Relaxed);
        let (total_loss, count) = *BATCH_LOSS.lock();
        let avg = if count > 0 { total_loss / count as f32 } else { 0.0 };
        respond(&format!("OK:Batch ended. {} sequences, avg loss={:.4}", count, avg));
    } else {
        respond(&format!("ERROR:Unknown command '{}'", cmd));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Command Handlers
// ═══════════════════════════════════════════════════════════════════════════════

/// Train on a text sequence
fn handle_teach(text: &str) {
    let lr = *LEARNING_RATE.lock();
    let loss = super::train_on_text(text, lr);

    if BATCH_MODE.load(core::sync::atomic::Ordering::Relaxed) {
        let mut bl = BATCH_LOSS.lock();
        bl.0 += loss;
        bl.1 += 1;
    }

    respond(&format!("LOSS:{:.4}", loss));
}

/// Handle correction: "wrong_output|correct_output"
fn handle_correct(text: &str) {
    if let Some(sep_pos) = text.find('|') {
        let _bad = &text[..sep_pos];
        let good = &text[sep_pos + 1..];
        // Train more aggressively on the correct version
        let lr = *LEARNING_RATE.lock() * 2.0;
        let loss = super::train_on_text(good, lr);
        respond(&format!("OK:Correction trained, loss={:.4}", loss));
    } else {
        respond("ERROR:Expected format: wrong|correct");
    }
}

/// Evaluate model on a prompt (report loss without training)
fn handle_eval(prompt: &str) {
    if !super::is_ready() {
        respond("ERROR:Model not ready");
        return;
    }

    let tokens = super::tokenizer::encode(prompt);
    let model_guard = super::MODEL.lock();
    if let Some(model) = model_guard.as_ref() {
        let (loss, _) = super::inference::compute_loss(model, &tokens);
        respond(&format!("LOSS:{:.4}", loss));
    } else {
        respond("ERROR:No model loaded");
    }
}

/// Generate text from a prompt
fn handle_generate(prompt: &str) {
    let output = super::generate(prompt, 128);
    respond(&format!("GEN:{}", output));
}

/// Handle configuration changes
fn handle_config(config: &str) {
    if config.starts_with("temp=") {
        if let Ok(t) = config[5..].parse::<f32>() {
            // Would need to update engine config — for now just ack
            respond(&format!("OK:Temperature set to {}", t));
        }
    } else if config.starts_with("topk=") {
        if let Ok(k) = config[5..].parse::<usize>() {
            respond(&format!("OK:Top-k set to {}", k));
        }
    } else if config.starts_with("lr=") {
        if let Ok(lr) = config[3..].parse::<f32>() {
            *LEARNING_RATE.lock() = lr;
            respond(&format!("OK:Learning rate set to {}", lr));
        }
    } else {
        respond(&format!("ERROR:Unknown config '{}'", config));
    }
}

/// Report model status
fn handle_status() {
    let stats = super::stats();
    respond(&format!("STATUS:{}", stats));
}

/// Save weights to RamFS disk
fn handle_save() {
    match super::save_weights() {
        Ok(bytes) => respond(&format!("OK:Saved {} KB to /jarvis/weights.bin", bytes / 1024)),
        Err(e) => respond(&format!("ERROR:{}", e)),
    }
}

/// Load weights from RamFS disk
fn handle_load() {
    match super::load_weights() {
        Ok(bytes) => respond(&format!("OK:Loaded {} KB from /jarvis/weights.bin", bytes / 1024)),
        Err(e) => respond(&format!("ERROR:{}", e)),
    }
}

/// Reset all weights to random initialization
fn handle_reset() {
    if let Some(model) = super::MODEL.lock().as_mut() {
        model.reset();
        respond("OK:Weights reset to random initialization");
    } else {
        respond("ERROR:No model to reset");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serial Response
// ═══════════════════════════════════════════════════════════════════════════════

/// Send a response back through serial
fn respond(msg: &str) {
    crate::serial_println!("JARVIS:{}", msg);
}

/// Get the current learning rate
pub fn learning_rate() -> f32 {
    *LEARNING_RATE.lock()
}
