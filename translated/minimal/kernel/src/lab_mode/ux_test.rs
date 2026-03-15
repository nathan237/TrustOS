














extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use super::{LabState, PanelId, trace_bus};


struct A {
    j: &'static str,
    cg: bool,
    eu: String,
}


pub fn wbn(g: &mut LabState) {
    let mut hd: Vec<A> = Vec::new();

    
    {
        g.ckl = 0; 
        
        let spg: [usize; 7] = [1, 2, 3, 4, 5, 6, 0];
        let mut bq = true;
        let mut eu = String::new();
        for (a, bgz) in spg.iter().cf() {
            g.vr(0x09); 
            if g.ckl != *bgz {
                bq = false;
                eu = format!("step {}: expected slot {}, got slot {}", a, bgz, g.ckl);
                break;
            }
        }
        if bq { eu = String::from("7 tab presses cycle all slots correctly"); }
        hd.push(A { j: "Tab cycle", cg: bq, eu });
    }

    
    {
        let kjn: [(&str, PanelId); 7] = [
            ("hw", PanelId::Iq),
            ("trace", PanelId::Gk),
            ("help", PanelId::Hm),
            ("fs", PanelId::Hp),
            ("edit", PanelId::Gp),
            ("pipeline", PanelId::Iz),
            ("hex", PanelId::Hr),
        ];
        let mut bq = true;
        let mut eu = String::new();
        for (cmd, qy) in &kjn {
            g.bfh = String::from(*cmd);
            g.dvx = cmd.len();
            g.nrq();
            if g.eqp() != *qy {
                bq = false;
                eu = format!("'{}' => {:?}, expected {:?}", cmd, g.eqp(), qy);
                break;
            }
        }
        if bq { eu = String::from("7 shell commands focus correct panels"); }
        hd.push(A { j: "Shell cmds", cg: bq, eu });
    }

    
    {
        
        
        let hk = 1200u32;
        let mg = 800u32;
        let cx = 2i32;
        let ae = 30i32; 
        let dt = hk - 4;
        let bm = mg - 32;

        let cls = super::ioy(cx, ae, dt, bm);
        let mut bq = true;
        let mut eu = String::new();
        for (a, oc) in cls.iter().cf() {
            let agi = oc.b + oc.d as i32 / 2;
            let bbf = oc.c + oc.i as i32 / 2;
            g.ago(agi, bbf, hk, mg);
            if g.ckl != a {
                bq = false;
                eu = format!("click slot {} => focused_slot {}, expected {}", a, g.ckl, a);
                break;
            }
        }
        if bq { eu = String::from("clicks on all 7 slot areas set focus correctly"); }
        hd.push(A { j: "Click focus", cg: bq, eu });
    }

    
    {
        
        trace_bus::dgy(trace_bus::EventCategory::Gv, "ux_test_marker", 42);
        g.ccg.qs();
        g.ccg.qs(); 
        
        let oas = g.ccg.events.iter()
            .any(|aa| aa.message.contains("ux_test_marker"));
        let eu = if oas {
            String::from("test event propagated to trace panel")
        } else {
            String::from("test event NOT found in trace panel (may need more ticks)")
        };
        hd.push(A { j: "Trace recv", cg: oas, eu });
    }

    
    {
        
        trace_bus::ktb(1, [1, 0x1000, 32], 32);
        g.ccg.qs();
        let obb = g.ccg.events.iter()
            .any(|aa| aa.fvy == Some(1));
        let eu = if obb {
            String::from("syscall event with nr=1 (write) found with structured data")
        } else {
            String::from("syscall structured event NOT found")
        };
        hd.push(A { j: "Syscall data", cg: obb, eu });
    }

    
    {
        let jwl = g.ccg.ckk[0]; 
        g.ccg.vr(b'1'); 
        let uvt = g.ccg.ckk[0];
        let pue = jwl != uvt;
        
        g.ccg.vr(b'1');
        let eu = if pue {
            String::from("filter toggle via key '1' works")
        } else {
            String::from("filter toggle FAILED")
        };
        hd.push(A { j: "Filter key", cg: pue, eu });
    }

    
    {
        g.cri.nvo();
        g.cri.qs();
        let cyk = g.cri.cnn > 0 || g.cri.aul > 0;
        let eu = format!("uptime={}s heap={}B irq_rate={}",
            g.cri.cnn, g.cri.aul, g.cri.eds);
        hd.push(A { j: "HW live data", cg: cyk, eu });
    }

    
    {
        let cvu = g.egs.cqq.len();
        trace_bus::dgy(trace_bus::EventCategory::Hs, "ux_test_key", 0);
        trace_bus::dgy(trace_bus::EventCategory::Scheduler, "ux_test_sched", 0);
        
        for _ in 0..10 {
            g.egs.qs();
        }
        let ddv = g.egs.cqq.len();
        let nzp = ddv > cvu;
        let eu = format!("flows: {} -> {} (grew={})", cvu, ddv, nzp);
        hd.push(A { j: "Pipeline upd", cg: nzp, eu });
    }

    
    {
        g.crb.anw.clear();
        g.crb.gi = 0;
        g.crb.fka('l');
        g.crb.fka('s');
        let oaz = g.crb.anw == "ls";
        
        g.crb.anw.clear();
        g.crb.gi = 0;
        let eu = if oaz {
            String::from("guide search input 'ls' recorded correctly")
        } else {
            format!("guide search: expected 'ls', got '{}'", g.crb.anw)
        };
        hd.push(A { j: "Guide search", cg: oaz, eu });
    }

    
    let cg = hd.iter().hi(|m| m.cg).az();
    let es = hd.len();
    let awz = format!("=== UX TEST: {}/{} passed ===", cg, es);

    trace_bus::fj(trace_bus::EventCategory::Gv, awz, cg as u64);

    for m in &hd {
        let pa = if m.cg { "PASS" } else { "FAIL" };
        let fr = format!("[{}] {} | {}", pa, m.j, m.eu);
        trace_bus::fj(trace_bus::EventCategory::Gv, fr, if m.cg { 1 } else { 0 });
    }

    
    crate::serial_println!("=== TrustLab UX Test Results: {}/{} passed ===", cg, es);
    for m in &hd {
        let pa = if m.cg { "PASS" } else { "FAIL" };
        crate::serial_println!("  [{}] {} - {}", pa, m.j, m.eu);
    }

    
    g.cqr(PanelId::Gk);
    g.ccg.jc = 0;
}
