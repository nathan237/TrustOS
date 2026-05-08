














extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{LabState, PanelId, trace_bus};


struct D {
    name: &'static str,
    passed: bool,
    detail: String,
}


pub fn ojh(state: &mut LabState) {
    let mut results: Vec<D> = Vec::new();

    
    {
        state.focused_slot = 0; 
        
        let lsq: [usize; 7] = [1, 2, 3, 4, 5, 6, 0];
        let mut ok = true;
        let mut detail = String::new();
        for (i, afe) in lsq.iter().enumerate() {
            state.handle_key(0x09); 
            if state.focused_slot != *afe {
                ok = false;
                detail = format!("step {}: expected slot {}, got slot {}", i, afe, state.focused_slot);
                break;
            }
        }
        if ok { detail = String::from("7 tab presses cycle all slots correctly"); }
        results.push(D { name: "Tab cycle", passed: ok, detail });
    }

    
    {
        let fnl: [(&str, PanelId); 7] = [
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
        for (cmd, expected) in &fnl {
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
        results.push(D { name: "Shell cmds", passed: ok, detail });
    }

    
    {
        
        
        let ca = 1200u32;
        let er = 800u32;
        let cx = 2i32;
        let u = 30i32; 
        let aq = ca - 4;
        let ch = er - 32;

        let aoq = super::cvn(cx, u, aq, ch);
        let mut ok = true;
        let mut detail = String::new();
        for (i, ej) in aoq.iter().enumerate() {
            let qh = ej.x + ej.w as i32 / 2;
            let abv = ej.y + ej.h as i32 / 2;
            state.handle_click(qh, abv, ca, er);
            if state.focused_slot != i {
                ok = false;
                detail = format!("click slot {} => focused_slot {}, expected {}", i, state.focused_slot, i);
                break;
            }
        }
        if ok { detail = String::from("clicks on all 7 slot areas set focus correctly"); }
        results.push(D { name: "Click focus", passed: ok, detail });
    }

    
    {
        
        trace_bus::bgi(trace_bus::EventCategory::Custom, "ux_test_marker", 42);
        state.trace_state.update();
        state.trace_state.update(); 
        
        let idu = state.trace_state.events.iter()
            .any(|e| e.message.contains("ux_test_marker"));
        let detail = if idu {
            String::from("test event propagated to trace panel")
        } else {
            String::from("test event NOT found in trace panel (may need more ticks)")
        };
        results.push(D { name: "Trace recv", passed: idu, detail });
    }

    
    {
        
        trace_bus::fuj(1, [1, 0x1000, 32], 32);
        state.trace_state.update();
        let iee = state.trace_state.events.iter()
            .any(|e| e.syscall_nr == Some(1));
        let detail = if iee {
            String::from("syscall event with nr=1 (write) found with structured data")
        } else {
            String::from("syscall structured event NOT found")
        };
        results.push(D { name: "Syscall data", passed: iee, detail });
    }

    
    {
        let fez = state.trace_state.filters[0]; 
        state.trace_state.handle_key(b'1'); 
        let nlj = state.trace_state.filters[0];
        let jne = fez != nlj;
        
        state.trace_state.handle_key(b'1');
        let detail = if jne {
            String::from("filter toggle via key '1' works")
        } else {
            String::from("filter toggle FAILED")
        };
        results.push(D { name: "Filter key", passed: jne, detail });
    }

    
    {
        state.hw_state.force_refresh();
        state.hw_state.update();
        let has_data = state.hw_state.uptime_secs > 0 || state.hw_state.heap_total > 0;
        let detail = format!("uptime={}s heap={}B irq_rate={}",
            state.hw_state.uptime_secs, state.hw_state.heap_total, state.hw_state.irq_rate);
        results.push(D { name: "HW live data", passed: has_data, detail });
    }

    
    {
        let bak = state.pipeline_state.flows.len();
        trace_bus::bgi(trace_bus::EventCategory::Keyboard, "ux_test_key", 0);
        trace_bus::bgi(trace_bus::EventCategory::Scheduler, "ux_test_sched", 0);
        
        for _ in 0..10 {
            state.pipeline_state.update();
        }
        let beo = state.pipeline_state.flows.len();
        let ict = beo > bak;
        let detail = format!("flows: {} -> {} (grew={})", bak, beo, ict);
        results.push(D { name: "Pipeline upd", passed: ict, detail });
    }

    
    {
        state.guide_state.search.clear();
        state.guide_state.cursor = 0;
        state.guide_state.handle_char('l');
        state.guide_state.handle_char('s');
        let iec = state.guide_state.search == "ls";
        
        state.guide_state.search.clear();
        state.guide_state.cursor = 0;
        let detail = if iec {
            String::from("guide search input 'ls' recorded correctly")
        } else {
            format!("guide search: expected 'ls', got '{}'", state.guide_state.search)
        };
        results.push(D { name: "Guide search", passed: iec, detail });
    }

    
    let passed = results.iter().filter(|r| r.passed).count();
    let av = results.len();
    let summary = format!("=== UX TEST: {}/{} passed ===", passed, av);

    trace_bus::emit(trace_bus::EventCategory::Custom, summary, passed as u64);

    for r in &results {
        let icon = if r.passed { "PASS" } else { "FAIL" };
        let bk = format!("[{}] {} | {}", icon, r.name, r.detail);
        trace_bus::emit(trace_bus::EventCategory::Custom, bk, if r.passed { 1 } else { 0 });
    }

    
    crate::serial_println!("=== TrustLab UX Test Results: {}/{} passed ===", passed, av);
    for r in &results {
        let icon = if r.passed { "PASS" } else { "FAIL" };
        crate::serial_println!("  [{}] {} - {}", icon, r.name, r.detail);
    }

    
    state.focus_module(PanelId::KernelTrace);
    state.trace_state.scroll = 0;
}
