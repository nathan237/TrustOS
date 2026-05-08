










use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};






mod reg {
    pub const Yz: u32     = 0x00;  
    pub const Arn: u32     = 0x02;  
    pub const Arl: u32     = 0x03;  
    pub const Azx: u32   = 0x04;  
    pub const Axd: u32    = 0x06;  
    pub const Ef: u32     = 0x08;  
    pub const Bfl: u32   = 0x0C;  
    pub const Ux: u32 = 0x0E;  
    pub const Awv: u32     = 0x10;  
    pub const Sx: u32   = 0x20;  
    pub const Axh: u32   = 0x24;  
    pub const Arw: u32   = 0x30;  
    pub const Aej: u32    = 0x38;  

    
    pub const Ahv: u32 = 0x40;  
    pub const Ahw: u32 = 0x44;  
    pub const Rg: u32    = 0x48;  
    pub const Wz: u32    = 0x4A;  
    pub const Wy: u32   = 0x4C;  
    pub const Atl: u32   = 0x4D;  
    pub const Xa: u32  = 0x4E;  

    
    pub const Aoe: u32 = 0x50;  
    pub const Aog: u32 = 0x54;  
    pub const Ade: u32    = 0x58;  
    pub const Aod: u32   = 0x5A;  
    pub const Adb: u32   = 0x5C;  
    pub const Aof: u32   = 0x5D;  
    pub const Adc: u32  = 0x5E;  

    
    pub const Awz: u32  = 0x60;  
    pub const Axr: u32  = 0x64;  
    pub const Axb: u32 = 0x68;  

    
    pub const Aim: u32 = 0x70;  
    pub const Ain: u32 = 0x74;  

    
    pub const CVP_: u32 = 0x80;
    pub const CVQ_: u32 = 0x20;
}


mod sd {
    pub const Bf: u32    = 0x00;  
    pub const Ix: u32    = 0x03;  
    pub const Aap: u32   = 0x04;  
    pub const Oe: u32    = 0x08;  
    pub const Pg: u32    = 0x0C;  
    pub const Ajt: u32  = 0x10;  
    pub const Ov: u32    = 0x12;  
    pub const Ob: u32  = 0x18;  
    pub const Oc: u32  = 0x1C;  
}


mod gctl {
    pub const Hi: u32   = 1 << 0;   
    pub const Avr: u32 = 1 << 1;   
    pub const Afr: u32  = 1 << 8;   
}


mod sctl {
    pub const Fc: u32 = 1 << 0;     
    pub const Ir: u32  = 1 << 1;     
    pub const Aln: u32 = 1 << 2;     
    
    pub const AKF_: u32 = 20;
}


mod ssts {
    pub const Ahf: u8 = 1 << 2;   
    pub const Ajr: u8 = 1 << 3;  
    pub const Aif: u8 = 1 << 4;   
    pub const Ajs: u8 = 1 << 5; 
}





mod verb {
    
    pub const EW_: u32        = 0xF00;
    pub const CAA_: u32        = 0xF02;
    pub const CAB_: u32      = 0xF01;
    pub const AUZ_: u32      = 0xF07;
    pub const BZZ_: u32   = 0xF1C;
    pub const AUY_: u32             = 0xF0C;
    pub const UO_: u32      = 0xF05;
    pub const AUX_: u32   = 0xF06;

    
    pub const DQ_: u32         = 0xB00;  
    pub const CAH_: u32    = 0xA00;  

    
    pub const CVZ_: u32      = 0x701;
    pub const QI_: u32      = 0x705;
    pub const CVY_: u32   = 0x706;
    pub const AJQ_: u32      = 0x707;
    pub const AJP_: u32             = 0x70C;

    
    pub const EJF_: u32    = 0x300;
    pub const CWE_: u32    = 0x200;
    pub const EJJ_: u32       = 0x500;  
    pub const CWC_: u32        = 0x400;  
    pub const DQM_: u32       = 0xD00;  
    pub const DQR_: u32        = 0xC00;  

    
    pub const BHQ_: u32        = 0x715;
    pub const CWB_: u32        = 0x716;
    pub const CWA_: u32         = 0x717;
    pub const CAC_: u32        = 0xF15;
    pub const CAE_: u32        = 0xF16;
    pub const CAD_: u32         = 0xF17;
    pub const CLZ_: u32    = 0x11;  

    
    pub const AIC_: u32     = 0x00;
    pub const ECL_: u32      = 0x02;
    pub const BES_: u32    = 0x04;
    pub const CLY_: u32 = 0x05;
    pub const PS_: u32    = 0x09;  
    pub const ECJ_: u32     = 0x0A;  
    pub const ECM_: u32   = 0x0B;  
    pub const CMA_: u32      = 0x0C;  
    pub const XK_: u32   = 0x0D;  
    pub const CLX_: u32 = 0x0E;  
    pub const ECK_: u32  = 0x0F;  
    pub const PR_: u32  = 0x12;  
    pub const ECN_: u32 = 0x13;
}


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
    fn lzc(caps: u32) -> Self {
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


fn gnb(config: u32) -> &'static str {
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






#[derive(Debug, Clone)]
pub struct Aw {
    pub nid: u16,
    pub widget_type: WidgetType,
    pub caps: u32,
    pub pin_config: u32,
    pub connections: Vec<u16>,
    pub amp_in_caps: u32,
    pub amp_out_caps: u32,
}


#[derive(Debug, Clone)]
pub struct Wk {
    pub pin_nid: u16,
    pub dac_nid: u16,
    pub path: Vec<u16>,  
    pub device_type: &'static str,
}


#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
struct Ahj {
    address: u64,    
    length: u32,     
    ioc: u32,        
}


pub struct HdaController {
    
    mmio_base: u64,
    
    num_iss: u8,
    
    num_oss: u8,
    
    num_bss: u8,
    
    addr64: bool,

    
    corb_virt: u64,
    corb_phys: u64,
    corb_entries: u16,

    
    rirb_virt: u64,
    rirb_phys: u64,
    rirb_entries: u16,
    rirb_rp: u16,  

    
    codecs: Vec<u8>,
    
    widgets: Vec<Aw>,
    
    output_paths: Vec<Wk>,

    
    stream_tag: u8,
    
    audio_buf_virt: u64,
    audio_buf_phys: u64,
    audio_buf_size: u32,
    
    bdl_virt: u64,
    bdl_phys: u64,

    
    playing: bool,
    
    afg_amp_out_caps: u32,
    afg_amp_in_caps: u32,
}


static Cd: Mutex<Option<HdaController>> = Mutex::new(None);
static AXW_: AtomicBool = AtomicBool::new(false);





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

    
    fn osd_base(&self, ae: u8) -> u32 {
        reg::CVP_ + ((self.num_iss + ae) as u32) * reg::CVQ_
    }

    
    
    

    
    pub fn init(s: &crate::pci::L) -> Result<Self, &'static str> {
        crate::serial_println!("[HDA] Initializing Intel HDA controller...");
        crate::serial_println!("[HDA]   PCI {:02X}:{:02X}.{} {:04X}:{:04X}",
            s.bus, s.device, s.function, s.vendor_id, s.device_id);

        
        crate::pci::bzi(s);
        crate::pci::bzj(s);

        
        let cgc = s.bar_address(0).ok_or("HDA: no BAR0")?;
        crate::serial_println!("[HDA]   BAR0 phys = {:#010X}", cgc);

        
        let bz = crate::memory::hhdm_offset();
        let mmio_base = cgc + bz;

        
        for za in 0..4 {
            let phys = (cgc & !0xFFF) + za * 0x1000;
            let virt = phys + bz;
            crate::memory::paging::ilu(virt, phys)?;
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
            afg_amp_out_caps: 0,
            afg_amp_in_caps: 0,
        };

        
        unsafe {
            let agk = ctrl.read16(reg::Yz);
            let pst = ctrl.read8(reg::Arn);
            let psp = ctrl.read8(reg::Arl);

            ctrl.num_oss = ((agk >> 12) & 0xF) as u8;
            ctrl.num_iss = ((agk >> 8) & 0xF) as u8;
            ctrl.num_bss = ((agk >> 3) & 0x1F) as u8;
            ctrl.addr64 = (agk & 1) != 0;

            crate::serial_println!("[HDA]   Version {}.{}", psp, pst);
            crate::serial_println!("[HDA]   Streams: {} output, {} input, {} bidir",
                ctrl.num_oss, ctrl.num_iss, ctrl.num_bss);
            crate::serial_println!("[HDA]   64-bit: {}", ctrl.addr64);

            if ctrl.num_oss == 0 {
                return Err("HDA: no output streams available");
            }
        }

        
        ctrl.reset()?;

        
        ctrl.setup_corb_rirb()?;

        
        ctrl.discover_codecs()?;

        
        ctrl.find_output_paths();

        
        ctrl.setup_output_stream()?;

        crate::serial_println!("[HDA] Initialization complete!");
        Ok(ctrl)
    }

    
    fn reset(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[HDA] Resetting controller...");
        unsafe {
            
            self.write16(reg::Ux, 0xFFFF);

            
            let gctl = self.read32(reg::Ef);
            self.write32(reg::Ef, gctl & !gctl::Hi);

            
            for _ in 0..1000 {
                if self.read32(reg::Ef) & gctl::Hi == 0 {
                    break;
                }
                Self::aas(10);
            }
            if self.read32(reg::Ef) & gctl::Hi != 0 {
                return Err("HDA: reset enter timeout");
            }

            
            let gctl = self.read32(reg::Ef);
            self.write32(reg::Ef, gctl | gctl::Hi);

            
            for _ in 0..1000 {
                if self.read32(reg::Ef) & gctl::Hi != 0 {
                    break;
                }
                Self::aas(10);
            }
            if self.read32(reg::Ef) & gctl::Hi == 0 {
                return Err("HDA: reset exit timeout");
            }

            
            
            
            let mut bdv = 0u16;
            for attempt in 0..10 {
                Self::aas(if attempt == 0 { 1000 } else { 5000 });
                bdv = self.read16(reg::Ux);
                if bdv != 0 { break; }
            }

            
            let gctl = self.read32(reg::Ef);
            self.write32(reg::Ef, gctl | gctl::Afr);

            
            
            self.write32(reg::Aej, 0x00000000);

            
            
            self.write32(reg::Ain, 0);
            self.write32(reg::Aim, 0x01); 

            crate::serial_println!("[HDA]   STATESTS = {:#06X} (codec presence)", bdv);

            if bdv == 0 {
                return Err("HDA: no codecs detected after reset");
            }

            
            for i in 0..15u8 {
                if bdv & (1 << i) != 0 {
                    self.codecs.push(i);
                    crate::serial_println!("[HDA]   Codec {} present", i);
                }
            }
        }
        Ok(())
    }

    
    
    

    fn setup_corb_rirb(&mut self) -> Result<(), &'static str> {
        crate::serial_println!("[HDA] Setting up CORB/RIRB...");
        let bz = crate::memory::hhdm_offset();

        unsafe {
            
            self.write8(reg::Wy, 0);
            self.write8(reg::Adb, 0);
            Self::aas(100);

            
            let hnv = self.read8(reg::Xa);
            let (corb_sz_sel, corb_entries) = if hnv & 0x40 != 0 {
                (2u8, 256u16)
            } else if hnv & 0x20 != 0 {
                (1, 16)
            } else {
                (0, 2)
            };
            self.write8(reg::Xa, corb_sz_sel);
            self.corb_entries = corb_entries;
            crate::serial_println!("[HDA]   CORB: {} entries", corb_entries);

            
            let hnu = (corb_entries as usize) * 4;
            let hnt: Vec<u8> = vec![0u8; hnu + 4096]; 
            let kxx = hnt.as_ptr() as u64;
            let corb_virt = (kxx + 0xFFF) & !0xFFF; 
            core::mem::forget(hnt);

            let corb_phys = corb_virt.checked_sub(bz)
                .ok_or("HDA: CORB virt->phys failed")?;
            self.corb_virt = corb_virt;
            self.corb_phys = corb_phys;

            
            core::ptr::write_bytes(corb_virt as *mut u8, 0, hnu);

            
            self.write32(reg::Ahv, corb_phys as u32);
            self.write32(reg::Ahw, (corb_phys >> 32) as u32);

            
            self.write16(reg::Wz, 1 << 15); 
            Self::aas(100);
            
            self.write16(reg::Wz, 0);
            Self::aas(100);

            
            self.write16(reg::Rg, 0);

            
            let jaz = self.read8(reg::Adc);
            let (rirb_sz_sel, rirb_entries) = if jaz & 0x40 != 0 {
                (2u8, 256u16)
            } else if jaz & 0x20 != 0 {
                (1, 16)
            } else {
                (0, 2)
            };
            self.write8(reg::Adc, rirb_sz_sel);
            self.rirb_entries = rirb_entries;
            crate::serial_println!("[HDA]   RIRB: {} entries", rirb_entries);

            
            let jay = (rirb_entries as usize) * 8;
            let jax: Vec<u8> = vec![0u8; jay + 4096];
            let ohk = jax.as_ptr() as u64;
            let rirb_virt = (ohk + 0xFFF) & !0xFFF;
            core::mem::forget(jax);

            let rirb_phys = rirb_virt.checked_sub(bz)
                .ok_or("HDA: RIRB virt->phys failed")?;
            self.rirb_virt = rirb_virt;
            self.rirb_phys = rirb_phys;

            core::ptr::write_bytes(rirb_virt as *mut u8, 0, jay);

            
            self.write32(reg::Aoe, rirb_phys as u32);
            self.write32(reg::Aog, (rirb_phys >> 32) as u32);

            
            self.write16(reg::Ade, 1 << 15);
            Self::aas(100);

            
            self.write16(reg::Aod, 1);

            self.rirb_rp = 0;

            
            self.write8(reg::Wy, 0x02); 
            self.write8(reg::Adb, 0x02); 
            Self::aas(100);

            crate::serial_println!("[HDA]   CORB phys={:#010X}, RIRB phys={:#010X}",
                corb_phys, rirb_phys);
        }

        Ok(())
    }

    
    fn send_verb(&mut self, codec: u8, nid: u16, verb: u32, payload: u32) -> Result<u32, &'static str> {
        
        let cmd = ((codec as u32) << 28)
            | ((nid as u32 & 0xFF) << 20)
            | (verb & 0xFFFFF);
        
        
        
        let _ = payload; 

        unsafe {
            
            let ma = self.read16(reg::Rg) & 0xFF;
            let iql = ((ma + 1) % self.corb_entries) as u16;

            let kxw = self.corb_virt as *mut u32;
            core::ptr::write_volatile(kxw.add(iql as usize), cmd);

            
            self.write16(reg::Rg, iql);

            
            for _ in 0..10000 {
                let ohl = self.read16(reg::Ade) & 0xFF;
                if ohl != self.rirb_rp {
                    
                    self.rirb_rp = (self.rirb_rp + 1) % self.rirb_entries;
                    let ohj = self.rirb_virt as *const u64;
                    let fa = core::ptr::read_volatile(ohj.add(self.rirb_rp as usize));
                    let data = fa as u32;
                    
                    self.write8(reg::Aof, 0x05);
                    return Ok(data);
                }
                Self::aas(10);
            }
        }
        Err("HDA: RIRB timeout")
    }

    
    fn codec_cmd(&mut self, codec: u8, nid: u16, verb: u32, data: u8) -> Result<u32, &'static str> {
        let mai = (verb << 8) | (data as u32);
        self.send_verb(codec, nid, mai, 0)
    }

    
    fn get_param(&mut self, codec: u8, nid: u16, param: u32) -> Result<u32, &'static str> {
        self.codec_cmd(codec, nid, verb::EW_, param as u8)
    }

    
    fn set_verb_16(&mut self, codec: u8, nid: u16, verb_id: u32, payload: u16) -> Result<u32, &'static str> {
        
        
        let obn = ((verb_id & 0xF00) << 8) | (payload as u32);
        self.send_verb(codec, nid, obn, 0)
    }

    
    
    

    fn discover_codecs(&mut self) -> Result<(), &'static str> {
        let codecs = self.codecs.clone();
        for &caddr in &codecs {
            crate::serial_println!("[HDA] Walking codec {}...", caddr);

            
            let vendor = self.get_param(caddr, 0, verb::AIC_)?;
            crate::serial_println!("[HDA]   Vendor={:04X}, Device={:04X}",
                vendor >> 16, vendor & 0xFFFF);

            
            let node_count = self.get_param(caddr, 0, verb::BES_)?;
            let fbn = ((node_count >> 16) & 0xFF) as u16;
            let iro = (node_count & 0xFF) as u16;
            crate::serial_println!("[HDA]   Root: subnodes {}..{}", fbn, fbn + iro - 1);

            
            for fg_nid in fbn..(fbn + iro) {
                let luy = self.get_param(caddr, fg_nid, verb::CLY_)?;
                let fwm = luy & 0xFF;
                crate::serial_println!("[HDA]   FG NID {}: type={} ({})", fg_nid, fwm,
                    if fwm == 1 { "Audio" } else { "Other" });

                if fwm != 1 { continue; } 

                
                let _ = self.codec_cmd(caddr, fg_nid, verb::QI_, 0x00); 

                
                self.afg_amp_out_caps = self.get_param(caddr, fg_nid, verb::PR_).unwrap_or(0);
                self.afg_amp_in_caps = self.get_param(caddr, fg_nid, verb::XK_).unwrap_or(0);
                crate::serial_println!("[HDA]   AFG amp caps: out={:#010X} in={:#010X}",
                    self.afg_amp_out_caps, self.afg_amp_in_caps);

                
                let jjl = self.get_param(caddr, fg_nid, verb::BES_)?;
                let fet = ((jjl >> 16) & 0xFF) as u16;
                let jqr = (jjl & 0xFF) as u16;
                crate::serial_println!("[HDA]   AFG widgets: {}..{}", fet, fet + jqr - 1);

                
                for nid in fet..(fet + jqr) {
                    let caps = self.get_param(caddr, nid, verb::PS_)?;
                    let wt = WidgetType::lzc(caps);

                    let mut akq = Aw {
                        nid,
                        widget_type: wt,
                        caps,
                        pin_config: 0,
                        connections: Vec::new(),
                        amp_in_caps: 0,
                        amp_out_caps: 0,
                    };

                    
                    let hni = self.get_param(caddr, nid, verb::CLX_)?;
                    let fod = (hni & 0x7F) as u16;
                    let naq = (hni & 0x80) != 0;

                    if fod > 0 && !naq {
                        
                        let mut offset = 0u8;
                        while (offset as u16) < fod {
                            let eo = self.codec_cmd(caddr, nid, verb::CAA_, offset)?;
                            for i in 0..4u32 {
                                if (offset as u16) + (i as u16) >= fod { break; }
                                let foe = ((eo >> (i * 8)) & 0xFF) as u16;
                                akq.connections.push(foe);
                            }
                            offset += 4;
                        }
                    }

                    
                    if wt == WidgetType::PinComplex {
                        akq.pin_config = self.codec_cmd(caddr, nid, verb::BZZ_, 0)?;
                    }

                    
                    
                    
                    let hfc = caps & (1 << 3) != 0;
                    if caps & (1 << 2) != 0 { 
                        if hfc {
                            akq.amp_out_caps = self.get_param(caddr, nid, verb::PR_)?;
                            if akq.amp_out_caps == 0 {
                                akq.amp_out_caps = self.afg_amp_out_caps;
                            }
                        } else {
                            
                            akq.amp_out_caps = self.afg_amp_out_caps;
                        }
                    }
                    if caps & (1 << 1) != 0 { 
                        if hfc {
                            akq.amp_in_caps = self.get_param(caddr, nid, verb::XK_)?;
                            if akq.amp_in_caps == 0 {
                                akq.amp_in_caps = self.afg_amp_in_caps;
                            }
                        } else {
                            akq.amp_in_caps = self.afg_amp_in_caps;
                        }
                    }

                    crate::serial_println!("[HDA]     NID {:3}: {} conns={:?}{}",
                        nid, wt.name(),
                        akq.connections,
                        if wt == WidgetType::PinComplex {
                            alloc::format!(" [{}]", gnb(akq.pin_config))
                        } else {
                            String::new()
                        }
                    );

                    self.widgets.push(akq);
                }
            }
        }
        Ok(())
    }

    
    fn find_output_paths(&mut self) {
        crate::serial_println!("[HDA] Searching output paths...");

        
        let nuv: Vec<(u16, u32, Vec<u16>)> = self.widgets.iter()
            .filter(|w| w.widget_type == WidgetType::PinComplex)
            .filter(|w| {
                
                let dli = (w.pin_config >> 30) & 0x3;
                let lcy = (w.pin_config >> 20) & 0xF;
                
                
                
                let mtk = matches!(lcy,
                    0x0 | 0x1 | 0x2 | 0x4 | 0x5 | 0x6 | 0xF);
                
                
                let kwz = dli == 0 || dli == 2; 
                (dli != 1 && mtk) || kwz
            })
            .map(|w| (w.nid, w.pin_config, w.connections.clone()))
            .collect();

        for (pin_nid, pin_config, pin_conns) in &nuv {
            
            if let Some(path) = self.trace_to_dac(*pin_nid, &mut Vec::new()) {
                let device = gnb(*pin_config);
                crate::serial_println!("[HDA]   Path found: {} -> {:?}", device,
                    path.iter().map(|ae| alloc::format!("{}", ae)).collect::<Vec<_>>());
                self.output_paths.push(Wk {
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

    
    fn trace_to_dac(&self, nid: u16, anc: &mut Vec<u16>) -> Option<Vec<u16>> {
        if anc.contains(&nid) { return None; } 
        anc.push(nid);

        let akq = self.widgets.iter().find(|w| w.nid == nid)?;

        if akq.widget_type == WidgetType::AudioOutput {
            return Some(vec![nid]); 
        }

        
        for &foe in &akq.connections {
            if let Some(mut path) = self.trace_to_dac(foe, anc) {
                path.insert(0, nid);
                return Some(path);
            }
        }

        None
    }

    
    
    

    fn setup_output_stream(&mut self) -> Result<(), &'static str> {
        if self.output_paths.is_empty() {
            return Err("HDA: no output paths to configure");
        }

        let bz = crate::memory::hhdm_offset();
        let codec = self.codecs[0];
        let path = self.output_paths[0].clone();

        crate::serial_println!("[HDA] Setting up output stream for path: {:?}", path.path);

        
        
        for &nid in &path.path {
            let _ = self.codec_cmd(codec, nid, verb::QI_, 0x00); 
        }

        
        
        let vendor_id = self.get_param(codec, 0, verb::AIC_).unwrap_or(0);
        let hbj = (vendor_id >> 16) & 0xFFFF;
        let kux = vendor_id & 0xFFFF;
        crate::serial_println!("[HDA]   Codec vendor={:#06X} device={:#06X}", hbj, kux);

        
        
        let gdp = hbj == 0x11D4; 
        let mse = hbj == 0x14F1; 
        let nhy = gdp || mse;

        if nhy {
            crate::serial_println!("[HDA]   Applying {} codec quirks",
                if gdp { "Analog Devices AD198x" } else { "Conexant CX205xx" });

            
            let jup: Vec<u16> = self.widgets.iter().map(|w| w.nid).collect();
            let nop: Vec<u16> = self.widgets.iter()
                .filter(|w| w.widget_type == WidgetType::PinComplex
                    && matches!((w.pin_config >> 20) & 0xF, 0x0 | 0x1 | 0x2))
                .map(|w| w.nid)
                .collect();

            
            let _ = self.codec_cmd(codec, 1, verb::QI_, 0x00);
            HdaController::aas(10_000); 
            for &nid in &jup {
                let _ = self.codec_cmd(codec, nid, verb::QI_, 0x00);
            }
            
            HdaController::aas(100_000); 

            
            
            for &nid in &nop {
                let _ = self.codec_cmd(codec, nid, verb::AJQ_, 0xC0);
                
                
                let _ = self.codec_cmd(codec, nid, verb::AJP_, 0x02);
                crate::serial_println!("[HDA]   Pin NID {} -> EAPD=0x02, PIN_CTL=0xC0", nid);
            }

            
            
            
            if gdp {
                let _ = self.set_verb_16(codec, 1, verb::CWC_, 0x0008);
                crate::serial_println!("[HDA]   AD1984 DMIC COEF: val=0x08 (default index)");
            }

            
            
            let _ = self.set_verb_16(codec, 1, 0x300,
                (1u16 << 15) | (1 << 13) | (1 << 12) | 0x27);
            let _ = self.set_verb_16(codec, 1, 0x300,
                (1u16 << 14) | (1 << 13) | (1 << 12) | 0x27);

            
            
            
            
            let _ = self.codec_cmd(codec, 1, verb::CWB_, 0x02); 
            let _ = self.codec_cmd(codec, 1, verb::CWA_,  0x02); 
            let _ = self.codec_cmd(codec, 1, verb::BHQ_, 0x02); 
            crate::serial_println!("[HDA]   GPIO1 HIGH (speaker amp power on)");
        }

        
        let _ = self.codec_cmd(codec, path.pin_nid, verb::AJQ_, 0xC0);
        
        
        let _ = self.codec_cmd(codec, path.pin_nid, verb::AJP_, 0x02);
        crate::serial_println!("[HDA]   Output pin {} -> EAPD=0x02, PIN_CTL=0xC0", path.pin_nid);

        
        
        let stream_tag = self.stream_tag;
        let channel = 0u8;
        let fmt: u16 = 0x0011; 

        let jul: Vec<u16> = self.widgets.iter()
            .filter(|w| w.widget_type == WidgetType::AudioOutput)
            .map(|w| w.nid)
            .collect();

        for &dac_nid in &jul {
            let _ = self.codec_cmd(codec, dac_nid, verb::CVY_,
                (stream_tag << 4) | channel);
            let _ = self.set_verb_16(codec, dac_nid, verb::CWE_, fmt);
            crate::serial_println!("[HDA]   DAC NID {} -> stream_tag={}, fmt=0x{:04X}",
                dac_nid, stream_tag, fmt);
        }

        
        HdaController::aas(5000);

        
        
        
        let hes: Vec<(u16, u32, usize, u32, u32)> = self.widgets.iter()
            .map(|w| (w.nid, w.caps, w.connections.len(), w.amp_out_caps, w.amp_in_caps))
            .collect();

        
        let efd = ((self.afg_amp_out_caps >> 8) & 0x7F) as u16;
        let heg = ((self.afg_amp_in_caps >> 8) & 0x7F) as u16;

        for &(nid, caps, num_conns, glg, gcd) in &hes {
            
            
            let dcc = ((glg >> 8) & 0x7F) as u16;
            let czq = ((gcd >> 8) & 0x7F) as u16;
            
            
            
            let evu = if dcc > 0 { dcc } else { efd };
            let drv = if czq > 0 { czq } else { heg };

            
            
            
            let fhb: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (evu & 0x7F);
            let _ = self.set_verb_16(codec, nid, 0x300, fhb);
            
            let fgz: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (drv & 0x7F);
            let _ = self.set_verb_16(codec, nid, 0x300, fgz);

            
            if num_conns > 1 {
                for idx in 1..num_conns.min(16) {
                    let jvm: u16 = (1 << 14) | (1 << 13) | (1 << 12) | ((idx as u16 & 0xF) << 8) | (drv & 0x7F);
                    let _ = self.set_verb_16(codec, nid, 0x300, jvm);
                }
            }
        }
        crate::serial_println!("[HDA]   Unmuted all {} widget amps (separate OUT/IN, afg_out={} afg_in={})",
            hes.len(), efd, heg);

        
        
        
        let fgk = if efd > 0 { efd } else { 3u16 };
        let juq: Vec<(u16, Vec<u16>)> = self.output_paths.iter()
            .map(|aa| (aa.pin_nid, aa.path.clone()))
            .collect();

        for (pin_nid, path_nids) in &juq {
            
            for &nid in path_nids {
                let _ = self.codec_cmd(codec, nid, verb::QI_, 0x00);
            }
            
            let nuo: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (fgk & 0x7F);
            let _ = self.set_verb_16(codec, *pin_nid, 0x300, nuo);
            
            let nup: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (fgk & 0x7F);
            let _ = self.set_verb_16(codec, *pin_nid, 0x300, nup);
            
            let _ = self.codec_cmd(codec, *pin_nid, verb::AJQ_, 0xC0);
            let _ = self.codec_cmd(codec, *pin_nid, verb::AJP_, 0x02);
            crate::serial_println!("[HDA]   Path pin NID {} -> amp forced gain={}, EAPD+OUT",
                pin_nid, fgk);
        }

        
        
        
        let jur: Vec<Vec<u16>> = self.output_paths.iter()
            .map(|aa| aa.path.clone())
            .collect();

        for ccc in &jur {
            let nry: Vec<(u16, WidgetType, Vec<u16>)> = ccc.iter()
                .filter_map(|&nid| self.widgets.iter().find(|w| w.nid == nid)
                    .map(|w| (nid, w.widget_type, w.connections.clone())))
                .collect();

            for (nid, _wtype, connections) in &nry {
                
                
                
                let nkc = ccc.iter()
                    .position(|&ae| ae == *nid)
                    .and_then(|pos| ccc.get(pos + 1))
                    .copied();
                if let Some(next_nid) = nkc {
                    if let Some(idx) = connections.iter().position(|&c| c == next_nid) {
                        let _ = self.codec_cmd(codec, *nid, verb::CVZ_, idx as u8);
                        crate::serial_println!("[HDA]   NID {} conn_sel={} (-> NID {})",
                            nid, idx, next_nid);
                    }
                }
            }
        }

        
        let jk = self.osd_base(0); 

        unsafe {
            
            let ajj = self.read32(jk + sd::Bf) & 0xFF;
            self.write8(jk + sd::Bf, (ajj as u8) | sctl::Fc as u8);
            Self::aas(100);
            
            for _ in 0..1000 {
                if self.read8(jk + sd::Bf) & (sctl::Fc as u8) != 0 { break; }
                Self::aas(10);
            }
            
            self.write8(jk + sd::Bf, 0);
            for _ in 0..1000 {
                if self.read8(jk + sd::Bf) & (sctl::Fc as u8) == 0 { break; }
                Self::aas(10);
            }

            
            self.write8(jk + sd::Ix, 0x1C);

            
            let fxp: u32 = 524288; 
            let cby: u32 = 2;
            let total_size = fxp * cby;

            let hfz: Vec<u8> = vec![0u8; total_size as usize + 4096];
            let keq = hfz.as_ptr() as u64;
            let kt = (keq + 0xFFF) & !0xFFF;
            core::mem::forget(hfz);

            let hg = kt.checked_sub(bz)
                .ok_or("HDA: audio buf virt->phys failed")?;

            self.audio_buf_virt = kt;
            self.audio_buf_phys = hg;
            self.audio_buf_size = total_size;

            
            core::ptr::write_bytes(kt as *mut u8, 0, total_size as usize);

            
            let hgz: Vec<u8> = vec![0u8; 256 + 4096]; 
            let kau = hgz.as_ptr() as u64;
            let bdl_virt = (kau + 127) & !127; 
            core::mem::forget(hgz);

            let bdl_phys = bdl_virt.checked_sub(bz)
                .ok_or("HDA: BDL virt->phys failed")?;

            self.bdl_virt = bdl_virt;
            self.bdl_phys = bdl_phys;

            
            let kat = bdl_virt as *mut Ahj;
            for i in 0..cby {
                let entry = &mut *kat.add(i as usize);
                entry.address = hg + (i as u64) * (fxp as u64);
                entry.length = fxp;
                entry.ioc = 1; 
            }

            
            self.write32(jk + sd::Oe, total_size);  
            self.write16(jk + sd::Pg, (cby - 1) as u16); 
            self.write16(jk + sd::Ov, fmt);  
            self.write32(jk + sd::Ob, bdl_phys as u32);
            self.write32(jk + sd::Oc, (bdl_phys >> 32) as u32);

            
            let dlu = (stream_tag as u32) << (sctl::AKF_ - 16);
            self.write8(jk + sd::Bf + 2, dlu as u8);

            crate::serial_println!("[HDA]   Stream configured: 48kHz 16-bit stereo");
            crate::serial_println!("[HDA]   Audio buf phys={:#010X} size={}",
                hg, total_size);
            crate::serial_println!("[HDA]   BDL phys={:#010X} entries={}", bdl_phys, cby);
        }

        Ok(())
    }

    
    
    

    
    
    
    
    fn reset_output_stream(&mut self) {
        if self.audio_buf_size == 0 { return; }
        let jk = self.osd_base(0);
        unsafe {
            
            let ajj = self.read8(jk + sd::Bf);
            self.write8(jk + sd::Bf, (ajj & !(sctl::Ir as u8)) | sctl::Fc as u8);
            for _ in 0..1000 {
                if self.read8(jk + sd::Bf) & (sctl::Fc as u8) != 0 { break; }
                HdaController::aas(10);
            }
            
            self.write8(jk + sd::Bf, 0);
            for _ in 0..1000 {
                if self.read8(jk + sd::Bf) & (sctl::Fc as u8) == 0 { break; }
                HdaController::aas(10);
            }
            
            self.write8(jk + sd::Ix, 0x1C);

            
            let cby: u16 = 2;
            let fmt: u16 = 0x0011; 
            self.write32(jk + sd::Oe, self.audio_buf_size);
            self.write16(jk + sd::Pg, cby - 1);
            self.write16(jk + sd::Ov, fmt);
            self.write32(jk + sd::Ob, self.bdl_phys as u32);
            self.write32(jk + sd::Oc, (self.bdl_phys >> 32) as u32);

            
            let dlu = (self.stream_tag as u32) << (sctl::AKF_ - 16);
            self.write8(jk + sd::Bf + 2, dlu as u8);
        }
        self.playing = false;
        crate::serial_println!("[HDA] Stream reset (LPIB→0, reconfig done)");
    }

    pub fn fill_tone(&mut self, freq_hz: u32, duration_ms: u32) {
        let sample_rate = 48000u32;
        let channels = 2u32;
        let hjl = 2u32; 
        let aai = (sample_rate * duration_ms / 1000) as usize;
        let kep = (self.audio_buf_size / (channels * hjl)) as usize;
        let jcl = aai.min(kep);

        let buf = self.audio_buf_virt as *mut i16;

        
        let zd = sample_rate / freq_hz;
        if zd == 0 { return; }
        let ccz = (zd / 4).max(1);
        let bqf: i32 = 16000; 

        unsafe {
            for i in 0..jcl {
                let pos = (i as u32) % zd;
                
                
                let ojz: i32 = if pos < ccz {
                    bqf * pos as i32 / ccz as i32
                } else if pos < 3 * ccz {
                    bqf * (2 * ccz as i32 - pos as i32) / ccz as i32
                } else {
                    bqf * (pos as i32 - zd as i32) / ccz as i32
                };
                
                let sample = ojz.clamp(-32000, 32000) as i16;

                
                let idx = i * channels as usize;
                *buf.add(idx) = sample;
                *buf.add(idx + 1) = sample;
            }

            
            let fwu = jcl * channels as usize * hjl as usize;
            if fwu < self.audio_buf_size as usize {
                core::ptr::write_bytes(
                    (self.audio_buf_virt as *mut u8).add(fwu),
                    0,
                    self.audio_buf_size as usize - fwu
                );
            }
        }
    }

    
    pub fn play(&mut self, start: bool) {
        let jk = self.osd_base(0);
        unsafe {
            if start {
                
                let gdb = self.read32(reg::Sx);
                let oxy = 1u32 << (self.num_iss as u32); 
                self.write32(reg::Sx, gdb | (1 << 31) | (1 << 30) | oxy);

                
                self.write8(jk + sd::Ix, 0x1C);

                
                
                let ajj = self.read8(jk + sd::Bf);
                self.write8(jk + sd::Bf, (ajj | sctl::Ir as u8) & !(sctl::Aln as u8));

                self.playing = true;
                crate::serial_println!("[HDA] Playback started");
            } else {
                
                let ajj = self.read8(jk + sd::Bf);
                self.write8(jk + sd::Bf, ajj & !(sctl::Ir as u8));

                self.playing = false;
                crate::serial_println!("[HDA] Playback stopped");
            }
        }
    }

    
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    
    pub fn stream_position(&self) -> u32 {
        let jk = self.osd_base(0);
        unsafe { self.read32(jk + sd::Aap) }
    }

    
    
    

    fn aas(us: u64) {
        
        for _ in 0..us {
            unsafe {
                let mut port: crate::arch::Port<u8> = crate::arch::Port::new(0x80);
                port.write(0);
            }
        }
    }

    
    pub fn status_info(&self) -> String {
        let mut j = String::new();
        j.push_str(&format!("Intel HDA Controller\n"));
        j.push_str(&format!("  Streams: {} out, {} in, {} bidir\n",
            self.num_oss, self.num_iss, self.num_bss));
        j.push_str(&format!("  Codecs: {:?}\n", self.codecs));
        j.push_str(&format!("  Widgets: {}\n", self.widgets.len()));
        j.push_str(&format!("  Output paths: {}\n", self.output_paths.len()));
        for (i, aa) in self.output_paths.iter().enumerate() {
            j.push_str(&format!("    [{}] {} -> path {:?}\n", i, aa.device_type, aa.path));
        }
        j.push_str(&format!("  Playing: {}\n", self.playing));
        if self.playing {
            j.push_str(&format!("  Position: {}\n", self.stream_position()));
        }
        j
    }
}






pub fn init() -> Result<(), &'static str> {
    
    let devices = crate::pci::bsp(crate::pci::class::Abd);
    let gae = devices.iter()
        .find(|d| d.subclass == 0x03) 
        .or_else(|| devices.iter().find(|d| d.subclass == 0x01)) 
        .ok_or("HDA: no Intel HDA device found on PCI bus")?
        .clone();

    let ctrl = HdaController::init(&gae)?;
    *Cd.lock() = Some(ctrl);
    AXW_.store(true, Ordering::SeqCst);

    Ok(())
}


pub fn is_initialized() -> bool {
    AXW_.load(Ordering::SeqCst)
}


pub fn ivg(freq_hz: u32, duration_ms: u32) -> Result<(), &'static str> {
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;

    
    ctrl.reset_output_stream();
    ctrl.fill_tone(freq_hz, duration_ms);

    
    let nwg = ctrl.stream_position();
    ctrl.play(true);

    
    HdaController::aas(5000); 
    let ivo = ctrl.stream_position();

    
    let sample_rate = 48000u32;
    let total_bytes = (sample_rate * duration_ms / 1000) * 4; 
    let target = total_bytes.min(ctrl.audio_buf_size);

    for _ in 0..(duration_ms * 10) {
        HdaController::aas(100);
        let pos = ctrl.stream_position();
        if pos >= target {
            break;
        }
    }

    let ivn = ctrl.stream_position();
    ctrl.play(false);

    
    crate::serial_println!("[HDA] play_tone: LPIB before={} early={} after={} target={}",
        nwg, ivo, ivn, target);
    if ivo == 0 && ivn == 0 {
        crate::serial_println!("[HDA] WARNING: LPIB never advanced! DMA may not be running.");
    }

    Ok(())
}


pub fn ooz(val: u8) -> Result<(), &'static str> {
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;
    if ctrl.codecs.is_empty() { return Err("No codecs"); }
    let codec = ctrl.codecs[0];
    let _ = ctrl.codec_cmd(codec, 1, verb::BHQ_, val);
    crate::serial_println!("[HDA] GPIO DATA set to {:#04X}", val);
    Ok(())
}


pub fn stop() -> Result<(), &'static str> {
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;
    ctrl.play(false);
    Ok(())
}


pub fn mdk() -> u32 {
    let hda = Cd.lock();
    match hda.as_ref() {
        Some(ctrl) => ctrl.stream_position(),
        None => 0,
    }
}





pub fn eyn() {
    let hda = Cd.lock();
    if let Some(ctrl) = hda.as_ref() {
        if ctrl.audio_buf_size == 0 { return; }
        let jk = ctrl.osd_base(0);
        unsafe {
            
            let ajj = ctrl.read8(jk + sd::Bf);
            ctrl.write8(jk + sd::Bf, (ajj & !(sctl::Ir as u8)) | sctl::Fc as u8);
            for _ in 0..1000 {
                if ctrl.read8(jk + sd::Bf) & (sctl::Fc as u8) != 0 { break; }
                HdaController::aas(10);
            }
            
            ctrl.write8(jk + sd::Bf, 0);
            for _ in 0..1000 {
                if ctrl.read8(jk + sd::Bf) & (sctl::Fc as u8) == 0 { break; }
                HdaController::aas(10);
            }
            
            ctrl.write8(jk + sd::Ix, 0x1C);

            
            let cby: u16 = 2;
            let fmt: u16 = 0x0011; 
            ctrl.write32(jk + sd::Oe, ctrl.audio_buf_size);
            ctrl.write16(jk + sd::Pg, cby - 1);
            ctrl.write16(jk + sd::Ov, fmt);
            ctrl.write32(jk + sd::Ob, ctrl.bdl_phys as u32);
            ctrl.write32(jk + sd::Oc, (ctrl.bdl_phys >> 32) as u32);

            
            let dlu = (ctrl.stream_tag as u32) << (sctl::AKF_ - 16);
            ctrl.write8(jk + sd::Bf + 2, dlu as u8);
        }
        crate::serial_println!("[HDA] Stream reset (LPIB→0, reconfig done)");
    }
}






pub fn bdu(jo: &[i16]) -> Result<(), &'static str> {
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;

    
    if ctrl.playing {
        ctrl.play(false);
    }

    
    ctrl.reset_output_stream();

    
    if ctrl.audio_buf_virt == 0 || ctrl.audio_buf_size == 0 {
        return Err("HDA: DMA buffer not initialized");
    }

    
    let buf = ctrl.audio_buf_virt as *mut i16;
    let cgn = (ctrl.audio_buf_size / 2) as usize;
    let od = jo.len().min(cgn);

    unsafe {
        core::ptr::copy_nonoverlapping(jo.as_ptr(), buf, od);
        
        if od < cgn {
            core::ptr::write_bytes(buf.add(od), 0, cgn - od);
        }
    }

    let hqo = (od * 2) as u32;
    crate::serial_println!("[HDA] Looped playback: {} bytes ({} ms), buf={}",
        hqo, hqo / (48000 * 4 / 1000), ctrl.audio_buf_size);

    
    
    
    ctrl.play(true);
    Ok(())
}




pub fn cym() -> Option<(*mut i16, usize)> {
    let hda = Cd.lock();
    let ctrl = hda.as_ref()?;
    if ctrl.audio_buf_virt == 0 { return None; }
    Some((ctrl.audio_buf_virt as *mut i16, (ctrl.audio_buf_size / 2) as usize))
}


pub fn is_playing() -> bool {
    let hda = Cd.lock();
    match hda.as_ref() {
        Some(ctrl) => ctrl.is_playing(),
        None => false,
    }
}


pub fn dqq() -> u32 {
    let hda = Cd.lock();
    match hda.as_ref() {
        Some(ctrl) => ctrl.stream_position(),
        None => 0,
    }
}



pub fn owe() -> Result<(), &'static str> {
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;
    if !ctrl.playing {
        ctrl.play(true);
    }
    Ok(())
}




pub fn hli() {
    let hda = Cd.lock();
    if let Some(ctrl) = hda.as_ref() {
        let jk = ctrl.osd_base(0);
        unsafe {
            
            ctrl.write8(jk + sd::Ix, 0x1C);
        }
    }
}




pub fn hwa() -> bool {
    let hda = Cd.lock();
    if let Some(ctrl) = hda.as_ref() {
        if !ctrl.playing { return false; }
        let jk = ctrl.osd_base(0);
        unsafe {
            let ajj = ctrl.read8(jk + sd::Bf);
            if ajj & (sctl::Ir as u8) == 0 {
                
                ctrl.write8(jk + sd::Ix, 0x1C);
                ctrl.write8(jk + sd::Bf, ajj | sctl::Ir as u8);
                crate::serial_println!("[HDA] Stream stalled — restarted (LPIB={})",
                    ctrl.stream_position());
                return true;
            }
        }
    }
    false
}


pub fn status() -> String {
    let hda = Cd.lock();
    match hda.as_ref() {
        Some(ctrl) => ctrl.status_info(),
        None => String::from("HDA: not initialized"),
    }
}


pub fn cwm() -> String {
    let mut j = String::new();
    let mut hda = Cd.lock();
    let ctrl = match hda.as_mut() {
        Some(c) => c,
        None => {
            j.push_str("HDA: not initialized\n");
            return j;
        }
    };

    unsafe {
        
        let agk = ctrl.read16(reg::Yz);
        let gctl = ctrl.read32(reg::Ef);
        let gdb = ctrl.read32(reg::Sx);
        let mrb = ctrl.read32(0x24); 
        let ovs = ctrl.read32(reg::Aej);
        let pto = ctrl.read32(reg::Arw);
        let bdv = ctrl.read16(reg::Ux);

        j.push_str(&format!("=== HDA Hardware Diagnostic ===\n"));
        j.push_str(&format!("GCAP={:#06X} GCTL={:#010X}\n", agk, gctl));
        j.push_str(&format!("INTCTL={:#010X} INTSTS={:#010X}\n", gdb, mrb));
        j.push_str(&format!("SSYNC={:#010X} WALCLK={}\n", ovs, pto));
        j.push_str(&format!("STATESTS={:#06X}\n", bdv));
        j.push_str(&format!("CRST={} UNSOL={}\n",
            if gctl & gctl::Hi != 0 { "OK" } else { "IN RESET!" },
            if gctl & gctl::Afr != 0 { "on" } else { "off" }));

        
        let jk = ctrl.osd_base(0);
        let fpg = ctrl.read8(jk + sd::Bf);
        let hpe = ctrl.read8(jk + sd::Bf + 2);
        let eao = ctrl.read8(jk + sd::Ix);
        let alw = ctrl.read32(jk + sd::Aap);
        let khs = ctrl.read32(jk + sd::Oe);
        let nbh = ctrl.read16(jk + sd::Pg);
        let lvb = ctrl.read16(jk + sd::Ajt);
        let fmt = ctrl.read16(jk + sd::Ov);
        let kav = ctrl.read32(jk + sd::Ob);
        let kaw = ctrl.read32(jk + sd::Oc);

        j.push_str(&format!("\n--- Output Stream 0 (base={:#X}) ---\n", jk));
        j.push_str(&format!("CTL[0]={:#04X} CTL[2]={:#04X} (RUN={} SRST={} TAG={})\n",
            fpg, hpe,
            if fpg & sctl::Ir as u8 != 0 { "YES" } else { "no" },
            if fpg & sctl::Fc as u8 != 0 { "YES!" } else { "no" },
            hpe >> 4));
        j.push_str(&format!("STS={:#04X} (BCIS={} FIFOE={} DESE={} FIFORDY={})\n",
            eao,
            if eao & ssts::Ahf != 0 { "Y" } else { "n" },
            if eao & ssts::Ajr != 0 { "ERR" } else { "ok" },
            if eao & ssts::Aif != 0 { "ERR" } else { "ok" },
            if eao & ssts::Ajs != 0 { "Y" } else { "n" }));
        j.push_str(&format!("LPIB={} CBL={} LVI={} FIFOS={}\n", alw, khs, nbh, lvb));
        j.push_str(&format!("FMT={:#06X} (48kHz/16bit/stereo=0x0011)\n", fmt));
        j.push_str(&format!("BDL={:#010X}:{:#010X}\n", kaw, kav));
        j.push_str(&format!("Audio buf phys={:#010X} size={}\n", ctrl.audio_buf_phys, ctrl.audio_buf_size));

        
        if !ctrl.codecs.is_empty() && !ctrl.output_paths.is_empty() {
            let codec = ctrl.codecs[0];
            let path = ctrl.output_paths[0].clone();
            j.push_str(&format!("\n--- Codec {} Path ---\n", codec));
            j.push_str(&format!("Path: {:?} Type={}\n", path.path, path.device_type));

            
            for &nid in &path.path {
                if let Ok(exe) = ctrl.codec_cmd(codec, nid, verb::UO_, 0) {
                    let bxh = exe & 0xF;
                    let target = (exe >> 4) & 0xF;
                    j.push_str(&format!("  NID {}: power D{}/D{}{}\n",
                        nid, bxh, target,
                        if bxh != 0 { " NOT D0!" } else { "" }));
                }
            }

            
            if let Ok(pc) = ctrl.codec_cmd(codec, path.pin_nid, verb::AUZ_, 0) {
                j.push_str(&format!("  Pin {} PIN_CTL={:#04X} (out={})\n",
                    path.pin_nid, pc, if pc & 0x40 != 0 { "YES" } else { "NO!" }));
            }
            if let Ok(ea) = ctrl.codec_cmd(codec, path.pin_nid, verb::AUY_, 0) {
                j.push_str(&format!("  Pin {} EAPD={:#04X} (on={})\n",
                    path.pin_nid, ea, if ea & 0x02 != 0 { "YES" } else { "NO!" }));
            }

            
            if let Ok(dr) = ctrl.codec_cmd(codec, path.dac_nid, verb::AUX_, 0) {
                let tag = (dr >> 4) & 0xF;
                j.push_str(&format!("  DAC {} STREAM_TAG={} (expect {})\n",
                    path.dac_nid, tag, ctrl.stream_tag));
            }
        }
    }
    j
}



pub fn kuy() -> String {
    let mut j = String::new();
    let mut hda = Cd.lock();
    let ctrl = match hda.as_mut() {
        Some(c) => c,
        None => {
            j.push_str("HDA: not initialized\n");
            return j;
        }
    };

    if ctrl.codecs.is_empty() {
        j.push_str("No codecs found\n");
        return j;
    }

    let codec = ctrl.codecs[0];

    
    if let Ok(bpx) = ctrl.codec_cmd(codec, 0, verb::EW_, verb::AIC_ as u8) {
        j.push_str(&format!("Codec {}: vendor={:#06X} device={:#06X}\n",
            codec, (bpx >> 16) & 0xFFFF, bpx & 0xFFFF));
    }

    j.push_str(&format!("Widgets: {} discovered\n", ctrl.widgets.len()));
    j.push_str(&format!("Output paths: {}\n", ctrl.output_paths.len()));
    for (i, aa) in ctrl.output_paths.iter().enumerate() {
        j.push_str(&format!("  Path[{}]: pin={} dac={} type={} route={:?}\n",
            i, aa.pin_nid, aa.dac_nid, aa.device_type, aa.path));
    }

    
    let mfn = ctrl.codec_cmd(codec, 1, verb::CAC_, 0).unwrap_or(0);
    let mfp = ctrl.codec_cmd(codec, 1, verb::CAE_, 0).unwrap_or(0);
    let mfo  = ctrl.codec_cmd(codec, 1, verb::CAD_,  0).unwrap_or(0);
    let mfm = ctrl.get_param(codec, 1, verb::CLZ_).unwrap_or(0);
    j.push_str(&format!("GPIO: count={} mask={:#04X} dir={:#04X} data={:#04X}\n",
        mfm & 0xFF, mfp, mfo, mfn));

    
    j.push_str(&format!("\n--- Pin Widgets ---\n"));
    let nur: Vec<(u16, u32)> = ctrl.widgets.iter()
        .filter(|w| w.widget_type == WidgetType::PinComplex)
        .map(|w| (w.nid, w.pin_config))
        .collect();

    for (nid, cfg) in &nur {
        let s = gnb(*cfg);
        let dli = match (*cfg >> 30) & 0x3 {
            0 => "Jack",
            1 => "None",
            2 => "Fixed",
            3 => "Both",
            _ => "?",
        };
        let axx = (*cfg >> 24) & 0x3F;

        
        let gna = ctrl.codec_cmd(codec, *nid, verb::AUZ_, 0).unwrap_or(0);
        let hus = ctrl.codec_cmd(codec, *nid, verb::AUY_, 0).unwrap_or(0);
        let dwt = ctrl.codec_cmd(codec, *nid, verb::UO_, 0).unwrap_or(0);
        let ast = ctrl.codec_cmd(codec, *nid, verb::EW_, verb::PS_ as u8).unwrap_or(0);
        
        let dho = ctrl.set_verb_16(codec, *nid, verb::DQ_, 0xA000).unwrap_or(0); 
        let dhp = ctrl.set_verb_16(codec, *nid, verb::DQ_, 0x8000).unwrap_or(0); 
        let gmz = ctrl.codec_cmd(codec, *nid, verb::EW_, verb::CMA_ as u8).unwrap_or(0);
        
        let pum = ctrl.widgets.iter().find(|w| w.nid == *nid).map(|w| w.amp_out_caps).unwrap_or(0);

        let nog = gna & 0x40 != 0;
        let mmk = gna & 0x80 != 0;
        let lnd = hus & 0x02 != 0;
        let mji = gmz & (1 << 16) != 0;
        let qkp = gmz & (1 << 4) != 0;
        let bmr = ast & (1 << 2) != 0;
        let mjb = ast & (1 << 3) != 0;

        j.push_str(&format!("  NID {:2}: {} ({}) loc={:#04X} cfg={:#010X}\n",
            nid, s, dli, axx, cfg));
        j.push_str(&format!("         wcaps={:#010X}(out_amp={} amp_ovrd={}) pin_caps={:#010X}\n",
            ast, bmr, mjb, gmz));
        j.push_str(&format!("         pin_ctl={:#04X}(out={} hp={}) eapd={:#04X}(on={} has={})\n",
            gna, nog, mmk, hus, lnd, mji));
        j.push_str(&format!("         power=D{} amp_out L={:#04X} R={:#04X} amp_caps={:#010X}\n",
            dwt & 0xF, dho, dhp, pum));
    }

    
    j.push_str(&format!("\n--- DAC Widgets ---\n"));
    let lbj: Vec<u16> = ctrl.widgets.iter()
        .filter(|w| w.widget_type == WidgetType::AudioOutput)
        .map(|w| w.nid)
        .collect();

    for nid in &lbj {
        let dwt = ctrl.codec_cmd(codec, *nid, verb::UO_, 0).unwrap_or(0);
        let jja = ctrl.codec_cmd(codec, *nid, verb::AUX_, 0).unwrap_or(0);
        
        let fmt = ctrl.set_verb_16(codec, *nid, verb::CAH_, 0).unwrap_or(0);
        let ast = ctrl.codec_cmd(codec, *nid, verb::EW_, verb::PS_ as u8).unwrap_or(0);
        let bmr = ast & (1 << 2) != 0;
        let ckj = ast & (1 << 1) != 0;
        
        let dho = ctrl.set_verb_16(codec, *nid, verb::DQ_, 0xA000).unwrap_or(0); 
        let dhp = ctrl.set_verb_16(codec, *nid, verb::DQ_, 0x8000).unwrap_or(0); 
        
        let fha = if bmr { ctrl.codec_cmd(codec, *nid, verb::EW_, verb::PR_ as u8).unwrap_or(0) } else { 0 };

        j.push_str(&format!("  NID {:2}: power=D{} stream_tag={} chan={} fmt={:#06X}\n",
            nid, dwt & 0xF, (jja >> 4) & 0xF, jja & 0xF, fmt));
        j.push_str(&format!("         wcaps={:#010X}(out_amp={} in_amp={})\n",
            ast, bmr, ckj));
        j.push_str(&format!("         amp_out L={:#04X} R={:#04X} caps={:#010X}\n",
            dho, dhp, fha));
    }

    
    j.push_str(&format!("\n--- Path Mixer/Selector ---\n"));
    let mut jeg: Vec<u16> = Vec::new();
    let nsa: Vec<Vec<u16>> = ctrl.output_paths.iter().map(|aa| aa.path.clone()).collect();
    let pun: Vec<(u16, WidgetType, Vec<u16>)> = ctrl.widgets.iter()
        .map(|w| (w.nid, w.widget_type, w.connections.clone()))
        .collect();
    for path in &nsa {
        for &nid in path {
            let puv = pun.iter().find(|w| w.0 == nid);
            if let Some((_, wt, nc)) = puv {
                if *wt != WidgetType::AudioMixer && *wt != WidgetType::AudioSelector { continue; }
                if jeg.contains(&nid) { continue; }
                jeg.push(nid);
                let ast = ctrl.codec_cmd(codec, nid, verb::EW_, verb::PS_ as u8).unwrap_or(0);
                let bmr = ast & (1 << 2) != 0;
                let ckj = ast & (1 << 1) != 0;
                
                let dho = ctrl.set_verb_16(codec, nid, verb::DQ_, 0xA000).unwrap_or(0);
                let dhp = ctrl.set_verb_16(codec, nid, verb::DQ_, 0x8000).unwrap_or(0);
                let jvn = ctrl.set_verb_16(codec, nid, verb::DQ_, 0x2000).unwrap_or(0);
                let jvo = ctrl.set_verb_16(codec, nid, verb::DQ_, 0x0000).unwrap_or(0);
                let fha = if bmr { ctrl.codec_cmd(codec, nid, verb::EW_, verb::PR_ as u8).unwrap_or(0) } else { 0 };
                let jvl = if ckj { ctrl.codec_cmd(codec, nid, verb::EW_, verb::XK_ as u8).unwrap_or(0) } else { 0 };
                let kxa = ctrl.codec_cmd(codec, nid, verb::CAB_, 0).unwrap_or(0);
                let dwt = ctrl.codec_cmd(codec, nid, verb::UO_, 0).unwrap_or(0);

                j.push_str(&format!("  NID {:2}: {} conns={:?}\n", nid, wt.name(), nc));
                j.push_str(&format!("         wcaps={:#010X}(out_amp={} in_amp={})\n",
                    ast, bmr, ckj));
                j.push_str(&format!("         out L={:#04X} R={:#04X} ocaps={:#010X}\n",
                    dho, dhp, fha));
                j.push_str(&format!("         in[0] L={:#04X} R={:#04X} icaps={:#010X}\n",
                    jvn, jvo, jvl));
                j.push_str(&format!("         conn_sel={} power=D{}\n", kxa, dwt & 0xF));
            }
        }
    }

    j
}




pub fn jvp() -> String {
    let mut j = String::new();
    let mut hda = Cd.lock();
    let ctrl = match hda.as_mut() {
        Some(c) => c,
        None => {
            j.push_str("HDA: not initialized\n");
            return j;
        }
    };

    if ctrl.codecs.is_empty() || ctrl.output_paths.is_empty() {
        j.push_str("No codecs or paths\n");
        return j;
    }

    let codec = ctrl.codecs[0];
    j.push_str("=== Amp Probe (SET then GET) ===\n");

    
    let mut iwt: Vec<u16> = Vec::new();
    let nrz: Vec<Vec<u16>> = ctrl.output_paths.iter().map(|aa| aa.path.clone()).collect();
    for path in &nrz {
        for &nid in path {
            if iwt.contains(&nid) { continue; }
            iwt.push(nid);

            let ast = ctrl.codec_cmd(codec, nid, verb::EW_, verb::PS_ as u8).unwrap_or(0);
            let wt = (ast >> 20) & 0xF;
            let bmr = ast & (1 << 2) != 0;
            let ckj = ast & (1 << 1) != 0;

            
            let hhf = ctrl.set_verb_16(codec, nid, verb::DQ_, 0xA000).unwrap_or(0xDEAD); 
            let hhe = ctrl.set_verb_16(codec, nid, verb::DQ_, 0x2000).unwrap_or(0xDEAD);  

            
            let glg = if bmr { ctrl.get_param(codec, nid, verb::PR_).unwrap_or(0) } else { 0 };
            let gcd = if ckj { ctrl.get_param(codec, nid, verb::XK_).unwrap_or(0) } else { 0 };
            let dcc = ((glg >> 8) & 0x7F) as u16;
            let czq = ((gcd >> 8) & 0x7F) as u16;
            let evu = if dcc > 0 { dcc } else { 0x1F };
            let drv = if czq > 0 { czq } else { 0x1F };

            
            let opg: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (evu & 0x7F);
            let nol = ctrl.set_verb_16(codec, nid, 0x300, opg);
            
            let opb: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (drv & 0x7F);
            let mol = ctrl.set_verb_16(codec, nid, 0x300, opb);

            
            let hei = ctrl.set_verb_16(codec, nid, verb::DQ_, 0xA000).unwrap_or(0xDEAD);
            let heh = ctrl.set_verb_16(codec, nid, verb::DQ_, 0x2000).unwrap_or(0xDEAD);

            let kim = hhf != hei;
            let kil = hhe != heh;

            j.push_str(&format!("NID {:2} type={} oamp={}({}) iamp={}({})\n",
                nid, wt, bmr, dcc, ckj, czq));
            j.push_str(&format!("  OUT: {:#04X}->{:#04X} {} set={} gain={}\n",
                hhf, hei,
                if kim { "CHANGED" } else { "same" },
                if nol.is_ok() { "ok" } else { "ERR" },
                evu));
            j.push_str(&format!("  IN:  {:#04X}->{:#04X} {} set={} gain={}\n",
                hhe, heh,
                if kil { "CHANGED" } else { "same" },
                if mol.is_ok() { "ok" } else { "ERR" },
                drv));
        }
    }

    j
}



pub fn bxb(jo: &[i16], duration_ms: u32) -> Result<(), &'static str> {
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;

    
    if ctrl.playing {
        ctrl.play(false);
    }

    
    ctrl.reset_output_stream();

    let buf = ctrl.audio_buf_virt as *mut i16;
    let cgn = (ctrl.audio_buf_size / 2) as usize; 

    let od = jo.len().min(cgn);

    unsafe {
        
        core::ptr::copy_nonoverlapping(jo.as_ptr(), buf, od);

        
        if od < cgn {
            core::ptr::write_bytes(buf.add(od), 0, cgn - od);
        }
    }

    
    ctrl.play(true);

    
    let total_bytes = (od * 2) as u32; 
    let target = total_bytes.min(ctrl.audio_buf_size);

    for _ in 0..(duration_ms * 10 + 500) {
        HdaController::aas(100);
        let pos = ctrl.stream_position();
        if pos >= target {
            break;
        }
    }

    ctrl.play(false);
    Ok(())
}






static Nu: Mutex<u8> = Mutex::new(80);


pub fn set_volume(level: u8) -> Result<(), &'static str> {
    let level = level.min(100);
    *Nu.lock() = level;
    
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;
    
    if ctrl.codecs.is_empty() || ctrl.output_paths.is_empty() {
        return Ok(());
    }
    
    let codec = ctrl.codecs[0];
    let path = ctrl.output_paths[0].clone();
    
    
    
    let nrx = path.dac_nid;
    let eua = ctrl.widgets.iter()
        .find(|w| w.nid == nrx)
        .map(|w| ((w.amp_out_caps >> 8) & 0x7F) as u16)
        .unwrap_or(39);
    let eua = if eua == 0 { 39 } else { eua };
    let bmi = ((level as u32) * (eua as u32) / 100) as u16;
    
    
    for &nid in &path.path {
        
        let fhb: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (bmi & 0x7F);
        let _ = ctrl.set_verb_16(codec, nid, 0x300, fhb);
        
        let fgz: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (bmi & 0x7F);
        let _ = ctrl.set_verb_16(codec, nid, 0x300, fgz);
    }
    
    crate::serial_println!("[HDA] Volume set to {}% (gain={})", level, bmi);
    Ok(())
}


pub fn ica() -> u8 {
    *Nu.lock()
}


pub fn qpf() -> Result<(), &'static str> {
    let mut hda = Cd.lock();
    let ctrl = hda.as_mut().ok_or("HDA: not initialized")?;
    
    if ctrl.codecs.is_empty() || ctrl.output_paths.is_empty() {
        return Ok(());
    }
    
    let codec = ctrl.codecs[0];
    let path = ctrl.output_paths[0].clone();
    
    for &nid in &path.path {
        
        let nhg: u16 = (1 << 15) | (1 << 13) | (1 << 12) | (1 << 7);
        let _ = ctrl.set_verb_16(codec, nid, 0x300, nhg);
        
        let nhf: u16 = (1 << 14) | (1 << 13) | (1 << 12) | (1 << 7);
        let _ = ctrl.set_verb_16(codec, nid, 0x300, nhf);
    }
    
    Ok(())
}


pub fn rbk() -> Result<(), &'static str> {
    let level = *Nu.lock();
    set_volume(level)
}







pub fn cyi(freq_hz: u32, duration_ms: u32, bqf: i16) -> Vec<i16> {
    let sample_rate = 48000u32;
    let cbz = (sample_rate as u64 * duration_ms as u64 / 1000) as usize;
    let mut jo = Vec::with_capacity(cbz * 2);
    
    let vd = *Nu.lock() as i32;
    let oku = (bqf as i32 * vd / 100) as i16;
    
    for i in 0..cbz {
        
        let nue = ((freq_hz as u64 * i as u64 * 256) / sample_rate as u64) as u32;
        let nuc = (nue & 0xFF) as u8;
        
        let sample = otg(nuc, oku);
        jo.push(sample); 
        jo.push(sample); 
    }
    
    
    let emb = (sample_rate as usize * 5 / 1000).min(cbz / 2);
    for i in 0..emb {
        let ha = i as i32 * 256 / emb as i32;
        jo[i * 2] = (jo[i * 2] as i32 * ha / 256) as i16;
        jo[i * 2 + 1] = (jo[i * 2 + 1] as i32 * ha / 256) as i16;
    }
    for i in 0..emb {
        let idx = cbz - 1 - i;
        let ha = i as i32 * 256 / emb as i32;
        if idx * 2 + 1 < jo.len() {
            jo[idx * 2] = (jo[idx * 2] as i32 * ha / 256) as i16;
            jo[idx * 2 + 1] = (jo[idx * 2 + 1] as i32 * ha / 256) as i16;
        }
    }
    
    jo
}



fn otg(phase: u8, bqf: i16) -> i16 {
    let x = phase as i32;
    
    let mhe = if x < 128 {
        let t = x - 64; 
        let dm = -(t * t) + 64 * 64;
        dm * 127 / (64 * 64)
    } else {
        let t = (x - 128) - 64;
        let dm = (t * t) - 64 * 64;
        dm * 127 / (64 * 64)
    };
    
    (mhe as i32 * bqf as i32 / 127) as i16
}


pub fn nvn(freq_hz: u32, duration_ms: u32) -> Result<(), &'static str> {
    let jo = cyi(freq_hz, duration_ms, 24000);
    bxb(&jo, duration_ms)
}






#[derive(Debug, Clone)]
pub struct Agr {
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub data_offset: usize,
    pub data_size: usize,
}


pub fn ewj(data: &[u8]) -> Result<Agr, &'static str> {
    if data.len() < 44 { return Err("WAV: too short"); }
    if &data[0..4] != b"RIFF" { return Err("WAV: missing RIFF"); }
    if &data[8..12] != b"WAVE" { return Err("WAV: missing WAVE"); }
    
    let mut offset = 12;
    let mut channels = 0u16;
    let mut sample_rate = 0u32;
    let mut bits_per_sample = 0u16;
    let mut data_offset = 0usize;
    let mut data_size = 0usize;
    
    while offset + 8 <= data.len() {
        let hkx = &data[offset..offset+4];
        let rs = u32::from_le_bytes([
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]
        ]) as usize;
        
        if hkx == b"fmt " && rs >= 16 {
            let jyc = u16::from_le_bytes([data[offset+8], data[offset+9]]);
            if jyc != 1 { return Err("WAV: not PCM format"); }
            channels = u16::from_le_bytes([data[offset+10], data[offset+11]]);
            sample_rate = u32::from_le_bytes([
                data[offset+12], data[offset+13], data[offset+14], data[offset+15]
            ]);
            bits_per_sample = u16::from_le_bytes([data[offset+22], data[offset+23]]);
        } else if hkx == b"data" {
            data_offset = offset + 8;
            data_size = rs.min(data.len() - data_offset);
            break;
        }
        
        offset += 8 + rs;
        if offset % 2 != 0 { offset += 1; } 
    }
    
    if data_offset == 0 || channels == 0 {
        return Err("WAV: missing fmt or data chunk");
    }
    
    Ok(Agr { channels, sample_rate, bits_per_sample, data_offset, data_size })
}


pub fn qqp(data: &[u8]) -> Result<(), &'static str> {
    let info = ewj(data)?;
    
    if info.bits_per_sample != 16 {
        return Err("WAV: only 16-bit PCM supported");
    }
    
    let dcl = &data[info.data_offset..info.data_offset + info.data_size];
    let dvq = info.data_size / (2 * info.channels as usize);
    
    let dfi = 48000u32;
    let dbw = (dvq as u64 * dfi as u64
        / info.sample_rate as u64) as usize;
    let mut output = Vec::with_capacity(dbw * 2);
    
    let vd = *Nu.lock() as i32;
    
    for dst_frame in 0..dbw {
        let eaf = (dst_frame as u64 * info.sample_rate as u64
            / dfi as u64) as usize;
        
        if eaf >= dvq { break; }
        
        let idx = eaf * info.channels as usize;
        let yk = idx * 2;
        
        let left = if yk + 1 < dcl.len() {
            i16::from_le_bytes([dcl[yk], dcl[yk + 1]])
        } else { 0 };
        
        let right = if info.channels >= 2 {
            let fkg = (idx + 1) * 2;
            if fkg + 1 < dcl.len() {
                i16::from_le_bytes([dcl[fkg], dcl[fkg + 1]])
            } else { left }
        } else { left };
        
        output.push((left as i32 * vd / 100) as i16);
        output.push((right as i32 * vd / 100) as i16);
    }
    
    let duration_ms = (dbw as u64 * 1000 / dfi as u64) as u32;
    bxb(&output, duration_ms + 100)
}






#[derive(Clone, Copy, Debug)]
pub enum SoundEffect {
    
    BootChime,
    
    Click,
    
    Error,
    
    Notification,
    
    Warning,
    
    Success,
    
    Keypress,
}


pub fn nvi(aoa: SoundEffect) -> Result<(), &'static str> {
    let plc: Vec<(u32, u32, i16)> = match aoa {
        SoundEffect::BootChime => vec![
            (523, 150, 20000),  
            (659, 150, 20000),  
            (784, 250, 22000),  
        ],
        SoundEffect::Click => vec![(1000, 15, 16000)],
        SoundEffect::Error => vec![
            (400, 120, 22000),
            (0, 60, 0),    
            (400, 120, 22000),
        ],
        SoundEffect::Notification => vec![
            (880, 100, 18000),   
            (1109, 100, 18000),  
            (1319, 200, 20000),  
        ],
        SoundEffect::Warning => vec![
            (880, 200, 20000),
            (660, 300, 18000),
        ],
        SoundEffect::Success => vec![
            (523, 100, 18000),  
            (659, 200, 20000),  
        ],
        SoundEffect::Keypress => vec![(2000, 8, 8000)],
    };
    
    let mut cfu: Vec<i16> = Vec::new();
    let mut total_ms = 0u32;
    
    for (freq, dur_ms, ank) in &plc {
        if *freq == 0 {
            let ost = (48000u32 * *dur_ms / 1000) as usize;
            cfu.extend(core::iter::repeat(0i16).take(ost * 2));
        } else {
            let gzj = cyi(*freq, *dur_ms, *ank);
            cfu.extend_from_slice(&gzj);
        }
        total_ms += dur_ms;
    }
    
    bxb(&cfu, total_ms + 50)
}






#[derive(Clone, Copy, Debug)]
pub struct Note {
    
    pub midi_note: u8,
    
    pub duration_16th: u8,
    
    pub velocity: u8,
}

impl Note {
    pub fn new(aad: u8, blz: u8, anb: u8) -> Self {
        Self { midi_note: aad, duration_16th: blz, velocity: anb }
    }
    
    pub fn ef(blz: u8) -> Self {
        Self { midi_note: 0, duration_16th: blz, velocity: 0 }
    }
    
    
    
    pub fn freq_hz(&self) -> u32 {
        if self.midi_note == 0 { return 0; }
        let faa = self.midi_note as i32 - 69;
        let gki = faa.div_euclid(12);
        let dee = faa.rem_euclid(12) as usize;
        
        
        const CVT_: [u32; 12] = [
            1000, 1059, 1122, 1189, 1260, 1335, 1414, 1498, 1587, 1682, 1782, 1888
        ];
        
        let hgw = CVT_[dee] * 440 / 1000;
        
        if gki >= 0 {
            hgw << gki as u32
        } else {
            hgw >> (-gki) as u32
        }
    }
}


pub fn nvm(notes: &[Note], bpm: u32) -> Result<(), &'static str> {
    if notes.is_empty() { return Ok(()); }
    
    let oti = 60_000 / (bpm * 4);
    
    let mut cfu: Vec<i16> = Vec::new();
    let mut total_ms = 0u32;
    
    for note in notes {
        let dur_ms = oti * note.duration_16th as u32;
        let freq = note.freq_hz();
        
        if freq == 0 || note.velocity == 0 {
            let fba = (48000u32 * dur_ms / 1000) as usize;
            cfu.extend(core::iter::repeat(0i16).take(fba * 2));
        } else {
            let ank = (note.velocity as i32 * 24000 / 127) as i16;
            let gzj = cyi(freq, dur_ms, ank);
            cfu.extend_from_slice(&gzj);
        }
        total_ms += dur_ms;
    }
    
    bxb(&cfu, total_ms + 50)
}










pub fn nvl(bcq: &str, bpm: u32) -> Result<(), &'static str> {
    let mut notes = Vec::new();
    
    for abm in bcq.split_whitespace() {
        if abm.is_empty() { continue; }
        
        let bytes = abm.as_bytes();
        if bytes[0] == b'R' || bytes[0] == b'r' {
            notes.push(Note::ef(its(&bytes[1..])));
            continue;
        }
        
        let (note_base, ef) = nqs(bytes);
        if note_base == 255 { continue; }
        
        let (octave, rest2) = if !ef.is_empty() && ef[0] >= b'0' && ef[0] <= b'9' {
            (ef[0] - b'0', &ef[1..])
        } else {
            (4, ef)
        };
        
        let blz = its(rest2);
        let aad = 12 * (octave + 1) + note_base;
        notes.push(Note::new(aad, blz, 100));
    }
    
    nvm(&notes, bpm)
}


fn nqs(bytes: &[u8]) -> (u8, &[u8]) {
    if bytes.is_empty() { return (255, bytes); }
    
    let base = match bytes[0] {
        b'C' | b'c' => 0,
        b'D' | b'd' => 2,
        b'E' | b'e' => 4,
        b'F' | b'f' => 5,
        b'G' | b'g' => 7,
        b'A' | b'a' => 9,
        b'B' | b'b' => 11,
        _ => return (255, bytes),
    };
    
    if bytes.len() > 1 && bytes[1] == b'#' {
        return ((base + 1) % 12, &bytes[2..]);
    }
    
    (base, &bytes[1..])
}


fn its(bytes: &[u8]) -> u8 {
    if bytes.is_empty() { return 4; }
    match bytes[0] {
        b'w' => 16,
        b'h' => 8,
        b'q' => 4,
        b'e' => 2,
        b's' => 1,
        _ => 4,
    }
}


pub fn nvh() -> Result<(), &'static str> {
    nvl("E4q E4q F4q G4q G4q F4q E4q D4q C4q C4q D4q E4q E4q D4h", 120)
}
