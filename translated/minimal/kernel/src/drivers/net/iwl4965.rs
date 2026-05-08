
















use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

use super::wifi::{Nx, Fg, WifiSecurity, WifiState};
use super::{Dd, NetStats};
use crate::drivers::{Cw, Bb, DriverStatus, DriverCategory};
use crate::pci::L;





const VO_: u16 = 0x8086;


pub const AFI_: &[u16] = &[
    0x4229, 
    0x4230, 
];


const AFK_: &[u16] = &[
    0x4229, 0x4230,         
    0x4232, 0x4235, 0x4236, 
    0x4237, 0x4238, 0x4239, 
    0x008A, 0x008B,         
    0x0082, 0x0083, 0x0084, 
    0x0085, 0x0089,         
    0x0887, 0x0888,         
    0x0890, 0x0891,         
    0x0893, 0x0894,         
    0x088E, 0x088F,         
    0x24F3, 0x24F4,         
    0x2526,                 
    0x2723,                 
    0x2725,                 
    0x7A70,                 
];





const GT_:   u32 = 0x000;
const BSV_: u32 = 0x004;
const BQ_:            u32 = 0x008;
const GU_:       u32 = 0x00C;
const TL_:  u32 = 0x010;
const ARL_:        u32 = 0x018;
const CO_:          u32 = 0x020;
const AL_:       u32 = 0x024;
const ARM_:         u32 = 0x028;
const ACJ_:     u32 = 0x02C;
const BSM_:      u32 = 0x030;
const KM_:      u32 = 0x054;
const DLT_:  u32 = 0x058;
const TO_:  u32 = 0x05C;
const BSY_:      u32 = 0x060;
const BSP_:            u32 = 0x03C;
const KL_:       u32 = 0x048;
const BSQ_:      u32 = 0x050;


const IM_: u32 = 1 << 0;
const ACK_:       u32 = 1 << 2;
const ACL_:  u32 = 1 << 3;
const DLK_:  u32 = 1 << 4;
const ACM_:    u32 = 1 << 0;  
const DLL_:         u32 = 1 << 10;


const BSW_:   u32 = 1 << 0;
const DLS_:    u32 = 1 << 1;
const BSX_:     u32 = 1 << 7;
const ARN_: u32 = 1 << 8;
const TN_:  u32 = 1 << 9;


const BSO_:  u32 = 1 << 0;
const BSN_:         u32 = 1 << 1;
const DLJ_:        u32 = 0x0000FFFC;


const BSR_: u32 = 0x000FFF0;
const DLM_: u32 = 0x0000000;
const DLP_: u32 = 0x0000020;
const DLN_: u32 = 0x0000050;
const DLO_: u32 = 0x0000040;
const DLQ_: u32 = 0x0000070;





const MX_:        u32 = 0x3400;
const AOC_:     u32 = 0x3404;
const AOB_:     u32 = 0x3408;
const AOA_:     u32 = 0x340C;
const ANY_:  u32 = 0x3490;
const ANX_: u32 = 0x3494;
const ANW_:  u32 = 0x3498;
const ANV_: u32 = 0x349C;


const ANZ_:     u32 = 1 << 31;
const BOE_:  u32 = 1 << 30;






const DPQ_:           u32 = 0x1D00;
const DPR_:   u32 = 0x20;

const DPP_:           u32 = 9;

const DPS_: u32 = 0x80000000;


const DPM_:     u32 = 0x1F40;

const DPO_:         u32 = 0x1C44;

const DPN_:   u32 = 0x1BC0;


const HU_:          usize = 256;

const EMO_:          usize = 256;

const BGX_:            usize = 4096;





const CRY_:            u8 = 0x01;
const CRZ_:            u8 = 0x02;
const AIZ_:             u8 = 0x10;
const CSA_:       u8 = 0x11;
const EGC_:        u8 = 0x13;
const BGM_:         u8 = 0x80;
const CSB_:    u8 = 0x84;
const CSC_:               u8 = 0x1C;
const EGB_:          u8 = 0x18;


const CTY_: u8 = 0x83;






#[repr(C, packed)]
struct Alz {
    tu: u32,
    dsb: u32,     
    data_size: u32,     
    init_size: u32,     
    dry: u32, 
    cuf: u32,     
}


struct Aak {
    version: u32,
    
    inst: Vec<u8>,
    
    data: Vec<u8>,
    
    init_inst: Vec<u8>,
    
    init_data: Vec<u8>,
    
    boot: Vec<u8>,
}


static RN_: spin::Mutex<Option<Vec<u8>>> = spin::Mutex::new(None);


pub fn oox(data: &[u8]) {
    crate::serial_println!("[IWL4965] Firmware data available: {} bytes", data.len());
    *RN_.lock() = Some(data.to_vec());
}


pub fn eou() -> bool {
    RN_.lock().is_some()
}






const AZN_: usize = 5;

const AFJ_: usize = 4;

const AZM_: usize = 20;

const LZ_: usize = 256;


const BYB_: u32 = 0x19D0; 


const AUC_: u32 = 0x1F48;





const BYJ_: u32  = 0x1BC0;  
const BYI_: u32  = 0x1BC4;  
const BYK_: u32   = 0x1BC8;  


const BYC_: u32   = 0x1F80;  


const BYE_: u32  = 0x8000_0000;
const BYF_: u32  = 0x0100_0000;
const BYG_: u32    = 0x0000_0000;
const BYH_: u32  = 0x0000_8000;
const BYD_: u32        = 20;  






const CCF_: u32 = 0x40C;
const CCH_: u32 = 0x410;
const CCI_:  u32 = 0x418;
const CCG_:  u32 = 0x41C;


const OI_: u32 = 0x444;
const VA_: u32 = 0x448;
const OJ_:  u32 = 0x44C;
const VB_:  u32 = 0x450;


const CCJ_: u32 = 0x460;





const BMT_:   u32 = 0x3000;
const RX_:     u32 = 0x3004;
const BMU_:    u32 = 0x3008;
const BMV_:    u32 = 0x300C;
const MO_: u32 = 0x3010;

const AAH_: u32 = 0x0000_0200;
const AAG_: u32 = 0x0000_0800;
const AMM_: u32 = 0x0000_0002;


const TM_: u32 = 0x100;
const ARK_: u32 = 0x2000_0000;


const NN_: u32 = 0x0040_0000;


const ABD_: u32 = 0x3800;


const EHQ_: u32 = 0x2E00;
const CUA_: u32 = 0x2D00;


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Tg {
    
    lo: u32,
    
    hi_n_len: u16,
}

impl Tg {
    fn set(&mut self, addr: u64, len: u16) {
        self.lo = addr as u32;
        self.hi_n_len = ((addr >> 32) as u16 & 0xF) | ((len & 0x0FFF) << 4);
    }
}



#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IwlTfd {
    __reserved: [u8; 12],
    tbs: [Tg; AZM_],
    __pad: u32,
}

impl IwlTfd {
    const fn zeroed() -> Self {
        Self {
            __reserved: [0; 12],
            tbs: [Tg { lo: 0, hi_n_len: 0 }; AZM_],
            __pad: 0,
        }
    }
    
    fn qpv(&self) -> usize {
        
        let ptr = self.__reserved.as_ptr() as *const u32;
        let val = unsafe { core::ptr::read_unaligned(ptr) };
        (val & 0x1F) as usize
    }
    
    fn set_num_tbs(&mut self, count: usize) {
        let ptr = self.__reserved.as_mut_ptr() as *mut u32;
        unsafe { core::ptr::write_unaligned(ptr, (count & 0x1F) as u32) };
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Aly {
    cmd: u8,
    flags: u8,
    idx: u8,     
    qid: u8,     
}



const BBN_: usize = 512;


struct TxQueue {
    
    tfds: Vec<IwlTfd>,
    
    cmd_buffers: Vec<Vec<u8>>,
    
    write_ptr: usize,
    
    read_ptr: usize,
}

impl TxQueue {
    fn new() -> Self {
        let mut tfds = Vec::with_capacity(LZ_);
        tfds.resize(LZ_, IwlTfd::zeroed());
        
        let mut cmd_buffers = Vec::with_capacity(LZ_);
        for _ in 0..LZ_ {
            
            cmd_buffers.push(vec![0u8; 4 + BBN_]);
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
        crate::memory::lc(virt)
            .unwrap_or(virt.wrapping_sub(crate::memory::hhdm_offset()))
    }
}


struct RxQueue {
    
    bd: Vec<u32>,  
    
    buffers: Vec<Vec<u8>>,
    
    
    rb_stts: Vec<u8>,
    
    write_ptr: usize,
    
    read_ptr: usize,
}

impl RxQueue {
    fn new() -> Self {
        let mut bd = Vec::with_capacity(HU_);
        let mut buffers = Vec::with_capacity(HU_);
        
        for _ in 0..HU_ {
            let buf = vec![0u8; BGX_];
            let virt = buf.as_ptr() as u64;
            let phys = crate::memory::lc(virt)
                .unwrap_or(virt.wrapping_sub(crate::memory::hhdm_offset()));
            
            bd.push((phys >> 8) as u32);
            buffers.push(buf);
        }

        
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
        crate::memory::lc(virt)
            .unwrap_or(virt.wrapping_sub(crate::memory::hhdm_offset()))
    }
}





const ADK_: u16 = 0x0015;
const BWB_:     u16 = 0x0045;
const DOP_: u16 = 0x0062; 
const DOQ_: u16 = 0x0080; 





const DUM_: u16 = 0x0000;
const DUN_: u16 = 0x0080;
const DUO_: u16 = 0x0050;


const DFI_: u8 = 0;
const EOD_: u8 = 3;
const DFH_: u8 = 48;        
const DFJ_: u8 = 221;    





const CIZ_: usize = 32;
const CTZ_: u64 = 500; 

pub struct Iwl4965 {
    
    pci_bus: u8,
    pci_device: u8,
    pci_function: u8,
    device_id: u16,

    
    mmio_base: usize,
    mmio_size: usize,

    
    status: DriverStatus,
    wifi_state: WifiState,
    hw_rev: u32,
    mac_addr: [u8; 6],

    
    firmware_loaded: bool,
    fw_alive: bool,

    
    tx_queues: Vec<TxQueue>,
    rx_queue: Option<RxQueue>,
    
    cmd_seq: u16,

    
    scan_results: Vec<Fg>,
    scan_start_tick: u64,
    scanning: bool,

    
    connected_ssid: Option<String>,
    connected_bssid: [u8; 6],
    current_channel: u8,
    signal_dbm: i8,

    
    rx_pending: Vec<Vec<u8>>,

    
    stats: NetStats,

    
    initialized: bool,
}

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

    

    #[inline]
    fn read_reg(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 { return 0; }
        unsafe {
            let ptr = (self.mmio_base + offset as usize) as *const u32;
            read_volatile(ptr)
        }
    }

    #[inline]
    fn write_reg(&self, offset: u32, value: u32) {
        if self.mmio_base == 0 { return; }
        unsafe {
            let ptr = (self.mmio_base + offset as usize) as *mut u32;
            write_volatile(ptr, value);
        }
    }

    
    
    
    fn grab_nic_access(&self) -> bool {
        
        self.write_reg(AL_,
            self.read_reg(AL_) | ACL_);
        
        for _ in 0..5000u32 {
            if self.read_reg(AL_) & ACM_ != 0 {
                return true;
            }
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        false
    }

    
    fn release_nic_access(&self) {
        self.write_reg(AL_,
            self.read_reg(AL_) & !ACL_);
    }

    fn write_prph(&self, addr: u32, val: u32) {
        if !self.grab_nic_access() {
            crate::serial_println!("[IWL4965] write_prph FAILED: no NIC access for {:#X}", addr);
            return;
        }
        self.write_reg(OI_, (addr & 0x000F_FFFF) | (3 << 24));
        self.write_reg(OJ_, val);
        self.release_nic_access();
    }

    
    fn read_prph(&self, addr: u32) -> u32 {
        if !self.grab_nic_access() {
            crate::serial_println!("[IWL4965] read_prph FAILED: no NIC access for {:#X}", addr);
            return 0xFFFFFFFF;
        }
        self.write_reg(VA_, (addr & 0x000F_FFFF) | (3 << 24));
        let val = self.read_reg(VB_);
        self.release_nic_access();
        val
    }

    
    fn rdm(&self, addr: u32, val: u32) {
        if !self.grab_nic_access() {
            crate::serial_println!("[IWL4965] write_targ_mem FAILED: no NIC access for {:#X}", addr);
            return;
        }
        self.write_reg(CCH_, addr);
        self.write_reg(CCI_, val);
        self.release_nic_access();
    }

    

    
    fn map_bar0(&mut self, go: &L) -> Result<(), &'static str> {
        let bar0 = go.bar[0];
        if bar0 == 0 {
            return Err("BAR0 is zero");
        }

        
        let mtc = (bar0 & 1) == 0;
        if !mtc {
            return Err("BAR0 is I/O, need memory");
        }

        
        let arf = (bar0 >> 1) & 0x3 == 2;
        let phys_addr = if arf {
            let bqi = go.bar[1] as u64;
            (bqi << 32) | (bar0 & 0xFFFFFFF0) as u64
        } else {
            (bar0 & 0xFFFFFFF0) as u64
        };

        if phys_addr == 0 {
            return Err("BAR0 base address is zero");
        }

        
        self.mmio_size = 0x2000; 

        
        let virt_addr = crate::memory::yv(phys_addr, self.mmio_size)
            .map_err(|_| "Failed to map BAR0 MMIO region")?;

        self.mmio_base = virt_addr as usize;

        crate::serial_println!("[IWL4965] MMIO phys: {:#X} -> virt: {:#X}, size: {:#X}", phys_addr, virt_addr, self.mmio_size);

        Ok(())
    }

    
    fn apm_stop(&self) {
        
        self.write_reg(CO_, BSX_);
        
        for _ in 0..10_000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }

        
        let tj = self.read_reg(AL_);
        self.write_reg(AL_, tj & !ACK_);
        
        for _ in 0..10_000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
    }

    
    fn apm_init(&self) -> Result<(), &'static str> {
        
        self.write_reg(GT_,
            self.read_reg(GT_) | NN_);
        for _ in 0..5000u32 {
            if self.read_reg(GT_) & NN_ != 0 {
                break;
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        
        self.write_reg(TM_,
            self.read_reg(TM_) | ARK_);

        
        self.write_reg(AL_,
            self.read_reg(AL_) | ACK_);

        
        
        for i in 0..25000u32 {
            let val = self.read_reg(AL_);
            if val & IM_ != 0 {
                crate::serial_println!("[IWL4965] MAC clock ready after {} iterations, GP_CNTRL={:#010X}", i, val);

                
                self.write_reg(AL_,
                    self.read_reg(AL_) | ACL_);
                let mut ill = false;
                for ay in 0..10_000u32 {
                    if self.read_reg(AL_) & ACM_ != 0 {
                        crate::serial_println!("[IWL4965] MAC access granted after {} iters", ay);
                        ill = true;
                        break;
                    }
                    
                    unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
                }
                if !ill {
                    crate::serial_println!("[IWL4965] WARNING: MAC access not granted, GP={:#010X}",
                        self.read_reg(AL_));
                }

                
                self.write_prph(RX_,
                    AAH_ | AAG_);
                
                for _ in 0..20_000 { core::hint::spin_loop(); }

                
                let gwq = self.read_prph(MO_);
                self.write_prph(MO_, gwq | AMM_);

                return Ok(());
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        let tj = self.read_reg(AL_);
        crate::serial_println!("[IWL4965] APM init FAILED: MAC clock not ready, GP_CNTRL={:#010X}", tj);
        Err("MAC clock not ready")
    }

    
    fn hw_init(&mut self) -> Result<(), &'static str> {
        
        self.write_reg(GU_, 0);
        self.write_reg(BQ_, 0xFFFFFFFF);
        self.write_reg(TL_, 0xFFFFFFFF);

        
        self.hw_rev = self.read_reg(ARM_);
        let drq = (self.hw_rev & BSR_) >> 4;
        let mms = match drq {
            0x00 => "4965",
            0x02 => "5300",
            0x04 => "5150",
            0x05 => "5100",
            0x07 => "6000",
            _ => "unknown",
        };
        crate::serial_println!("[IWL4965] HW rev: {:#010X} (type: {} = {})", self.hw_rev, drq, mms);

        
        self.apm_init()?;

        crate::serial_println!("[IWL4965] APM init complete");

        
        self.read_eeprom_mac()?;

        crate::serial_println!("[IWL4965] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac_addr[0], self.mac_addr[1], self.mac_addr[2],
            self.mac_addr[3], self.mac_addr[4], self.mac_addr[5]);

        self.initialized = true;
        Ok(())
    }

    
    fn eeprom_read(&self, addr: u16) -> u16 {
        
        let oeh = ((addr as u32) << 2) | BSN_;
        self.write_reg(ACJ_, oeh);

        
        for _ in 0..5000 {
            let val = self.read_reg(ACJ_);
            if val & BSO_ != 0 {
                return (val >> 16) as u16;
            }
            for _ in 0..50 { core::hint::spin_loop(); }
        }

        crate::serial_println!("[IWL4965] EEPROM read timeout at addr {:#06X}", addr);
        0
    }

    
    fn read_eeprom_mac(&mut self) -> Result<(), &'static str> {
        let avr = self.eeprom_read(ADK_);
        let ahg = self.eeprom_read(ADK_ + 1);
        let aeo = self.eeprom_read(ADK_ + 2);

        self.mac_addr[0] = (avr & 0xFF) as u8;
        self.mac_addr[1] = (avr >> 8) as u8;
        self.mac_addr[2] = (ahg & 0xFF) as u8;
        self.mac_addr[3] = (ahg >> 8) as u8;
        self.mac_addr[4] = (aeo & 0xFF) as u8;
        self.mac_addr[5] = (aeo >> 8) as u8;

        
        if self.mac_addr == [0; 6] || self.mac_addr == [0xFF; 6] {
            
            
            crate::serial_println!("[IWL4965] EEPROM MAC invalid, generating from PCI");
            
            self.mac_addr = [
                0x00, 0x13, 0xE8, 
                self.pci_bus,
                self.pci_device,
                self.pci_function | 0x40,
            ];
        }

        Ok(())
    }

    

    
    fn nqi(data: &[u8]) -> Result<Aak, &'static str> {
        let gag = core::mem::size_of::<Alz>();
        if data.len() < gag {
            return Err("Firmware too small for header");
        }

        
        let tu = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let dsb = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        let data_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let init_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]) as usize;
        let dry = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize;
        let cuf = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;

        let av = gag + dsb + data_size + init_size + dry + cuf;
        if data.len() < av {
            crate::serial_println!("[IWL4965] FW: need {} bytes, have {}", av, data.len());
            return Err("Firmware file truncated");
        }

        let axz = (tu >> 24) & 0xFF;
        let ayh = (tu >> 16) & 0xFF;
        let api = (tu >> 8) & 0xFF;
        crate::serial_println!("[IWL4965] FW version: {}.{}.{} (raw {:#010X})", axz, ayh, api, tu);
        crate::serial_println!("[IWL4965] FW sections: inst={} data={} init={} init_data={} boot={}",
            dsb, data_size, init_size, dry, cuf);

        let mut off = gag;
        let inst = data[off..off + dsb].to_vec(); off += dsb;
        let lbr = data[off..off + data_size].to_vec(); off += data_size;
        let init_inst = data[off..off + init_size].to_vec(); off += init_size;
        let init_data = data[off..off + dry].to_vec(); off += dry;
        let boot = data[off..off + cuf].to_vec();

        Ok(Aak {
            version: tu,
            inst,
            data: lbr,
            init_inst,
            init_data,
            boot,
        })
    }

    
    fn load_firmware(&mut self) -> Result<(), &'static str> {
        
        let mas = {
            let jg = RN_.lock();
            match jg.as_ref() {
                Some(d) => d.clone(),
                None => {
                    
                    match crate::ramfs::bh(|fs| fs.read_file("/firmware/iwlwifi-4965-2.ucode").map(|d| d.to_vec())) {
                        Ok(d) => d,
                        Err(_) => return Err("No firmware available (need iwlwifi-4965-2.ucode)"),
                    }
                }
            }
        };

        let fo = Self::nqi(&mas)?;
        crate::println!("    FW parsed: boot={} inst={} data={} init_inst={} init_data={}",
            fo.boot.len(), fo.inst.len(), fo.data.len(), fo.init_inst.len(), fo.init_data.len());

        
        
        
        let (init_inst_dma, ii_off) = Self::efj(&fo.init_inst);
        let (init_data_dma, id_off) = Self::efj(&fo.init_data);
        let (inst_dma, i_off) = Self::efj(&fo.inst);
        let (data_dma, d_off) = Self::efj(&fo.data);
        crate::println!("    DMA buffers allocated (page-aligned)");

        
        if !fo.init_inst.is_empty() {
            crate::println!("    === Phase 1: Init firmware (calibration) ===");
            crate::serial_println!("[IWL4965] === INIT firmware phase ===");

            crate::println!("    stop_device...");
            self.stop_device()?;
            let tj = self.read_reg(AL_);
            let nbr = if tj & ACM_ != 0 { "granted" } else { "DENIED" };
            crate::println!("    stop_device OK, GP={:#010X} MAC_ACCESS={}", tj, nbr);

            crate::println!("    bsm_load_bootstrap ({} bytes)...", fo.boot.len());
            self.bsm_load_bootstrap(&fo.boot)?;
            crate::println!("    bsm_load_bootstrap OK");

            let mob = &init_inst_dma[ii_off..ii_off + fo.init_inst.len()];
            let mnk = &init_data_dma[id_off..id_off + fo.init_data.len()];
            crate::println!("    bsm_set_dram_addrs (init_inst={}, init_data={})...", fo.init_inst.len(), fo.init_data.len());
            self.bsm_set_dram_addrs(mob, mnk)?;
            crate::println!("    bsm_set_dram_addrs OK");

            crate::println!("    bsm_start (boot={})...", fo.boot.len());
            self.bsm_start(fo.boot.len())?;
            crate::println!("    bsm_start OK");
            self.verify_inst_sram(&fo.boot);

            
            self.write_reg(TO_, 0x2 | 0x4);
            self.write_reg(TO_, 0x2); 
            self.write_reg(BQ_, 0xFFFFFFFF);
            
            self.write_reg(GU_, 0xAB00_000B);
            crate::println!("    nic_start: CSR_RESET=0 (release CPU)...");
            self.write_reg(CO_, 0);
            
            for _ in 0..25_000 {
                unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
            let gsa = self.read_reg(CO_);
            let fzh = self.read_reg(AL_);
            crate::println!("    nic_start done: RST={:#X} GP={:#010X}", gsa, fzh);

            crate::println!("    wait_alive (init)...");
            match self.wait_alive() {
                Ok(()) => {
                    crate::println!("    Init firmware ALIVE!");
                    
                    
                    for i in 0..50u32 {
                        for _ in 0..10_000u32 {
                            for _ in 0..1000 { core::hint::spin_loop(); }
                        }
                        self.poll_rx(); 
                        if i == 25 {
                            let eqw = self.read_reg(BQ_);
                            crate::serial_println!("[IWL4965] Init cal progress: INT={:#X} rxpkts so far", eqw);
                        }
                    }
                    crate::println!("    Init calibration done");
                }
                Err(e) => {
                    crate::println!("    Init ALIVE FAILED: {} — trying runtime directly", e);
                }
            }
        }

        
        
        
        
        crate::println!("    === Phase 2: Runtime firmware ===");
        crate::serial_println!("[IWL4965] === RUNTIME firmware phase (soft reset) ===");

        
        self.write_reg(CO_, TN_);
        for _ in 0..1000u32 {
            if self.read_reg(CO_) & ARN_ != 0 {
                break;
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }
        
        self.write_reg(CO_, BSW_ | TN_);
        
        for _ in 0..5_000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }

        
        self.write_prph(RX_,
            AAH_ | AAG_);
        for _ in 0..20_000 { core::hint::spin_loop(); }

        let tj = self.read_reg(AL_);
        crate::println!("    soft reset OK (NEVO held), GP={:#010X}", tj);

        crate::println!("    bsm_load_bootstrap ({} bytes)...", fo.boot.len());
        self.bsm_load_bootstrap(&fo.boot)?;
        crate::println!("    bsm_load_bootstrap OK");

        let mna = &inst_dma[i_off..i_off + fo.inst.len()];
        let lbe = &data_dma[d_off..d_off + fo.data.len()];
        crate::println!("    bsm_set_dram_addrs (inst={}, data={})...", fo.inst.len(), fo.data.len());
        self.bsm_set_dram_addrs(mna, lbe)?;
        crate::println!("    bsm_set_dram_addrs OK");

        crate::println!("    bsm_start (boot={})...", fo.boot.len());
        self.bsm_start(fo.boot.len())?;
        crate::println!("    bsm_start OK");
        self.verify_inst_sram(&fo.boot);

        
        self.write_reg(TO_, 0x2 | 0x4);
        self.write_reg(TO_, 0x2); 
        self.write_reg(BQ_, 0xFFFFFFFF);
        
        self.write_reg(GU_, 0xAB00_000B);
        crate::println!("    nic_start: CSR_RESET=0 (release CPU)...");
        self.write_reg(CO_, 0);
        
        for _ in 0..25_000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        let gsa = self.read_reg(CO_);
        let fzh = self.read_reg(AL_);
        crate::println!("    nic_start done: RST={:#X} GP={:#010X}", gsa, fzh);

        crate::println!("    wait_alive (runtime)...");
        self.wait_alive()?;

        self.firmware_loaded = true;
        self.fw_alive = true;
        crate::println!("    Runtime firmware ALIVE!");
        crate::serial_println!("[IWL4965] Runtime firmware loaded and alive!");

        Ok(())
    }

    
    
    
    fn efj(data: &[u8]) -> (Vec<u8>, usize) {
        let mut buf = vec![0u8; data.len() + 4095];
        let ptr = buf.as_ptr() as usize;
        let offset = (4096 - (ptr & 0xFFF)) & 0xFFF;
        buf[offset..offset + data.len()].copy_from_slice(data);
        (buf, offset)
    }

    
    fn verify_inst_sram(&self, boot: &[u8]) {
        if !self.grab_nic_access() {
            crate::println!("      INST SRAM verify: cannot grab NIC access");
            return;
        }
        let dko = core::cmp::min(boot.len() / 4, 4);
        let mut inx = 0;
        for i in 0..dko {
            self.write_reg(CCF_, (i * 4) as u32);
            let val = self.read_reg(CCG_);
            let off = i * 4;
            let expect = u32::from_le_bytes([boot[off], boot[off+1], boot[off+2], boot[off+3]]);
            if val != expect {
                crate::println!("      INST SRAM [{}]: {:#010X} expect {:#010X} MISMATCH", i, val, expect);
                inx += 1;
            }
        }
        self.release_nic_access();
        if inx == 0 {
            crate::println!("      INST SRAM verify: first {} dwords OK", dko);
        }
    }

    
    fn stop_device(&mut self) -> Result<(), &'static str> {
        
        self.write_reg(GU_, 0);
        self.write_reg(BQ_, 0xFFFFFFFF);
        self.write_reg(TL_, 0xFFFFFFFF);

        
        self.write_reg(CO_, TN_);
        for _ in 0..1000u32 {
            if self.read_reg(CO_) & ARN_ != 0 {
                break;
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }

        
        self.apm_stop();

        
        self.apm_init()?;

        
        let cmd = crate::pci::ms(self.pci_bus, self.pci_device, self.pci_function, 0x04);
        if cmd & 0x04 == 0 {
            crate::serial_println!("[IWL4965] Re-enabling PCI bus master after reset");
            crate::pci::qj(self.pci_bus, self.pci_device, self.pci_function, 0x04, cmd | 0x06);
        }

        Ok(())
    }

    
    
    fn bsm_load_bootstrap(&self, boot: &[u8]) -> Result<(), &'static str> {
        let dnz = (boot.len() + 3) / 4;

        
        if !self.grab_nic_access() {
            return Err("bsm_load_bootstrap: cannot grab NIC access");
        }

        
        for i in 0..dnz {
            let offset = i * 4;
            let fx = if offset + 4 <= boot.len() {
                u32::from_le_bytes([boot[offset], boot[offset+1], boot[offset+2], boot[offset+3]])
            } else {
                let mut bytes = [0u8; 4];
                for ay in 0..(boot.len() - offset) {
                    bytes[ay] = boot[offset + ay];
                }
                u32::from_le_bytes(bytes)
            };

            
            let addr = ABD_ + (i * 4) as u32;
            self.write_reg(OI_, (addr & 0x000F_FFFF) | (3 << 24));
            self.write_reg(OJ_, fx);
        }

        
        let jtz = ABD_;
        self.write_reg(VA_, (jtz & 0x000F_FFFF) | (3 << 24));
        let hbk = self.read_reg(VB_);

        self.release_nic_access();

        let fvs = u32::from_le_bytes([boot[0], boot[1], boot[2], boot[3]]);
        crate::println!("      SRAM verify: [0]={:#010X} expect={:#010X} {}",
            hbk, fvs, if hbk == fvs { "OK" } else { "MISMATCH!" });
        crate::serial_println!("[IWL4965] Bootstrap: {} dwords written to SRAM @ {:#X}, verify={}",
            dnz, ABD_, if hbk == fvs { "OK" } else { "FAIL" });
        Ok(())
    }

    
    
    fn bsm_set_dram_addrs(&self, inst: &[u8], data: &[u8]) -> Result<(), &'static str> {
        let bz = crate::memory::hhdm_offset();

        
        
        
        

        
        if !inst.is_empty() {
            let virt = inst.as_ptr() as u64;
            let phys = crate::memory::lc(virt).unwrap_or(virt.wrapping_sub(bz));
            self.write_prph(ANY_, (phys >> 4) as u32);
            self.write_prph(ANX_, inst.len() as u32);
            crate::serial_println!("[IWL4965] FW inst @ phys {:#010X} >> 4 = {:#010X} ({} bytes)", phys, phys >> 4, inst.len());
            crate::println!("      inst phys={:#010X} >>4={:#010X} len={}", phys, phys >> 4, inst.len());
        }

        
        if !data.is_empty() {
            let virt = data.as_ptr() as u64;
            let phys = crate::memory::lc(virt).unwrap_or(virt.wrapping_sub(bz));
            self.write_prph(ANW_, (phys >> 4) as u32);
            self.write_prph(ANV_, data.len() as u32);
            crate::serial_println!("[IWL4965] FW data @ phys {:#010X} >> 4 = {:#010X} ({} bytes)", phys, phys >> 4, data.len());
            crate::println!("      data phys={:#010X} >>4={:#010X} len={}", phys, phys >> 4, data.len());
        }

        Ok(())
    }

    
    fn bsm_start(&self, cuf: usize) -> Result<(), &'static str> {
        
        
        self.write_prph(MX_, 0);
        for _ in 0..1000 { core::hint::spin_loop(); }

        
        self.write_prph(AOC_, 0);
        self.write_prph(AOB_, 0);

        
        let dnz = (cuf + 3) / 4;
        self.write_prph(AOA_, dnz as u32);
        crate::serial_println!("[IWL4965] BSM: src=0 dst=0 dwcount={}", dnz);

        
        self.write_prph(MX_, ANZ_);

        
        for _ in 0..10000u32 {
            let ctrl = self.read_prph(MX_);
            if ctrl & ANZ_ == 0 {
                crate::serial_println!("[IWL4965] BSM load complete");
                
                
                self.write_prph(MX_, BOE_);
                return Ok(());
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        
        let tj = self.read_reg(AL_);
        if tj & IM_ != 0 {
            crate::serial_println!("[IWL4965] BSM: device appears running (GP_CNTRL={:#X})", tj);
            return Ok(());
        }

        Err("BSM start timeout")
    }

    
    fn wait_alive(&mut self) -> Result<(), &'static str> {
        const BSS_: u32 = 1 << 0;
        const DLR_: u32 = 1 << 7;
        const BST_: u32 = 1 << 26;
        const BSU_: u32 = 1 << 29;

        
        let dds = self.read_reg(CO_);
        crate::println!("      CSR_RESET={:#X} (expect 0 for CPU running)", dds);

        
        for attempt in 0..300u32 {
            let aoj = self.read_reg(BQ_);

            
            if aoj & BSS_ != 0 {
                self.write_reg(BQ_, aoj); 
                let bta = self.read_reg(KM_);
                let bmn = self.read_reg(KL_);
                crate::println!("      ALIVE! INT={:#X} GP1={:#X} UCODE={:#X} (attempt {})",
                    aoj, bta, bmn, attempt);
                crate::serial_println!("[IWL4965] FW alive: INT={:#X} GP1={:#X} UCODE={:#X}",
                    aoj, bta, bmn);
                return Ok(());
            }

            
            if aoj & BSU_ != 0 {
                crate::println!("      HW ERROR! INT={:#X} (attempt {})", aoj, attempt);
                self.write_reg(BQ_, aoj);
                return Err("Hardware error during firmware load");
            }

            
            if aoj != 0 && aoj != 0xFFFFFFFF {
                
                if aoj & BST_ != 0 {
                    self.write_reg(BQ_, aoj);
                    crate::println!("      ALIVE via FH_RX! INT={:#X} (attempt {})", aoj, attempt);
                    return Ok(());
                }
            }

            
            let bmn = self.read_reg(KL_);
            if bmn != 0 && bmn != 0xFFFFFFFF {
                crate::println!("      ALIVE via GP_UCODE={:#X} (attempt {})", bmn, attempt);
                crate::serial_println!("[IWL4965] FW alive via GP_UCODE: {:#X}", bmn);
                return Ok(());
            }

            if self.fw_alive {
                crate::println!("      ALIVE via RX (attempt {})", attempt);
                return Ok(());
            }

            
            if attempt % 50 == 49 {
                let tj = self.read_reg(AL_);
                crate::print!("      [{}] RST={:#X} GP={:#010X} INT={:#X} GP1={:#X} UCODE={:#X}\n",
                    attempt + 1,
                    self.read_reg(CO_),
                    tj,
                    self.read_reg(BQ_),
                    self.read_reg(KM_),
                    self.read_reg(KL_));
            }

            
            for _ in 0..10_000 {
                unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
        }

        
        let tj = self.read_reg(AL_);
        let bta = self.read_reg(KM_);
        let bmn = self.read_reg(KL_);
        let eqw = self.read_reg(BQ_);
        let dds = self.read_reg(CO_);
        crate::println!("      TIMEOUT: RST={:#X} GP={:#010X} GP1={:#X} UCODE={:#X} INT={:#X}",
            dds, tj, bta, bmn, eqw);
        crate::serial_println!("[IWL4965] FW ALIVE TIMEOUT: RST={:#X} GP={:#X} GP1={:#X} UCODE={:#X} INT={:#X}",
            dds, tj, bta, bmn, eqw);

        Err("Firmware ALIVE timeout")
    }

    

    
    fn init_queues(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[IWL4965] Initializing DMA queues...");

        
        self.tx_queues.clear();
        for q in 0..AZN_ {
            let avn = TxQueue::new();
            let phys = avn.tfd_phys_base();
            
            
            self.write_reg(BYB_ + (q as u32) * 4, phys as u32);
            crate::serial_println!("[IWL4965]   TXQ{}: TFD ring @ phys {:#X}", q, phys);
            
            self.tx_queues.push(avn);
        }

        
        let bvp = RxQueue::new();
        let iyf = bvp.bd_phys_base();

        
        self.write_reg(AUC_, 0);
        self.write_reg(BYC_, 0);

        
        self.write_reg(BYI_, (iyf >> 8) as u32);

        let jjh = bvp.rb_stts.as_ptr() as u64;
        let jjg = crate::memory::lc(jjh)
            .unwrap_or(jjh.wrapping_sub(crate::memory::hhdm_offset()));
        self.write_reg(BYK_, (jjg >> 4) as u32);

        
        
        let jbz = BYE_
            | BYF_
            | BYH_
            | BYG_
            | (8u32 << BYD_);  
        self.write_reg(AUC_, jbz);

        
        self.write_reg(BYJ_, (HU_ as u32) & !0x7);

        crate::serial_println!("[IWL4965]   RXQ: CONFIG={:#010X} BASE={:#X} STTS={:#X} WPTR={}",
            jbz, (iyf >> 8) as u32, (jjg >> 4) as u32, (HU_ as u32) & !0x7);

        self.rx_queue = Some(bvp);

        
        self.write_prph(CUA_, (1 << AZN_) - 1);

        crate::serial_println!("[IWL4965] DMA queues initialized");
        Ok(())
    }

    

    
    fn send_hcmd(&mut self, bfj: u8, data: &[u8]) -> Result<(), &'static str> {
        if !self.fw_alive {
            return Err("Firmware not alive");
        }
        if data.len() > BBN_ {
            return Err("HCMD payload too large");
        }

        let avn = &mut self.tx_queues[AFJ_];
        let idx = avn.write_ptr;

        
        let buf = &mut avn.cmd_buffers[idx];
        let kp = Aly {
            cmd: bfj,
            flags: 0,
            idx: idx as u8,
            qid: AFJ_ as u8,
        };
        
        buf[0] = kp.cmd;
        buf[1] = kp.flags;
        buf[2] = kp.idx;
        buf[3] = kp.qid;
        
        let payload_len = data.len();
        buf[4..4 + payload_len].copy_from_slice(data);
        let total_len = 4 + payload_len;

        
        let kt = buf.as_ptr() as u64;
        let hg = crate::memory::lc(kt)
            .unwrap_or(kt.wrapping_sub(crate::memory::hhdm_offset()));

        
        let tfd = &mut avn.tfds[idx];
        *tfd = IwlTfd::zeroed();
        tfd.tbs[0].set(hg, total_len as u16);
        tfd.set_num_tbs(1);

        
        avn.write_ptr = (idx + 1) % LZ_;

        
        let hcq = (avn.write_ptr as u32) | ((AFJ_ as u32) << 8);
        self.write_reg(CCJ_, hcq);

        self.cmd_seq = self.cmd_seq.wrapping_add(1);

        crate::serial_println!("[IWL4965] HCMD sent: cmd={:#04X} len={} idx={}", bfj, total_len, idx);
        Ok(())
    }

    
    fn poll_rx(&mut self) {
        if self.rx_queue.is_none() {
            return;
        }

        
        let dlt = self.read_reg(BQ_);
        if dlt != 0 && dlt != 0xFFFFFFFF {
            
            self.write_reg(BQ_, dlt);
            
            self.write_reg(GU_, 0xAB00_000B);
            if dlt & 0x80000000 != 0 {
                crate::serial_println!("[IWL4965] poll_rx: INT={:#010X} (FH_RX fired)", dlt);
            }
        }

        
        let gbj = {
            let bvp = match self.rx_queue.as_ref() {
                Some(q) => q,
                None => { crate::serial_println!("[IWL4965] poll_rx: no RX queue"); return; }
            };
            let fbv = &bvp.rb_stts;
            
            let dm = u32::from_le_bytes([fbv[0], fbv[1], fbv[2], fbv[3]]);
            (dm & 0xFFF) as usize % HU_
        };
        let read = match self.rx_queue.as_ref() {
            Some(q) => q.read_ptr,
            None => return,
        };
        
        if gbj == read {
            return;
        }

        crate::serial_println!("[IWL4965] poll_rx: hw_write={} read={} — new packets!", gbj, read);

        
        let mut packets: Vec<(u8, Vec<u8>)> = Vec::new();
        let mut idx = read;
        let mut count = 0;
        
        {
            let bvp = match self.rx_queue.as_ref() {
                Some(q) => q,
                None => return,
            };
            while idx != gbj && count < HU_ {
                let buf = &bvp.buffers[idx];
                if buf.len() >= 8 {
                    let aup = u16::from_le_bytes([buf[0], buf[1]]) as usize;
                    let bfj = buf[4];
                    if aup > 0 && aup <= BGX_ - 4 {
                        let end = (aup + 4).min(buf.len());
                        packets.push((bfj, buf[..end].to_vec()));
                    }
                }
                idx = (idx + 1) % HU_;
                count += 1;
            }
        }

        
        if let Some(bvp) = self.rx_queue.as_mut() {
            bvp.read_ptr = idx;
        }
        
        
        for (bfj, data) in packets {
            self.process_rx_packet(bfj, &data);
        }
    }

    
    fn process_rx_packet(&mut self, bfj: u8, data: &[u8]) {
        match bfj {
            CRY_ => {
                crate::serial_println!("[IWL4965] RX: ALIVE notification");
                self.fw_alive = true;
            }
            CRZ_ => {
                crate::serial_println!("[IWL4965] RX: ERROR from firmware");
            }
            CTY_ => {
                crate::serial_println!("[IWL4965] RX: Scan results notification ({} bytes)", data.len());
                self.parse_scan_notification(data);
            }
            CSB_ | BGM_ => {
                crate::serial_println!("[IWL4965] RX: Scan complete/response");
                self.scanning = false;
                self.wifi_state = if self.connected_ssid.is_some() {
                    WifiState::Connected
                } else {
                    WifiState::Disconnected
                };
            }
            AIZ_ | CSA_ => {
                crate::serial_println!("[IWL4965] RX: RXON response");
            }
            CSC_ => {
                
                self.stats.tx_packets += 1;
            }
            
            0xC1 | 0xC3 => {
                
                if data.len() > 32 {
                    
                    let frame = data[16..].to_vec(); 
                    self.stats.rx_packets += 1;
                    self.stats.rx_bytes += frame.len() as u64;
                    if self.rx_pending.len() < 64 {
                        self.rx_pending.push(frame);
                    }
                }
            }
            _ => {
                crate::serial_println!("[IWL4965] RX: Unknown cmd {:#04X} ({} bytes)", bfj, data.len());
            }
        }
    }

    
    fn parse_scan_notification(&mut self, data: &[u8]) {
        
        
        
        
        
        
        if data.len() < 12 {
            return;
        }

        let count = data[8] as usize;
        crate::serial_println!("[IWL4965] Scan: {} network(s) reported", count);

        let mut offset = 12; 
        for _ in 0..count {
            if offset + 20 > data.len() { break; }

            
            let mut bssid = [0u8; 6];
            bssid.copy_from_slice(&data[offset..offset + 6]);
            let channel = data[offset + 6];
            let ash = data[offset + 7] as i8;
            let mny = u16::from_le_bytes([data[offset + 8], data[offset + 9]]) as usize;

            
            let ifs = offset + 10;
            let ifr = (ifs + mny).min(data.len());
            let (ssid, security) = self.parse_ies(&data[ifs..ifr]);

            if !ssid.is_empty() && self.scan_results.len() < CIZ_ {
                
                let dnx = self.scan_results.iter().any(|ae| ae.bssid == bssid);
                if !dnx {
                    let freq = if channel <= 14 {
                        2407 + (channel as u16) * 5
                    } else {
                        5000 + (channel as u16) * 5
                    };
                    self.scan_results.push(Fg {
                        ssid,
                        bssid,
                        channel,
                        signal_dbm: ash,
                        security,
                        frequency_mhz: freq,
                    });
                }
            }

            offset = ifr;
        }
    }

    
    fn parse_ies(&self, data: &[u8]) -> (String, WifiSecurity) {
        let mut ssid = String::new();
        let mut security = WifiSecurity::Open;
        let mut i = 0;

        while i + 2 <= data.len() {
            let bbh = data[i];
            let ele = data[i + 1] as usize;
            let cxh = i + 2;
            let hvc = (cxh + ele).min(data.len());

            match bbh {
                DFI_ => {
                    if ele > 0 && ele <= 32 {
                        if let Ok(j) = core::str::from_utf8(&data[cxh..hvc]) {
                            ssid = String::from(j);
                        }
                    }
                }
                DFH_ => {
                    security = WifiSecurity::WPA2;
                }
                DFJ_ => {
                    
                    if ele >= 4 && data[cxh] == 0x00 && data[cxh + 1] == 0x50
                        && data[cxh + 2] == 0xF2 && data[cxh + 3] == 0x01
                    {
                        if security == WifiSecurity::Open {
                            security = WifiSecurity::WPA;
                        }
                    }
                }
                _ => {}
            }

            i = hvc;
        }

        (ssid, security)
    }

    

    
    fn start_scan_hw(&mut self) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Hardware not initialized");
        }

        
        crate::serial_println!("[IWL4965] Scan: fw_loaded={} fw_alive={} queues={} rx={}",
            self.firmware_loaded, self.fw_alive,
            self.tx_queues.len(), self.rx_queue.is_some());
        crate::println!("  FW: loaded={} alive={} queues={}",
            self.firmware_loaded, self.fw_alive, self.tx_queues.len());

        
        let tj = self.read_reg(AL_);
        let czu = self.read_reg(BQ_);
        let bta = self.read_reg(KM_);
        let cxt = self.read_reg(TL_);
        crate::serial_println!("[IWL4965] Pre-scan CSR: GP={:#X} INT={:#X} GP1={:#X} FH={:#X}", tj, czu, bta, cxt);
        crate::println!("  CSR: GP_CNTRL={:#X} INT={:#X}", tj, czu);

        self.scan_results.clear();
        self.scanning = true;
        self.scan_start_tick = crate::logger::eg();
        self.wifi_state = WifiState::Scanning;

        if self.firmware_loaded && self.fw_alive {
            
            crate::serial_println!("[IWL4965] Sending RXON to enable radio...");
            self.send_rxon()?;
            
            for _ in 0..500_000 { core::hint::spin_loop(); }
            self.poll_rx(); 
            let mqs = self.read_reg(BQ_);
            crate::serial_println!("[IWL4965] Post-RXON: INT={:#X}", mqs);

            crate::serial_println!("[IWL4965] Starting firmware-based scan (2.4 GHz + 5 GHz)...");
            self.send_scan_request()?;
        } else {
            
            self.write_reg(BSV_, 0x40);
            crate::serial_println!("[IWL4965] Passive scan started (no firmware)");
            crate::println!("  Warning: No firmware - passive scan only");
        }

        Ok(())
    }

    
    
    fn send_rxon(&mut self) -> Result<(), &'static str> {
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        let mut afp = [0u8; 44];

        
        afp[0..6].copy_from_slice(&self.mac_addr);

        
        afp[8..14].copy_from_slice(&[0xFF; 6]);

        
        afp[16..22].copy_from_slice(&[0xFF; 6]);

        
        afp[24] = 1;

        
        let ezi: u16 = 0x0001 | 0x0006 | 0x0030;
        afp[26..28].copy_from_slice(&ezi.to_le_bytes());

        
        afp[28] = 0x15;
        
        afp[29] = 0x0F;

        
        let flags: u32 = (1 << 0) | (1 << 4) | (1 << 5) | (1 << 15);
        afp[32..36].copy_from_slice(&flags.to_le_bytes());

        
        let filter: u32 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 6);
        afp[36..40].copy_from_slice(&filter.to_le_bytes());

        
        afp[40] = 1;

        self.send_hcmd(AIZ_, &afp)?;
        crate::serial_println!("[IWL4965] RXON sent: ch=1 STA flags={:#X} filter={:#X} rx_chain={:#X}",
            flags, filter, ezi);
        Ok(())
    }

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    fn send_scan_request(&mut self) -> Result<(), &'static str> {
        let channels: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 36, 40, 44, 48];
        let ehq = channels.len();

        
        const BHD_: usize = 224;
        let total_len = BHD_ + ehq * 12;
        let mut cmd = vec![0u8; total_len];

        
        let atl = (ehq * 12) as u16;

        
        cmd[0..2].copy_from_slice(&atl.to_le_bytes());
        
        
        cmd[3] = ehq as u8;
        
        
        
        
        let ezi: u16 = 0x0001 | 0x0006 | 0x0030;
        cmd[10..12].copy_from_slice(&ezi.to_le_bytes());
        
        cmd[12..16].copy_from_slice(&200000u32.to_le_bytes());
        
        cmd[16..20].copy_from_slice(&100000u32.to_le_bytes());
        
        let flags: u32 = (1 << 0) | (1 << 4) | (1 << 5);
        cmd[20..24].copy_from_slice(&flags.to_le_bytes());
        
        let filter: u32 = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 6);
        cmd[24..28].copy_from_slice(&filter.to_le_bytes());

        
        
        
        cmd[32..36].copy_from_slice(&0x0000_000Du32.to_le_bytes());
        
        
        cmd[44..48].copy_from_slice(&0x0400_000Du32.to_le_bytes());
        
        cmd[48] = 0xFF;
        
        cmd[72..76].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        
        cmd[82] = 1;

        

        
        let mut pos = BHD_;
        for &ch in channels {
            
            cmd[pos..pos + 4].copy_from_slice(&0u32.to_le_bytes());
            
            cmd[pos + 4..pos + 6].copy_from_slice(&(ch as u16).to_le_bytes());
            
            cmd[pos + 6] = 0x28;
            
            cmd[pos + 7] = 110;
            
            cmd[pos + 8..pos + 10].copy_from_slice(&20u16.to_le_bytes());
            
            cmd[pos + 10..pos + 12].copy_from_slice(&120u16.to_le_bytes());
            pos += 12;
        }

        self.send_hcmd(BGM_, &cmd)?;

        crate::serial_println!("[IWL4965] Scan request sent: {} channels, {} bytes, passive mode",
            ehq, total_len);
        Ok(())
    }

    
    fn poll_scan(&mut self) {
        if !self.scanning {
            return;
        }

        let gx = crate::logger::eg();
        let bb = gx.saturating_sub(self.scan_start_tick);

        
        self.poll_rx();

        
        if bb >= CTZ_ {
            self.scanning = false;
            self.wifi_state = if self.connected_ssid.is_some() {
                WifiState::Connected
            } else {
                WifiState::Disconnected
            };
            crate::serial_println!("[IWL4965] Scan complete: {} networks", self.scan_results.len());

            
            if self.scan_results.is_empty() {
                self.detect_networks_from_ether();
            }
        }
    }

    
    
    
    fn detect_networks_from_ether(&mut self) {
        
        
        

        let gpio = self.read_reg(ARL_);
        let mfk = self.read_reg(AL_);

        crate::serial_println!("[IWL4965] GPIO: {:#010X}, GP_CNTRL: {:#010X}", gpio, mfk);

        
        let fbc = self.eeprom_read(BWB_);
        let miy = (fbc & 0x01) != 0 || fbc == 0; 
        let miz = (fbc & 0x02) != 0;
        crate::serial_println!("[IWL4965] SKU: {:#06X}, 2.4GHz: {}, 5GHz: {}", fbc, miy, miz);

        
        
        
        
    }

    

    fn do_connect(&mut self, ssid: &str, _password: &str) -> Result<(), &'static str> {
        if !self.initialized {
            return Err("Hardware not initialized");
        }

        
        let network = self.scan_results.iter()
            .find(|ae| ae.ssid == ssid)
            .cloned();

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

                
                if self.fw_alive && !self.tx_queues.is_empty() {
                    self.send_rxon_cmd(&net)?;
                    self.wifi_state = WifiState::Authenticating;
                    
                    
                    
                    
                    match net.security {
                        WifiSecurity::Open => {
                            
                            self.connected_ssid = Some(String::from(ssid));
                            self.wifi_state = WifiState::Connected;
                            crate::serial_println!("[IWL4965] Connected to '{}' (Open, {} dBm)", ssid, net.signal_dbm);
                        }
                        WifiSecurity::WPA2 | WifiSecurity::WPA | WifiSecurity::WPA3 => {
                            
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

    
    fn send_rxon_cmd(&mut self, net: &Fg) -> Result<(), &'static str> {
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        
        

        let mut afp = [0u8; 56];

        
        afp[0..6].copy_from_slice(&net.bssid);
        
        afp[8..14].copy_from_slice(&self.mac_addr);
        
        afp[16..22].copy_from_slice(&net.bssid);
        
        afp[22] = 0x01;
        afp[23] = 0x00;
        
        afp[24] = 0x03;
        
        afp[25] = 0x03;
        
        afp[26] = net.channel;

        self.send_hcmd(AIZ_, &afp)?;
        crate::serial_println!("[IWL4965] RXON sent: ch{} BSSID {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            net.channel, net.bssid[0], net.bssid[1], net.bssid[2],
            net.bssid[3], net.bssid[4], net.bssid[5]);

        Ok(())
    }
}





impl Cw for Iwl4965 {
    fn info(&self) -> &Bb {
        &CA_
    }

    fn probe(&mut self, go: &L) -> Result<(), &'static str> {
        self.pci_bus = go.bus;
        self.pci_device = go.device;
        self.pci_function = go.function;
        self.device_id = go.device_id;
        self.status = DriverStatus::Loading;

        crate::println!("    [iwl4965] probe: map_bar0...");
        
        self.map_bar0(go)?;
        crate::println!("    [iwl4965] probe: map_bar0 OK (base={:#X})", self.mmio_base);

        
        crate::println!("    [iwl4965] probe: PCI bus master...");
        let cmd = crate::pci::ms(go.bus, go.device, go.function, 0x04);
        crate::pci::qj(go.bus, go.device, go.function, 0x04,
            cmd | 0x06); 
        crate::println!("    [iwl4965] probe: done");

        Ok(())
    }

    fn start(&mut self) -> Result<(), &'static str> {
        crate::println!("  [wifi start] hw_init...");
        self.hw_init()?;
        let ico = self.read_reg(AL_);
        crate::println!("  [wifi start] hw_init OK  GP={:#010X} MAC_CLK={}",
            ico,
            if ico & IM_ != 0 { "ready" } else { "DEAD" });
        self.wifi_state = WifiState::Disconnected;
        self.status = DriverStatus::Running;

        
        
        if eou() {
            let mat = RN_.lock().as_ref().map(|d| d.len()).unwrap_or(0);
            crate::println!("  [wifi start] Firmware available ({} bytes), loading...", mat);
            match self.load_firmware() {
                Ok(()) => {
                    crate::println!("  [wifi start] Firmware loaded and ALIVE!");
                    crate::serial_println!("[IWL4965] Firmware loaded and alive");

                    
                    crate::println!("  [wifi start] init_queues...");
                    match self.init_queues() {
                        Ok(()) => {
                            crate::println!("  [wifi start] DMA queues ready — full WiFi mode");
                            crate::serial_println!("[IWL4965] DMA queues ready — full WiFi mode");
                        }
                        Err(e) => {
                            crate::println!("  [wifi start] Queue init FAILED: {}", e);
                        }
                    }

                    
                    self.write_reg(BQ_, 0xFFFFFFFF); 
                    self.write_reg(GU_, 0xAB00_000B);
                }
                Err(e) => {
                    crate::println!("  [wifi start] FW load FAILED: {}", e);
                    crate::serial_println!("[IWL4965] Firmware load failed: {} — passive mode", e);
                }
            }
        } else {
            crate::println!("  [wifi start] No firmware file — passive scan only");
        }

        
        let tj = self.read_reg(AL_);
        let kld = if tj & IM_ != 0 { "ready" } else { "DEAD" };
        crate::println!("  [wifi start] SUMMARY: MAC_CLK={} FW={} ALIVE={} GP={:#010X}",
            kld,
            if self.firmware_loaded { "yes" } else { "no" },
            if self.fw_alive { "yes" } else { "no" },
            tj);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), &'static str> {
        
        self.write_reg(GU_, 0);
        
        self.write_reg(CO_, TN_);
        self.status = DriverStatus::Suspended;
        self.wifi_state = WifiState::Disabled;
        Ok(())
    }

    fn status(&self) -> DriverStatus {
        self.status
    }

    fn btc(&mut self) {
        let aoj = self.read_reg(BQ_);
        if aoj == 0 || aoj == 0xFFFFFFFF {
            return;
        }
        self.write_reg(BQ_, aoj);

        
        self.poll_rx();

        
        if self.scanning {
            self.poll_scan();
        }
    }
}

impl Dd for Iwl4965 {
    fn mac_address(&self) -> [u8; 6] {
        self.mac_addr
    }

    fn link_up(&self) -> bool {
        self.wifi_state == WifiState::Connected
    }

    fn cbj(&self) -> u32 {
        if self.wifi_state == WifiState::Connected { 54 } else { 0 } 
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

        
        let avn = &mut self.tx_queues[0];
        let idx = avn.write_ptr;

        
        let buf = &mut avn.cmd_buffers[idx];
        
        
        let len = data.len().min(buf.len());
        buf[..len].copy_from_slice(&data[..len]);

        
        let kt = buf.as_ptr() as u64;
        let hg = crate::memory::lc(kt)
            .unwrap_or(kt.wrapping_sub(crate::memory::hhdm_offset()));
        
        let tfd = &mut avn.tfds[idx];
        *tfd = IwlTfd::zeroed();
        tfd.tbs[0].set(hg, len as u16);
        tfd.set_num_tbs(1);

        avn.write_ptr = (idx + 1) % LZ_;

        
        let hcq = avn.write_ptr as u32;
        self.write_reg(0x060, hcq); 

        self.stats.tx_packets += 1;
        self.stats.tx_bytes += len as u64;
        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        
        self.poll_rx();
        self.rx_pending.pop()
    }

    fn poll(&mut self) {
        
        self.poll_rx();
        
        if self.scanning {
            self.poll_scan();
        }
    }

    fn stats(&self) -> NetStats {
        self.stats
    }
}

impl Nx for Iwl4965 {
    fn wifi_state(&self) -> WifiState {
        self.wifi_state
    }

    fn scan(&mut self) -> Result<(), &'static str> {
        
        if !self.initialized {
            crate::serial_println!("[IWL4965] Lazy start: initializing hardware...");
            self.start().map_err(|e| {
                crate::serial_println!("[IWL4965] Lazy start failed: {}", e);
                "WiFi hardware init failed"
            })?;
        }
        self.start_scan_hw()
    }

    fn scan_results(&self) -> Vec<Fg> {
        self.scan_results.clone()
    }

    fn connect(&mut self, ssid: &str, uy: &str) -> Result<(), &'static str> {
        if !self.initialized {
            crate::serial_println!("[IWL4965] Lazy start for connect...");
            self.start().map_err(|_| "WiFi hardware init failed")?;
        }
        self.do_connect(ssid, uy)
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


unsafe impl Send for Iwl4965 {}
unsafe impl Sync for Iwl4965 {}





static CA_: Bb = Bb {
    name: "Intel WiFi (iwl4965)",
    version: "0.1.0",
    author: "TrustOS",
    category: DriverCategory::Network,
    vendor_ids: &[(VO_, 0xFFFF)], 
};


pub fn probe(go: &L) -> Option<Box<dyn Nx>> {
    
    if go.vendor_id != VO_ {
        return None;
    }

    if !AFK_.contains(&go.device_id) {
        return None;
    }

    crate::serial_println!("[IWL4965] Probing Intel WiFi {:04X}:{:04X}...", 
        go.vendor_id, go.device_id);

    let mut driver = Iwl4965::new();
    match driver.probe(go) {
        Ok(()) => {
            
            
            crate::serial_println!("[IWL4965] PCI probe OK for {:04X}:{:04X} — start deferred", 
                go.vendor_id, go.device_id);
            Some(Box::new(driver))
        }
        Err(e) => {
            crate::serial_println!("[IWL4965] Probe failed: {}", e);
            None
        }
    }
}



pub fn hqu() {
    use super::wifi::EN_;
    let jg = EN_.lock();
    if let Some(ref driver) = *jg {
        
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
        if let Some(yp) = driver.signal_strength() {
            crate::println!("  Signal:     {} dBm", yp);
        }
    } else {
        crate::println!("  No WiFi driver loaded");
    }

    crate::println!("  Firmware:   {}", if eou() { "available" } else { "NOT loaded" });
    if let Some(ref fo) = *RN_.lock() {
        crate::println!("  FW size:    {} bytes", fo.len());
    }
}



pub fn byp(offset: u32) -> Option<u32> {
    
    if offset >= 0x2000 {
        return None;
    }
    
    let devices = crate::pci::aqs();
    for s in &devices {
        if s.vendor_id == VO_ && AFK_.contains(&s.device_id) {
            let bar0 = s.bar[0];
            if bar0 == 0 || (bar0 & 1) != 0 { return None; }
            let arf = (bar0 >> 1) & 0x3 == 2;
            let phys = if arf {
                let bqi = s.bar[1] as u64;
                (bqi << 32) | (bar0 & 0xFFFFFFF0) as u64
            } else {
                (bar0 & 0xFFFFFFF0) as u64
            };
            if phys == 0 { return None; }
            
            let virt = match crate::memory::yv(phys, 0x2000) {
                Ok(v) => v as usize,
                Err(_) => return None,
            };
            unsafe {
                let ptr = (virt + offset as usize) as *const u32;
                return Some(core::ptr::read_volatile(ptr));
            }
        }
    }
    None
}


pub fn hqv() {
    let regs: &[(&str, u32)] = &[
        ("HW_IF_CONFIG",  GT_),
        ("INT",           BQ_),
        ("INT_MASK",      GU_),
        ("FH_INT_STATUS", TL_),
        ("GPIO_IN",       ARL_),
        ("RESET",         CO_),
        ("GP_CNTRL",      AL_),
        ("HW_REV",        ARM_),
        ("EEPROM_REG",    ACJ_),
        ("EEPROM_GP",     BSM_),
        ("UCODE_DRV_GP1", KM_),
        ("UCODE_DRV_GP2", BSY_),
        ("GIO_REG",       BSP_),
        ("GP_UCODE",      KL_),
        ("GP_DRIVER",     BSQ_),
    ];

    for (name, offset) in regs {
        match byp(*offset) {
            Some(val) => crate::println!("  CSR {:<16} [0x{:03X}] = 0x{:08X}", name, offset, val),
            None => crate::println!("  CSR {:<16} [0x{:03X}] = <unavailable>", name, offset),
        }
    }
}






fn ejy() -> Option<usize> {
    let devices = crate::pci::aqs();
    for s in &devices {
        if s.vendor_id == VO_ && AFK_.contains(&s.device_id) {
            let bar0 = s.bar[0];
            if bar0 == 0 || (bar0 & 1) != 0 { return None; }
            let arf = (bar0 >> 1) & 0x3 == 2;
            let phys = if arf {
                let bqi = s.bar[1] as u64;
                (bqi << 32) | (bar0 & 0xFFFFFFF0) as u64
            } else {
                (bar0 & 0xFFFFFFF0) as u64
            };
            if phys == 0 { return None; }
            return crate::memory::yv(phys, 0x2000).ok().map(|v| v as usize);
        }
    }
    None
}

fn brs(base: usize, offset: u32) -> u32 {
    unsafe { core::ptr::read_volatile((base + offset as usize) as *const u32) }
}

fn baz(base: usize, offset: u32, val: u32) {
    unsafe { core::ptr::write_volatile((base + offset as usize) as *mut u32, val); }
}


pub fn lce(offset: u32, val: u32) -> bool {
    if offset >= 0x2000 { return false; }
    if let Some(base) = ejy() {
        baz(base, offset, val);
        true
    } else { false }
}


pub fn fqy(addr: u32) -> Option<u32> {
    let base = ejy()?;
    baz(base, VA_, (addr & 0x000F_FFFF) | (3 << 24));
    Some(brs(base, VB_))
}


pub fn lcf(addr: u32, val: u32) -> bool {
    if let Some(base) = ejy() {
        baz(base, OI_, (addr & 0x000F_FFFF) | (3 << 24));
        baz(base, OJ_, val);
        true
    } else { false }
}


pub fn lbz() -> Result<(), &'static str> {
    let base = ejy().ok_or("No MMIO")?;
    
    crate::println!("  [1] Setting NIC_READY...");
    let mlg = brs(base, GT_);
    baz(base, GT_, mlg | NN_);
    for _ in 0..5000u32 {
        if brs(base, GT_) & NN_ != 0 { break; }
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
    let nkl = brs(base, GT_) & NN_ != 0;
    crate::println!("      NIC_READY = {}", if nkl { "YES" } else { "NO" });
    
    crate::println!("  [2] GIO chicken bits (disable L0s timer)...");
    let meo = brs(base, TM_);
    baz(base, TM_, meo | ARK_);
    
    crate::println!("  [3] Setting INIT_DONE...");
    let tj = brs(base, AL_);
    baz(base, AL_, tj | ACK_);
    
    crate::println!("  [4] Polling MAC clock...");
    for i in 0..25000u32 {
        let val = brs(base, AL_);
        if val & IM_ != 0 {
            crate::println!("      MAC clock READY after {} iters (GP={:#010X})", i, val);
            
            crate::println!("  [5] Enabling APMG DMA+BSM clocks...");
            
            baz(base, OI_, (RX_ & 0x000F_FFFF) | (3 << 24));
            baz(base, OJ_, AAH_ | AAG_);
            for _ in 0..20_000 { core::hint::spin_loop(); }
            
            crate::println!("  [6] Disabling L1-Active...");
            baz(base, VA_, (MO_ & 0x000F_FFFF) | (3 << 24));
            let gwq = brs(base, VB_);
            baz(base, OI_, (MO_ & 0x000F_FFFF) | (3 << 24));
            baz(base, OJ_, gwq | AMM_);
            
            crate::println!("  APM init DONE!");
            return Ok(());
        }
        for _ in 0..1000 { core::hint::spin_loop(); }
        if i % 5000 == 4999 { crate::print!("."); }
    }
    
    let lvm = brs(base, AL_);
    crate::println!("      MAC clock TIMEOUT (GP={:#010X})", lvm);
    Err("MAC clock not ready")
}


pub fn lcc() -> Result<(), &'static str> {
    use super::wifi::EN_;
    
    
    let mut jg = EN_.lock();
    let driver = jg.as_mut().ok_or("No WiFi driver — run 'drv reprobe wifi' first")?;
    
    crate::println!("  Driver found: {:?}", driver.status());
    
    
    crate::println!("  Calling driver.start()...");
    crate::println!("    This does: hw_init → apm_init → firmware load → DMA queues");
    crate::println!("    Watch for step-by-step output below:");
    crate::println!();
    
    match driver.start() {
        Ok(()) => {
            crate::println!();
            let tj = byp(AL_).unwrap_or(0);
            let bta = byp(KM_).unwrap_or(0);
            let asp = byp(KL_).unwrap_or(0);
            crate::println!("  Result: OK!");
            crate::println!("    GP_CNTRL={:#010X} GP1={:#010X} UCODE={:#010X}", tj, bta, asp);
            crate::println!("    MAC_CLK={}", if tj & IM_ != 0 { "ready" } else { "DEAD" });
            Ok(())
        }
        Err(e) => {
            crate::println!();
            let tj = byp(AL_).unwrap_or(0);
            crate::println!("  Result: FAILED: {}", e);
            crate::println!("    GP_CNTRL={:#010X}", tj);
            crate::println!("    MAC_CLK={}", if tj & IM_ != 0 { "ready" } else { "DEAD" });
            Err(e)
        }
    }
}


pub fn lcb() {
    let kea: &[(&str, u32)] = &[
        ("BSM_WR_CTRL",        MX_),
        ("BSM_WR_MEM_SRC",     AOC_),
        ("BSM_WR_MEM_DST",     AOB_),
        ("BSM_WR_DWCOUNT",     AOA_),
        ("BSM_DRAM_INST_PTR",  ANY_),
        ("BSM_DRAM_INST_SIZE", ANX_),
        ("BSM_DRAM_DATA_PTR",  ANW_),
        ("BSM_DRAM_DATA_SIZE", ANV_),
    ];
    
    crate::println!("  BSM Registers (via PRPH bus):");
    for (name, addr) in kea {
        match fqy(*addr) {
            Some(val) => crate::println!("    {:<20} [0x{:04X}] = 0x{:08X}", name, addr, val),
            None => crate::println!("    {:<20} [0x{:04X}] = <unavailable>", name, addr),
        }
    }
}


pub fn lca() {
    let jwu: &[(&str, u32)] = &[
        ("APMG_CLK_CTRL",   BMT_),
        ("APMG_CLK_EN",     RX_),
        ("APMG_CLK_DIS",    BMU_),
        ("APMG_PS_CTRL",    BMV_),
        ("APMG_PCIDEV_STT", MO_),
    ];
    
    crate::println!("  APMG Registers (via PRPH bus):");
    for (name, addr) in jwu {
        match fqy(*addr) {
            Some(val) => crate::println!("    {:<20} [0x{:04X}] = 0x{:08X}", name, addr, val),
            None => crate::println!("    {:<20} [0x{:04X}] = <unavailable>", name, addr),
        }
    }
}
