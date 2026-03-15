//! Virtual GIC (Generic Interrupt Controller) for ARM EL2 Hypervisor
//!
//! The GIC is how ARM CPUs route hardware interrupts. When we run a
//! guest at EL1, we need to virtualize interrupt delivery:
//!
//! 1. Physical IRQ fires → trapped to EL2
//! 2. We read the IAR (Interrupt Acknowledge Register), get IRQ number
//! 3. We inject a virtual IRQ into the guest via List Registers
//! 4. Guest handles it as if it were a normal interrupt
//!
//! ARM GIC Architecture:
//! ```text
//!   GIC Distributor (GICD) — routes IRQs to CPUs
//!   GIC Redistributor (GICR) — per-CPU interface (GICv3+)
//!   GIC CPU Interface (ICC_*_EL1) — system registers for ack/eoi
//!   GIC Hypervisor (ICH_*_EL2) — system registers for virtual IRQs
//! ```
//!
//! GICv3 virtual interrupt injection uses List Registers (LRs):
//!   ICH_LR<n>_EL2: [63]=HW, [62]=Group, [61:60]=State, [55:48]=Priority,
//!                  [44:32]=pINTID (physical), [31:0]=vINTID (virtual)

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

/// Maximum number of List Registers (most GICv3 have 4-16)
const MAX_LRS: usize = 16;

/// GICv3 ICC System Register offsets (for EL2 access)
pub mod gic_regs {
    /// ICC_IAR1_EL1 — Interrupt Acknowledge (Group 1)
    pub const ICC_IAR1_EL1: u32 = 0;
    /// ICC_EOIR1_EL1 — End of Interrupt (Group 1)
    pub const ICC_EOIR1_EL1: u32 = 1;
    /// ICC_PMR_EL1 — Priority Mask
    pub const ICC_PMR_EL1: u32 = 2;
    /// ICC_SRE_EL2 — System Register Enable (EL2)
    pub const ICC_SRE_EL2: u32 = 3;
    /// ICH_HCR_EL2 — Hypervisor Control Register
    pub const ICH_HCR_EL2: u32 = 4;
    /// ICH_VTR_EL2 — VGIC Type Register
    pub const ICH_VTR_EL2: u32 = 5;
}

/// ICH_HCR_EL2 bits
pub mod ich_hcr {
    /// Enable virtual CPU interface
    pub const EN: u64 = 1 << 0;
    /// Underflow Interrupt Enable
    pub const UIE: u64 = 1 << 1;
    /// List Register Entry Not Present Interrupt Enable
    pub const LRENPIE: u64 = 1 << 2;
    /// No Pending Interrupt Enable
    pub const NPIE: u64 = 1 << 3;
    /// VGRP0 Enable
    pub const VGRP0EIE: u64 = 1 << 4;
    /// VGRP1 Enable
    pub const VGRP1EIE: u64 = 1 << 5;
    /// Trap all EL1 accesses to ICC_* regs
    pub const TALL0: u64 = 1 << 6;
    /// Trap all EL1 accesses to ICC_* Group 1 regs
    pub const TALL1: u64 = 1 << 7;
    /// EOI count (maintenance interrupt)
    pub const EOI_COUNT_SHIFT: u64 = 27;
}

/// ICH_LR<n>_EL2 bit fields
pub mod ich_lr {
    /// State: Invalid (empty LR)
    pub const STATE_INVALID: u64 = 0b00 << 62;   // bits [63:62] = State, shifted by 1
    /// State: Pending
    pub const STATE_PENDING: u64 = 0b01 << 60;
    /// State: Active
    pub const STATE_ACTIVE: u64 = 0b10 << 60;
    /// State: Pending + Active
    pub const STATE_PEND_ACT: u64 = 0b11 << 60;
    /// HW bit — links virtual to physical IRQ
    pub const HW: u64 = 1 << 63;
    /// Group bit (0 = Group 0, 1 = Group 1)
    pub const GROUP1: u64 = 1 << 60;

    /// Build a List Register value
    pub fn build_lr(vintid: u32, pintid: u32, priority: u8, hw: bool) -> u64 {
        let mut lr: u64 = 0;
        lr |= (vintid as u64) & 0xFFFF_FFFF;    // vINTID [31:0]
        if hw {
            lr |= HW;
            lr |= ((pintid as u64) & 0x1FFF) << 32;  // pINTID [44:32]
        }
        lr |= ((priority as u64) & 0xFF) << 48;  // Priority [55:48]
        lr |= STATE_PENDING;   // Start as Pending
        lr |= GROUP1;          // Group 1 (non-secure)
        lr
    }
}

/// Virtual GIC state
pub struct VirtualGic {
    /// Number of available List Registers
    num_lrs: u32,
    /// Pending virtual IRQs that couldn't be injected yet
    pending_queue: [u32; 64],
    pending_count: usize,
    /// Whether the VGIC is initialized
    initialized: bool,
}

/// Global VGIC instance
static VGIC_INITIALIZED: AtomicBool = AtomicBool::new(false);

impl VirtualGic {
    /// Create a new virtual GIC (uninitialized)
    pub const fn new() -> Self {
        VirtualGic {
            num_lrs: 0,
            pending_queue: [0; 64],
            pending_count: 0,
            initialized: false,
        }
    }

    /// Initialize the virtual GIC
    ///
    /// Must be called at EL2 to read ICH_VTR_EL2
    pub fn init(&mut self) {
        #[cfg(target_arch = "aarch64")]
        {
            // Read ICH_VTR_EL2 to get number of LRs
            let vtr: u64;
            unsafe {
                core::arch::asm!(
                    "mrs {vtr}, ich_vtr_el2",
                    vtr = out(reg) vtr,
                    options(nomem, nostack)
                );
            }

            // ListRegs field is bits [4:0], value is (n+1) LRs
            self.num_lrs = ((vtr & 0x1F) + 1) as u32;
            if self.num_lrs > MAX_LRS as u32 {
                self.num_lrs = MAX_LRS as u32;
            }

            // Enable the virtual CPU interface
            unsafe {
                // First, ensure SRE is enabled at EL2
                core::arch::asm!(
                    "mrs {tmp}, icc_sre_el2",
                    "orr {tmp}, {tmp}, #0x1",   // SRE bit
                    "orr {tmp}, {tmp}, #0x8",   // Enable for lower ELs
                    "msr icc_sre_el2, {tmp}",
                    "isb",
                    tmp = out(reg) _,
                    options(nomem, nostack)
                );

                // Enable ICH_HCR_EL2
                let hcr_val = ich_hcr::EN;
                core::arch::asm!(
                    "msr ich_hcr_el2, {val}",
                    "isb",
                    val = in(reg) hcr_val,
                    options(nomem, nostack)
                );
            }

            self.initialized = true;
            VGIC_INITIALIZED.store(true, Ordering::Release);
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            self.num_lrs = 4;
            self.initialized = true;
            VGIC_INITIALIZED.store(true, Ordering::Release);
        }
    }

    /// Is the VGIC initialized?
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Number of List Registers available
    pub fn num_list_registers(&self) -> u32 {
        self.num_lrs
    }

    /// Inject a virtual interrupt into the guest
    ///
    /// Finds a free List Register and writes the interrupt.
    /// If all LRs are full, queues it for later injection.
    pub fn inject_irq(&mut self, vintid: u32, pintid: u32, priority: u8) -> bool {
        #[cfg(target_arch = "aarch64")]
        {
            // Try to find a free List Register
            for i in 0..self.num_lrs {
                let lr_val = self.read_lr(i);
                let state = (lr_val >> 60) & 0x3;
                if state == 0 {
                    // Free LR — inject!
                    let lr = ich_lr::build_lr(vintid, pintid, priority, true);
                    self.write_lr(i, lr);
                    return true;
                }
            }

            // All LRs full — queue for later
            if self.pending_count < self.pending_queue.len() {
                self.pending_queue[self.pending_count] = vintid;
                self.pending_count += 1;
            }
            false
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = (vintid, pintid, priority);
            false
        }
    }

    /// Drain pending queue — call after guest handles some interrupts
    pub fn drain_pending(&mut self) {
        if self.pending_count == 0 {
            return;
        }

        let mut remaining = 0;
        for i in 0..self.pending_count {
            let vintid = self.pending_queue[i];
            if !self.inject_irq(vintid, vintid, 0xA0) {
                // Still can't inject, keep in queue
                self.pending_queue[remaining] = vintid;
                remaining += 1;
            }
        }
        self.pending_count = remaining;
    }

    /// Read a List Register
    fn read_lr(&self, idx: u32) -> u64 {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let val: u64;
            match idx {
                0 => core::arch::asm!("mrs {v}, ich_lr0_el2", v = out(reg) val, options(nomem, nostack)),
                1 => core::arch::asm!("mrs {v}, ich_lr1_el2", v = out(reg) val, options(nomem, nostack)),
                2 => core::arch::asm!("mrs {v}, ich_lr2_el2", v = out(reg) val, options(nomem, nostack)),
                3 => core::arch::asm!("mrs {v}, ich_lr3_el2", v = out(reg) val, options(nomem, nostack)),
                4 => core::arch::asm!("mrs {v}, ich_lr4_el2", v = out(reg) val, options(nomem, nostack)),
                5 => core::arch::asm!("mrs {v}, ich_lr5_el2", v = out(reg) val, options(nomem, nostack)),
                6 => core::arch::asm!("mrs {v}, ich_lr6_el2", v = out(reg) val, options(nomem, nostack)),
                7 => core::arch::asm!("mrs {v}, ich_lr7_el2", v = out(reg) val, options(nomem, nostack)),
                8 => core::arch::asm!("mrs {v}, ich_lr8_el2", v = out(reg) val, options(nomem, nostack)),
                9 => core::arch::asm!("mrs {v}, ich_lr9_el2", v = out(reg) val, options(nomem, nostack)),
                10 => core::arch::asm!("mrs {v}, ich_lr10_el2", v = out(reg) val, options(nomem, nostack)),
                11 => core::arch::asm!("mrs {v}, ich_lr11_el2", v = out(reg) val, options(nomem, nostack)),
                12 => core::arch::asm!("mrs {v}, ich_lr12_el2", v = out(reg) val, options(nomem, nostack)),
                13 => core::arch::asm!("mrs {v}, ich_lr13_el2", v = out(reg) val, options(nomem, nostack)),
                14 => core::arch::asm!("mrs {v}, ich_lr14_el2", v = out(reg) val, options(nomem, nostack)),
                15 => core::arch::asm!("mrs {v}, ich_lr15_el2", v = out(reg) val, options(nomem, nostack)),
                _ => return 0,
            }
            val
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = idx;
            0
        }
    }

    /// Write a List Register
    fn write_lr(&self, idx: u32, val: u64) {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            match idx {
                0 => core::arch::asm!("msr ich_lr0_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                1 => core::arch::asm!("msr ich_lr1_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                2 => core::arch::asm!("msr ich_lr2_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                3 => core::arch::asm!("msr ich_lr3_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                4 => core::arch::asm!("msr ich_lr4_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                5 => core::arch::asm!("msr ich_lr5_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                6 => core::arch::asm!("msr ich_lr6_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                7 => core::arch::asm!("msr ich_lr7_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                8 => core::arch::asm!("msr ich_lr8_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                9 => core::arch::asm!("msr ich_lr9_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                10 => core::arch::asm!("msr ich_lr10_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                11 => core::arch::asm!("msr ich_lr11_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                12 => core::arch::asm!("msr ich_lr12_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                13 => core::arch::asm!("msr ich_lr13_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                14 => core::arch::asm!("msr ich_lr14_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                15 => core::arch::asm!("msr ich_lr15_el2, {v}", v = in(reg) val, options(nomem, nostack)),
                _ => {}
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let _ = (idx, val);
        }
    }
}

/// Handle a physical IRQ at EL2 — acknowledge and inject to guest
pub fn handle_el2_irq(vgic: &mut VirtualGic) {
    #[cfg(target_arch = "aarch64")]
    {
        // Read ICC_IAR1_EL1 to acknowledge the physical IRQ
        let intid: u64;
        unsafe {
            core::arch::asm!(
                "mrs {id}, icc_iar1_el1",
                id = out(reg) intid,
                options(nomem, nostack)
            );
        }

        let intid = intid as u32;

        // Spurious interrupt?
        if intid >= 1020 {
            return;
        }

        // Inject as virtual IRQ to guest
        vgic.inject_irq(intid, intid, 0xA0);

        // Signal EOI for the physical interrupt
        unsafe {
            core::arch::asm!(
                "msr icc_eoir1_el1, {id}",
                id = in(reg) intid as u64,
                options(nomem, nostack)
            );
        }

        // Also deactivate
        unsafe {
            core::arch::asm!(
                "msr icc_dir_el1, {id}",
                id = in(reg) intid as u64,
                options(nomem, nostack)
            );
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = vgic;
    }
}
