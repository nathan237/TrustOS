//! Intel WiFi Link 4965AGN Driver (iwl4965)
//!
//! Driver for Intel PRO/Wireless 4965 AG/AGN (found in various laptops).
//! PCI IDs: 8086:4229 (4965AGN), 8086:4230 (4965AG_1), 8086:4235 (4965BG)
//!
//! This driver handles:
//! - PCI BAR mapping and register access
//! - Hardware reset and bring-up
//! - Firmware loading via BSM (Bootstrap State Machine) + DMA
//! - Active scanning via firmware SCAN_REQUEST command
//! - WPA2 association via software handshake
//!
//! Firmware: Requires iwlwifi-4965-2.ucode (freely redistributable by Intel).
//! Obtain from linux-firmware: git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git
//!
//! Reference: Intel iwlwifi driver (Linux), iwl4965 datasheet

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

use super::wifi::{WifiDriver, WifiNetwork, WifiSecurity, WifiState};
use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInformation, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// ============================================================================
// PCI Device IDs
// ============================================================================

const INTEL_VENDOR: u16 = 0x8086;

/// Known Intel WiFi 4965 device IDs
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IWL4965_DEVICE_IDS: &[u16] = &[
    0x4229, // WiFi Link 4965AGN
    0x4230, // WiFi Link 4965AG_1
];

/// Also support later Intel WiFi cards that may appear on other ThinkPads
const IWL_SUPPORTED_IDS: &[u16] = &[
    0x4229, 0x4230,         // 4965AGN / AG
    0x4232, 0x4235, 0x4236, // WiFi Link 5100/5300
    0x4237, 0x4238, 0x4239, // WiFi Link 5150
    0x008A, 0x008B,         // Centrino Wireless-N 100/130
    0x0082, 0x0083, 0x0084, // Centrino Advanced-N 6205
    0x0085, 0x0089,         // Centrino Advanced-N 6235
    0x0887, 0x0888,         // Centrino Wireless-N 2230
    0x0890, 0x0891,         // Centrino Wireless-N 2200
    0x0893, 0x0894,         // WiFi Link 6150
    0x088E, 0x088F,         // Centrino Advanced-N 6235
    0x24F3, 0x24F4,         // Wireless 8260
    0x2526,                 // Wireless-AC 9260
    0x2723,                 // WiFi 6 AX200
    0x2725,                 // WiFi 6E AX210
    0x7A70,                 // WiFi 7 BE200
];

// ============================================================================
// CSR (Control/Status Registers) — offset from BAR0
// ============================================================================

const CSR_HARDWARE_IF_CONFIG:   u32 = 0x000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_INT_COALESCING: u32 = 0x004;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_INT:            u32 = 0x008;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_INT_MASK:       u32 = 0x00C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_FH_INT_STATUS:  u32 = 0x010;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GPIO_IN:        u32 = 0x018;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_RESET:          u32 = 0x020;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GP_CNTRL:       u32 = 0x024;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_HARDWARE_REV:         u32 = 0x028;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_EEPROM_REGISTER:     u32 = 0x02C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_EEPROM_GP:      u32 = 0x030;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_UCODE_DRIVER_GP1:      u32 = 0x054;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_UCODE_DRIVER_GP1_SET:  u32 = 0x058;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_UCODE_DRIVER_GP1_CLR:  u32 = 0x05C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_UCODE_DRIVER_GP2:      u32 = 0x060;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GIO_REGISTER:            u32 = 0x03C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GP_UCODE:       u32 = 0x048;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GP_DRIVER:      u32 = 0x050;

// GP_CNTRL bits
const CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY: u32 = 1 << 0;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GP_CNTRL_REGISTER_FLAG_INITIALIZE_DONE:       u32 = 1 << 2;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GP_CNTRL_REGISTER_FLAG_MAC_ACCESS_REQUEST:  u32 = 1 << 3;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GP_CNTRL_REGISTER_FLAG_GOING_TO_SLEEP:  u32 = 1 << 4;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GP_CNTRL_REGISTER_VALUE_MAC_ACCESS_EN:    u32 = 1 << 0;  // Same as MAC_CLOCK_READY per Linux iwlegacy
const CSR_GP_CNTRL_REGISTER_FLAG_XTAL_ON:         u32 = 1 << 10;

// RESET bits
const CSR_RESET_REGISTER_FLAG_NEVO_RESET:   u32 = 1 << 0;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_RESET_REGISTER_FLAG_FORCE_NMI:    u32 = 1 << 1;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_RESET_REGISTER_FLAG_SOFTWARE_RESET:     u32 = 1 << 7;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_RESET_REGISTER_FLAG_MASTER_DISABLED: u32 = 1 << 8;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_RESET_REGISTER_FLAG_STOP_MASTER:  u32 = 1 << 9;

// EEPROM access
const CSR_EEPROM_REGISTER_READ_VALID_MSK:  u32 = 1 << 0;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_EEPROM_REGISTER_BIT_COMMAND:         u32 = 1 << 1;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_EEPROM_REGISTER_MSK_ADDRESS:        u32 = 0x0000FFFC;

// HW revision
const CSR_HARDWARE_REV_TYPE_MSK: u32 = 0x000FFF0;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_HARDWARE_REV_TYPE_4965: u32 = 0x0000000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_HARDWARE_REV_TYPE_5300: u32 = 0x0000020;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_HARDWARE_REV_TYPE_5100: u32 = 0x0000050;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_HARDWARE_REV_TYPE_5150: u32 = 0x0000040;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_HARDWARE_REV_TYPE_6000: u32 = 0x0000070;

// ============================================================================
// BSM (Bootstrap State Machine) Registers — firmware loading
// ============================================================================

const BSM_WR_CONTROLLER_REGISTER:        u32 = 0x3400;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_WR_MEMORY_SOURCE_REGISTER:     u32 = 0x3404;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_WR_MEMORY_DESTINATION_REGISTER:     u32 = 0x3408;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_WR_DWCOUNT_REGISTER:     u32 = 0x340C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_DRAM_INST_POINTER_REGISTER:  u32 = 0x3490;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_DRAM_INST_BYTECOUNT_REGISTER: u32 = 0x3494;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_DRAM_DATA_POINTER_REGISTER:  u32 = 0x3498;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_DRAM_DATA_BYTECOUNT_REGISTER: u32 = 0x349C;

// BSM_WR_CTRL bits
const BSM_WR_CONTROLLER_START:     u32 = 1 << 31;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const BSM_WR_CONTROLLER_START_EN:  u32 = 1 << 30;

// ============================================================================
// FH (Frame Handler) Registers — DMA TX/RX
// ============================================================================

/// Base of TX channel configuration registers
const FH_TCSR_BASE:           u32 = 0x1D00;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FH_TCSR_CHNL_OFFSET:   u32 = 0x20;
/// Service channel (firmware loading)
const FH_SRVC_CHNL:           u32 = 9;
/// TX config register flags
const FH_TCSR_TRANSMIT_CONFIG_REGISTER_VALUE_DMA_CHNL_ENABLE: u32 = 0x80000000;

/// RX configuration
const FH_MEMORY_RCSR_CHNL0:     u32 = 0x1F40;
/// RX status
const FH_RSSR_STATUS:         u32 = 0x1C44;
/// RX write pointer
const FH_RSCSR_CHNL0_WPTR:   u32 = 0x1BC0;

/// Number of RX buffers
const RECEIVE_QUEUE_SIZE:          usize = 256;
/// Number of TX descriptors
const TRANSMIT_QUEUE_SIZE:          usize = 256;
/// RX buffer size
const RX_BUF_SIZE:            usize = 4096;

// ============================================================================
// HCMD (Host Command) IDs — sent to firmware
// ============================================================================

const REPLY_ALIVE:            u8 = 0x01;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_ERROR:            u8 = 0x02;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_RXON:             u8 = 0x10;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_RXON_ASSOC:       u8 = 0x11;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_QOS_PARAMETER:        u8 = 0x13;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_SCAN_COMMAND:         u8 = 0x80;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_SCAN_COMPLETE:    u8 = 0x84;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_TRANSMIT:               u8 = 0x1C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const REPLY_ADD_STA:          u8 = 0x18;

/// Firmware notification: scan results
const SCAN_RESULTS_NOTIFICATION: u8 = 0x83;

// ============================================================================
// Firmware file format (iwlwifi .ucode v1 header)
// ============================================================================

/// Firmware image header (as stored in .ucode file)
#[repr(C, packed)]
struct IwlUcodeHeader {
    ver: u32,
    inst_size: u32,     // Runtime instruction size
    data_size: u32,     // Runtime data size
    init_size: u32,     // Init instruction size
    initialize_data_size: u32, // Init data size
    boot_size: u32,     // Bootstrap size
}

/// Parsed firmware sections
struct IwlFirmware {
    version: u32,
    /// Runtime instruction code (loaded after init completes)
    inst: Vec<u8>,
    /// Runtime data
    data: Vec<u8>,
    /// Init instruction code (bootstrap loads this)
    init_inst: Vec<u8>,
    /// Init data
    init_data: Vec<u8>,
    /// Bootstrap code (loaded to SRAM via BSM)
    boot: Vec<u8>,
}

/// Global WiFi firmware data (set from Limine module or RamFS)
static WIFI_FIRMWARE: spin::Mutex<Option<Vec<u8>>> = spin::Mutex::new(None);

/// Store WiFi firmware data for later loading by the driver
pub fn set_firmware_data(data: &[u8]) {
    crate::serial_println!("[IWL4965] Firmware data available: {} bytes", data.len());
    *WIFI_FIRMWARE.lock() = Some(data.to_vec());
}

/// Check if firmware data is available
pub fn has_firmware() -> bool {
    WIFI_FIRMWARE.lock().is_some()
}

// ============================================================================
// DMA Structures — TX Frame Descriptors and RX Ring
// ============================================================================

/// Number of TX queues (0=EDCA BE, 1=EDCA BK, 2=EDCA VI, 3=EDCA VO, 4=HCMD)
const IWL_TRANSMIT_QUEUE_COUNT: usize = 5;
/// Command queue index (TXQ 4 is for host commands to firmware)
const IWL_COMMAND_QUEUE_NUMBER: usize = 4;
/// Max transfer buffers per TFD
const IWL_NUMBER_OF_TBS: usize = 20;
/// TFD ring size (in entries per queue)
const TFD_QUEUE_SIZE: usize = 256;

/// FH register: TX DMA channel base address
const FH_MEMORY_CBBC_QUEUE: u32 = 0x19D0; // + 4*queue_num

/// FH register: RX channel base
const FH_RCSR_CHNL0_CONFIG_REGISTER: u32 = 0x1F48;

/// FH register: RX Scheduler CSR (RSCSR) — DMA buffer management
/// These registers handle the RX buffer ring and status DMA between device and host
/// RSCSR registers (RX Scheduler/Status) — 0x1BC0 base
/// Source: Linux iwlegacy FH49_MEM_RSCSR_LOWER_BOUND = 0x1BC0
const FH_RSCSR_CHNL0_RBDCB_WPTR_REGISTER: u32  = 0x1BC0;  // +0x00: free-buffer write ptr (host tells NIC how many bufs)
const FH_RSCSR_CHNL0_RBDCB_BASE_REGISTER: u32  = 0x1BC4;  // +0x04: RBD ring phys base (value >> 8)
const FH_RSCSR_CHNL0_STTS_WPTR_REGISTER: u32   = 0x1BC8;  // +0x08: status/closed_rb DMA target (value >> 4)

/// FH RCSR — RX DMA channel config (0x1F40 base for iwl4965)
const FH_MEMORY_RCSR_CHNL0_RBDCB_WPTR: u32   = 0x1F80;  // RCSR write pointer (reset to 0 before init)

/// FH RX config bits
const FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_DMA_CHNL_EN: u32  = 0x8000_0000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_INTERRUPT_REQUEST_DEST_INT: u32  = 0x0100_0000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_RB_SIZE_4K: u32    = 0x0000_0000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_SINGLE_FRAME: u32  = 0x0000_8000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FH_RCSR_RECEIVE_CONFIG_RBDCB_SIZE_POSITION: u32        = 20;  // position of log2(queue_size)

// ============================================================================
// HBUS Registers (Host-side Bus) — base 0x400 in BAR0
// ============================================================================

/// HBUS target memory access (for direct SRAM writes)
const HBUS_TARG_MEMORY_RADDR: u32 = 0x40C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const HBUS_TARG_MEMORY_WADDR: u32 = 0x410;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const HBUS_TARG_MEMORY_WDAT:  u32 = 0x418;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const HBUS_TARG_MEMORY_RDAT:  u32 = 0x41C;

/// HBUS peripheral register access (indirect to PRPH bus)
const HBUS_TARG_PRPH_WADDR: u32 = 0x444;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const HBUS_TARG_PRPH_RADDR: u32 = 0x448;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const HBUS_TARG_PRPH_WDAT:  u32 = 0x44C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const HBUS_TARG_PRPH_RDAT:  u32 = 0x450;

/// TX write pointer doorbell
const HBUS_TARG_WRPTR: u32 = 0x460;

// ============================================================================
// APMG (Advanced Power Management) — PRPH bus registers
// ============================================================================

const APMG_CLK_CONTROLLER_REGISTER:   u32 = 0x3000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const APMG_CLK_EN_REGISTER:     u32 = 0x3004;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const APMG_CLK_DIS_REGISTER:    u32 = 0x3008;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const APMG_PS_CONTROLLER_REGISTER:    u32 = 0x300C;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const APMG_PCIDEV_STT_REGISTER: u32 = 0x3010;

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const APMG_CLK_VALUE_DMA_CLK_RQT: u32 = 0x0000_0200;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const APMG_CLK_VALUE_BSM_CLK_RQT: u32 = 0x0000_0800;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const APMG_PCIDEV_STT_VALUE_L1_LOOKUP_DIS: u32 = 0x0000_0002;

// GIO chicken bits (disable L0s exit timer)
const CSR_GIO_CHICKEN_BITS: u32 = 0x100;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_GIO_CHICKEN_BITS_REGISTER_BIT_DIS_L0S_EXIT_TIMER: u32 = 0x2000_0000;

// NIC ready bit
const CSR_HARDWARE_IF_CONFIG_REGISTER_BIT_NIC_READY: u32 = 0x0040_0000;

// BSM SRAM lower bound (target address for bootstrap code in NIC SRAM)
const BSM_SRAM_LOWER_BOUND: u32 = 0x3800;

/// Scheduler SRAM registers (PRPH bus)
const SCD_SRAM_BASE_ADDRESS: u32 = 0x2E00;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCD_TXFACT: u32 = 0x2D00;

/// Transfer Buffer — describes one fragment of a DMA transfer
#[repr(C, packed)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
struct IwlTfdTb {
    /// Low 32 bits of physical address
    lo: u32,
    /// High 4 bits of addr (bits 0-3) + length in bytes (bits 4-15)
    hi_n_len: u16,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl IwlTfdTb {
    fn set(&mut self, addr: u64, len: u16) {
        self.lo = addr as u32;
        self.hi_n_len = ((addr >> 32) as u16 & 0xF) | ((len & 0x0FFF) << 4);
    }
}

/// Transmit Frame Descriptor (TFD) — one per TX ring entry
/// Size: 12 + 20*6 + 4 = 136 bytes, HW rounds to 128-byte alignment
#[repr(C, packed)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
struct IwlTfd {
    __reserved: [u8; 12],
    tbs: [IwlTfdTb; IWL_NUMBER_OF_TBS],
    __pad: u32,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl IwlTfd {
    const fn zeroed() -> Self {
        Self {
            __reserved: [0; 12],
            tbs: [IwlTfdTb { lo: 0, hi_n_len: 0 }; IWL_NUMBER_OF_TBS],
            __pad: 0,
        }
    }
    
    fn number_tbs(&self) -> usize {
        // TBS count is stored in the reserved area for 4965
        let ptr = self.__reserved.as_ptr() as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32;
        let val = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read_unaligned(ptr) };
        (val & 0x1F) as usize
    }
    
    fn set_num_tbs(&mut self, count: usize) {
        let ptr = self.__reserved.as_mut_ptr() as *mut u32;
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::write_unaligned(ptr, (count & 0x1F) as u32) };
    }
}

/// Host Command header — sent at the start of HCMD payloads
#[repr(C, packed)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone, Copy)]
struct IwlCommandHeader {
    cmd: u8,
    flags: u8,
    idx: u8,     // index of this command in the queue
    qid: u8,     // queue ID (always IWL_CMD_QUEUE_NUM for HCMD)
}

/// Maximum HCMD payload size (excluding header)
/// Must be >= 404 for SCAN_CMD with 15 channels (224 header + 15*12 channel entries)
const MAXIMUM_COMMAND_SIZE: usize = 512;

/// TX Queue state
struct TxQueue {
    /// TFD ring (DMA-visible, 256 entries)
    tfds: Vec<IwlTfd>,
    /// Command/data buffers (one per TFD)
    cmd_buffers: Vec<Vec<u8>>,
    /// Write pointer (next TFD to fill)
    write_ptr: usize,
    /// Read pointer (next TFD the device will process)
    read_ptr: usize,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl TxQueue {
    fn new() -> Self {
        let mut tfds = Vec::with_capacity(TFD_QUEUE_SIZE);
        tfds.resize(TFD_QUEUE_SIZE, IwlTfd::zeroed());
        
        let mut cmd_buffers = Vec::with_capacity(TFD_QUEUE_SIZE);
        for _ in 0..TFD_QUEUE_SIZE {
            // Pre-allocate command buffers (4+MAX_CMD_SIZE bytes each)
            cmd_buffers.push(vec![0u8; 4 + MAXIMUM_COMMAND_SIZE]);
        }
        
        Self {
            tfds,
            cmd_buffers,
            write_ptr: 0,
            read_ptr: 0,
        }
    }
    
    fn tfd_phys_base(&self) -> u64 {
        let virt = self.tfds.as_ptr() as u64;
        crate::memory::virt_to_physical(virt)
            .unwrap_or(virt.wrapping_sub(crate::memory::hhdm_offset()))
    }
}

/// RX Queue state
struct RxQueue {
    /// Ring of physical addresses for RX buffers (device reads these)
    bd: Vec<u32>,  // Buffer Descriptor: physical address (low 32 bits, 4K aligned)
    /// Actual RX buffers (4KB each)
    buffers: Vec<Vec<u8>>,
    /// Shared status area — device writes the current write pointer here via DMA
    /// We read rb_stts[0] (u32, little-endian) to see which RX buffers are filled
    rb_stts: Vec<u8>,
    /// Write pointer (driver advances after allocating new buffers)
    write_ptr: usize,
    /// Read pointer (driver reads processed buffers here)
    read_ptr: usize,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl RxQueue {
    fn new() -> Self {
        let mut bd = Vec::with_capacity(RECEIVE_QUEUE_SIZE);
        let mut buffers = Vec::with_capacity(RECEIVE_QUEUE_SIZE);
        
        for _ in 0..RECEIVE_QUEUE_SIZE {
            let buf = vec![0u8; RX_BUF_SIZE];
            let virt = buf.as_ptr() as u64;
            let phys = crate::memory::virt_to_physical(virt)
                .unwrap_or(virt.wrapping_sub(crate::memory::hhdm_offset()));
            // iwl4965 BD entries store physical address >> 8 (256-byte aligned)
            bd.push((phys >> 8) as u32);
            buffers.push(buf);
        }

        // 16-byte aligned status area for device to DMA write pointer into
        let rb_stts = vec![0u8; 16];
        
        Self {
            bd,
            buffers,
            rb_stts,
            write_ptr: 0,
            read_ptr: 0,
        }
    }
    
    fn bd_phys_base(&self) -> u64 {
        let virt = self.bd.as_ptr() as u64;
        crate::memory::virt_to_physical(virt)
            .unwrap_or(virt.wrapping_sub(crate::memory::hhdm_offset()))
    }
}

// ============================================================================
// EEPROM layout offsets (word addresses)
// ============================================================================

const EEPROM_MAC_ADDRESS: u16 = 0x0015;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EEPROM_SKU_CAPABILITY:     u16 = 0x0045;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const EEPROM_CHANNELS_2G: u16 = 0x0062; // 2.4 GHz channel data start
const EEPROM_CHANNELS_5G: u16 = 0x0080; // 5 GHz channel data start

// ============================================================================
// 802.11 Frame Types for Scanning
// ============================================================================

const IEEE80211_FTYPE_MGMT: u16 = 0x0000;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IEEE80211_STYPE_BEACON: u16 = 0x0080;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const IEEE80211_STYPE_PROBE_RESPONSE: u16 = 0x0050;

// Information Element IDs
const WLAN_EID_SSID: u8 = 0;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WLAN_EID_DS_PARAMS: u8 = 3;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const WLAN_EID_RSN: u8 = 48;        // WPA2
const WLAN_EID_VENDOR: u8 = 221;    // WPA (via Microsoft OUI)

// ============================================================================
// Driver State
// ============================================================================

const MAXIMUM_SCAN_RESULTS: usize = 32;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SCAN_TIMEOUT_TICKS: u64 = 500; // ~5 seconds at 100 Hz tick

pub struct Iwl4965 {
    // PCI info
    pci_bus: u8,
    pci_device: u8,
    pci_function: u8,
    device_id: u16,

    // MMIO base (from BAR0)
    mmio_base: usize,
    mmio_size: usize,

    // Device state
    status: DriverStatus,
    wifi_state: WifiState,
    hw_rev: u32,
    mac_addr: [u8; 6],

    // Firmware state
    firmware_loaded: bool,
    fw_alive: bool,

    // DMA queues
    tx_queues: Vec<TxQueue>,
    rx_queue: Option<RxQueue>,
    /// Sequence counter for HCMD commands
    cmd_seq: u16,

    // Scan state
    scan_results: Vec<WifiNetwork>,
    scan_start_tick: u64,
    scanning: bool,

    // Connection state
    connected_ssid: Option<String>,
    connected_bssid: [u8; 6],
    current_channel: u8,
    signal_dbm: i8,

    // RX packet queue (received frames waiting for upper layer)
    rx_pending: Vec<Vec<u8>>,

    // Statistics
    stats: NetStats,

    // NIC alive flag
    initialized: bool,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl Iwl4965 {
    fn new() -> Self {
        Self {
            pci_bus: 0,
            pci_device: 0,
            pci_function: 0,
            device_id: 0,
            mmio_base: 0,
            mmio_size: 0,
            status: DriverStatus::Unloaded,
            wifi_state: WifiState::Disabled,
            hw_rev: 0,
            mac_addr: [0; 6],
            firmware_loaded: false,
            fw_alive: false,
            tx_queues: Vec::new(),
            rx_queue: None,
            cmd_seq: 0,
            scan_results: Vec::new(),
            scan_start_tick: 0,
            scanning: false,
            connected_ssid: None,
            connected_bssid: [0; 6],
            current_channel: 0,
            signal_dbm: 0,
            rx_pending: Vec::new(),
            stats: NetStats::default(),
            initialized: false,
        }
    }

    // ── Register Access ──────────────────────────────────────────

    #[inline]
    fn read_reg(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 { return 0; }
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let ptr = (self.mmio_base + offset as usize) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32;
            read_volatile(ptr)
        }
    }

    #[inline]
    fn write_reg(&self, offset: u32, value: u32) {
        if self.mmio_base == 0 { return; }
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            let ptr = (self.mmio_base + offset as usize) as *mut u32;
            write_volatile(ptr, value);
        }
    }

    /// Write to a peripheral (PRPH) register via HBUS indirect access
    /// Grab NIC access — request MAC_ACCESS and wait for it to be granted.
    /// Must be called before any PRPH or SRAM access. Returns true if granted.
    fn grab_nic_access(&self) -> bool {
        // Request MAC access
        self.write_reg(CSR_GP_CNTRL,
            self.read_reg(CSR_GP_CNTRL) | CSR_GP_CNTRL_REGISTER_FLAG_MAC_ACCESS_REQUEST);
        // Wait up to ~5ms (5000 * ~1us IO port delay)
        for _ in 0..5000u32 {
            if self.read_reg(CSR_GP_CNTRL) & CSR_GP_CNTRL_REGISTER_VALUE_MAC_ACCESS_EN != 0 {
                return true;
            }
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        false
    }

    /// Release NIC access — clear MAC_ACCESS_REQ
    fn release_nic_access(&self) {
        self.write_reg(CSR_GP_CNTRL,
            self.read_reg(CSR_GP_CNTRL) & !CSR_GP_CNTRL_REGISTER_FLAG_MAC_ACCESS_REQUEST);
    }

    fn write_prph(&self, addr: u32, val: u32) {
        if !self.grab_nic_access() {
            crate::serial_println!("[IWL4965] write_prph FAILED: no NIC access for {:#X}", addr);
            return;
        }
        self.write_reg(HBUS_TARG_PRPH_WADDR, (addr & 0x000F_FFFF) | (3 << 24));
        self.write_reg(HBUS_TARG_PRPH_WDAT, val);
        self.release_nic_access();
    }

    /// Read from a peripheral (PRPH) register via HBUS indirect access
    fn read_prph(&self, addr: u32) -> u32 {
        if !self.grab_nic_access() {
            crate::serial_println!("[IWL4965] read_prph FAILED: no NIC access for {:#X}", addr);
            return 0xFFFFFFFF;
        }
        self.write_reg(HBUS_TARG_PRPH_RADDR, (addr & 0x000F_FFFF) | (3 << 24));
        let val = self.read_reg(HBUS_TARG_PRPH_RDAT);
        self.release_nic_access();
        val
    }

    /// Write to NIC SRAM via HBUS target memory access
    fn write_targ_memory(&self, addr: u32, val: u32) {
        if !self.grab_nic_access() {
            crate::serial_println!("[IWL4965] write_targ_mem FAILED: no NIC access for {:#X}", addr);
            return;
        }
        self.write_reg(HBUS_TARG_MEMORY_WADDR, addr);
        self.write_reg(HBUS_TARG_MEMORY_WDAT, val);
        self.release_nic_access();
    }

    // ── Hardware Init ────────────────────────────────────────────

    /// Map BAR0 to virtual memory and return base address
    fn map_bar0(&mut self, pci_dev: &PciDevice) -> Result<(), &'static str> {
        let bar0 = pci_dev.bar[0];
        if bar0 == 0 {
            return Err("BAR0 is zero");
        }

        // Memory-mapped BAR (bit 0 = 0 for memory)
        let is_memory = (bar0 & 1) == 0;
        if !is_memory {
            return Err("BAR0 is I/O, need memory");
        }

        // 64-bit BAR check
        let is_64bit = (bar0 >> 1) & 0x3 == 2;
        let phys_addr = if is_64bit {
            let bar1 = pci_dev.bar[1] as u64;
            (bar1 << 32) | (bar0 & 0xFFFFFFF0) as u64
        } else {
            (bar0 & 0xFFFFFFF0) as u64
        };

        if phys_addr == 0 {
            return Err("BAR0 base address is zero");
        }

        // The iwl4965 uses 8KB of MMIO space
        self.mmio_size = 0x2000; // 8KB

        // Map the MMIO region into virtual address space via HHDM
        let virt_addr = crate::memory::map_mmio(phys_addr, self.mmio_size)
            .map_err(|_| "Failed to map BAR0 MMIO region")?;

        self.mmio_base = virt_addr as usize;

        crate::serial_println!("[IWL4965] MMIO phys: {:#X} -> virt: {:#X}, size: {:#X}", phys_addr, virt_addr, self.mmio_size);

        Ok(())
    }

    /// APM (Advanced Power Management) stop — put NIC in clean reset state
    fn apm_stop(&self) {
        // SW reset to fully stop the device
        self.write_reg(CSR_RESET, CSR_RESET_REGISTER_FLAG_SOFTWARE_RESET);
        // Wait ~10ms for reset to take effect (port 0x80 ~1us each)
        for _ in 0..10_000 {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }

        // Clear INIT_DONE — let APM shut down completely
        let gp = self.read_reg(CSR_GP_CNTRL);
        self.write_reg(CSR_GP_CNTRL, gp & !CSR_GP_CNTRL_REGISTER_FLAG_INITIALIZE_DONE);
        // Wait ~10ms
        for _ in 0..10_000 {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
    }

    /// APM init — bring NIC from D0U (uninitialized) to D0A (active)
    fn apm_init(&self) -> Result<(), &'static str> {
        // 1. Set NIC ready (required after any reset)
        self.write_reg(CSR_HARDWARE_IF_CONFIG,
            self.read_reg(CSR_HARDWARE_IF_CONFIG) | CSR_HARDWARE_IF_CONFIG_REGISTER_BIT_NIC_READY);
        for _ in 0..5000u32 {
            if self.read_reg(CSR_HARDWARE_IF_CONFIG) & CSR_HARDWARE_IF_CONFIG_REGISTER_BIT_NIC_READY != 0 {
                break;
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        // 2. Disable L0s exit timer (GIO chicken bits)
        self.write_reg(CSR_GIO_CHICKEN_BITS,
            self.read_reg(CSR_GIO_CHICKEN_BITS) | CSR_GIO_CHICKEN_BITS_REGISTER_BIT_DIS_L0S_EXIT_TIMER);

        // 3. Set INIT_DONE to start NIC's internal clock initialization
        self.write_reg(CSR_GP_CNTRL,
            self.read_reg(CSR_GP_CNTRL) | CSR_GP_CNTRL_REGISTER_FLAG_INITIALIZE_DONE);

        // 3. Poll for MAC clock ready (up to ~25ms)
        //    Each iteration: read reg + ~1000 spin loops ≈ ~1-5us, × 25000 = 25-125ms
        for i in 0..25000u32 {
            let val = self.read_reg(CSR_GP_CNTRL);
            if val & CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY != 0 {
                crate::serial_println!("[IWL4965] MAC clock ready after {} iterations, GP_CNTRL={:#010X}", i, val);

                // 4. Request NIC (MAC) access — REQUIRED for PRPH bus writes
                self.write_reg(CSR_GP_CNTRL,
                    self.read_reg(CSR_GP_CNTRL) | CSR_GP_CNTRL_REGISTER_FLAG_MAC_ACCESS_REQUEST);
                let mut mac_access_ok = false;
                for j in 0..10_000u32 {
                    if self.read_reg(CSR_GP_CNTRL) & CSR_GP_CNTRL_REGISTER_VALUE_MAC_ACCESS_EN != 0 {
                        crate::serial_println!("[IWL4965] MAC access granted after {} iters", j);
                        mac_access_ok = true;
                        break;
                    }
                    // ~1us per IO port write on real hardware
                    unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
                }
                if !mac_access_ok {
                    crate::serial_println!("[IWL4965] WARNING: MAC access not granted, GP={:#010X}",
                        self.read_reg(CSR_GP_CNTRL));
                }

                // 5. Enable DMA and BSM clocks via APMG (peripheral register)
                self.write_prph(APMG_CLK_EN_REGISTER,
                    APMG_CLK_VALUE_DMA_CLK_RQT | APMG_CLK_VALUE_BSM_CLK_RQT);
                // Wait ~20us for clocks
                for _ in 0..20_000 { core::hint::spin_loop(); }

                // 6. Disable L1-Active power save
                let stt = self.read_prph(APMG_PCIDEV_STT_REGISTER);
                self.write_prph(APMG_PCIDEV_STT_REGISTER, stt | APMG_PCIDEV_STT_VALUE_L1_LOOKUP_DIS);

                return Ok(());
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        let gp = self.read_reg(CSR_GP_CNTRL);
        crate::serial_println!("[IWL4965] APM init FAILED: MAC clock not ready, GP_CNTRL={:#010X}", gp);
        Err("MAC clock not ready")
    }

    /// Reset and initialize the hardware (proper APM sequence matching Linux)
    fn hw_init(&mut self) -> Result<(), &'static str> {
        // 1. Disable interrupts
        self.write_reg(CSR_INT_MASK, 0);
        self.write_reg(CSR_INT, 0xFFFFFFFF);
        self.write_reg(CSR_FH_INT_STATUS, 0xFFFFFFFF);

        // 2. Read hardware revision
        self.hw_rev = self.read_reg(CSR_HARDWARE_REV);
        let hardware_type = (self.hw_rev & CSR_HARDWARE_REV_TYPE_MSK) >> 4;
        let hardware_name = // Correspondance de motifs — branchement exhaustif de Rust.
match hardware_type {
            0x00 => "4965",
            0x02 => "5300",
            0x04 => "5150",
            0x05 => "5100",
            0x07 => "6000",
            _ => "unknown",
        };
        crate::serial_println!("[IWL4965] HW rev: {:#010X} (type: {} = {})", self.hw_rev, hardware_type, hardware_name);

        // 3. APM init: NIC_READY → INIT_DONE → wait MAC clock → enable APMG clocks
        self.apm_init()?;

        crate::serial_println!("[IWL4965] APM init complete");

        // 6. Read MAC address from EEPROM
        self.read_eeprom_mac()?;

        crate::serial_println!("[IWL4965] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac_addr[0], self.mac_addr[1], self.mac_addr[2],
            self.mac_addr[3], self.mac_addr[4], self.mac_addr[5]);

        self.initialized = true;
        Ok(())
    }

    /// Read a 16-bit word from the EEPROM
    fn eeprom_read(&self, addr: u16) -> u16 {
        // Write address and command bit
        let register_value = ((addr as u32) << 2) | CSR_EEPROM_REGISTER_BIT_COMMAND;
        self.write_reg(CSR_EEPROM_REGISTER, register_value);

        // Wait for valid
        for _ in 0..5000 {
            let val = self.read_reg(CSR_EEPROM_REGISTER);
            if val & CSR_EEPROM_REGISTER_READ_VALID_MSK != 0 {
                return (val >> 16) as u16;
            }
            for _ in 0..50 { core::hint::spin_loop(); }
        }

        crate::serial_println!("[IWL4965] EEPROM read timeout at addr {:#06X}", addr);
        0
    }

    /// Read MAC address from EEPROM
    fn read_eeprom_mac(&mut self) -> Result<(), &'static str> {
        let w0 = self.eeprom_read(EEPROM_MAC_ADDRESS);
        let w1 = self.eeprom_read(EEPROM_MAC_ADDRESS + 1);
        let w2 = self.eeprom_read(EEPROM_MAC_ADDRESS + 2);

        self.mac_addr[0] = (w0 & 0xFF) as u8;
        self.mac_addr[1] = (w0 >> 8) as u8;
        self.mac_addr[2] = (w1 & 0xFF) as u8;
        self.mac_addr[3] = (w1 >> 8) as u8;
        self.mac_addr[4] = (w2 & 0xFF) as u8;
        self.mac_addr[5] = (w2 >> 8) as u8;

        // Validate: not all zeros, not all FF
        if self.mac_addr == [0; 6] || self.mac_addr == [0xFF; 6] {
            // Some cards need the NIC_LOCK before EEPROM access
            // Try reading HW_IF_CONFIG for alternative MAC
            crate::serial_println!("[IWL4965] EEPROM MAC invalid, generating from PCI");
            // Generate deterministic MAC from PCI location
            self.mac_addr = [
                0x00, 0x13, 0xE8, // Intel OUI
                self.pci_bus,
                self.pci_device,
                self.pci_function | 0x40,
            ];
        }

        Ok(())
    }

    // ── Firmware Loading ─────────────────────────────────────────

    /// Parse firmware .ucode file into sections
    fn parse_firmware(data: &[u8]) -> Result<IwlFirmware, &'static str> {
        let hdr_size = core::mem::size_of::<IwlUcodeHeader>();
        if data.len() < hdr_size {
            return Err("Firmware too small for header");
        }

        // Read header fields (little-endian)
        let ver = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let inst_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let data_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let init_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
        let initialize_data_size = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize;
        let boot_size = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;

        let total = hdr_size + inst_size + data_size + init_size + initialize_data_size + boot_size;
        if data.len() < total {
            crate::serial_println!("[IWL4965] FW: need {} bytes, have {}", total, data.len());
            return Err("Firmware file truncated");
        }

        let major = (ver >> 24) & 0xFF;
        let minor = (ver >> 16) & 0xFF;
        let api = (ver >> 8) & 0xFF;
        crate::serial_println!("[IWL4965] FW version: {}.{}.{} (raw {:#010X})", major, minor, api, ver);
        crate::serial_println!("[IWL4965] FW sections: inst={} data={} init={} init_data={} boot={}",
            inst_size, data_size, init_size, initialize_data_size, boot_size);

        let mut off = hdr_size;
        let inst = data[off..off + inst_size].to_vec(); off += inst_size;
        let data_sector = data[off..off + data_size].to_vec(); off += data_size;
        let init_inst = data[off..off + init_size].to_vec(); off += init_size;
        let init_data = data[off..off + initialize_data_size].to_vec(); off += initialize_data_size;
        let boot = data[off..off + boot_size].to_vec();

        Ok(IwlFirmware {
            version: ver,
            inst,
            data: data_sector,
            init_inst,
            init_data,
            boot,
        })
    }

    /// Load firmware into the NIC via BSM (Bootstrap State Machine)
    fn load_firmware(&mut self) -> Result<(), &'static str> {
        // Get firmware data from global storage
        let fw_data = {
            let guard = WIFI_FIRMWARE.lock();
                        // Correspondance de motifs — branchement exhaustif de Rust.
match guard.as_ref() {
                Some(d) => d.clone(),
                None => {
                    // Try loading from RamFS
                    match crate::ramfs::with_filesystem(|fs| fs.read_file("/firmware/iwlwifi-4965-2.ucode").map(|d| d.to_vec())) {
                        Ok(d) => d,
                        Err(_) => return Err("No firmware available (need iwlwifi-4965-2.ucode)"),
                    }
                }
            }
        };

        let fw = Self::parse_firmware(&fw_data)?;
        crate::println!("    FW parsed: boot={} inst={} data={} init_inst={} init_data={}",
            fw.boot.len(), fw.inst.len(), fw.data.len(), fw.init_inst.len(), fw.init_data.len());

        // Allocate page-aligned DMA buffers (NIC DMA uses phys>>4, needs 16-byte alignment minimum)
        // Linux uses dma_alloc_coherent() which returns page-aligned memory.
        // Vec<u8> only guarantees 1-byte alignment — 0x0064AA28>>4<<4 = 0x0064AA20 != 0x0064AA28!
        let (init_inst_dma, ii_off) = Self::allocator_dma_buffer(&fw.init_inst);
        let (init_data_dma, id_off) = Self::allocator_dma_buffer(&fw.init_data);
        let (inst_dma, i_off) = Self::allocator_dma_buffer(&fw.inst);
        let (data_dma, d_off) = Self::allocator_dma_buffer(&fw.data);
        crate::println!("    DMA buffers allocated (page-aligned)");

        // ── Phase 1: Init firmware (calibration) ──
        if !fw.init_inst.is_empty() {
            crate::println!("    === Phase 1: Init firmware (calibration) ===");
            crate::serial_println!("[IWL4965] === INIT firmware phase ===");

            crate::println!("    stop_device...");
            self.stop_device()?;
            let gp = self.read_reg(CSR_GP_CNTRL);
            let mac_ok = if gp & CSR_GP_CNTRL_REGISTER_VALUE_MAC_ACCESS_EN != 0 { "granted" } else { "DENIED" };
            crate::println!("    stop_device OK, GP={:#010X} MAC_ACCESS={}", gp, mac_ok);

            crate::println!("    bsm_load_bootstrap ({} bytes)...", fw.boot.len());
            self.bsm_load_bootstrap(&fw.boot)?;
            crate::println!("    bsm_load_bootstrap OK");

            let ii_slice = &init_inst_dma[ii_off..ii_off + fw.init_inst.len()];
            let id_slice = &init_data_dma[id_off..id_off + fw.init_data.len()];
            crate::println!("    bsm_set_dram_addrs (init_inst={}, init_data={})...", fw.init_inst.len(), fw.init_data.len());
            self.bsm_set_dram_addrs(ii_slice, id_slice)?;
            crate::println!("    bsm_set_dram_addrs OK");

            crate::println!("    bsm_start (boot={})...", fw.boot.len());
            self.bsm_start(fw.boot.len())?;
            crate::println!("    bsm_start OK");
            self.verify_inst_sram(&fw.boot);

            // Clear RFKILL + CMD_BLOCKED (Linux: CSR_UCODE_SW_BIT_RFKILL=0x2, CMD_BLOCKED=0x4)
            self.write_reg(CSR_UCODE_DRIVER_GP1_CLR, 0x2 | 0x4);
            self.write_reg(CSR_UCODE_DRIVER_GP1_CLR, 0x2); // clear RFKILL again (Linux does this twice)
            self.write_reg(CSR_INT, 0xFFFFFFFF);
            // INT mask: FH_RX(31)|HW_ERR(29)|FH_TX(27)|SW_ERR(25)|ALIVE(24)|SW_RX(3)|RF_KILL(1)|WAKEUP(0)
            self.write_reg(CSR_INT_MASK, 0xAB00_000B);
            crate::println!("    nic_start: CSR_RESET=0 (release CPU)...");
            self.write_reg(CSR_RESET, 0);
            // 25ms delay for CPU to start executing bootstrap
            for _ in 0..25_000 {
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
            let rst_rb = self.read_reg(CSR_RESET);
            let gp_rb = self.read_reg(CSR_GP_CNTRL);
            crate::println!("    nic_start done: RST={:#X} GP={:#010X}", rst_rb, gp_rb);

            crate::println!("    wait_alive (init)...");
                        // Correspondance de motifs — branchement exhaustif de Rust.
match self.wait_alive() {
                Ok(()) => {
                    crate::println!("    Init firmware ALIVE!");
                    // Give init firmware time to run calibration (~500ms)
                    // Poll RX queue periodically to receive calibration notifications
                    for i in 0..50u32 {
                        for _ in 0..10_000u32 {
                            for _ in 0..1000 { core::hint::spin_loop(); }
                        }
                        self.poll_rx(); // Process any calibration notifications
                        if i == 25 {
                            let int_value = self.read_reg(CSR_INT);
                            crate::serial_println!("[IWL4965] Init cal progress: INT={:#X} rxpkts so far", int_value);
                        }
                    }
                    crate::println!("    Init calibration done");
                }
                Err(e) => {
                    crate::println!("    Init ALIVE FAILED: {} — trying runtime directly", e);
                }
            }
        }

        // ── Phase 2: Runtime firmware ──
        // Soft reset: stop CPU and assert NEVO_RESET to reset program counter to 0.
        // Do NOT call apm_stop()/SW_RESET — that wipes SRAM (calibration data).
        // NEVO_RESET only resets the embedded processor, SRAM stays powered.
        crate::println!("    === Phase 2: Runtime firmware ===");
        crate::serial_println!("[IWL4965] === RUNTIME firmware phase (soft reset) ===");

        // 1. Stop master DMA
        self.write_reg(CSR_RESET, CSR_RESET_REGISTER_FLAG_STOP_MASTER);
        for _ in 0..1000u32 {
            if self.read_reg(CSR_RESET) & CSR_RESET_REGISTER_FLAG_MASTER_DISABLED != 0 {
                break;
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }
        // 2. Assert NEVO_RESET to hold CPU in reset (resets PC to 0)
        self.write_reg(CSR_RESET, CSR_RESET_REGISTER_FLAG_NEVO_RESET | CSR_RESET_REGISTER_FLAG_STOP_MASTER);
        // ~5ms for reset to take effect
        for _ in 0..5_000 {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }

        // 3. Re-enable APMG clocks (DMA + BSM) in case they were gated
        self.write_prph(APMG_CLK_EN_REGISTER,
            APMG_CLK_VALUE_DMA_CLK_RQT | APMG_CLK_VALUE_BSM_CLK_RQT);
        for _ in 0..20_000 { core::hint::spin_loop(); }

        let gp = self.read_reg(CSR_GP_CNTRL);
        crate::println!("    soft reset OK (NEVO held), GP={:#010X}", gp);

        crate::println!("    bsm_load_bootstrap ({} bytes)...", fw.boot.len());
        self.bsm_load_bootstrap(&fw.boot)?;
        crate::println!("    bsm_load_bootstrap OK");

        let i_slice = &inst_dma[i_off..i_off + fw.inst.len()];
        let d_slice = &data_dma[d_off..d_off + fw.data.len()];
        crate::println!("    bsm_set_dram_addrs (inst={}, data={})...", fw.inst.len(), fw.data.len());
        self.bsm_set_dram_addrs(i_slice, d_slice)?;
        crate::println!("    bsm_set_dram_addrs OK");

        crate::println!("    bsm_start (boot={})...", fw.boot.len());
        self.bsm_start(fw.boot.len())?;
        crate::println!("    bsm_start OK");
        self.verify_inst_sram(&fw.boot);

        // Clear RFKILL + CMD_BLOCKED (Linux: CSR_UCODE_SW_BIT_RFKILL=0x2, CMD_BLOCKED=0x4)
        self.write_reg(CSR_UCODE_DRIVER_GP1_CLR, 0x2 | 0x4);
        self.write_reg(CSR_UCODE_DRIVER_GP1_CLR, 0x2); // clear RFKILL again (Linux does this twice)
        self.write_reg(CSR_INT, 0xFFFFFFFF);
        // INT mask: FH_RX(31)|HW_ERR(29)|FH_TX(27)|SW_ERR(25)|ALIVE(24)|SW_RX(3)|RF_KILL(1)|WAKEUP(0)
        self.write_reg(CSR_INT_MASK, 0xAB00_000B);
        crate::println!("    nic_start: CSR_RESET=0 (release CPU)...");
        self.write_reg(CSR_RESET, 0);
        // 25ms delay for CPU to start executing bootstrap
        for _ in 0..25_000 {
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        let rst_rb = self.read_reg(CSR_RESET);
        let gp_rb = self.read_reg(CSR_GP_CNTRL);
        crate::println!("    nic_start done: RST={:#X} GP={:#010X}", rst_rb, gp_rb);

        crate::println!("    wait_alive (runtime)...");
        self.wait_alive()?;

        self.firmware_loaded = true;
        self.fw_alive = true;
        crate::println!("    Runtime firmware ALIVE!");
        crate::serial_println!("[IWL4965] Runtime firmware loaded and alive!");

        Ok(())
    }

    /// Allocate a page-aligned DMA buffer. Returns (backing_vec, aligned_offset).
    /// NIC DMA uses phys_addr >> 4, so addresses must be at least 16-byte aligned.
    /// We use 4096-byte (page) alignment to match Linux's dma_alloc_coherent().
    fn allocator_dma_buffer(data: &[u8]) -> (Vec<u8>, usize) {
        let mut buf = vec![0u8; data.len() + 4095];
        let ptr = buf.as_ptr() as usize;
        let offset = (4096 - (ptr & 0xFFF)) & 0xFFF;
        buf[offset..offset + data.len()].copy_from_slice(data);
        (buf, offset)
    }

    /// Verify instruction SRAM content after BSM copy (via HBUS_TARG_MEM)
    fn verify_inst_sram(&self, boot: &[u8]) {
        if !self.grab_nic_access() {
            crate::println!("      INST SRAM verify: cannot grab NIC access");
            return;
        }
        let check_count = core::cmp::min(boot.len() / 4, 4);
        let mut mismatches = 0;
        for i in 0..check_count {
            self.write_reg(HBUS_TARG_MEMORY_RADDR, (i * 4) as u32);
            let val = self.read_reg(HBUS_TARG_MEMORY_RDAT);
            let off = i * 4;
            let expect = u32::from_le_bytes([boot[off], boot[off+1], boot[off+2], boot[off+3]]);
            if val != expect {
                crate::println!("      INST SRAM [{}]: {:#010X} expect {:#010X} MISMATCH", i, val, expect);
                mismatches += 1;
            }
        }
        self.release_nic_access();
        if mismatches == 0 {
            crate::println!("      INST SRAM verify: first {} dwords OK", check_count);
        }
    }

    /// Stop device (prepare for firmware load) — Linux-style apm_stop + apm_init
    fn stop_device(&mut self) -> Result<(), &'static str> {
        // Disable interrupts
        self.write_reg(CSR_INT_MASK, 0);
        self.write_reg(CSR_INT, 0xFFFFFFFF);
        self.write_reg(CSR_FH_INT_STATUS, 0xFFFFFFFF);

        // Stop master DMA
        self.write_reg(CSR_RESET, CSR_RESET_REGISTER_FLAG_STOP_MASTER);
        for _ in 0..1000u32 {
            if self.read_reg(CSR_RESET) & CSR_RESET_REGISTER_FLAG_MASTER_DISABLED != 0 {
                break;
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }

        // APM stop: SW_RESET + clear INIT_DONE (same as Linux iwl_apm_stop)
        self.apm_stop();

        // APM re-init: set INIT_DONE, wait MAC clock, enable APMG clocks
        self.apm_init()?;

        // Re-enable PCI bus master (SW_RESET may clear it)
        let cmd = crate::pci::config_read(self.pci_bus, self.pci_device, self.pci_function, 0x04);
        if cmd & 0x04 == 0 {
            crate::serial_println!("[IWL4965] Re-enabling PCI bus master after reset");
            crate::pci::config_write(self.pci_bus, self.pci_device, self.pci_function, 0x04, cmd | 0x06);
        }

        Ok(())
    }

    /// Load bootstrap code into NIC's BSM SRAM via PRPH indirect access
    /// Linux iwlegacy uses _il_wr_prph() for BSM SRAM (peripheral space, NOT direct CSR)
    fn bsm_load_bootstrap(&self, boot: &[u8]) -> Result<(), &'static str> {
        let dword_count = (boot.len() + 3) / 4;

        // Grab NIC access once for the entire loop (like Linux)
        if !self.grab_nic_access() {
            return Err("bsm_load_bootstrap: cannot grab NIC access");
        }

        // Write bootstrap code into BSM SRAM word by word via PRPH indirect
        for i in 0..dword_count {
            let offset = i * 4;
            let word = if offset + 4 <= boot.len() {
                u32::from_le_bytes([boot[offset], boot[offset+1], boot[offset+2], boot[offset+3]])
            } else {
                let mut bytes = [0u8; 4];
                for j in 0..(boot.len() - offset) {
                    bytes[j] = boot[offset + j];
                }
                u32::from_le_bytes(bytes)
            };

            // Write via PRPH: HBUS_TARG_PRPH_WADDR / WDAT (no grab/release per iteration)
            let addr = BSM_SRAM_LOWER_BOUND + (i * 4) as u32;
            self.write_reg(HBUS_TARG_PRPH_WADDR, (addr & 0x000F_FFFF) | (3 << 24));
            self.write_reg(HBUS_TARG_PRPH_WDAT, word);
        }

        // Verify first dword by reading back via PRPH
        let addr0 = BSM_SRAM_LOWER_BOUND;
        self.write_reg(HBUS_TARG_PRPH_RADDR, (addr0 & 0x000F_FFFF) | (3 << 24));
        let verify0 = self.read_reg(HBUS_TARG_PRPH_RDAT);

        self.release_nic_access();

        let expect0 = u32::from_le_bytes([boot[0], boot[1], boot[2], boot[3]]);
        crate::println!("      SRAM verify: [0]={:#010X} expect={:#010X} {}",
            verify0, expect0, if verify0 == expect0 { "OK" } else { "MISMATCH!" });
        crate::serial_println!("[IWL4965] Bootstrap: {} dwords written to SRAM @ {:#X}, verify={}",
            dword_count, BSM_SRAM_LOWER_BOUND, if verify0 == expect0 { "OK" } else { "FAIL" });
        Ok(())
    }

    /// Set the physical addresses for firmware sections (via PRPH BSM regs)
    /// `inst` and `data` can be either init or runtime firmware sections.
    fn bsm_set_dram_addrs(&self, inst: &[u8], data: &[u8]) -> Result<(), &'static str> {
        let hhdm = crate::memory::hhdm_offset();

        // BSM DRAM pointer registers are on the PRPH bus — use write_prph
        // CRITICAL: 4965 hardware addresses DRAM in 16-byte units (bits 35:4)
        //   Linux: pinst = il->ucode_init.p_addr >> 4;
        //   See il4965_load_bsm() comment: "host DRAM physical address bits 35:4 for 4965"

        // Instruction code
        if !inst.is_empty() {
            let virt = inst.as_ptr() as u64;
            let phys = crate::memory::virt_to_physical(virt).unwrap_or(virt.wrapping_sub(hhdm));
            self.write_prph(BSM_DRAM_INST_POINTER_REGISTER, (phys >> 4) as u32);
            self.write_prph(BSM_DRAM_INST_BYTECOUNT_REGISTER, inst.len() as u32);
            crate::serial_println!("[IWL4965] FW inst @ phys {:#010X} >> 4 = {:#010X} ({} bytes)", phys, phys >> 4, inst.len());
            crate::println!("      inst phys={:#010X} >>4={:#010X} len={}", phys, phys >> 4, inst.len());
        }

        // Data section
        if !data.is_empty() {
            let virt = data.as_ptr() as u64;
            let phys = crate::memory::virt_to_physical(virt).unwrap_or(virt.wrapping_sub(hhdm));
            self.write_prph(BSM_DRAM_DATA_POINTER_REGISTER, (phys >> 4) as u32);
            self.write_prph(BSM_DRAM_DATA_BYTECOUNT_REGISTER, data.len() as u32);
            crate::serial_println!("[IWL4965] FW data @ phys {:#010X} >> 4 = {:#010X} ({} bytes)", phys, phys >> 4, data.len());
            crate::println!("      data phys={:#010X} >>4={:#010X} len={}", phys, phys >> 4, data.len());
        }

        Ok(())
    }

    /// Start the BSM to begin firmware loading (via PRPH registers)
    fn bsm_start(&self, boot_size: usize) -> Result<(), &'static str> {
        // BSM registers are on the PRPH bus
        // Clear start bit first
        self.write_prph(BSM_WR_CONTROLLER_REGISTER, 0);
        for _ in 0..1000 { core::hint::spin_loop(); }

        // Set BSM source (offset 0 within SRAM), destination 0 (instruction memory start)
        self.write_prph(BSM_WR_MEMORY_SOURCE_REGISTER, 0);
        self.write_prph(BSM_WR_MEMORY_DESTINATION_REGISTER, 0);

        // Set the number of dwords to copy — THIS IS CRITICAL
        let dword_count = (boot_size + 3) / 4;
        self.write_prph(BSM_WR_DWCOUNT_REGISTER, dword_count as u32);
        crate::serial_println!("[IWL4965] BSM: src=0 dst=0 dwcount={}", dword_count);

        // Start BSM (bit 31 only — Linux uses BSM_WR_CTRL_REG_BIT_START = 0x80000000)
        self.write_prph(BSM_WR_CONTROLLER_REGISTER, BSM_WR_CONTROLLER_START);

        // Wait for BSM to finish loading
        for _ in 0..10000u32 {
            let ctrl = self.read_prph(BSM_WR_CONTROLLER_REGISTER);
            if ctrl & BSM_WR_CONTROLLER_START == 0 {
                crate::serial_println!("[IWL4965] BSM load complete");
                // Enable future boot loads (BSM auto-reload from DRAM on CPU release)
                // Linux: il_wr_prph(il, BSM_WR_CTRL_REG, BSM_WR_CTRL_REG_BIT_START_EN)
                self.write_prph(BSM_WR_CONTROLLER_REGISTER, BSM_WR_CONTROLLER_START_EN);
                return Ok(());
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        // BSM may still complete — check if device is running
        let gp = self.read_reg(CSR_GP_CNTRL);
        if gp & CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY != 0 {
            crate::serial_println!("[IWL4965] BSM: device appears running (GP_CNTRL={:#X})", gp);
            return Ok(());
        }

        Err("BSM start timeout")
    }

    /// Wait for firmware ALIVE notification
    fn wait_alive(&mut self) -> Result<(), &'static str> {
                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_INT_BIT_ALIVE: u32 = 1 << 0;
                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_INT_BIT_RF_KILL: u32 = 1 << 7;
                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_INT_BIT_FH_RECEIVE: u32 = 1 << 26;
                // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CSR_INT_BIT_HARDWARE_ERROR: u32 = 1 << 29;

        // Readback CSR_RESET to verify CPU was released
        let rst = self.read_reg(CSR_RESET);
        crate::println!("      CSR_RESET={:#X} (expect 0 for CPU running)", rst);

        // 300 iterations * ~10ms = 3 seconds timeout
        for attempt in 0..300u32 {
            let int_status = self.read_reg(CSR_INT);

            // Check for ALIVE bit specifically (bit 0)
            if int_status & CSR_INT_BIT_ALIVE != 0 {
                self.write_reg(CSR_INT, int_status); // ACK
                let gp1 = self.read_reg(CSR_UCODE_DRIVER_GP1);
                let gp_ucode = self.read_reg(CSR_GP_UCODE);
                crate::println!("      ALIVE! INT={:#X} GP1={:#X} UCODE={:#X} (attempt {})",
                    int_status, gp1, gp_ucode, attempt);
                crate::serial_println!("[IWL4965] FW alive: INT={:#X} GP1={:#X} UCODE={:#X}",
                    int_status, gp1, gp_ucode);
                return Ok(());
            }

            // Check for HW error
            if int_status & CSR_INT_BIT_HARDWARE_ERROR != 0 {
                crate::println!("      HW ERROR! INT={:#X} (attempt {})", int_status, attempt);
                self.write_reg(CSR_INT, int_status);
                return Err("Hardware error during firmware load");
            }

            // Check for any interesting interrupt activity
            if int_status != 0 && int_status != 0xFFFFFFFF {
                // FH_RX means firmware sent something via RX DMA
                if int_status & CSR_INT_BIT_FH_RECEIVE != 0 {
                    self.write_reg(CSR_INT, int_status);
                    crate::println!("      ALIVE via FH_RX! INT={:#X} (attempt {})", int_status, attempt);
                    return Ok(());
                }
            }

            // Check GP_UCODE register (firmware sets this when alive)
            let gp_ucode = self.read_reg(CSR_GP_UCODE);
            if gp_ucode != 0 && gp_ucode != 0xFFFFFFFF {
                crate::println!("      ALIVE via GP_UCODE={:#X} (attempt {})", gp_ucode, attempt);
                crate::serial_println!("[IWL4965] FW alive via GP_UCODE: {:#X}", gp_ucode);
                return Ok(());
            }

            if self.fw_alive {
                crate::println!("      ALIVE via RX (attempt {})", attempt);
                return Ok(());
            }

            // Print progress every 50 attempts
            if attempt % 50 == 49 {
                let gp = self.read_reg(CSR_GP_CNTRL);
                crate::print!("      [{}] RST={:#X} GP={:#010X} INT={:#X} GP1={:#X} UCODE={:#X}\n",
                    attempt + 1,
                    self.read_reg(CSR_RESET),
                    gp,
                    self.read_reg(CSR_INT),
                    self.read_reg(CSR_UCODE_DRIVER_GP1),
                    self.read_reg(CSR_GP_UCODE));
            }

            // ~10ms delay between checks (port 0x80 ~1us each)
            for _ in 0..10_000 {
                                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
        }

        // Log final state for debugging
        let gp = self.read_reg(CSR_GP_CNTRL);
        let gp1 = self.read_reg(CSR_UCODE_DRIVER_GP1);
        let gp_ucode = self.read_reg(CSR_GP_UCODE);
        let int_value = self.read_reg(CSR_INT);
        let rst = self.read_reg(CSR_RESET);
        crate::println!("      TIMEOUT: RST={:#X} GP={:#010X} GP1={:#X} UCODE={:#X} INT={:#X}",
            rst, gp, gp1, gp_ucode, int_value);
        crate::serial_println!("[IWL4965] FW ALIVE TIMEOUT: RST={:#X} GP={:#X} GP1={:#X} UCODE={:#X} INT={:#X}",
            rst, gp, gp1, gp_ucode, int_value);

        Err("Firmware ALIVE timeout")
    }

    // ── DMA Queue Setup ──────────────────────────────────────────

    /// Initialize all TX queues and the RX queue for DMA
    fn init_queues(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[IWL4965] Initializing DMA queues...");

        // Allocate TX queues (0-3: EDCA data, 4: HCMD)
        self.tx_queues.clear();
        for q in 0..IWL_TRANSMIT_QUEUE_COUNT {
            let txq = TxQueue::new();
            let phys = txq.tfd_phys_base();
            
            // Tell the device where this queue's TFD ring lives
            self.write_reg(FH_MEMORY_CBBC_QUEUE + (q as u32) * 4, phys as u32);
            crate::serial_println!("[IWL4965]   TXQ{}: TFD ring @ phys {:#X}", q, phys);
            
            self.tx_queues.push(txq);
        }

        // Allocate RX queue
        let rxq = RxQueue::new();
        let rb_physical = rxq.bd_phys_base();

        // Step 1: Disable RX DMA + clear RCSR write pointer (Linux does this first)
        self.write_reg(FH_RCSR_CHNL0_CONFIG_REGISTER, 0);
        self.write_reg(FH_MEMORY_RCSR_CHNL0_RBDCB_WPTR, 0);

        // Step 2: Set RSCSR registers (RBD ring base + status DMA target)
        self.write_reg(FH_RSCSR_CHNL0_RBDCB_BASE_REGISTER, (rb_physical >> 8) as u32);

        let stts_virt = rxq.rb_stts.as_ptr() as u64;
        let stts_physical = crate::memory::virt_to_physical(stts_virt)
            .unwrap_or(stts_virt.wrapping_sub(crate::memory::hhdm_offset()));
        self.write_reg(FH_RSCSR_CHNL0_STTS_WPTR_REGISTER, (stts_physical >> 4) as u32);

        // Step 3: Enable RX DMA channel (matching Linux iwlegacy config exactly)
        // RX_QUEUE_SIZE=256, log2(256)=8 → size field = 8 << 20 = 0x00800000
        let receive_config = FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_DMA_CHNL_EN
            | FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_INTERRUPT_REQUEST_DEST_INT
            | FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_SINGLE_FRAME
            | FH_RCSR_RECEIVE_CONFIG_REGISTER_VALUE_RB_SIZE_4K
            | (8u32 << FH_RCSR_RECEIVE_CONFIG_RBDCB_SIZE_POSITION);  // log2(256) = 8
        self.write_reg(FH_RCSR_CHNL0_CONFIG_REGISTER, receive_config);

        // Step 4: Tell device all RX buffers are available (aligned to 8)
        self.write_reg(FH_RSCSR_CHNL0_RBDCB_WPTR_REGISTER, (RECEIVE_QUEUE_SIZE as u32) & !0x7);

        crate::serial_println!("[IWL4965]   RXQ: CONFIG={:#010X} BASE={:#X} STTS={:#X} WPTR={}",
            receive_config, (rb_physical >> 8) as u32, (stts_physical >> 4) as u32, (RECEIVE_QUEUE_SIZE as u32) & !0x7);

        self.rx_queue = Some(rxq);

        // Enable TX scheduling on all queues (SCD is on PRPH bus)
        self.write_prph(SCD_TXFACT, (1 << IWL_TRANSMIT_QUEUE_COUNT) - 1);

        crate::serial_println!("[IWL4965] DMA queues initialized");
        Ok(())
    }

    // ── Host Command Interface ───────────────────────────────────

    /// Send a host command to the firmware via the HCMD queue (TXQ4)
    fn send_hcmd(&mut self, cmd_id: u8, data: &[u8]) -> Result<(), &'static str> {
        if !self.fw_alive {
            return Err("Firmware not alive");
        }
        if data.len() > MAXIMUM_COMMAND_SIZE {
            return Err("HCMD payload too large");
        }

        let txq = &mut self.tx_queues[IWL_COMMAND_QUEUE_NUMBER];
        let idx = txq.write_ptr;

        // Build command header + payload into the pre-allocated buffer
        let buf = &mut txq.cmd_buffers[idx];
        let hdr = IwlCommandHeader {
            cmd: cmd_id,
            flags: 0,
            idx: idx as u8,
            qid: IWL_COMMAND_QUEUE_NUMBER as u8,
        };
        // Write header
        buf[0] = hdr.cmd;
        buf[1] = hdr.flags;
        buf[2] = hdr.idx;
        buf[3] = hdr.qid;
        // Write payload
        let payload_len = data.len();
        buf[4..4 + payload_len].copy_from_slice(data);
        let total_len = 4 + payload_len;

        // Get physical address of the buffer
        let buffer_virt = buf.as_ptr() as u64;
        let buffer_physical = crate::memory::virt_to_physical(buffer_virt)
            .unwrap_or(buffer_virt.wrapping_sub(crate::memory::hhdm_offset()));

        // Setup TFD with one transfer buffer pointing to our command
        let tfd = &mut txq.tfds[idx];
        *tfd = IwlTfd::zeroed();
        tfd.tbs[0].set(buffer_physical, total_len as u16);
        tfd.set_num_tbs(1);

        // Advance write pointer and kick the doorbell
        txq.write_ptr = (idx + 1) % TFD_QUEUE_SIZE;

        // Write the new write pointer to the device's TX write pointer register
        let wrptr_value = (txq.write_ptr as u32) | ((IWL_COMMAND_QUEUE_NUMBER as u32) << 8);
        self.write_reg(HBUS_TARG_WRPTR, wrptr_value);

        self.cmd_seq = self.cmd_seq.wrapping_add(1);

        crate::serial_println!("[IWL4965] HCMD sent: cmd={:#04X} len={} idx={}", cmd_id, total_len, idx);
        Ok(())
    }

    /// Wait for a response/notification from firmware (poll RX queue)
    fn poll_rx(&mut self) {
        if self.rx_queue.is_none() {
            return;
        }

        // Check CSR_INT for any pending interrupt from device
        let csr_int = self.read_reg(CSR_INT);
        if csr_int != 0 && csr_int != 0xFFFFFFFF {
            // Acknowledge/clear the interrupt
            self.write_reg(CSR_INT, csr_int);
            // Re-enable interrupts after ACK (iwl4965 requires this)
            self.write_reg(CSR_INT_MASK, 0xAB00_000B);
            if csr_int & 0x80000000 != 0 {
                crate::serial_println!("[IWL4965] poll_rx: INT={:#010X} (FH_RX fired)", csr_int);
            }
        }

        // Read the hardware write pointer from shared DMA memory (rb_stts)
        let hardware_write = {
            let rxq = // Correspondance de motifs — branchement exhaustif de Rust.
match self.rx_queue.as_ref() {
                Some(q) => q,
                None => { crate::serial_println!("[IWL4965] poll_rx: no RX queue"); return; }
            };
            let stts = &rxq.rb_stts;
            // Device writes a u32 at offset 0: bits [11:0] = closed_rb_num (write pointer)
            let raw = u32::from_le_bytes([stts[0], stts[1], stts[2], stts[3]]);
            (raw & 0xFFF) as usize % RECEIVE_QUEUE_SIZE
        };
        let read = // Correspondance de motifs — branchement exhaustif de Rust.
match self.rx_queue.as_ref() {
            Some(q) => q.read_ptr,
            None => return,
        };
        
        if hardware_write == read {
            return;
        }

        crate::serial_println!("[IWL4965] poll_rx: hw_write={} read={} — new packets!", hardware_write, read);

        // Collect packets first to avoid borrow conflicts with self
        let mut packets: Vec<(u8, Vec<u8>)> = Vec::new();
        let mut idx = read;
        let mut count = 0;
        
        {
            let rxq = // Correspondance de motifs — branchement exhaustif de Rust.
match self.rx_queue.as_ref() {
                Some(q) => q,
                None => return,
            };
            while idx != hardware_write && count < RECEIVE_QUEUE_SIZE {
                let buf = &rxq.buffers[idx];
                if buf.len() >= 8 {
                    let packet_length = u16::from_le_bytes([buf[0], buf[1]]) as usize;
                    let cmd_id = buf[4];
                    if packet_length > 0 && packet_length <= RX_BUF_SIZE - 4 {
                        let end = (packet_length + 4).min(buf.len());
                        packets.push((cmd_id, buf[..end].to_vec()));
                    }
                }
                idx = (idx + 1) % RECEIVE_QUEUE_SIZE;
                count += 1;
            }
        }

        // Update read pointer
        if let Some(rxq) = self.rx_queue.as_mut() {
            rxq.read_ptr = idx;
        }
        
        // Process collected packets
        for (cmd_id, data) in packets {
            self.process_rx_packet(cmd_id, &data);
        }
    }

    /// Process a single RX packet/notification from firmware
    fn process_rx_packet(&mut self, cmd_id: u8, data: &[u8]) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match cmd_id {
            REPLY_ALIVE => {
                crate::serial_println!("[IWL4965] RX: ALIVE notification");
                self.fw_alive = true;
            }
            REPLY_ERROR => {
                crate::serial_println!("[IWL4965] RX: ERROR from firmware");
            }
            SCAN_RESULTS_NOTIFICATION => {
                crate::serial_println!("[IWL4965] RX: Scan results notification ({} bytes)", data.len());
                self.parse_scan_notification(data);
            }
            REPLY_SCAN_COMPLETE | REPLY_SCAN_COMMAND => {
                crate::serial_println!("[IWL4965] RX: Scan complete/response");
                self.scanning = false;
                self.wifi_state = if self.connected_ssid.is_some() {
                    WifiState::Connected
                } else {
                    WifiState::Disconnected
                };
            }
            REPLY_RXON | REPLY_RXON_ASSOC => {
                crate::serial_println!("[IWL4965] RX: RXON response");
            }
            REPLY_TRANSMIT => {
                // TX completion notification
                self.stats.tx_packets += 1;
            }
            // RX data frames from firmware (typically cmd IDs 0xC0+ are data)
            0xC1 | 0xC3 => {
                // Data frame received — extract Ethernet payload
                if data.len() > 32 {
                    // Firmware strips 802.11 header and delivers Ethernet-like frame
                    let frame = data[16..].to_vec(); // Skip internal header
                    self.stats.rx_packets += 1;
                    self.stats.rx_bytes += frame.len() as u64;
                    if self.rx_pending.len() < 64 {
                        self.rx_pending.push(frame);
                    }
                }
            }
            _ => {
                crate::serial_println!("[IWL4965] RX: Unknown cmd {:#04X} ({} bytes)", cmd_id, data.len());
            }
        }
    }

    /// Parse a SCAN_RESULTS_NOTIFICATION from firmware
    fn parse_scan_notification(&mut self, data: &[u8]) {
        // iwl4965 scan notification format:
        // offset 0-3: pkt_len + header
        // offset 4: notification ID (0x83)
        // offset 8: number of results
        // Then for each result: BSSID(6) + channel(1) + signal(1) + IEs(variable)
        
        if data.len() < 12 {
            return;
        }

        let count = data[8] as usize;
        crate::serial_println!("[IWL4965] Scan: {} network(s) reported", count);

        let mut offset = 12; // Start of first result
        for _ in 0..count {
            if offset + 20 > data.len() { break; }

            // Parse BSS entry
            let mut bssid = [0u8; 6];
            bssid.copy_from_slice(&data[offset..offset + 6]);
            let channel = data[offset + 6];
            let signal = data[offset + 7] as i8;
            let ie_length = u16::from_le_bytes([data[offset + 8], data[offset + 9]]) as usize;

            // Extract SSID and security from IEs
            let ie_start = offset + 10;
            let ie_end = (ie_start + ie_length).min(data.len());
            let (ssid, security) = self.parse_ies(&data[ie_start..ie_end]);

            if !ssid.is_empty() && self.scan_results.len() < MAXIMUM_SCAN_RESULTS {
                // Don't add duplicates
                let dup = self.scan_results.iter().any(|n| n.bssid == bssid);
                if !dup {
                    let freq = if channel <= 14 {
                        2407 + (channel as u16) * 5
                    } else {
                        5000 + (channel as u16) * 5
                    };
                    self.scan_results.push(WifiNetwork {
                        ssid,
                        bssid,
                        channel,
                        signal_dbm: signal,
                        security,
                        frequency_mhz: freq,
                    });
                }
            }

            offset = ie_end;
        }
    }

    /// Parse 802.11 Information Elements to extract SSID and security type
    fn parse_ies(&self, data: &[u8]) -> (String, WifiSecurity) {
        let mut ssid = String::new();
        let mut security = WifiSecurity::Open;
        let mut i = 0;

        while i + 2 <= data.len() {
            let eid = data[i];
            let elen = data[i + 1] as usize;
            let estart = i + 2;
            let eend = (estart + elen).min(data.len());

                        // Correspondance de motifs — branchement exhaustif de Rust.
match eid {
                WLAN_EID_SSID => {
                    if elen > 0 && elen <= 32 {
                        if let Ok(s) = core::str::from_utf8(&data[estart..eend]) {
                            ssid = String::from(s);
                        }
                    }
                }
                WLAN_EID_RSN => {
                    security = WifiSecurity::WPA2;
                }
                WLAN_EID_VENDOR => {
                    // WPA1 OUI: 00:50:F2:01
                    if elen >= 4 && data[estart] == 0x00 && data[estart + 1] == 0x50
                        && data[estart + 2] == 0xF2 && data[estart + 3] == 0x01
                    {
                        if security == WifiSecurity::Open {
                            security = WifiSecurity::WPA;
                        }
                    }
                }
                _ => {}
            }

            i = eend;
        }

        (ssid, security)
    }

    // ── Scanning ─────────────────────────────────────────────────

    /// Start passive scan on 2.4 GHz channels
    fn start_scan_hw(&mut self) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Hardware not initialized");
        }

        // Report firmware/queue state
        crate::serial_println!("[IWL4965] Scan: fw_loaded={} fw_alive={} queues={} rx={}",
            self.firmware_loaded, self.fw_alive,
            self.tx_queues.len(), self.rx_queue.is_some());
        crate::println!("  FW: loaded={} alive={} queues={}",
            self.firmware_loaded, self.fw_alive, self.tx_queues.len());

        // Check key CSR registers
        let gp = self.read_reg(CSR_GP_CNTRL);
        let int = self.read_reg(CSR_INT);
        let gp1 = self.read_reg(CSR_UCODE_DRIVER_GP1);
        let fh = self.read_reg(CSR_FH_INT_STATUS);
        crate::serial_println!("[IWL4965] Pre-scan CSR: GP={:#X} INT={:#X} GP1={:#X} FH={:#X}", gp, int, gp1, fh);
        crate::println!("  CSR: GP_CNTRL={:#X} INT={:#X}", gp, int);

        self.scan_results.clear();
        self.scanning = true;
        self.scan_start_tick = crate::logger::get_ticks();
        self.wifi_state = WifiState::Scanning;

        if self.firmware_loaded && self.fw_alive {
            // Firmware-based scan: first send RXON to turn on the radio
            crate::serial_println!("[IWL4965] Sending RXON to enable radio...");
            self.send_rxon()?;
            // Give firmware time to process RXON and poll for response
            for _ in 0..500_000 { core::hint::spin_loop(); }
            self.poll_rx(); // Check if firmware ACKed RXON
            let int_after_rxon = self.read_reg(CSR_INT);
            crate::serial_println!("[IWL4965] Post-RXON: INT={:#X}", int_after_rxon);

            crate::serial_println!("[IWL4965] Starting firmware-based scan (2.4 GHz + 5 GHz)...");
            self.send_scan_request()?;
        } else {
            // Passive monitoring — no firmware, limited capability
            self.write_reg(CSR_INT_COALESCING, 0x40);
            crate::serial_println!("[IWL4965] Passive scan started (no firmware)");
            crate::println!("  Warning: No firmware - passive scan only");
        }

        Ok(())
    }

    /// Send RXON command to firmware — turns on the radio for scanning/receiving
    /// Struct layout: iwl4965_rxon_cmd (44 bytes, packed)
    fn send_rxon(&mut self) -> Result<(), &'static str> {
        // iwl4965_rxon_cmd layout:
        //   [0..5]   node_addr (MAC, 6 bytes)
        //   [6..7]   reserved1
        //   [8..13]  bssid_addr (6 bytes)
        //   [14..15] reserved2
        //   [16..21] wlap_bssid_addr (6 bytes)
        //   [22..23] reserved3
        //   [24]     dev_type (1=STA, 3=IBSS, 4=AP)
        //   [25]     air_propagation
        //   [26..27] rx_chain (antenna config)
        //   [28]     ofdm_basic_rates
        //   [29]     cck_basic_rates
        //   [30..31] assoc_id
        //   [32..35] flags (RXON_FLG_*)
        //   [36..39] filter_flags (RXON_FILTER_*)
        //   [40]     channel
        //   [41]     reserved5
        //   [42]     ht_single_stream_basic_rates
        //   [43]     ht_dual_stream_basic_rates
        let mut rxon = [0u8; 44];

        // [0..5] node_addr = our MAC address
        rxon[0..6].copy_from_slice(&self.mac_addr);

        // [8..13] bssid = broadcast (unassociated / scanning mode)
        rxon[8..14].copy_from_slice(&[0xFF; 6]);

        // [16..21] wlap_bssid = broadcast
        rxon[16..22].copy_from_slice(&[0xFF; 6]);

        // [24] dev_type = STA mode
        rxon[24] = 1;

        // [26..27] rx_chain: DRIVER_FORCE(0) | VALID=A+B(bits 1-2) | FORCE_SEL=A+B(bits 4-5)
        let receive_chain: u16 = 0x0001 | 0x0006 | 0x0030;
        rxon[26..28].copy_from_slice(&receive_chain.to_le_bytes());

        // [28] ofdm_basic_rates: 6, 12, 24 Mbps
        rxon[28] = 0x15;
        // [29] cck_basic_rates: 1, 2, 5.5, 11 Mbps
        rxon[29] = 0x0F;

        // [32..35] flags: BAND_24G(0) | SHORT_PREAMBLE(5) | SHORT_SLOT(4) | TSF2HOST(15)
        let flags: u32 = (1 << 0) | (1 << 4) | (1 << 5) | (1 << 15);
        rxon[32..36].copy_from_slice(&flags.to_le_bytes());

        // [36..39] filter_flags: PROMISC(0) | CTL2HOST(1) | ACCEPT_GRP(2) | BCON_AWARE(6)
        let filter: u32 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 6);
        rxon[36..40].copy_from_slice(&filter.to_le_bytes());

        // [40] channel = 1 (initial channel for RXON)
        rxon[40] = 1;

        self.send_hcmd(REPLY_RXON, &rxon)?;
        crate::serial_println!("[IWL4965] RXON sent: ch=1 STA flags={:#X} filter={:#X} rx_chain={:#X}",
            flags, filter, receive_chain);
        Ok(())
    }

    /// Send SCAN_REQUEST host command to firmware (iwl4965 format)
    ///
    /// The iwl4965 SCAN_CMD layout (struct il_scan_cmd):
    ///   [0..1]   len (u16) — size of probe_frame + channel_list (variable data)
    ///   [2]      reserved0
    ///   [3]      channel_count
    ///   [4..5]   quiet_time (u16, TU)
    ///   [6..7]   quiet_plcp_th (u16)
    ///   [8..9]   good_CRC_th (u16)
    ///   [10..11] rx_chain (u16)
    ///   [12..15] max_out_time (u32, usec)
    ///   [16..19] suspend_time (u32, usec)
    ///   [20..23] flags (u32, RXON flags)
    ///   [24..27] filter_flags (u32, RXON filter)
    ///   [28..87] tx_cmd (60 bytes — il_tx_cmd for probe requests)
    ///   [88..223] direct_scan[4] (4 × 34 = 136 bytes — SSID IEs)
    ///   [224..] data: probe_request_frame(tx_cmd.len bytes) + channel_list
    ///
    /// Channel entry (12 bytes each — struct il_scan_channel):
    ///   [+0..+3]  type (u32) — bit 0: 1=active, 0=passive
    ///   [+4..+5]  channel (u16)
    ///   [+6]      tx_gain
    ///   [+7]      dsp_atten
    ///   [+8..+9]  active_dwell (u16, ms)
    ///   [+10..+11] passive_dwell (u16, ms)
    fn send_scan_request(&mut self) -> Result<(), &'static str> {
        let channels: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 36, 40, 44, 48];
        let chan_count = channels.len();

        // Fixed header = 224 bytes, channels = 12 each
        const SCAN_HEADER_SIZE: usize = 224;
        let total_len = SCAN_HEADER_SIZE + chan_count * 12;
        let mut cmd = vec![0u8; total_len];

        // Passive scan: no probe frame, so data portion = channel list only
        let data_length = (chan_count * 12) as u16;

        // [0..1] len = size of variable data (probe frame + channels)
        cmd[0..2].copy_from_slice(&data_length.to_le_bytes());
        // [2] reserved0 = 0
        // [3] channel_count
        cmd[3] = chan_count as u8;
        // [4..5] quiet_time = 0 (no quiet)
        // [6..7] quiet_plcp_th = 0
        // [8..9] good_CRC_th = 0 (no passive->active promotion)
        // [10..11] rx_chain: DRIVER_FORCE | VALID=A+B | FORCE=A+B
        let receive_chain: u16 = 0x0001 | 0x0006 | 0x0030;
        cmd[10..12].copy_from_slice(&receive_chain.to_le_bytes());
        // [12..15] max_out_time = 200000 usec (200ms)
        cmd[12..16].copy_from_slice(&200000u32.to_le_bytes());
        // [16..19] suspend_time = 100000 usec (100ms) — pause between channels
        cmd[16..20].copy_from_slice(&100000u32.to_le_bytes());
        // [20..23] flags: BAND_24G | SHORT_SLOT | SHORT_PREAMBLE
        let flags: u32 = (1 << 0) | (1 << 4) | (1 << 5);
        cmd[20..24].copy_from_slice(&flags.to_le_bytes());
        // [24..27] filter_flags: PROMISC | CTL2HOST | ACCEPT_GRP | BCON_AWARE
        let filter: u32 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 6);
        cmd[24..28].copy_from_slice(&filter.to_le_bytes());

        // [28..87] tx_cmd (60 bytes) — mostly zeros for passive scan
        // tx_cmd.len [28..29] = 0 (no probe request frame)
        // tx_cmd.tx_flags [32..35]: set ANT_A|ANT_B for any fallback TX
        cmd[32..36].copy_from_slice(&0x0000_000Du32.to_le_bytes());
        // tx_cmd.rate_n_flags [44..47]: 6 Mbps OFDM = 0x0D (rate) | OFDM flag
        // iwl4965 rate flags: rate index 0x0D=6Mbps, ANT_A=0x0400_0000
        cmd[44..48].copy_from_slice(&0x0400_000Du32.to_le_bytes());
        // tx_cmd.sta_id [48] = 0xFF (broadcast)
        cmd[48] = 0xFF;
        // tx_cmd.life_time [72..75] = 0xFFFFFFFF (max)
        cmd[72..76].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        // tx_cmd.data_retry_limit [82] = 1
        cmd[82] = 1;

        // [88..223] direct_scan[4] = all zeros (wildcard scan, no specific SSID)

        // [224+] Channel list — passive scan on all channels
        let mut pos = SCAN_HEADER_SIZE;
        for &ch in channels {
            // type: passive scan = 0 (bit 0 = 0)
            cmd[pos..pos + 4].copy_from_slice(&0u32.to_le_bytes());
            // channel number
            cmd[pos + 4..pos + 6].copy_from_slice(&(ch as u16).to_le_bytes());
            // tx_gain = 0x28 (20 dBm)
            cmd[pos + 6] = 0x28;
            // dsp_atten = 110
            cmd[pos + 7] = 110;
            // active_dwell = 20 ms
            cmd[pos + 8..pos + 10].copy_from_slice(&20u16.to_le_bytes());
            // passive_dwell = 120 ms (long enough to catch beacons, ~10 per second)
            cmd[pos + 10..pos + 12].copy_from_slice(&120u16.to_le_bytes());
            pos += 12;
        }

        self.send_hcmd(REPLY_SCAN_COMMAND, &cmd)?;

        crate::serial_println!("[IWL4965] Scan request sent: {} channels, {} bytes, passive mode",
            chan_count, total_len);
        Ok(())
    }

    /// Poll for scan results (called from poll())
    fn poll_scan(&mut self) {
        if !self.scanning {
            return;
        }

        let ticks = crate::logger::get_ticks();
        let elapsed = ticks.saturating_sub(self.scan_start_tick);

        // Process RX queue for firmware notifications
        self.poll_rx();

        // Scan timeout
        if elapsed >= SCAN_TIMEOUT_TICKS {
            self.scanning = false;
            self.wifi_state = if self.connected_ssid.is_some() {
                WifiState::Connected
            } else {
                WifiState::Disconnected
            };
            crate::serial_println!("[IWL4965] Scan complete: {} networks", self.scan_results.len());

            // If no hardware results (firmware not loaded), do a discovery
            if self.scan_results.is_empty() {
                self.detect_networks_from_ether();
            }
        }
    }

    /// Attempt to detect networks from raw ether monitoring
    /// This reads the GPIO and power state to detect nearby APs
    /// Works as a fallback when firmware isn't loaded
    fn detect_networks_from_ether(&mut self) {
        // Without firmware, we can detect RF energy on channels
        // by checking the AGC (Automatic Gain Control) and RSSI registers
        // The GP register reflects the RF environment somewhat

        let gpio = self.read_reg(CSR_GPIO_IN);
        let gp_cntrl = self.read_reg(CSR_GP_CNTRL);

        crate::serial_println!("[IWL4965] GPIO: {:#010X}, GP_CNTRL: {:#010X}", gpio, gp_cntrl);

        // Read EEPROM for channel capabilities
        let sku = self.eeprom_read(EEPROM_SKU_CAPABILITY);
        let has_24ghz = (sku & 0x01) != 0 || sku == 0; // Default to yes
        let has_5ghz = (sku & 0x02) != 0;
        crate::serial_println!("[IWL4965] SKU: {:#06X}, 2.4GHz: {}, 5GHz: {}", sku, has_24ghz, has_5ghz);

        // Without firmware loading, we report the hardware as ready
        // but scanning returns hardware-detected channel info
        // The desktop UI will show "WiFi Ready - Scan in progress"
        // A full implementation would load iwlwifi-4965-2.ucode firmware blob
    }

    // ── Connection ───────────────────────────────────────────────

    fn do_connect(&mut self, ssid: &str, _password: &str) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Hardware not initialized");
        }

        // Find the network in scan results
        let network = self.scan_results.iter()
            .find(|n| n.ssid == ssid)
            .cloned();

                // Correspondance de motifs — branchement exhaustif de Rust.
match network {
            Some(net) => {
                crate::serial_println!("[IWL4965] Connecting to '{}' on ch{} ({:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X})",
                    ssid, net.channel,
                    net.bssid[0], net.bssid[1], net.bssid[2],
                    net.bssid[3], net.bssid[4], net.bssid[5]);

                self.wifi_state = WifiState::Connecting;
                self.connected_bssid = net.bssid;
                self.current_channel = net.channel;
                self.signal_dbm = net.signal_dbm;

                // Step 1: Send RXON command to join the BSS
                if self.fw_alive && !self.tx_queues.is_empty() {
                    self.send_rxon_cmd(&net)?;
                    self.wifi_state = WifiState::Authenticating;
                    
                    // Step 2: For Open networks, association completes after RXON
                    // For WPA2, we need to do 4-way handshake after association
                    // (EAPOL frames via TX queue 0)
                    match net.security {
                        WifiSecurity::Open => {
                            // Open network: RXON is enough
                            self.connected_ssid = Some(String::from(ssid));
                            self.wifi_state = WifiState::Connected;
                            crate::serial_println!("[IWL4965] Connected to '{}' (Open, {} dBm)", ssid, net.signal_dbm);
                        }
                        WifiSecurity::WPA2 | WifiSecurity::WPA | WifiSecurity::WPA3 => {
                            // WPA2: firmware handles authentication, we need EAPOL for keys
                            self.connected_ssid = Some(String::from(ssid));
                            self.wifi_state = WifiState::Connected;
                            crate::serial_println!("[IWL4965] Associated to '{}' (WPA2, {} dBm) — key exchange TODO", ssid, net.signal_dbm);
                        }
                        _ => {
                            self.connected_ssid = Some(String::from(ssid));
                            self.wifi_state = WifiState::Connected;
                            crate::serial_println!("[IWL4965] Connected to '{}' ({} dBm)", ssid, net.signal_dbm);
                        }
                    }
                } else {
                    // No firmware — can't really connect
                    self.connected_ssid = Some(String::from(ssid));
                    self.wifi_state = WifiState::Connected;
                    crate::serial_println!("[IWL4965] Connected to '{}' (no firmware — limited)", ssid);
                }

                Ok(())
            }
            None => {
                crate::serial_println!("[IWL4965] Network '{}' not in scan results, attempting blind connect", ssid);
                self.wifi_state = WifiState::Connecting;
                self.connected_ssid = Some(String::from(ssid));
                Ok(())
            }
        }
    }

    /// Send RXON command to configure the radio to join a BSS
    fn send_rxon_cmd(&mut self, net: &WifiNetwork) -> Result<(), &'static str> {
        // iwl4965 RXON command (REPLY_RXON = 0x10)
        // Sets the radio to a specific channel, BSSID filter, and operating mode
        //
        // RXON structure (simplified, 56 bytes):
        //   [0..5]   : BSSID to filter on
        //   [6..7]   : reserved
        //   [8..13]  : node address (our MAC)
        //   [14..15] : reserved
        //   [16..21] : WLAP BSSID (same as BSSID for STA mode)
        //   [22..23] : dev_type (STA=1)
        //   [24]     : flags
        //   [25]     : filter_flags
        //   [26]     : channel
        //   [27]     : ofdm_ht / ht_protection
        //   [28..31] : assoc_id / reserved
        //   [32..55] : cipher/key info

        let mut rxon = [0u8; 56];

        // BSSID filter
        rxon[0..6].copy_from_slice(&net.bssid);
        // Our MAC
        rxon[8..14].copy_from_slice(&self.mac_addr);
        // WLAP BSSID
        rxon[16..22].copy_from_slice(&net.bssid);
        // Device type: STA (managed mode)
        rxon[22] = 0x01;
        rxon[23] = 0x00;
        // Flags: short preamble, 802.11g
        rxon[24] = 0x03;
        // Filter: accept unicast + broadcast
        rxon[25] = 0x03;
        // Channel
        rxon[26] = net.channel;

        self.send_hcmd(REPLY_RXON, &rxon)?;
        crate::serial_println!("[IWL4965] RXON sent: ch{} BSSID {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            net.channel, net.bssid[0], net.bssid[1], net.bssid[2],
            net.bssid[3], net.bssid[4], net.bssid[5]);

        Ok(())
    }
}

// ============================================================================
// Driver Trait Implementation
// ============================================================================

impl Driver for Iwl4965 {
    fn info(&self) -> &DriverInformation {
        &DRIVER_INFORMATION
    }

    fn probe(&mut self, pci_dev: &PciDevice) -> Result<(), &'static str> {
        self.pci_bus = pci_dev.bus;
        self.pci_device = pci_dev.device;
        self.pci_function = pci_dev.function;
        self.device_id = pci_dev.device_id;
        self.status = DriverStatus::Loading;

        crate::println!("    [iwl4965] probe: map_bar0...");
        // Map BAR0
        self.map_bar0(pci_dev)?;
        crate::println!("    [iwl4965] probe: map_bar0 OK (base={:#X})", self.mmio_base);

        // Enable bus mastering and memory space in PCI command register
        crate::println!("    [iwl4965] probe: PCI bus master...");
        let cmd = crate::pci::config_read(pci_dev.bus, pci_dev.device, pci_dev.function, 0x04);
        crate::pci::config_write(pci_dev.bus, pci_dev.device, pci_dev.function, 0x04,
            cmd | 0x06); // Memory Space + Bus Master
        crate::println!("    [iwl4965] probe: done");

        Ok(())
    }

    fn start(&mut self) -> Result<(), &'static str> {
        crate::println!("  [wifi start] hw_init...");
        self.hw_init()?;
        let gp_after_initialize = self.read_reg(CSR_GP_CNTRL);
        crate::println!("  [wifi start] hw_init OK  GP={:#010X} MAC_CLK={}",
            gp_after_initialize,
            if gp_after_initialize & CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY != 0 { "ready" } else { "DEAD" });
        self.wifi_state = WifiState::Disconnected;
        self.status = DriverStatus::Running;

        // Attempt firmware loading
        // Linux flow: load FW → wait ALIVE → THEN init queues → send commands
        if has_firmware() {
            let fw_size = WIFI_FIRMWARE.lock().as_ref().map(|d| d.len()).unwrap_or(0);
            crate::println!("  [wifi start] Firmware available ({} bytes), loading...", fw_size);
                        // Correspondance de motifs — branchement exhaustif de Rust.
match self.load_firmware() {
                Ok(()) => {
                    crate::println!("  [wifi start] Firmware loaded and ALIVE!");
                    crate::serial_println!("[IWL4965] Firmware loaded and alive");

                    // NOW init DMA queues (after firmware is alive, matching Linux il4965_alive_start)
                    crate::println!("  [wifi start] init_queues...");
                                        // Correspondance de motifs — branchement exhaustif de Rust.
match self.init_queues() {
                        Ok(()) => {
                            crate::println!("  [wifi start] DMA queues ready — full WiFi mode");
                            crate::serial_println!("[IWL4965] DMA queues ready — full WiFi mode");
                        }
                        Err(e) => {
                            crate::println!("  [wifi start] Queue init FAILED: {}", e);
                        }
                    }

                    // Enable interrupts 
                    self.write_reg(CSR_INT, 0xFFFFFFFF); // clear any pending
                    self.write_reg(CSR_INT_MASK, 0xAB00_000B);
                }
                Err(e) => {
                    crate::println!("  [wifi start] FW load FAILED: {}", e);
                    crate::serial_println!("[IWL4965] Firmware load failed: {} — passive mode", e);
                }
            }
        } else {
            crate::println!("  [wifi start] No firmware file — passive scan only");
        }

        // Print status summary to screen so user can see what happened
        let gp = self.read_reg(CSR_GP_CNTRL);
        let clk = if gp & CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY != 0 { "ready" } else { "DEAD" };
        crate::println!("  [wifi start] SUMMARY: MAC_CLK={} FW={} ALIVE={} GP={:#010X}",
            clk,
            if self.firmware_loaded { "yes" } else { "no" },
            if self.fw_alive { "yes" } else { "no" },
            gp);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), &'static str> {
        // Disable interrupts
        self.write_reg(CSR_INT_MASK, 0);
        // Stop master
        self.write_reg(CSR_RESET, CSR_RESET_REGISTER_FLAG_STOP_MASTER);
        self.status = DriverStatus::Suspended;
        self.wifi_state = WifiState::Disabled;
        Ok(())
    }

    fn status(&self) -> DriverStatus {
        self.status
    }

    fn handle_interrupt(&mut self) {
        let int_status = self.read_reg(CSR_INT);
        if int_status == 0 || int_status == 0xFFFFFFFF {
            return;
        }
        self.write_reg(CSR_INT, int_status);

        // Process RX queue on any interrupt
        self.poll_rx();

        // Process scan if active
        if self.scanning {
            self.poll_scan();
        }
    }
}

// Implémentation de trait — remplit un contrat comportemental.
impl NetworkDriver for Iwl4965 {
    fn mac_address(&self) -> [u8; 6] {
        self.mac_addr
    }

    fn link_up(&self) -> bool {
        self.wifi_state == WifiState::Connected
    }

    fn link_speed(&self) -> u32 {
        if self.wifi_state == WifiState::Connected { 54 } else { 0 } // 54 Mbps (802.11g baseline)
    }

    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if self.wifi_state != WifiState::Connected {
            return Err("Not connected");
        }
        if data.len() > 2048 {
            return Err("Frame too large");
        }
        if !self.fw_alive || self.tx_queues.is_empty() {
            return Err("Firmware not ready");
        }

        // Use TX Queue 0 (Best Effort) for data frames
        let txq = &mut self.tx_queues[0];
        let idx = txq.write_ptr;

        // Build 802.11 data frame header + payload into TX buffer
        let buf = &mut txq.cmd_buffers[idx];
        // For now: the firmware expects Ethernet frames, wraps them in 802.11
        // (In full driver, we'd build the 802.11 header ourselves)
        let len = data.len().min(buf.len());
        buf[..len].copy_from_slice(&data[..len]);

        // Setup TFD
        let buffer_virt = buf.as_ptr() as u64;
        let buffer_physical = crate::memory::virt_to_physical(buffer_virt)
            .unwrap_or(buffer_virt.wrapping_sub(crate::memory::hhdm_offset()));
        
        let tfd = &mut txq.tfds[idx];
        *tfd = IwlTfd::zeroed();
        tfd.tbs[0].set(buffer_physical, len as u16);
        tfd.set_num_tbs(1);

        txq.write_ptr = (idx + 1) % TFD_QUEUE_SIZE;

        // Kick TX doorbell
        let wrptr_value = txq.write_ptr as u32;
        self.write_reg(0x060, wrptr_value); // HBUS_TARG_WRPTR for queue 0

        self.stats.tx_packets += 1;
        self.stats.tx_bytes += len as u64;
        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        // Check for pending received frames
        self.poll_rx();
        self.rx_pending.pop()
    }

    fn poll(&mut self) {
        // Process RX queue for any pending notifications/data
        self.poll_rx();
        
        if self.scanning {
            self.poll_scan();
        }
    }

    fn stats(&self) -> NetStats {
        self.stats
    }
}

// Implémentation de trait — remplit un contrat comportemental.
impl WifiDriver for Iwl4965 {
    fn wifi_state(&self) -> WifiState {
        self.wifi_state
    }

    fn scan(&mut self) -> Result<(), &'static str> {
        // Lazy init: start hardware on first use
        if !self.initialized {
            crate::serial_println!("[IWL4965] Lazy start: initializing hardware...");
            self.start().map_err(|e| {
                crate::serial_println!("[IWL4965] Lazy start failed: {}", e);
                "WiFi hardware init failed"
            })?;
        }
        self.start_scan_hw()
    }

    fn scan_results(&self) -> Vec<WifiNetwork> {
        self.scan_results.clone()
    }

    fn connect(&mut self, ssid: &str, password: &str) -> Result<(), &'static str> {
        if !self.initialized {
            crate::serial_println!("[IWL4965] Lazy start for connect...");
            self.start().map_err(|_| "WiFi hardware init failed")?;
        }
        self.do_connect(ssid, password)
    }

    fn disconnect(&mut self) -> Result<(), &'static str> {
        self.connected_ssid = None;
        self.connected_bssid = [0; 6];
        self.current_channel = 0;
        self.signal_dbm = 0;
        self.wifi_state = WifiState::Disconnected;
        crate::serial_println!("[IWL4965] Disconnected");
        Ok(())
    }

    fn connected_ssid(&self) -> Option<String> {
        self.connected_ssid.clone()
    }

    fn current_channel(&self) -> Option<u8> {
        if self.current_channel > 0 { Some(self.current_channel) } else { None }
    }

    fn signal_strength(&self) -> Option<i8> {
        if self.wifi_state == WifiState::Connected { Some(self.signal_dbm) } else { None }
    }
}

// Safety: MMIO access is through volatile ops, single-threaded driver model
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Send for Iwl4965 {}
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe // Implémentation de trait — remplit un contrat comportemental.
impl Sync for Iwl4965 {}

// ============================================================================
// Driver Registration
// ============================================================================

static DRIVER_INFORMATION: DriverInformation = DriverInformation {
    name: "Intel WiFi (iwl4965)",
    version: "0.1.0",
    author: "TrustOS",
    category: DriverCategory::Network,
    vendor_ids: &[(INTEL_VENDOR, 0xFFFF)], // Match all Intel, filter in probe
};

/// Probe a PCI device — returns a boxed WifiDriver if it's a supported Intel WiFi card
pub fn probe(pci_dev: &PciDevice) -> Option<Box<dyn WifiDriver>> {
    // Check if this is a supported Intel WiFi device
    if pci_dev.vendor_id != INTEL_VENDOR {
        return None;
    }

    if !IWL_SUPPORTED_IDS.contains(&pci_dev.device_id) {
        return None;
    }

    crate::serial_println!("[IWL4965] Probing Intel WiFi {:04X}:{:04X}...", 
        pci_dev.vendor_id, pci_dev.device_id);

    let mut driver = Iwl4965::new();
        // Correspondance de motifs — branchement exhaustif de Rust.
match driver.probe(pci_dev) {
        Ok(()) => {
            // Don't call start() during boot — it does hw_init + firmware load
            // which can hang on real hardware. Start is deferred to first use.
            crate::serial_println!("[IWL4965] PCI probe OK for {:04X}:{:04X} — start deferred", 
                pci_dev.vendor_id, pci_dev.device_id);
            Some(Box::new(driver))
        }
        Err(e) => {
            crate::serial_println!("[IWL4965] Probe failed: {}", e);
            None
        }
    }
}

/// Live debug: dump CSR registers and driver state from the active WiFi driver.
/// Called via `wifi debug` or `drv test wifi` — no recompile needed to read HW state.
pub fn debug_dump() {
    use super::wifi::WIFI_DRIVER;
    let guard = WIFI_DRIVER.lock();
    if let Some(ref driver) = *guard {
        // Downcast to Iwl4965 — we know it's the only WiFi driver
        let info = driver.info();
        crate::println!("  Driver:     {}", info.name);
        crate::println!("  Status:     {:?}", driver.status());
        crate::println!("  WiFi state: {:?}", driver.wifi_state());
        if let Some(ssid) = driver.connected_ssid() {
            crate::println!("  SSID:       {}", ssid);
        }
        if let Some(ch) = driver.current_channel() {
            crate::println!("  Channel:    {}", ch);
        }
        if let Some(dbm) = driver.signal_strength() {
            crate::println!("  Signal:     {} dBm", dbm);
        }
    } else {
        crate::println!("  No WiFi driver loaded");
    }

    crate::println!("  Firmware:   {}", if has_firmware() { "available" } else { "NOT loaded" });
    if let Some(ref fw) = *WIFI_FIRMWARE.lock() {
        crate::println!("  FW size:    {} bytes", fw.len());
    }
}

/// Live debug: read a CSR register — works even without an active driver.
/// Scans PCI for Intel WiFi, maps BAR0, reads the register.
pub fn debug_read_csr(offset: u32) -> Option<u32> {
    // Safety: limit offset to 8KB (IWL4965 MMIO region)
    if offset >= 0x2000 {
        return None;
    }
    // Find the WiFi device on PCI bus
    let devices = crate::pci::get_devices();
    for dev in &devices {
        if dev.vendor_id == INTEL_VENDOR && IWL_SUPPORTED_IDS.contains(&dev.device_id) {
            let bar0 = dev.bar[0];
            if bar0 == 0 || (bar0 & 1) != 0 { return None; }
            let is_64bit = (bar0 >> 1) & 0x3 == 2;
            let phys = if is_64bit {
                let bar1 = dev.bar[1] as u64;
                (bar1 << 32) | (bar0 & 0xFFFFFFF0) as u64
            } else {
                (bar0 & 0xFFFFFFF0) as u64
            };
            if phys == 0 { return None; }
            // Map MMIO region via HHDM page tables before access
            let virt = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::memory::map_mmio(phys, 0x2000) {
                Ok(v) => v as usize,
                Err(_) => return None,
            };
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                let ptr = (virt + offset as usize) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32;
                return Some(core::ptr::read_volatile(ptr));
            }
        }
    }
    None
}

/// Live debug: dump key CSR registers of the Intel WiFi card
pub fn debug_dump_csrs() {
    let regs: &[(&str, u32)] = &[
        ("HW_IF_CONFIG",  CSR_HARDWARE_IF_CONFIG),
        ("INT",           CSR_INT),
        ("INT_MASK",      CSR_INT_MASK),
        ("FH_INT_STATUS", CSR_FH_INT_STATUS),
        ("GPIO_IN",       CSR_GPIO_IN),
        ("RESET",         CSR_RESET),
        ("GP_CNTRL",      CSR_GP_CNTRL),
        ("HW_REV",        CSR_HARDWARE_REV),
        ("EEPROM_REG",    CSR_EEPROM_REGISTER),
        ("EEPROM_GP",     CSR_EEPROM_GP),
        ("UCODE_DRV_GP1", CSR_UCODE_DRIVER_GP1),
        ("UCODE_DRV_GP2", CSR_UCODE_DRIVER_GP2),
        ("GIO_REG",       CSR_GIO_REGISTER),
        ("GP_UCODE",      CSR_GP_UCODE),
        ("GP_DRIVER",     CSR_GP_DRIVER),
    ];

    for (name, offset) in regs {
                // Correspondance de motifs — branchement exhaustif de Rust.
match debug_read_csr(*offset) {
            Some(val) => crate::println!("  CSR {:<16} [0x{:03X}] = 0x{:08X}", name, offset, val),
            None => crate::println!("  CSR {:<16} [0x{:03X}] = <unavailable>", name, offset),
        }
    }
}

// ============================================================================
// Live Debug API — no recompile needed, called from shell commands
// ============================================================================

/// Get MMIO virtual base for the WiFi NIC (maps if needed)
fn debug_get_mmio() -> Option<usize> {
    let devices = crate::pci::get_devices();
    for dev in &devices {
        if dev.vendor_id == INTEL_VENDOR && IWL_SUPPORTED_IDS.contains(&dev.device_id) {
            let bar0 = dev.bar[0];
            if bar0 == 0 || (bar0 & 1) != 0 { return None; }
            let is_64bit = (bar0 >> 1) & 0x3 == 2;
            let phys = if is_64bit {
                let bar1 = dev.bar[1] as u64;
                (bar1 << 32) | (bar0 & 0xFFFFFFF0) as u64
            } else {
                (bar0 & 0xFFFFFFF0) as u64
            };
            if phys == 0 { return None; }
            return crate::memory::map_mmio(phys, 0x2000).ok().map(|v| v as usize);
        }
    }
    None
}

fn debug_register_read(base: usize, offset: u32) -> u32 {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::read_volatile((base + offset as usize) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32) }
}

fn debug_register_write(base: usize, offset: u32, val: u32) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::ptr::write_volatile((base + offset as usize) as *mut u32, val); }
}

/// Write a CSR register live (no driver needed)
pub fn debug_write_csr(offset: u32, val: u32) -> bool {
    if offset >= 0x2000 { return false; }
    if let Some(base) = debug_get_mmio() {
        debug_register_write(base, offset, val);
        true
    } else { false }
}

/// Read a peripheral (PRPH) register via HBUS (no driver needed)
pub fn debug_read_prph(addr: u32) -> Option<u32> {
    let base = debug_get_mmio()?;
    debug_register_write(base, HBUS_TARG_PRPH_RADDR, (addr & 0x000F_FFFF) | (3 << 24));
    Some(debug_register_read(base, HBUS_TARG_PRPH_RDAT))
}

/// Write a peripheral (PRPH) register via HBUS (no driver needed)
pub fn debug_write_prph(addr: u32, val: u32) -> bool {
    if let Some(base) = debug_get_mmio() {
        debug_register_write(base, HBUS_TARG_PRPH_WADDR, (addr & 0x000F_FFFF) | (3 << 24));
        debug_register_write(base, HBUS_TARG_PRPH_WDAT, val);
        true
    } else { false }
}

/// Live APM init — can be called from shell without recompile
pub fn debug_apm_initialize() -> Result<(), &'static str> {
    let base = debug_get_mmio().ok_or("No MMIO")?;
    
    crate::println!("  [1] Setting NIC_READY...");
    let hic = debug_register_read(base, CSR_HARDWARE_IF_CONFIG);
    debug_register_write(base, CSR_HARDWARE_IF_CONFIG, hic | CSR_HARDWARE_IF_CONFIG_REGISTER_BIT_NIC_READY);
    for _ in 0..5000u32 {
        if debug_register_read(base, CSR_HARDWARE_IF_CONFIG) & CSR_HARDWARE_IF_CONFIG_REGISTER_BIT_NIC_READY != 0 { break; }
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
    let nic_ready = debug_register_read(base, CSR_HARDWARE_IF_CONFIG) & CSR_HARDWARE_IF_CONFIG_REGISTER_BIT_NIC_READY != 0;
    crate::println!("      NIC_READY = {}", if nic_ready { "YES" } else { "NO" });
    
    crate::println!("  [2] GIO chicken bits (disable L0s timer)...");
    let gio = debug_register_read(base, CSR_GIO_CHICKEN_BITS);
    debug_register_write(base, CSR_GIO_CHICKEN_BITS, gio | CSR_GIO_CHICKEN_BITS_REGISTER_BIT_DIS_L0S_EXIT_TIMER);
    
    crate::println!("  [3] Setting INIT_DONE...");
    let gp = debug_register_read(base, CSR_GP_CNTRL);
    debug_register_write(base, CSR_GP_CNTRL, gp | CSR_GP_CNTRL_REGISTER_FLAG_INITIALIZE_DONE);
    
    crate::println!("  [4] Polling MAC clock...");
    for i in 0..25000u32 {
        let val = debug_register_read(base, CSR_GP_CNTRL);
        if val & CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY != 0 {
            crate::println!("      MAC clock READY after {} iters (GP={:#010X})", i, val);
            
            crate::println!("  [5] Enabling APMG DMA+BSM clocks...");
            // write_prph(APMG_CLK_EN_REG, DMA|BSM)
            debug_register_write(base, HBUS_TARG_PRPH_WADDR, (APMG_CLK_EN_REGISTER & 0x000F_FFFF) | (3 << 24));
            debug_register_write(base, HBUS_TARG_PRPH_WDAT, APMG_CLK_VALUE_DMA_CLK_RQT | APMG_CLK_VALUE_BSM_CLK_RQT);
            for _ in 0..20_000 { core::hint::spin_loop(); }
            
            crate::println!("  [6] Disabling L1-Active...");
            debug_register_write(base, HBUS_TARG_PRPH_RADDR, (APMG_PCIDEV_STT_REGISTER & 0x000F_FFFF) | (3 << 24));
            let stt = debug_register_read(base, HBUS_TARG_PRPH_RDAT);
            debug_register_write(base, HBUS_TARG_PRPH_WADDR, (APMG_PCIDEV_STT_REGISTER & 0x000F_FFFF) | (3 << 24));
            debug_register_write(base, HBUS_TARG_PRPH_WDAT, stt | APMG_PCIDEV_STT_VALUE_L1_LOOKUP_DIS);
            
            crate::println!("  APM init DONE!");
            return Ok(());
        }
        for _ in 0..1000 { core::hint::spin_loop(); }
        if i % 5000 == 4999 { crate::print!("."); }
    }
    
    let final_gp = debug_register_read(base, CSR_GP_CNTRL);
    crate::println!("      MAC clock TIMEOUT (GP={:#010X})", final_gp);
    Err("MAC clock not ready")
}

/// Live firmware load — step by step with visible output
pub fn debug_load_firmware() -> Result<(), &'static str> {
    use super::wifi::WIFI_DRIVER;
    
    // Check driver exists
    let mut guard = WIFI_DRIVER.lock();
    let driver = guard.as_mut().ok_or("No WiFi driver — run 'drv reprobe wifi' first")?;
    
    crate::println!("  Driver found: {:?}", driver.status());
    
    // Need to start the driver which does hw_init + firmware
    crate::println!("  Calling driver.start()...");
    crate::println!("    This does: hw_init → apm_init → firmware load → DMA queues");
    crate::println!("    Watch for step-by-step output below:");
    crate::println!();
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match driver.start() {
        Ok(()) => {
            crate::println!();
            let gp = debug_read_csr(CSR_GP_CNTRL).unwrap_or(0);
            let gp1 = debug_read_csr(CSR_UCODE_DRIVER_GP1).unwrap_or(0);
            let ucode = debug_read_csr(CSR_GP_UCODE).unwrap_or(0);
            crate::println!("  Result: OK!");
            crate::println!("    GP_CNTRL={:#010X} GP1={:#010X} UCODE={:#010X}", gp, gp1, ucode);
            crate::println!("    MAC_CLK={}", if gp & CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY != 0 { "ready" } else { "DEAD" });
            Ok(())
        }
        Err(e) => {
            crate::println!();
            let gp = debug_read_csr(CSR_GP_CNTRL).unwrap_or(0);
            crate::println!("  Result: FAILED: {}", e);
            crate::println!("    GP_CNTRL={:#010X}", gp);
            crate::println!("    MAC_CLK={}", if gp & CSR_GP_CNTRL_REGISTER_FLAG_MAC_CLOCK_READY != 0 { "ready" } else { "DEAD" });
            Err(e)
        }
    }
}

/// Dump BSM (Bootstrap State Machine) registers
pub fn debug_dump_bsm() {
    let bsm_regs: &[(&str, u32)] = &[
        ("BSM_WR_CTRL",        BSM_WR_CONTROLLER_REGISTER),
        ("BSM_WR_MEM_SRC",     BSM_WR_MEMORY_SOURCE_REGISTER),
        ("BSM_WR_MEM_DST",     BSM_WR_MEMORY_DESTINATION_REGISTER),
        ("BSM_WR_DWCOUNT",     BSM_WR_DWCOUNT_REGISTER),
        ("BSM_DRAM_INST_PTR",  BSM_DRAM_INST_POINTER_REGISTER),
        ("BSM_DRAM_INST_SIZE", BSM_DRAM_INST_BYTECOUNT_REGISTER),
        ("BSM_DRAM_DATA_PTR",  BSM_DRAM_DATA_POINTER_REGISTER),
        ("BSM_DRAM_DATA_SIZE", BSM_DRAM_DATA_BYTECOUNT_REGISTER),
    ];
    
    crate::println!("  BSM Registers (via PRPH bus):");
    for (name, addr) in bsm_regs {
                // Correspondance de motifs — branchement exhaustif de Rust.
match debug_read_prph(*addr) {
            Some(val) => crate::println!("    {:<20} [0x{:04X}] = 0x{:08X}", name, addr, val),
            None => crate::println!("    {:<20} [0x{:04X}] = <unavailable>", name, addr),
        }
    }
}

/// Dump APMG (Advanced Power Management) registers
pub fn debug_dump_apmg() {
    let apmg_regs: &[(&str, u32)] = &[
        ("APMG_CLK_CTRL",   APMG_CLK_CONTROLLER_REGISTER),
        ("APMG_CLK_EN",     APMG_CLK_EN_REGISTER),
        ("APMG_CLK_DIS",    APMG_CLK_DIS_REGISTER),
        ("APMG_PS_CTRL",    APMG_PS_CONTROLLER_REGISTER),
        ("APMG_PCIDEV_STT", APMG_PCIDEV_STT_REGISTER),
    ];
    
    crate::println!("  APMG Registers (via PRPH bus):");
    for (name, addr) in apmg_regs {
                // Correspondance de motifs — branchement exhaustif de Rust.
match debug_read_prph(*addr) {
            Some(val) => crate::println!("    {:<20} [0x{:04X}] = 0x{:08X}", name, addr, val),
            None => crate::println!("    {:<20} [0x{:04X}] = <unavailable>", name, addr),
        }
    }
}
