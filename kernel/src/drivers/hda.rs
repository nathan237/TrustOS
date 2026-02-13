//! Intel High Definition Audio (HDA) Driver
//!
//! Implements Intel HDA controller support for audio output.
//! Reference: Intel High Definition Audio Specification Rev 1.0a
//!
//! Architecture:
//!   Controller (PCI device, class 0x04/0x03) ←→ Codec(s) via link
//!   Commands sent via CORB (Command Output Ring Buffer)
//!   Responses received via RIRB (Response Input Ring Buffer)  
//!   Audio data streamed via DMA through BDL (Buffer Descriptor List)

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

// ═══════════════════════════════════════════════════════════════════════════════
// Register Offsets — Intel HDA Spec §3
// ═══════════════════════════════════════════════════════════════════════════════

/// Global registers
mod reg {
    pub const GCAP: u32     = 0x00;  // 16-bit: Global Capabilities
    pub const VMIN: u32     = 0x02;  // 8-bit: Minor Version
    pub const VMAJ: u32     = 0x03;  // 8-bit: Major Version
    pub const OUTPAY: u32   = 0x04;  // 16-bit: Output Payload Capability
    pub const INPAY: u32    = 0x06;  // 16-bit: Input Payload Capability
    pub const GCTL: u32     = 0x08;  // 32-bit: Global Control
    pub const WAKEEN: u32   = 0x0C;  // 16-bit: Wake Enable
    pub const STATESTS: u32 = 0x0E;  // 16-bit: State Change Status
    pub const GSTS: u32     = 0x10;  // 16-bit: Global Status
    pub const INTCTL: u32   = 0x20;  // 32-bit: Interrupt Control
    pub const INTSTS: u32   = 0x24;  // 32-bit: Interrupt Status
    pub const WALCLK: u32   = 0x30;  // 32-bit: Wall Clock Counter

    // CORB registers
    pub const CORBLBASE: u32 = 0x40;  // 32-bit: CORB Lower Base Address
    pub const CORBUBASE: u32 = 0x44;  // 32-bit: CORB Upper Base Address
    pub const CORBWP: u32    = 0x48;  // 16-bit: CORB Write Pointer
    pub const CORBRP: u32    = 0x4A;  // 16-bit: CORB Read Pointer
    pub const CORBCTL: u32   = 0x4C;  // 8-bit: CORB Control
    pub const CORBSTS: u32   = 0x4D;  // 8-bit: CORB Status
    pub const CORBSIZE: u32  = 0x4E;  // 8-bit: CORB Size

    // RIRB registers
    pub const RIRBLBASE: u32 = 0x50;  // 32-bit: RIRB Lower Base Address
    pub const RIRBUBASE: u32 = 0x54;  // 32-bit: RIRB Upper Base Address
    pub const RIRBWP: u32    = 0x58;  // 16-bit: RIRB Write Pointer
    pub const RINTCNT: u32   = 0x5A;  // 16-bit: Response Interrupt Count
    pub const RIRBCTL: u32   = 0x5C;  // 8-bit: RIRB Control
    pub const RIRBSTS: u32   = 0x5D;  // 8-bit: RIRB Status
    pub const RIRBSIZE: u32  = 0x5E;  // 8-bit: RIRB Size

    // Immediate Command (alternative to CORB/RIRB)
    pub const IC: u32  = 0x60;  // 32-bit: Immediate Command
    pub const IR: u32  = 0x64;  // 32-bit: Immediate Response
    pub const ICS: u32 = 0x68;  // 16-bit: Immediate Command Status

    // DMA Position Buffer
    pub const DPLBASE: u32 = 0x70;  // 32-bit: DMA Position Lower Base
    pub const DPUBASE: u32 = 0x74;  // 32-bit: DMA Position Upper Base

    // Stream Descriptor base (0x80 + n*0x20)
    pub const SD_BASE: u32 = 0x80;
    pub const SD_SIZE: u32 = 0x20;
}

/// Stream descriptor register offsets (relative to stream base)
mod sd {
    pub const CTL: u32    = 0x00;  // 24-bit (3 bytes): Stream Control
    pub const STS: u32    = 0x03;  // 8-bit: Stream Status
    pub const LPIB: u32   = 0x04;  // 32-bit: Link Position In Buffer
    pub const CBL: u32    = 0x08;  // 32-bit: Cyclic Buffer Length
    pub const LVI: u32    = 0x0C;  // 16-bit: Last Valid Index
    pub const FIFOS: u32  = 0x10;  // 16-bit: FIFO Size
    pub const FMT: u32    = 0x12;  // 16-bit: Stream Format
    pub const BDLPL: u32  = 0x18;  // 32-bit: BDL Lower Address
    pub const BDLPU: u32  = 0x1C;  // 32-bit: BDL Upper Address
}

/// Global Control bits
mod gctl {
    pub const CRST: u32   = 1 << 0;   // Controller Reset
    pub const FCNTRL: u32 = 1 << 1;   // Flush Control
    pub const UNSOL: u32  = 1 << 8;   // Accept Unsolicited Responses
}

/// Stream Control bits
mod sctl {
    pub const SRST: u32 = 1 << 0;     // Stream Reset
    pub const RUN: u32  = 1 << 1;     // Stream Run (DMA enable)
    pub const IOCE: u32 = 1 << 2;     // Interrupt On Completion Enable
    // Bits [23:20] = Stream Number (tag)
    pub const STREAM_TAG_SHIFT: u32 = 20;
}

/// Stream Status bits
mod ssts {
    pub const BCIS: u8 = 1 << 2;   // Buffer Completion Interrupt Status
    pub const FIFOE: u8 = 1 << 3;  // FIFO Error
    pub const DESE: u8 = 1 << 4;   // Descriptor Error
    pub const FIFORDY: u8 = 1 << 5; // FIFO Ready
}

// ═══════════════════════════════════════════════════════════════════════════════
// Codec Verbs & Parameters — Intel HDA Spec §7
// ═══════════════════════════════════════════════════════════════════════════════

mod verb {
    // GET verbs (12-bit)
    pub const GET_PARAMETER: u32        = 0xF00;
    pub const GET_CONN_LIST: u32        = 0xF02;
    pub const GET_CONN_SELECT: u32      = 0xF01;
    pub const GET_AMP_GAIN: u32         = 0xB00;
    pub const GET_PIN_CONTROL: u32      = 0xF07;
    pub const GET_CONFIG_DEFAULT: u32   = 0xF1C;
    pub const GET_EAPD: u32             = 0xF0C;
    pub const GET_POWER_STATE: u32      = 0xF05;
    pub const GET_STREAM_FORMAT: u32    = 0xA00;
    pub const GET_CHANNEL_STREAM: u32   = 0xF06;

    // SET verbs (4-bit verb + 8-bit payload → or 12-bit verb variants)
    pub const SET_CONN_SELECT: u32      = 0x701;
    pub const SET_POWER_STATE: u32      = 0x705;
    pub const SET_CHANNEL_STREAM: u32   = 0x706;
    pub const SET_PIN_CONTROL: u32      = 0x707;
    pub const SET_EAPD: u32             = 0x70C;
    pub const SET_AMP_GAIN_MUTE: u32    = 0x300;  // 12-bit verb, 8-bit payload in command
    pub const SET_STREAM_FORMAT: u32    = 0x200;  // 12-bit verb, 16-bit payload

    // Parameters (used with GET_PARAMETER)
    pub const PARAM_VENDOR_ID: u32     = 0x00;
    pub const PARAM_REVISION: u32      = 0x02;
    pub const PARAM_NODE_COUNT: u32    = 0x04;
    pub const PARAM_FN_GROUP_TYPE: u32 = 0x05;
    pub const PARAM_AUDIO_CAPS: u32    = 0x09;  // Audio Widget Capabilities
    pub const PARAM_PCM_RATES: u32     = 0x0A;  // Supported PCM sizes/rates
    pub const PARAM_STREAM_FMTS: u32   = 0x0B;  // Supported stream formats
    pub const PARAM_PIN_CAPS: u32      = 0x0C;  // Pin Capabilities
    pub const PARAM_AMP_IN_CAPS: u32   = 0x0D;  // Input Amp Capabilities
    pub const PARAM_CONN_LIST_LEN: u32 = 0x0E;  // Connection List Length
    pub const PARAM_POWER_STATES: u32  = 0x0F;  // Supported Power States
    pub const PARAM_AMP_OUT_CAPS: u32  = 0x12;  // Output Amp Capabilities
    pub const PARAM_VOL_KNOB_CAPS: u32 = 0x13;
}

/// Widget types (bits [23:20] of Audio Widget Capabilities)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WidgetType {
    AudioOutput  = 0,
    AudioInput   = 1,
    AudioMixer   = 2,
    AudioSelector = 3,
    PinComplex   = 4,
    Power        = 5,
    VolumeKnob   = 6,
    BeepGen      = 7,
    VendorDef    = 0xF,
    Unknown      = 0xFF,
}

impl WidgetType {
    fn from_caps(caps: u32) -> Self {
        match (caps >> 20) & 0xF {
            0 => Self::AudioOutput,
            1 => Self::AudioInput,
            2 => Self::AudioMixer,
            3 => Self::AudioSelector,
            4 => Self::PinComplex,
            5 => Self::Power,
            6 => Self::VolumeKnob,
            7 => Self::BeepGen,
            0xF => Self::VendorDef,
            _ => Self::Unknown,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Self::AudioOutput => "Audio Output (DAC)",
            Self::AudioInput => "Audio Input (ADC)",
            Self::AudioMixer => "Audio Mixer",
            Self::AudioSelector => "Audio Selector",
            Self::PinComplex => "Pin Complex",
            Self::Power => "Power Widget",
            Self::VolumeKnob => "Volume Knob",
            Self::BeepGen => "Beep Generator",
            Self::VendorDef => "Vendor Defined",
            Self::Unknown => "Unknown",
        }
    }
}

/// Pin default config — device type from bits [23:20]
fn pin_default_device(config: u32) -> &'static str {
    match (config >> 20) & 0xF {
        0x0 => "Line Out",
        0x1 => "Speaker",
        0x2 => "HP Out",
        0x3 => "CD",
        0x4 => "SPDIF Out",
        0x5 => "Digital Other Out",
        0x6 => "Modem Line Side",
        0x7 => "Modem Handset",
        0x8 => "Line In",
        0x9 => "AUX",
        0xA => "Mic In",
        0xB => "Telephony",
        0xC => "SPDIF In",
        0xD => "Digital Other In",
        0xE => "Reserved",
        0xF => "Other",
        _ => "?",
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Structures
// ═══════════════════════════════════════════════════════════════════════════════

/// Widget info discovered from codec
#[derive(Debug, Clone)]
pub struct Widget {
    pub nid: u16,
    pub widget_type: WidgetType,
    pub caps: u32,
    pub pin_config: u32,
    pub connections: Vec<u16>,
    pub amp_in_caps: u32,
    pub amp_out_caps: u32,
}

/// A discovered audio path: PinComplex → ... → DAC
#[derive(Debug, Clone)]
pub struct AudioPath {
    pub pin_nid: u16,
    pub dac_nid: u16,
    pub path: Vec<u16>,  // NIDs from pin to DAC
    pub device_type: &'static str,
}

/// BDL Entry — Buffer Descriptor List entry (16 bytes, §3.6.3)
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
struct BdlEntry {
    address: u64,    // Physical address of audio buffer
    length: u32,     // Byte length
    ioc: u32,        // Bit 0 = Interrupt On Completion
}

/// HDA Controller state
pub struct HdaController {
    /// MMIO base virtual address
    mmio_base: u64,
    /// Number of input streams
    num_iss: u8,
    /// Number of output streams
    num_oss: u8,
    /// Number of bidirectional streams
    num_bss: u8,
    /// 64-bit addressing supported
    addr64: bool,

    /// CORB buffer (virtual address, leaked allocation)
    corb_virt: u64,
    corb_phys: u64,
    corb_entries: u16,

    /// RIRB buffer (virtual address, leaked allocation)
    rirb_virt: u64,
    rirb_phys: u64,
    rirb_entries: u16,
    rirb_rp: u16,  // Software read pointer

    /// Discovered codec addresses
    codecs: Vec<u8>,
    /// Discovered widgets per codec
    widgets: Vec<Widget>,
    /// Discovered audio output paths
    output_paths: Vec<AudioPath>,

    /// Output stream state
    stream_tag: u8,
    /// Audio buffer (virtual)
    audio_buf_virt: u64,
    audio_buf_phys: u64,
    audio_buf_size: u32,
    /// BDL (virtual)
    bdl_virt: u64,
    bdl_phys: u64,

    /// Is audio currently playing?
    playing: bool,
}

/// Global HDA controller instance
static HDA: Mutex<Option<HdaController>> = Mutex::new(None);
static HDA_INITIALIZED: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// MMIO Helpers
// ═══════════════════════════════════════════════════════════════════════════════

impl HdaController {
    #[inline]
    unsafe fn read8(&self, offset: u32) -> u8 {
        core::ptr::read_volatile((self.mmio_base + offset as u64) as *const u8)
    }

    #[inline]
    unsafe fn read16(&self, offset: u32) -> u16 {
        core::ptr::read_volatile((self.mmio_base + offset as u64) as *const u16)
    }

    #[inline]
    unsafe fn read32(&self, offset: u32) -> u32 {
        core::ptr::read_volatile((self.mmio_base + offset as u64) as *const u32)
    }

    #[inline]
    unsafe fn write8(&self, offset: u32, val: u8) {
        core::ptr::write_volatile((self.mmio_base + offset as u64) as *mut u8, val);
    }

    #[inline]
    unsafe fn write16(&self, offset: u32, val: u16) {
        core::ptr::write_volatile((self.mmio_base + offset as u64) as *mut u16, val);
    }

    #[inline]
    unsafe fn write32(&self, offset: u32, val: u32) {
        core::ptr::write_volatile((self.mmio_base + offset as u64) as *mut u32, val);
    }

    /// Stream descriptor register base for output stream index `n`
    fn osd_base(&self, n: u8) -> u32 {
        reg::SD_BASE + ((self.num_iss + n) as u32) * reg::SD_SIZE
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Phase 1: Controller Initialization
    // ═════════════════════════════════════════════════════════════════════════

    /// Initialize the HDA controller from a PCI device
    pub fn init(dev: &crate::pci::PciDevice) -> Result<Self, &'static str> {
        crate::serial_println!("[HDA] Initializing Intel HDA controller...");
        crate::serial_println!("[HDA]   PCI {:02X}:{:02X}.{} {:04X}:{:04X}",
            dev.bus, dev.device, dev.function, dev.vendor_id, dev.device_id);

        // Enable bus mastering + memory space
        crate::pci::enable_bus_master(dev);
        crate::pci::enable_memory_space(dev);

        // Get BAR0 (MMIO base)
        let bar0_phys = dev.bar_address(0).ok_or("HDA: no BAR0")?;
        crate::serial_println!("[HDA]   BAR0 phys = {:#010X}", bar0_phys);

        // Map MMIO pages (HDA register space is typically 16 KB)
        let hhdm = crate::memory::hhdm_offset();
        let mmio_base = bar0_phys + hhdm;

        // Map 4 pages (16 KB) for register space
        for page in 0..4 {
            let phys = (bar0_phys & !0xFFF) + page * 0x1000;
            let virt = phys + hhdm;
            crate::memory::paging::map_kernel_mmio_page(virt, phys)?;
        }

        crate::serial_println!("[HDA]   MMIO mapped at virt {:#018X}", mmio_base);

        let mut ctrl = HdaController {
            mmio_base,
            num_iss: 0, num_oss: 0, num_bss: 0,
            addr64: false,
            corb_virt: 0, corb_phys: 0, corb_entries: 0,
            rirb_virt: 0, rirb_phys: 0, rirb_entries: 0,
            rirb_rp: 0,
            codecs: Vec::new(),
            widgets: Vec::new(),
            output_paths: Vec::new(),
            stream_tag: 1,
            audio_buf_virt: 0, audio_buf_phys: 0, audio_buf_size: 0,
            bdl_virt: 0, bdl_phys: 0,
            playing: false,
        };

        // Read capabilities
        unsafe {
            let gcap = ctrl.read16(reg::GCAP);
            let vmin = ctrl.read8(reg::VMIN);
            let vmaj = ctrl.read8(reg::VMAJ);

            ctrl.num_oss = ((gcap >> 12) & 0xF) as u8;
            ctrl.num_iss = ((gcap >> 8) & 0xF) as u8;
            ctrl.num_bss = ((gcap >> 3) & 0x1F) as u8;
            ctrl.addr64 = (gcap & 1) != 0;

            crate::serial_println!("[HDA]   Version {}.{}", vmaj, vmin);
            crate::serial_println!("[HDA]   Streams: {} output, {} input, {} bidir",
                ctrl.num_oss, ctrl.num_iss, ctrl.num_bss);
            crate::serial_println!("[HDA]   64-bit: {}", ctrl.addr64);

            if ctrl.num_oss == 0 {
                return Err("HDA: no output streams available");
            }
        }

        // Controller reset
        ctrl.reset()?;

        // Setup CORB/RIRB
        ctrl.setup_corb_rirb()?;

        // Discover codecs
        ctrl.discover_codecs()?;

        // Find output paths
        ctrl.find_output_paths();

        // Setup output stream
        ctrl.setup_output_stream()?;

        crate::serial_println!("[HDA] Initialization complete!");
        Ok(ctrl)
    }

    /// Reset the controller (§4.2.2)
    fn reset(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[HDA] Resetting controller...");
        unsafe {
            // Clear STATESTS
            self.write16(reg::STATESTS, 0xFFFF);

            // Enter reset: clear CRST
            let gctl = self.read32(reg::GCTL);
            self.write32(reg::GCTL, gctl & !gctl::CRST);

            // Wait for CRST to read 0
            for _ in 0..1000 {
                if self.read32(reg::GCTL) & gctl::CRST == 0 {
                    break;
                }
                Self::delay_us(10);
            }
            if self.read32(reg::GCTL) & gctl::CRST != 0 {
                return Err("HDA: reset enter timeout");
            }

            // Exit reset: set CRST
            let gctl = self.read32(reg::GCTL);
            self.write32(reg::GCTL, gctl | gctl::CRST);

            // Wait for CRST to read 1
            for _ in 0..1000 {
                if self.read32(reg::GCTL) & gctl::CRST != 0 {
                    break;
                }
                Self::delay_us(10);
            }
            if self.read32(reg::GCTL) & gctl::CRST == 0 {
                return Err("HDA: reset exit timeout");
            }

            // Wait for codecs to initialize (~521 µs per spec)
            Self::delay_us(600);

            // Enable unsolicited responses
            let gctl = self.read32(reg::GCTL);
            self.write32(reg::GCTL, gctl | gctl::UNSOL);

            // Check which codecs are present
            let statests = self.read16(reg::STATESTS);
            crate::serial_println!("[HDA]   STATESTS = {:#06X} (codec presence)", statests);

            if statests == 0 {
                return Err("HDA: no codecs detected after reset");
            }

            // Record codec addresses
            for i in 0..15u8 {
                if statests & (1 << i) != 0 {
                    self.codecs.push(i);
                    crate::serial_println!("[HDA]   Codec {} present", i);
                }
            }
        }
        Ok(())
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Phase 2: CORB / RIRB Setup
    // ═════════════════════════════════════════════════════════════════════════

    fn setup_corb_rirb(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[HDA] Setting up CORB/RIRB...");
        let hhdm = crate::memory::hhdm_offset();

        unsafe {
            // ── Stop CORB & RIRB ──
            self.write8(reg::CORBCTL, 0);
            self.write8(reg::RIRBCTL, 0);
            Self::delay_us(100);

            // ── CORB size: pick largest supported ──
            let corbsize_cap = self.read8(reg::CORBSIZE);
            let (corb_sz_sel, corb_entries) = if corbsize_cap & 0x40 != 0 {
                (2u8, 256u16)
            } else if corbsize_cap & 0x20 != 0 {
                (1, 16)
            } else {
                (0, 2)
            };
            self.write8(reg::CORBSIZE, corb_sz_sel);
            self.corb_entries = corb_entries;
            crate::serial_println!("[HDA]   CORB: {} entries", corb_entries);

            // Allocate CORB buffer (4 bytes per entry, page-aligned)
            let corb_bytes = (corb_entries as usize) * 4;
            let corb_buf: Vec<u8> = vec![0u8; corb_bytes + 4096]; // extra for alignment
            let corb_virt_raw = corb_buf.as_ptr() as u64;
            let corb_virt = (corb_virt_raw + 0xFFF) & !0xFFF; // page-align
            core::mem::forget(corb_buf);

            let corb_phys = corb_virt.checked_sub(hhdm)
                .ok_or("HDA: CORB virt->phys failed")?;
            self.corb_virt = corb_virt;
            self.corb_phys = corb_phys;

            // Zero the buffer
            core::ptr::write_bytes(corb_virt as *mut u8, 0, corb_bytes);

            // Set CORB base address
            self.write32(reg::CORBLBASE, corb_phys as u32);
            self.write32(reg::CORBUBASE, (corb_phys >> 32) as u32);

            // Reset CORB read pointer
            self.write16(reg::CORBRP, 1 << 15); // Set reset bit
            Self::delay_us(100);
            // Some controllers need reset bit cleared
            self.write16(reg::CORBRP, 0);
            Self::delay_us(100);

            // Set CORB write pointer to 0
            self.write16(reg::CORBWP, 0);

            // ── RIRB size ──
            let rirbsize_cap = self.read8(reg::RIRBSIZE);
            let (rirb_sz_sel, rirb_entries) = if rirbsize_cap & 0x40 != 0 {
                (2u8, 256u16)
            } else if rirbsize_cap & 0x20 != 0 {
                (1, 16)
            } else {
                (0, 2)
            };
            self.write8(reg::RIRBSIZE, rirb_sz_sel);
            self.rirb_entries = rirb_entries;
            crate::serial_println!("[HDA]   RIRB: {} entries", rirb_entries);

            // Allocate RIRB buffer (8 bytes per entry)
            let rirb_bytes = (rirb_entries as usize) * 8;
            let rirb_buf: Vec<u8> = vec![0u8; rirb_bytes + 4096];
            let rirb_virt_raw = rirb_buf.as_ptr() as u64;
            let rirb_virt = (rirb_virt_raw + 0xFFF) & !0xFFF;
            core::mem::forget(rirb_buf);

            let rirb_phys = rirb_virt.checked_sub(hhdm)
                .ok_or("HDA: RIRB virt->phys failed")?;
            self.rirb_virt = rirb_virt;
            self.rirb_phys = rirb_phys;

            core::ptr::write_bytes(rirb_virt as *mut u8, 0, rirb_bytes);

            // Set RIRB base address
            self.write32(reg::RIRBLBASE, rirb_phys as u32);
            self.write32(reg::RIRBUBASE, (rirb_phys >> 32) as u32);

            // Reset RIRB write pointer
            self.write16(reg::RIRBWP, 1 << 15);
            Self::delay_us(100);

            // Set response interrupt count
            self.write16(reg::RINTCNT, 1);

            self.rirb_rp = 0;

            // ── Start CORB & RIRB ──
            self.write8(reg::CORBCTL, 0x02); // CORBRUN
            self.write8(reg::RIRBCTL, 0x02); // RIRBDMAEN
            Self::delay_us(100);

            crate::serial_println!("[HDA]   CORB phys={:#010X}, RIRB phys={:#010X}",
                corb_phys, rirb_phys);
        }

        Ok(())
    }

    /// Send a codec verb via CORB and wait for response via RIRB
    fn send_verb(&mut self, codec: u8, nid: u16, verb: u32, payload: u32) -> Result<u32, &'static str> {
        // Build command word: [31:28]=codec, [27:20]=nid, [19:0]=verb+payload
        let cmd = ((codec as u32) << 28)
            | ((nid as u32 & 0xFF) << 20)
            | (verb & 0xFFFFF);
        // For 4-bit verbs: verb is [19:16], payload is [15:0]
        // For 12-bit verbs: verb is [19:8], payload is [7:0]
        // Actually the caller should pre-compose verb+payload into the bottom 20 bits
        let _ = payload; // payload already included in verb for our API

        unsafe {
            // Write command to CORB
            let wp = self.read16(reg::CORBWP) & 0xFF;
            let new_wp = ((wp + 1) % self.corb_entries) as u16;

            let corb_ptr = self.corb_virt as *mut u32;
            core::ptr::write_volatile(corb_ptr.add(new_wp as usize), cmd);

            // Advance CORB write pointer
            self.write16(reg::CORBWP, new_wp);

            // Wait for RIRB response
            for _ in 0..10000 {
                let rirb_wp = self.read16(reg::RIRBWP) & 0xFF;
                if rirb_wp != self.rirb_rp {
                    // Read response
                    self.rirb_rp = (self.rirb_rp + 1) % self.rirb_entries;
                    let rirb_ptr = self.rirb_virt as *const u64;
                    let response = core::ptr::read_volatile(rirb_ptr.add(self.rirb_rp as usize));
                    let data = response as u32;
                    // Clear RIRB status
                    self.write8(reg::RIRBSTS, 0x05);
                    return Ok(data);
                }
                Self::delay_us(10);
            }
        }
        Err("HDA: RIRB timeout")
    }

    /// Higher-level: send a 12-bit verb with 8-bit data
    fn codec_cmd(&mut self, codec: u8, nid: u16, verb: u32, data: u8) -> Result<u32, &'static str> {
        let full_verb = (verb << 8) | (data as u32);
        self.send_verb(codec, nid, full_verb, 0)
    }

    /// Get parameter from a codec node
    fn get_param(&mut self, codec: u8, nid: u16, param: u32) -> Result<u32, &'static str> {
        self.codec_cmd(codec, nid, verb::GET_PARAMETER, param as u8)
    }

    /// Set verb (4-bit verb ID in bits [19:16], 16-bit payload in [15:0])
    fn set_verb_16(&mut self, codec: u8, nid: u16, verb_id: u32, payload: u16) -> Result<u32, &'static str> {
        // 4-bit verbs: [19:16]=verb, [15:0]=payload 
        // verb_id like 0x200, 0x300 already include position
        let full = (verb_id & 0xF0000) | ((verb_id & 0xFFF) << 8) | 0;
        // Actually let's use the raw 20-bit approach:
        // For SET_STREAM_FORMAT (verb 0x2, 16-bit payload): cmd[19:16]=0x2, cmd[15:0]=payload
        let raw20 = ((verb_id & 0xF00) << 8) | (payload as u32);
        self.send_verb(codec, nid, raw20, 0)
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Phase 3: Codec Discovery
    // ═════════════════════════════════════════════════════════════════════════

    fn discover_codecs(&mut self) -> Result<(), &'static str> {
        let codecs = self.codecs.clone();
        for &caddr in &codecs {
            crate::serial_println!("[HDA] Walking codec {}...", caddr);

            // Get vendor/device
            let vendor = self.get_param(caddr, 0, verb::PARAM_VENDOR_ID)?;
            crate::serial_println!("[HDA]   Vendor={:04X}, Device={:04X}",
                vendor >> 16, vendor & 0xFFFF);

            // Get sub-node count from root (NID 0)
            let node_count = self.get_param(caddr, 0, verb::PARAM_NODE_COUNT)?;
            let start_nid = ((node_count >> 16) & 0xFF) as u16;
            let num_nodes = (node_count & 0xFF) as u16;
            crate::serial_println!("[HDA]   Root: subnodes {}..{}", start_nid, start_nid + num_nodes - 1);

            // Walk function groups
            for fg_nid in start_nid..(start_nid + num_nodes) {
                let fg_type = self.get_param(caddr, fg_nid, verb::PARAM_FN_GROUP_TYPE)?;
                let fg_type_id = fg_type & 0xFF;
                crate::serial_println!("[HDA]   FG NID {}: type={} ({})", fg_nid, fg_type_id,
                    if fg_type_id == 1 { "Audio" } else { "Other" });

                if fg_type_id != 1 { continue; } // Only Audio Function Group

                // Power on the AFG
                let _ = self.codec_cmd(caddr, fg_nid, verb::SET_POWER_STATE, 0x00); // D0

                // Get sub-nodes of this function group
                let sub_count = self.get_param(caddr, fg_nid, verb::PARAM_NODE_COUNT)?;
                let w_start = ((sub_count >> 16) & 0xFF) as u16;
                let w_count = (sub_count & 0xFF) as u16;
                crate::serial_println!("[HDA]   AFG widgets: {}..{}", w_start, w_start + w_count - 1);

                // Walk each widget
                for nid in w_start..(w_start + w_count) {
                    let caps = self.get_param(caddr, nid, verb::PARAM_AUDIO_CAPS)?;
                    let wtype = WidgetType::from_caps(caps);

                    let mut widget = Widget {
                        nid,
                        widget_type: wtype,
                        caps,
                        pin_config: 0,
                        connections: Vec::new(),
                        amp_in_caps: 0,
                        amp_out_caps: 0,
                    };

                    // Get connection list
                    let conn_len_raw = self.get_param(caddr, nid, verb::PARAM_CONN_LIST_LEN)?;
                    let conn_len = (conn_len_raw & 0x7F) as u16;
                    let long_form = (conn_len_raw & 0x80) != 0;

                    if conn_len > 0 && !long_form {
                        // Read connection list entries (4 per response for short form)
                        let mut offset = 0u8;
                        while (offset as u16) < conn_len {
                            let resp = self.codec_cmd(caddr, nid, verb::GET_CONN_LIST, offset)?;
                            for i in 0..4u32 {
                                if (offset as u16) + (i as u16) >= conn_len { break; }
                                let conn_nid = ((resp >> (i * 8)) & 0xFF) as u16;
                                widget.connections.push(conn_nid);
                            }
                            offset += 4;
                        }
                    }

                    // Pin-specific data
                    if wtype == WidgetType::PinComplex {
                        widget.pin_config = self.codec_cmd(caddr, nid, verb::GET_CONFIG_DEFAULT, 0)?;
                    }

                    // Amp capabilities
                    if caps & (1 << 1) != 0 { // Has output amp
                        widget.amp_out_caps = self.get_param(caddr, nid, verb::PARAM_AMP_OUT_CAPS)?;
                    }
                    if caps & (1 << 2) != 0 { // Has input amp
                        widget.amp_in_caps = self.get_param(caddr, nid, verb::PARAM_AMP_IN_CAPS)?;
                    }

                    crate::serial_println!("[HDA]     NID {:3}: {} conns={:?}{}",
                        nid, wtype.name(),
                        widget.connections,
                        if wtype == WidgetType::PinComplex {
                            alloc::format!(" [{}]", pin_default_device(widget.pin_config))
                        } else {
                            String::new()
                        }
                    );

                    self.widgets.push(widget);
                }
            }
        }
        Ok(())
    }

    /// Find output audio paths: Pin Complex (output) → ... → DAC
    fn find_output_paths(&mut self) {
        crate::serial_println!("[HDA] Searching output paths...");

        // Find all output pin complexes
        let pins: Vec<(u16, u32, Vec<u16>)> = self.widgets.iter()
            .filter(|w| w.widget_type == WidgetType::PinComplex)
            .filter(|w| {
                // Check if pin is an output type (connectivity != "No connection")
                let connectivity = (w.pin_config >> 30) & 0x3;
                let default_device = (w.pin_config >> 20) & 0xF;
                connectivity != 1 && // Not "no connection"
                (default_device <= 0x2 || default_device == 0x5) // Line Out, Speaker, HP, SPDIF
            })
            .map(|w| (w.nid, w.pin_config, w.connections.clone()))
            .collect();

        for (pin_nid, pin_config, pin_conns) in &pins {
            // Walk backward from pin to find a DAC
            if let Some(path) = self.trace_to_dac(*pin_nid, &mut Vec::new()) {
                let device = pin_default_device(*pin_config);
                crate::serial_println!("[HDA]   Path found: {} -> {:?}", device,
                    path.iter().map(|n| alloc::format!("{}", n)).collect::<Vec<_>>());
                self.output_paths.push(AudioPath {
                    pin_nid: *pin_nid,
                    dac_nid: *path.last().unwrap_or(&0),
                    path: path,
                    device_type: device,
                });
            }
        }

        if self.output_paths.is_empty() {
            crate::serial_println!("[HDA]   WARNING: No output paths found!");
        } else {
            crate::serial_println!("[HDA]   {} output path(s) found", self.output_paths.len());
        }
    }

    /// Recursively trace from a widget to a DAC (AudioOutput)
    fn trace_to_dac(&self, nid: u16, visited: &mut Vec<u16>) -> Option<Vec<u16>> {
        if visited.contains(&nid) { return None; } // Cycle detection
        visited.push(nid);

        let widget = self.widgets.iter().find(|w| w.nid == nid)?;

        if widget.widget_type == WidgetType::AudioOutput {
            return Some(vec![nid]); // Found a DAC!
        }

        // Try each connection
        for &conn_nid in &widget.connections {
            if let Some(mut path) = self.trace_to_dac(conn_nid, visited) {
                path.insert(0, nid);
                return Some(path);
            }
        }

        None
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Phase 4: Output Stream Setup
    // ═════════════════════════════════════════════════════════════════════════

    fn setup_output_stream(&mut self) -> Result<(), &'static str> {
        if self.output_paths.is_empty() {
            return Err("HDA: no output paths to configure");
        }

        let hhdm = crate::memory::hhdm_offset();
        let codec = self.codecs[0];
        let path = self.output_paths[0].clone();

        crate::serial_println!("[HDA] Setting up output stream for path: {:?}", path.path);

        // ── Configure the codec path ──
        // Power on all widgets in the path
        for &nid in &path.path {
            let _ = self.codec_cmd(codec, nid, verb::SET_POWER_STATE, 0x00); // D0
        }

        // Configure the pin
        let _ = self.codec_cmd(codec, path.pin_nid, verb::SET_PIN_CONTROL, 0xC0); // HP amp + out
        // Try EAPD
        let eapd = self.codec_cmd(codec, path.pin_nid, verb::GET_EAPD, 0).unwrap_or(0);
        let _ = self.codec_cmd(codec, path.pin_nid, verb::SET_EAPD, (eapd as u8) | 0x02);

        // Set stream tag on DAC
        let stream_tag = self.stream_tag;
        let channel = 0u8;
        let _ = self.codec_cmd(codec, path.dac_nid, verb::SET_CHANNEL_STREAM,
            (stream_tag << 4) | channel);

        // Set stream format on DAC: 48 kHz, 16-bit, stereo
        // FMT: base=48kHz(0), mult=x1(0), div=/1(0), bits=16(001), chan=2-1(0001)
        let fmt: u16 = 0x0011; // 48kHz, 16-bit, stereo
        let _ = self.set_verb_16(codec, path.dac_nid, verb::SET_STREAM_FORMAT, fmt);

        // Unmute output amps along the path
        for &nid in &path.path {
            let widget = self.widgets.iter().find(|w| w.nid == nid);
            if let Some(w) = widget {
                if w.caps & (1 << 1) != 0 { // Has output amp
                    // SET_AMP_GAIN_MUTE: [15]=1(output), [14:13]=11(L+R), [12]=0(not mute), [6:0]=gain
                    let amp_cmd = 0xB000 | (1 << 15) | (3 << 13) | 0x7F; // Max gain
                    let _ = self.send_verb(codec, nid, (0x300 << 8) | (amp_cmd & 0xFF), 0);
                    // Actually SET_AMP_GAIN_MUTE is verb 0x3, payload is 16 bits
                    // cmd[19:16] = 0x3, cmd[15:0] = amp_data
                    let amp_data: u16 = (1 << 15) | (1 << 13) | (1 << 12) | 0x7F;
                    let _ = self.set_verb_16(codec, nid, 0x300, amp_data);
                    let amp_data2: u16 = (1 << 15) | (1 << 14) | (1 << 12) | 0x7F;
                    let _ = self.set_verb_16(codec, nid, 0x300, amp_data2);
                }
            }
        }

        // ── Setup DMA stream ──
        let sd_base = self.osd_base(0); // First output stream

        unsafe {
            // Reset stream
            let ctl = self.read32(sd_base + sd::CTL) & 0xFF;
            self.write8(sd_base + sd::CTL, (ctl as u8) | sctl::SRST as u8);
            Self::delay_us(100);
            // Wait for reset
            for _ in 0..1000 {
                if self.read8(sd_base + sd::CTL) & (sctl::SRST as u8) != 0 { break; }
                Self::delay_us(10);
            }
            // Clear reset
            self.write8(sd_base + sd::CTL, 0);
            for _ in 0..1000 {
                if self.read8(sd_base + sd::CTL) & (sctl::SRST as u8) == 0 { break; }
                Self::delay_us(10);
            }

            // Clear status
            self.write8(sd_base + sd::STS, 0x1C);

            // Allocate audio buffer: 2 fragments × 16KB = 32KB
            let frag_size: u32 = 16384; // 16 KB per fragment
            let num_frags: u32 = 2;
            let total_size = frag_size * num_frags;

            let audio_buf: Vec<u8> = vec![0u8; total_size as usize + 4096];
            let buf_virt_raw = audio_buf.as_ptr() as u64;
            let buf_virt = (buf_virt_raw + 0xFFF) & !0xFFF;
            core::mem::forget(audio_buf);

            let buf_phys = buf_virt.checked_sub(hhdm)
                .ok_or("HDA: audio buf virt->phys failed")?;

            self.audio_buf_virt = buf_virt;
            self.audio_buf_phys = buf_phys;
            self.audio_buf_size = total_size;

            // Zero the audio buffer
            core::ptr::write_bytes(buf_virt as *mut u8, 0, total_size as usize);

            // Allocate BDL: 2 entries × 16 bytes = 32 bytes (128-byte aligned)
            let bdl_buf: Vec<u8> = vec![0u8; 256 + 4096]; // oversized for alignment
            let bdl_virt_raw = bdl_buf.as_ptr() as u64;
            let bdl_virt = (bdl_virt_raw + 127) & !127; // 128-byte align
            core::mem::forget(bdl_buf);

            let bdl_phys = bdl_virt.checked_sub(hhdm)
                .ok_or("HDA: BDL virt->phys failed")?;

            self.bdl_virt = bdl_virt;
            self.bdl_phys = bdl_phys;

            // Fill BDL entries
            let bdl = bdl_virt as *mut BdlEntry;
            for i in 0..num_frags {
                let entry = &mut *bdl.add(i as usize);
                entry.address = buf_phys + (i as u64) * (frag_size as u64);
                entry.length = frag_size;
                entry.ioc = 1; // Interrupt on completion
            }

            // Configure stream descriptor
            self.write32(sd_base + sd::CBL, total_size);  // Cyclic buffer length
            self.write16(sd_base + sd::LVI, (num_frags - 1) as u16); // Last valid index
            self.write16(sd_base + sd::FMT, fmt);  // Stream format
            self.write32(sd_base + sd::BDLPL, bdl_phys as u32);
            self.write32(sd_base + sd::BDLPU, (bdl_phys >> 32) as u32);

            // Set stream tag in CTL bits [23:20]
            let ctl_high = (stream_tag as u32) << (sctl::STREAM_TAG_SHIFT - 16);
            self.write8(sd_base + sd::CTL + 2, ctl_high as u8);

            crate::serial_println!("[HDA]   Stream configured: 48kHz 16-bit stereo");
            crate::serial_println!("[HDA]   Audio buf phys={:#010X} size={}",
                buf_phys, total_size);
            crate::serial_println!("[HDA]   BDL phys={:#010X} entries={}", bdl_phys, num_frags);
        }

        Ok(())
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Phase 5: Audio Playback
    // ═════════════════════════════════════════════════════════════════════════

    /// Fill the audio buffer with a sine tone
    pub fn fill_tone(&mut self, freq_hz: u32, duration_ms: u32) {
        let sample_rate = 48000u32;
        let channels = 2u32;
        let bytes_per_sample = 2u32; // 16-bit
        let total_samples = (sample_rate * duration_ms / 1000) as usize;
        let buf_samples = (self.audio_buf_size / (channels * bytes_per_sample)) as usize;
        let samples_to_fill = total_samples.min(buf_samples);

        let buf = self.audio_buf_virt as *mut i16;

        // Integer sine approximation (triangle wave approximation scaled)
        // Period in samples
        let period = sample_rate / freq_hz;
        let half = period / 2;
        let quarter = period / 4;

        unsafe {
            for i in 0..samples_to_fill {
                let pos = (i as u32) % period;
                // Triangle wave → pseudo-sine
                let sample: i16 = if pos < quarter {
                    // Rising 0→max
                    (pos as i32 * 24000 / quarter as i32) as i16
                } else if pos < half + quarter {
                    // Falling max→-max
                    ((half as i32 - (pos as i32 - quarter as i32)) * 24000 / quarter as i32) as i16
                } else {
                    // Rising -max→0
                    (((pos as i32) - period as i32) * 24000 / quarter as i32) as i16
                };

                // Write to both channels (stereo interleaved)
                let idx = i * channels as usize;
                *buf.add(idx) = sample;
                *buf.add(idx + 1) = sample;
            }

            // Zero remaining buffer
            let filled_bytes = samples_to_fill * channels as usize * bytes_per_sample as usize;
            if filled_bytes < self.audio_buf_size as usize {
                core::ptr::write_bytes(
                    (self.audio_buf_virt as *mut u8).add(filled_bytes),
                    0,
                    self.audio_buf_size as usize - filled_bytes
                );
            }
        }
    }

    /// Start or stop DMA playback
    pub fn play(&mut self, start: bool) {
        let sd_base = self.osd_base(0);
        unsafe {
            if start {
                // Enable interrupt control
                let intctl = self.read32(reg::INTCTL);
                let stream_bit = 1u32 << (self.num_iss as u32); // First output stream
                self.write32(reg::INTCTL, intctl | (1 << 31) | (1 << 30) | stream_bit);

                // Clear status bits
                self.write8(sd_base + sd::STS, 0x1C);

                // Start stream: RUN + IOCE
                let ctl = self.read8(sd_base + sd::CTL);
                self.write8(sd_base + sd::CTL, ctl | sctl::RUN as u8 | sctl::IOCE as u8);

                self.playing = true;
                crate::serial_println!("[HDA] Playback started");
            } else {
                // Stop stream
                let ctl = self.read8(sd_base + sd::CTL);
                self.write8(sd_base + sd::CTL, ctl & !(sctl::RUN as u8));

                self.playing = false;
                crate::serial_println!("[HDA] Playback stopped");
            }
        }
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Get stream position
    pub fn stream_position(&self) -> u32 {
        let sd_base = self.osd_base(0);
        unsafe { self.read32(sd_base + sd::LPIB) }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // Utility
    // ═════════════════════════════════════════════════════════════════════════

    fn delay_us(us: u64) {
        // Simple busy-wait delay using port 0x80 (POST code, ~1µs per access)
        for _ in 0..us {
            unsafe {
                let mut port: x86_64::instructions::port::Port<u8> = x86_64::instructions::port::Port::new(0x80);
                port.write(0);
            }
        }
    }

    /// Return status info string
    pub fn status_info(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Intel HDA Controller\n"));
        s.push_str(&format!("  Streams: {} out, {} in, {} bidir\n",
            self.num_oss, self.num_iss, self.num_bss));
        s.push_str(&format!("  Codecs: {:?}\n", self.codecs));
        s.push_str(&format!("  Widgets: {}\n", self.widgets.len()));
        s.push_str(&format!("  Output paths: {}\n", self.output_paths.len()));
        for (i, p) in self.output_paths.iter().enumerate() {
            s.push_str(&format!("    [{}] {} -> path {:?}\n", i, p.device_type, p.path));
        }
        s.push_str(&format!("  Playing: {}\n", self.playing));
        if self.playing {
            s.push_str(&format!("  Position: {}\n", self.stream_position()));
        }
        s
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize the HDA driver (called during boot or on-demand)
pub fn init() -> Result<(), &'static str> {
    // Find Intel HDA device (class 0x04, subclass 0x03)
    let devices = crate::pci::find_by_class(crate::pci::class::MULTIMEDIA);
    let hda_dev = devices.iter()
        .find(|d| d.subclass == 0x03) // Audio device (HDA subclass)
        .or_else(|| devices.iter().find(|d| d.subclass == 0x01)) // Also try multimedia audio
        .ok_or("HDA: no Intel HDA device found on PCI bus")?
        .clone();

    let ctrl = HdaController::init(&hda_dev)?;
    *HDA.lock() = Some(ctrl);
    HDA_INITIALIZED.store(true, Ordering::SeqCst);

    Ok(())
}

/// Check if HDA is initialized
pub fn is_initialized() -> bool {
    HDA_INITIALIZED.load(Ordering::SeqCst)
}

/// Play a tone at given frequency for given duration
pub fn play_tone(freq_hz: u32, duration_ms: u32) -> Result<(), &'static str> {
    let mut hda = HDA.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;

    ctrl.fill_tone(freq_hz, duration_ms);
    ctrl.play(true);

    // Busy-wait for the duration
    let sample_rate = 48000u32;
    let total_bytes = (sample_rate * duration_ms / 1000) * 4; // 16-bit stereo = 4 bytes/sample
    let target = total_bytes.min(ctrl.audio_buf_size);

    for _ in 0..(duration_ms * 10) {
        HdaController::delay_us(100);
        let pos = ctrl.stream_position();
        if pos >= target {
            break;
        }
    }

    ctrl.play(false);
    Ok(())
}

/// Stop any playing audio
pub fn stop() -> Result<(), &'static str> {
    let mut hda = HDA.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;
    ctrl.play(false);
    Ok(())
}

/// Get status info
pub fn status() -> String {
    let hda = HDA.lock();
    match hda.as_ref() {
        Some(ctrl) => ctrl.status_info(),
        None => String::from("HDA: not initialized"),
    }
}

/// Write raw audio samples to the DMA buffer and play for a given duration
/// Samples are stereo interleaved i16 (left, right, left, right, ...)
pub fn write_samples_and_play(samples: &[i16], duration_ms: u32) -> Result<(), &'static str> {
    let mut hda = HDA.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;

    // Stop any current playback first
    if ctrl.playing {
        ctrl.play(false);
    }

    let buf = ctrl.audio_buf_virt as *mut i16;
    let buf_capacity = (ctrl.audio_buf_size / 2) as usize; // capacity in i16 samples

    let to_copy = samples.len().min(buf_capacity);

    unsafe {
        // Copy samples to DMA buffer
        core::ptr::copy_nonoverlapping(samples.as_ptr(), buf, to_copy);

        // Zero the rest
        if to_copy < buf_capacity {
            core::ptr::write_bytes(buf.add(to_copy), 0, buf_capacity - to_copy);
        }
    }

    // Start playback
    ctrl.play(true);

    // Wait for the duration
    let total_bytes = (to_copy * 2) as u32; // i16 = 2 bytes
    let target = total_bytes.min(ctrl.audio_buf_size);

    for _ in 0..(duration_ms * 10 + 500) {
        HdaController::delay_us(100);
        let pos = ctrl.stream_position();
        if pos >= target {
            break;
        }
    }

    ctrl.play(false);
    Ok(())
}
