//! Apple Interrupt Controller (AIC) Driver
//!
//! AIC is Apple's custom interrupt controller used on all Apple Silicon.
//! It replaces the standard ARM GIC (Generic Interrupt Controller).
//!
//! AIC handles:
//! - Hardware IRQs from peripherals (mapped to AIC IRQ numbers)
//! - IPI (Inter-Processor Interrupts) for SMP
//! - FIQ-based timer interrupts (ARM architected timers route through FIQ on Apple)
//! - Performance monitor interrupts
//!
//! Architecture:
//! ```text
//!   Peripherals → AIC → CPU core (IRQ line)
//!                  ↑
//!   Timer ————→ FIQ (bypasses AIC, directly to core)
//!   IPI ——————→ AIC IPI register → target core's IRQ
//! ```
//!
//! AIC versions:
//! - AICv1: A7–A14, M1 (early) — what we target first
//! - AICv2: M1 Pro/Max/Ultra, A15+ — extended event numbering
//!
//! Register map (AICv1, base from Device Tree):
//!   +0x0000  AIC_INFO       — capabilities (num IRQs, version)
//!   +0x2008  AIC_EVENT       — current pending event (read to ACK)
//!   +0x2004  AIC_IPI_FLAG    — IPI pending flags
//!   +0x2028  AIC_IPI_ACK     — acknowledge IPI
//!   +0x2024  AIC_IPI_SET     — send IPI to self
//!   +0x2034  AIC_IPI_OTHER   — send IPI to another core
//!   +0x3000+ AIC_TARGET_CPU  — per-IRQ routing (which CPU handles it)
//!   +0x4000+ AIC_MASK_SET    — per-IRQ mask (disable interrupt)
//!   +0x4080+ AIC_MASK_CLR    — per-IRQ unmask (enable interrupt)
//!   +0x4100+ AIC_SW_SET      — software trigger interrupt
//!   +0x4180+ AIC_SW_CLR      — clear software trigger
//!
//! References:
//! - Asahi Linux: drivers/irqchip/irq-apple-aic.c
//! - Corellium: public AIC documentation
//! - TrustOS iOS Recon: tools/ios-recon/

use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════════
// AIC Register Offsets (AICv1)
// ═══════════════════════════════════════════════════════════════════════════

/// AIC hardware info register (read-only)
const AIC_INFO: usize         = 0x0004;
/// Interrupt event register (read = ack + get IRQ number)
const AIC_EVENT: usize        = 0x2004;
/// IPI flags for current CPU  
const AIC_IPI_FLAG: usize     = 0x2008;
/// IPI mask for current CPU
const AIC_IPI_MASK_SET: usize = 0x200C;
/// IPI mask clear  
const AIC_IPI_MASK_CLR: usize = 0x2010;
/// Acknowledge IPI (write to clear)
const AIC_IPI_ACK: usize      = 0x2014;
/// Set IPI to self
const AIC_IPI_SET_SELF: usize = 0x2008;
/// Set IPI to other CPU
const AIC_IPI_SET_OTHER: usize = 0x200C;

/// Base of per-IRQ target CPU registers (32-bit per IRQ)
const AIC_TARGET_CPU_BASE: usize = 0x3000;
/// Base of IRQ mask set registers (bit per IRQ, 32 IRQs per word)
const AIC_MASK_SET_BASE: usize   = 0x4000;
/// Base of IRQ mask clear registers  
const AIC_MASK_CLR_BASE: usize   = 0x4080;
/// Base of software set IRQ registers
const AIC_SW_SET_BASE: usize     = 0x4100;
/// Base of software clear IRQ registers
const AIC_SW_CLR_BASE: usize     = 0x4180;

// AIC_INFO bitfields
/// Number of external IRQs (bits 15:0)
const AIC_INFO_NUM_IRQ_MASK: u32 = 0xFFFF;
/// AIC version (bits 31:28) — 0=v1, 1=v2
const AIC_INFO_VERSION_SHIFT: u32 = 28;

// AIC_EVENT bitfields
/// Event type (bits 31:16)
const AIC_EVENT_TYPE_SHIFT: u32 = 16;
/// Event IRQ number (bits 15:0)
const AIC_EVENT_NUM_MASK: u32   = 0xFFFF;
/// Event type: no event pending
const AIC_EVENT_TYPE_NONE: u32  = 0;
/// Event type: hardware IRQ
const AIC_EVENT_TYPE_IRQ: u32   = 1;
/// Event type: IPI
const AIC_EVENT_TYPE_IPI: u32   = 4;

// IPI flags
/// Self IPI pending
const AIC_IPI_SELF: u32 = 1 << 31;
/// Other CPU IPI pending  
const AIC_IPI_OTHER: u32 = 1 << 0;

// ═══════════════════════════════════════════════════════════════════════════
// AIC Driver State
// ═══════════════════════════════════════════════════════════════════════════

/// Maximum supported IRQs
const MAX_IRQS: usize = 1024;

/// IRQ handler function type
pub type IrqHandler = fn(irq: u32);

/// Per-IRQ configuration
#[derive(Clone, Copy)]
struct IrqConfig {
    /// Handler function (None = no handler)
    handler: Option<IrqHandler>,
    /// Target CPU for this IRQ (0 = CPU0)
    target_cpu: u8,
    /// Whether this IRQ is enabled
    enabled: bool,
    /// Hit count for debugging
    count: u64,
    /// Device name for display
    name: &'static str,
}

impl Default for IrqConfig {
    fn default() -> Self {
        Self {
            handler: None,
            target_cpu: 0,
            enabled: false,
            count: 0,
            name: "unknown",
        }
    }
}

/// AIC driver instance
pub struct AppleAic {
    /// MMIO base address (physical)
    base: u64,
    /// Number of hardware IRQs supported
    num_irqs: u32,
    /// AIC version (0=v1, 1=v2)
    version: u32,
    /// Number of CPU cores
    num_cpus: u32,
    /// Per-IRQ configuration
    irqs: [IrqConfig; MAX_IRQS],
    /// Total interrupts handled
    total_irqs: u64,
    /// Total IPIs handled
    total_ipis: u64,
    /// Whether AIC is initialized
    initialized: bool,
}

static AIC: Mutex<Option<AppleAic>> = Mutex::new(None);
static AIC_INITIALIZED: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════
// MMIO access helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Read a 32-bit AIC register
#[inline(always)]
unsafe fn aic_read32(base: u64, offset: usize) -> u32 {
    let addr = (base as usize + offset) as *const u32;
    ptr::read_volatile(addr)
}

/// Write a 32-bit AIC register
#[inline(always)]
unsafe fn aic_write32(base: u64, offset: usize, val: u32) {
    let addr = (base as usize + offset) as *mut u32;
    ptr::write_volatile(addr, val);
}

// ═══════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════

/// Initialize AIC at the given MMIO base address.
///
/// `base` should come from the Device Tree "reg" property of the AIC node.
/// On A12 (T8020), this is typically 0x23B100000.
/// On A10 (T8010), this is typically 0x20E100000.
///
/// # Safety
/// Caller must ensure `base` points to valid AIC MMIO.
pub unsafe fn init(base: u64, num_cpus: u32) -> Result<(), &'static str> {
    crate::serial_println!("[AIC] Initializing Apple Interrupt Controller @ {:#x}", base);
    
    // Read AIC_INFO to discover capabilities
    let info = aic_read32(base, AIC_INFO);
    let num_irqs = info & AIC_INFO_NUM_IRQ_MASK;
    let version = (info >> AIC_INFO_VERSION_SHIFT) & 0xF;
    
    crate::serial_println!("[AIC] Version: AICv{}", version + 1);
    crate::serial_println!("[AIC] Hardware IRQs: {}", num_irqs);
    crate::serial_println!("[AIC] CPUs: {}", num_cpus);
    
    if num_irqs == 0 || num_irqs as usize > MAX_IRQS {
        return Err("AIC: invalid IRQ count from hardware");
    }
    
    let mut aic = AppleAic {
        base,
        num_irqs,
        version,
        num_cpus,
        irqs: [IrqConfig::default(); MAX_IRQS],
        total_irqs: 0,
        total_ipis: 0,
        initialized: false,
    };
    
    // Mask all IRQs initially
    let num_words = (num_irqs + 31) / 32;
    for w in 0..num_words as usize {
        aic_write32(base, AIC_MASK_SET_BASE + w * 4, 0xFFFFFFFF);
    }
    
    // Route all IRQs to CPU 0 initially
    for i in 0..num_irqs as usize {
        aic_write32(base, AIC_TARGET_CPU_BASE + i * 4, 1 << 0); // CPU0
    }
    
    // Enable IPI reception (unmask IPIs)
    aic_write32(base, AIC_IPI_MASK_CLR, AIC_IPI_SELF | AIC_IPI_OTHER);
    
    aic.initialized = true;
    
    *AIC.lock() = Some(aic);
    AIC_INITIALIZED.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[AIC] Initialization complete — all IRQs masked, IPIs enabled");
    Ok(())
}

/// Register an IRQ handler
pub fn register_irq(irq: u32, name: &'static str, handler: IrqHandler) -> Result<(), &'static str> {
    let mut guard = AIC.lock();
    let aic = guard.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    
    let config = &mut aic.irqs[irq as usize];
    config.handler = Some(handler);
    config.name = name;
    
    crate::serial_println!("[AIC] Registered IRQ {} → {}", irq, name);
    Ok(())
}

/// Enable (unmask) an IRQ
pub fn enable_irq(irq: u32) -> Result<(), &'static str> {
    let mut guard = AIC.lock();
    let aic = guard.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    
    let word = irq / 32;
    let bit = irq % 32;
    
    unsafe {
        aic_write32(aic.base, AIC_MASK_CLR_BASE + word as usize * 4, 1 << bit);
    }
    
    aic.irqs[irq as usize].enabled = true;
    Ok(())
}

/// Disable (mask) an IRQ
pub fn disable_irq(irq: u32) -> Result<(), &'static str> {
    let mut guard = AIC.lock();
    let aic = guard.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    
    let word = irq / 32;
    let bit = irq % 32;
    
    unsafe {
        aic_write32(aic.base, AIC_MASK_SET_BASE + word as usize * 4, 1 << bit);
    }
    
    aic.irqs[irq as usize].enabled = false;
    Ok(())
}

/// Set which CPU handles a given IRQ
pub fn set_irq_target(irq: u32, cpu: u32) -> Result<(), &'static str> {
    let mut guard = AIC.lock();
    let aic = guard.as_mut().ok_or("AIC not initialized")?;
    
    if irq >= aic.num_irqs {
        return Err("IRQ number out of range");
    }
    if cpu >= aic.num_cpus {
        return Err("CPU number out of range");
    }
    
    unsafe {
        aic_write32(aic.base, AIC_TARGET_CPU_BASE + irq as usize * 4, 1 << cpu);
    }
    
    aic.irqs[irq as usize].target_cpu = cpu as u8;
    Ok(())
}

/// Send an IPI (Inter-Processor Interrupt) to another CPU
pub fn send_ipi(target_cpu: u32) -> Result<(), &'static str> {
    let guard = AIC.lock();
    let aic = guard.as_ref().ok_or("AIC not initialized")?;
    
    if target_cpu >= aic.num_cpus {
        return Err("Target CPU out of range");
    }
    
    unsafe {
        // On AIC, IPI to other cores uses the AIC_IPI_SET register
        // The mechanism varies by AIC version; on AICv1 we write the
        // target CPU mask. On AICv2 there's a per-cluster register.
        // For now, implement AICv1 simple path:
        aic_write32(aic.base, AIC_IPI_SET_OTHER, 1 << target_cpu);
    }
    
    Ok(())
}

/// Handle an AIC interrupt (called from exception vector)
///
/// Returns true if an interrupt was handled, false if none pending.
///
/// # Safety
/// Must be called from IRQ exception handler context.
pub unsafe fn handle_irq() -> bool {
    if !AIC_INITIALIZED.load(Ordering::SeqCst) {
        return false;
    }
    
    let mut guard = AIC.lock();
    let aic = match guard.as_mut() {
        Some(a) => a,
        None => return false,
    };
    
    // Read AIC_EVENT — this atomically acknowledges the interrupt
    let event = aic_read32(aic.base, AIC_EVENT);
    let event_type = (event >> AIC_EVENT_TYPE_SHIFT) & 0xFFFF;
    let event_num = event & AIC_EVENT_NUM_MASK;
    
    match event_type {
        AIC_EVENT_TYPE_NONE => {
            // No pending event (spurious)
            false
        }
        
        AIC_EVENT_TYPE_IRQ => {
            // Hardware IRQ
            aic.total_irqs += 1;
            
            if (event_num as usize) < MAX_IRQS {
                let config = &mut aic.irqs[event_num as usize];
                config.count += 1;
                
                if let Some(handler) = config.handler {
                    // Drop lock before calling handler to prevent deadlocks
                    let handler_fn = handler;
                    drop(guard);
                    handler_fn(event_num);
                    return true;
                }
            }
            
            // No handler — log and ignore
            crate::serial_println!("[AIC] Unhandled IRQ {}", event_num);
            true
        }
        
        AIC_EVENT_TYPE_IPI => {
            // Inter-Processor Interrupt
            aic.total_ipis += 1;
            
            // Read IPI flags to determine type
            let ipi_flags = aic_read32(aic.base, AIC_IPI_FLAG);
            
            // Acknowledge the IPI
            aic_write32(aic.base, AIC_IPI_ACK, ipi_flags);
            
            if ipi_flags & AIC_IPI_SELF != 0 {
                crate::serial_println!("[AIC] Self-IPI received");
            }
            if ipi_flags & AIC_IPI_OTHER != 0 {
                // IPI from another CPU — typically used for:
                // - TLB shootdown
                // - Scheduler rebalance  
                // - Kernel panic broadcast
                crate::serial_println!("[AIC] Cross-CPU IPI received");
            }
            
            true
        }
        
        _ => {
            crate::serial_println!("[AIC] Unknown event type {} num {}", event_type, event_num);
            true
        }
    }
}

/// Check if AIC is initialized
pub fn is_initialized() -> bool {
    AIC_INITIALIZED.load(Ordering::SeqCst)
}

/// Get AIC status summary for shell display
pub fn status_summary() -> String {
    let guard = AIC.lock();
    match guard.as_ref() {
        None => String::from("AIC: not initialized"),
        Some(aic) => {
            let enabled = aic.irqs[..aic.num_irqs as usize]
                .iter()
                .filter(|c| c.enabled)
                .count();
            format!(
                "AIC v{} @ {:#x}: {} IRQs ({} enabled), {} total handled, {} IPIs",
                aic.version + 1, aic.base, aic.num_irqs,
                enabled, aic.total_irqs, aic.total_ipis
            )
        }
    }
}

/// List all registered IRQ handlers (for `irqinfo` shell command)
pub fn list_irqs() -> Vec<(u32, &'static str, bool, u64)> {
    let guard = AIC.lock();
    match guard.as_ref() {
        None => Vec::new(),
        Some(aic) => {
            let mut result = Vec::new();
            for i in 0..aic.num_irqs as usize {
                let config = &aic.irqs[i];
                if config.handler.is_some() || config.enabled {
                    result.push((
                        i as u32,
                        config.name,
                        config.enabled,
                        config.count,
                    ));
                }
            }
            result
        }
    }
}
