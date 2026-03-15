//! TrustLab UX Auto-Test — Automated usability testing for TrustLab
//!
//! Runs a sequence of simulated user interactions and checks that the
//! Lab UI responds correctly. Reports pass/fail for each test step.
//! Triggered by the `labtest` shell command from within TrustLab.
//!
//! Tests:
//!  1. Tab cycling visits all 7 panels
//!  2. Click dispatches to all panels
//!  3. Shell commands focus correct panels
//!  4. Kernel Trace shows events and supports filtering
//!  5. Event detail view works (click to select)
//!  6. Pipeline receives and displays events
//!  7. Hardware state updates with live data

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{LabState, PanelId, trace_bus};

/// Result of a single UX test
struct TestResult {
    name: &'static str,
    passed: bool,
    detail: String,
}

/// Run all UX tests and return a report as Vec of trace events
pub fn run_ux_tests(state: &mut LabState) {
    let mut results: Vec<TestResult> = Vec::new();

    // ── Test 1: Tab cycling visits all slots ──────────────────────
    {
        state.focused_slot = 0; // Start at slot 0
        // All 7 slots are now cycled (no skip)
        let expected_slots: [usize; 7] = [1, 2, 3, 4, 5, 6, 0];
        let mut ok = true;
        let mut detail = String::new();
        for (i, exp) in expected_slots.iter().enumerate() {
            state.handle_key(0x09); // Tab
            if state.focused_slot != *exp {
                ok = false;
                detail = format!("step {}: expected slot {}, got slot {}", i, exp, state.focused_slot);
                break;
            }
        }
        if ok { detail = String::from("7 tab presses cycle all slots correctly"); }
        results.push(TestResult { name: "Tab cycle", passed: ok, detail });
    }

    // ── Test 2: Shell commands focus correct panels ────────────────
    {
        let cmds: [(&str, PanelId); 7] = [
            ("hw", PanelId::HardwareStatus),
            ("trace", PanelId::KernelTrace),
            ("help", PanelId::CommandGuide),
            ("fs", PanelId::FileTree),
            ("edit", PanelId::TrustLangEditor),
            ("pipeline", PanelId::Pipeline),
            ("hex", PanelId::HexEditor),
        ];
        let mut ok = true;
        let mut detail = String::new();
        for (cmd, expected) in &cmds {
            state.shell_input = String::from(*cmd);
            state.shell_cursor = cmd.len();
            state.execute_shell_command();
            if state.focused_module() != *expected {
                ok = false;
                detail = format!("'{}' => {:?}, expected {:?}", cmd, state.focused_module(), expected);
                break;
            }
        }
        if ok { detail = String::from("7 shell commands focus correct panels"); }
        results.push(TestResult { name: "Shell cmds", passed: ok, detail });
    }

    // ── Test 3: Click dispatches set focus ─────────────────────────
    {
        // Simulate clicks in the center of each panel area
        // We use a reasonable window size (1200×800)
        let ww = 1200u32;
        let wh = 800u32;
        let cx = 2i32;
        let cy = 30i32; // TITLE_BAR_HEIGHT + 2
        let cw = ww - 4;
        let ch = wh - 32;

        let panels = super::compute_panels(cx, cy, cw, ch);
        let mut ok = true;
        let mut detail = String::new();
        for (i, pr) in panels.iter().enumerate() {
            let click_x = pr.x + pr.w as i32 / 2;
            let click_y = pr.y + pr.h as i32 / 2;
            state.handle_click(click_x, click_y, ww, wh);
            if state.focused_slot != i {
                ok = false;
                detail = format!("click slot {} => focused_slot {}, expected {}", i, state.focused_slot, i);
                break;
            }
        }
        if ok { detail = String::from("clicks on all 7 slot areas set focus correctly"); }
        results.push(TestResult { name: "Click focus", passed: ok, detail });
    }

    // ── Test 4: Kernel trace receives events ───────────────────────
    {
        // Emit a test event and check it shows up
        trace_bus::emit_static(trace_bus::EventCategory::Custom, "ux_test_marker", 42);
        state.trace_state.update();
        state.trace_state.update(); // force update (bypass counter)
        // Check last event
        let has_marker = state.trace_state.events.iter()
            .any(|e| e.message.contains("ux_test_marker"));
        let detail = if has_marker {
            String::from("test event propagated to trace panel")
        } else {
            String::from("test event NOT found in trace panel (may need more ticks)")
        };
        results.push(TestResult { name: "Trace recv", passed: has_marker, detail });
    }

    // ── Test 5: Syscall tracing emits structured data ──────────────
    {
        // Emit a synthetic syscall event
        trace_bus::emit_syscall(1, [1, 0x1000, 32], 32);
        state.trace_state.update();
        let has_syscall = state.trace_state.events.iter()
            .any(|e| e.syscall_nr == Some(1));
        let detail = if has_syscall {
            String::from("syscall event with nr=1 (write) found with structured data")
        } else {
            String::from("syscall structured event NOT found")
        };
        results.push(TestResult { name: "Syscall data", passed: has_syscall, detail });
    }

    // ── Test 6: Filter toggle works ────────────────────────────────
    {
        let was_enabled = state.trace_state.filters[0]; // Interrupt filter
        state.trace_state.handle_key(b'1'); // Toggle filter #1
        let now_enabled = state.trace_state.filters[0];
        let toggled = was_enabled != now_enabled;
        // Toggle back
        state.trace_state.handle_key(b'1');
        let detail = if toggled {
            String::from("filter toggle via key '1' works")
        } else {
            String::from("filter toggle FAILED")
        };
        results.push(TestResult { name: "Filter key", passed: toggled, detail });
    }

    // ── Test 7: Hardware state has live data ───────────────────────
    {
        state.hw_state.force_refresh();
        state.hw_state.update();
        let has_data = state.hw_state.uptime_secs > 0 || state.hw_state.heap_total > 0;
        let detail = format!("uptime={}s heap={}B irq_rate={}",
            state.hw_state.uptime_secs, state.hw_state.heap_total, state.hw_state.irq_rate);
        results.push(TestResult { name: "HW live data", passed: has_data, detail });
    }

    // ── Test 8: Pipeline updates from trace bus ────────────────────
    {
        let before = state.pipeline_state.flows.len();
        trace_bus::emit_static(trace_bus::EventCategory::Keyboard, "ux_test_key", 0);
        trace_bus::emit_static(trace_bus::EventCategory::Scheduler, "ux_test_sched", 0);
        // Force pipeline update (bypass frame counter)
        for _ in 0..10 {
            state.pipeline_state.update();
        }
        let after = state.pipeline_state.flows.len();
        let grew = after > before;
        let detail = format!("flows: {} -> {} (grew={})", before, after, grew);
        results.push(TestResult { name: "Pipeline upd", passed: grew, detail });
    }

    // ── Test 9: Guide search filters correctly ─────────────────────
    {
        state.guide_state.search.clear();
        state.guide_state.cursor = 0;
        state.guide_state.handle_char('l');
        state.guide_state.handle_char('s');
        let has_search = state.guide_state.search == "ls";
        // Reset
        state.guide_state.search.clear();
        state.guide_state.cursor = 0;
        let detail = if has_search {
            String::from("guide search input 'ls' recorded correctly")
        } else {
            format!("guide search: expected 'ls', got '{}'", state.guide_state.search)
        };
        results.push(TestResult { name: "Guide search", passed: has_search, detail });
    }

    // ── Emit results to trace bus ──────────────────────────────────
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    let summary = format!("=== UX TEST: {}/{} passed ===", passed, total);

    trace_bus::emit(trace_bus::EventCategory::Custom, summary, passed as u64);

    for r in &results {
        let icon = if r.passed { "PASS" } else { "FAIL" };
        let msg = format!("[{}] {} | {}", icon, r.name, r.detail);
        trace_bus::emit(trace_bus::EventCategory::Custom, msg, if r.passed { 1 } else { 0 });
    }

    // Also print to serial for external verification
    crate::serial_println!("=== TrustLab UX Test Results: {}/{} passed ===", passed, total);
    for r in &results {
        let icon = if r.passed { "PASS" } else { "FAIL" };
        crate::serial_println!("  [{}] {} - {}", icon, r.name, r.detail);
    }

    // Focus Kernel Trace to show results
    state.focus_module(PanelId::KernelTrace);
    state.trace_state.scroll = 0;
}
