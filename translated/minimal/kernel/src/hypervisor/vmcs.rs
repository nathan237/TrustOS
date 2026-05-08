









use super::{HypervisorError, Result};
use super::vmx::{self, hbu, jqk, edv, jql};
use alloc::boxed::Box;


pub mod fields {
    
    pub const Ars: u64 = 0x0000;
    
    
    pub const AWQ_: u64 = 0x0800;
    pub const AWI_: u64 = 0x0802;
    pub const AXN_: u64 = 0x0804;
    pub const AWM_: u64 = 0x0806;
    pub const AWU_: u64 = 0x0808;
    pub const AXA_: u64 = 0x080A;
    pub const AXH_: u64 = 0x080C;
    pub const AXR_: u64 = 0x080E;
    
    
    pub const CDD_: u64 = 0x0C00;
    pub const CDB_: u64 = 0x0C02;
    pub const CDP_: u64 = 0x0C04;
    pub const CDC_: u64 = 0x0C06;
    pub const CDF_: u64 = 0x0C08;
    pub const CDI_: u64 = 0x0C0A;
    pub const CDU_: u64 = 0x0C0C;
    
    
    pub const DVF_: u64 = 0x2000;
    pub const DVG_: u64 = 0x2002;
    pub const DYP_: u64 = 0x2004;
    pub const BXD_: u64 = 0x201A;
    
    
    pub const DEP_: u64 = 0x2800;
    pub const CBR_: u64 = 0x2802;
    pub const CBS_: u64 = 0x2804;
    pub const AXB_: u64 = 0x2806;
    
    
    pub const CDK_: u64 = 0x2C00;
    pub const CDJ_: u64 = 0x2C02;
    
    
    pub const CNA_: u64 = 0x4000;
    pub const BRO_: u64 = 0x4002;
    pub const BXK_: u64 = 0x4004;
    pub const ECE_: u64 = 0x4006;
    pub const ECF_: u64 = 0x4008;
    pub const BSF_: u64 = 0x400A;
    pub const DEU_: u64 = 0x400C;
    pub const DEX_: u64 = 0x400E;
    pub const DEW_: u64 = 0x4010;
    pub const DER_: u64 = 0x4012;
    pub const DET_: u64 = 0x4014;
    pub const DES_: u64 = 0x4016;
    pub const ENQ_: u64 = 0x4018;
    pub const ENR_: u64 = 0x401A;
    pub const CVR_: u64 = 0x401E;
    
    
    pub const AWP_: u64 = 0x4800;
    pub const AWH_: u64 = 0x4802;
    pub const AXM_: u64 = 0x4804;
    pub const AWL_: u64 = 0x4806;
    pub const AWT_: u64 = 0x4808;
    pub const AWZ_: u64 = 0x480A;
    pub const AXG_: u64 = 0x480C;
    pub const AXQ_: u64 = 0x480E;
    pub const AWW_: u64 = 0x4810;
    pub const AXD_: u64 = 0x4812;
    pub const AWN_: u64 = 0x4814;
    pub const AWF_: u64 = 0x4816;
    pub const AXK_: u64 = 0x4818;
    pub const AWJ_: u64 = 0x481A;
    pub const AWR_: u64 = 0x481C;
    pub const AWX_: u64 = 0x481E;
    pub const AXE_: u64 = 0x4820;
    pub const AXO_: u64 = 0x4822;
    pub const CBU_: u64 = 0x4824;
    pub const CBP_: u64 = 0x4826;
    pub const CCA_: u64 = 0x482A;
    
    
    pub const CDQ_: u64 = 0x4C00;
    
    
    pub const BSD_: u64 = 0x6000;
    pub const BSG_: u64 = 0x6002;
    pub const BSE_: u64 = 0x6004;
    pub const BSH_: u64 = 0x6006;
    
    
    pub const AWC_: u64 = 0x6800;
    pub const AWD_: u64 = 0x6802;
    pub const AWE_: u64 = 0x6804;
    pub const AWO_: u64 = 0x6806;
    pub const AWG_: u64 = 0x6808;
    pub const AXL_: u64 = 0x680A;
    pub const AWK_: u64 = 0x680C;
    pub const AWS_: u64 = 0x680E;
    pub const AWY_: u64 = 0x6810;
    pub const AXF_: u64 = 0x6812;
    pub const AXP_: u64 = 0x6814;
    pub const AWV_: u64 = 0x6816;
    pub const AXC_: u64 = 0x6818;
    pub const CBQ_: u64 = 0x681A;
    pub const AXJ_: u64 = 0x681C;
    pub const FV_: u64 = 0x681E;
    pub const AXI_: u64 = 0x6820;
    pub const CBW_: u64 = 0x6822;
    pub const CCC_: u64 = 0x6824;
    pub const CCB_: u64 = 0x6826;
    
    
    pub const CCY_: u64 = 0x6C00;
    pub const CCZ_: u64 = 0x6C02;
    pub const CDA_: u64 = 0x6C04;
    pub const CDE_: u64 = 0x6C06;
    pub const CDH_: u64 = 0x6C08;
    pub const CDT_: u64 = 0x6C0A;
    pub const CDG_: u64 = 0x6C0C;
    pub const CDL_: u64 = 0x6C0E;
    pub const CDS_: u64 = 0x6C10;
    pub const CDR_: u64 = 0x6C12;
    pub const CDN_: u64 = 0x6C14;
    pub const CDM_: u64 = 0x6C16;
    
    
    pub const DFA_: u64 = 0x4400;
    pub const DEY_: u64 = 0x4402;
    pub const ENU_: u64 = 0x4404;
    pub const ENT_: u64 = 0x4406;
    pub const DUL_: u64 = 0x4408;
    pub const DUK_: u64 = 0x440A;
    pub const DEV_: u64 = 0x440C;
    pub const ENS_: u64 = 0x440E;
    pub const BXN_: u64 = 0x6400;
    pub const CBV_: u64 = 0x640A;
    pub const CBX_: u64 = 0x2400;
}


pub mod exit_reason {
    pub const DOW_: u32 = 0;
    pub const DPB_: u32 = 1;
    pub const DCI_: u32 = 2;
    pub const DUU_: u32 = 3;
    pub const Bcq: u32 = 4;
    pub const DVL_: u32 = 5;
    pub const ECA_: u32 = 6;
    pub const DVA_: u32 = 7;
    pub const DYU_: u32 = 8;
    pub const DBE_: u32 = 9;
    pub const Rh: u32 = 10;
    pub const Awp: u32 = 11;
    pub const Su: u32 = 12;
    pub const Zx: u32 = 13;
    pub const Alm: u32 = 14;
    pub const Ano: u32 = 15;
    pub const Anp: u32 = 16;
    pub const Aok: u32 = 17;
    pub const Arm: u32 = 18;
    pub const Bew: u32 = 19;
    pub const Bey: u32 = 20;
    pub const Bfa: u32 = 21;
    pub const Bfb: u32 = 22;
    pub const Bfc: u32 = 23;
    pub const Bfd: u32 = 24;
    pub const Bfg: u32 = 25;
    pub const Bfh: u32 = 26;
    pub const Bfi: u32 = 27;
    pub const DLI_: u32 = 28;
    pub const DOD_: u32 = 29;
    pub const CFT_: u32 = 30;
    pub const Ann: u32 = 31;
    pub const Agn: u32 = 32;
    pub const CFK_: u32 = 33;
    pub const DYQ_: u32 = 34;
    pub const Abe: u32 = 36;
    pub const DYA_: u32 = 37;
    pub const Abb: u32 = 39;
    pub const Acb: u32 = 40;
    pub const DXG_: u32 = 41;
    pub const EMC_: u32 = 43;
    pub const DGK_: u32 = 44;
    pub const ENO_: u32 = 45;
    pub const DQI_: u32 = 46;
    pub const DWF_: u32 = 47;
    pub const BXE_: u32 = 48;
    pub const DOV_: u32 = 49;
    pub const Axi: u32 = 50;
    pub const Anq: u32 = 51;
    pub const EEQ_: u32 = 52;
    pub const Axl: u32 = 53;
    pub const Agh: u32 = 54;
    pub const Wb: u32 = 55;
    pub const DGL_: u32 = 56;
    pub const Baz: u32 = 57;
    pub const Axk: u32 = 58;
    pub const Bex: u32 = 59;
    pub const Auj: u32 = 60;
    pub const Bba: u32 = 61;
    pub const EEI_: u32 = 62;
    pub const Bfq: u32 = 63;
    pub const Bfp: u32 = 64;
}


#[repr(C, align(4096))]
pub struct VmcsRegion {
    pub azj: u32,
    pub abort_indicator: u32,
    pub data: [u8; 4088],
}

impl VmcsRegion {
    pub fn new(azj: u32) -> Self {
        VmcsRegion {
            azj,
            abort_indicator: 0,
            data: [0; 4088],
        }
    }
}


pub struct Vmcs {
    qd: Box<VmcsRegion>,
    phys_addr: u64,
    is_current: bool,
}

impl Vmcs {
    
    pub fn new(azj: u32) -> Result<Self> {
        let qd = Box::new(VmcsRegion::new(azj));
        let virt_addr = qd.as_ref() as *const VmcsRegion as u64;
        let phys_addr = vmx::bjv(virt_addr);
        
        crate::serial_println!("[VMCS] Allocated virt=0x{:016X} phys=0x{:016X} rev=0x{:08X}",
                              virt_addr, phys_addr, azj);
        
        
        hbu(phys_addr)?;
        
        Ok(Vmcs {
            qd,
            phys_addr,
            is_current: false,
        })
    }
    
    
    pub fn load(&mut self) -> Result<()> {
        jqk(self.phys_addr)?;
        self.is_current = true;
        Ok(())
    }
    
    
    pub fn write(&self, field: u64, value: u64) -> Result<()> {
        if !self.is_current {
            return Err(HypervisorError::InvalidConfiguration);
        }
        jql(field, value)
    }
    
    
    pub fn read(&self, field: u64) -> Result<u64> {
        if !self.is_current {
            return Err(HypervisorError::InvalidConfiguration);
        }
        edv(field)
    }
    
    
    pub fn setup_execution_controls(&self) -> Result<()> {
        use fields::*;
        
        
        let nuq = 0u32; 
        let iuw = vmx::dhg(vmx::nus(), nuq);
        self.write(CNA_, iuw as u64)?;
        crate::serial_println!("[VMCS] Pin-based controls: 0x{:08X}", iuw);
        
        
        let kyk = (1u32 << 7)   
                        | (1u32 << 24)  
                        | (1u32 << 31); 
        let fot = vmx::dhg(vmx::nyf(), kyk);
        self.write(BRO_, fot as u64)?;
        crate::serial_println!("[VMCS] Primary proc-based controls: 0x{:08X}", fot);
        
        
        if fot & (1 << 31) != 0 {
            let mut gth = 0u32;
            
            
            gth |= 1 << 1;
            
            
            gth |= super::vpid::mdt() as u32;
            
            
            
            
            let jdy = vmx::dhg(vmx::AYS_, gth);
            self.write(CVR_, jdy as u64)?;
            crate::serial_println!("[VMCS] Secondary controls: 0x{:08X}", jdy);
        }
        
        
        self.write(BXK_, 0)?;
        
        
        self.write(BSD_, 0)?;
        self.write(BSG_, 0)?;
        self.write(BSE_, 0)?;
        self.write(BSH_, 0)?;
        
        
        self.write(BSF_, 0)?;
        
        
        
        
        
        
        
        Ok(())
    }
    
    
    pub fn setup_vpid(&self, vpid: Option<u16>) -> Result<()> {
        use fields::*;
        
        let jqp = super::vpid::meb(vpid);
        self.write(Ars, jqp)?;
        
        if vpid.is_some() {
            crate::serial_println!("[VMCS] VPID set to {}", jqp);
        }
        
        Ok(())
    }
    
    
    pub fn setup_exit_controls(&self) -> Result<()> {
        use fields::*;
        
        let lsk = (1u32 << 9)   
                         | (1u32 << 15)  
                         | (1u32 << 20)  
                         | (1u32 << 21); 
        let hxb = vmx::dhg(vmx::lsj(), lsk);
        self.write(DEU_, hxb as u64)?;
        crate::serial_println!("[VMCS] Exit controls: 0x{:08X}", hxb);
        
        
        self.write(DEX_, 0)?;
        self.write(DEW_, 0)?;
        
        Ok(())
    }
    
    
    pub fn setup_entry_controls(&self) -> Result<()> {
        use fields::*;
        
        let lqp = (1u32 << 9)   
                          | (1u32 << 14)  
                          ;
        let hwg = vmx::dhg(vmx::lqo(), lqp);
        self.write(DER_, hwg as u64)?;
        crate::serial_println!("[VMCS] Entry controls: 0x{:08X}", hwg);
        
        
        self.write(DET_, 0)?;
        
        
        self.write(DES_, 0)?;
        
        Ok(())
    }
    
    
    pub fn setup_guest_state(&self, entry_point: u64, stack_ptr: u64) -> Result<()> {
        use fields::*;
        
        
        
        self.write(AWI_, 0x08)?;
        self.write(AWG_, 0)?;
        self.write(AWH_, 0xFFFFFFFF)?;
        self.write(AWF_, 0xA09B)?; 
        
        
        for (sel, base, jm, dhu) in [
            (AWM_, AWK_, AWL_, AWJ_),
            (AWQ_, AWO_, AWP_, AWN_),
            (AXN_, AXL_, AXM_, AXK_),
        ] {
            self.write(sel, 0x10)?;
            self.write(base, 0)?;
            self.write(jm, 0xFFFFFFFF)?;
            self.write(dhu, 0xC093)?; 
        }
        
        
        self.write(AWU_, 0)?;
        self.write(AWS_, 0)?;
        self.write(AWT_, 0xFFFF)?;
        self.write(AWR_, 0x10000)?; 
        
        self.write(AXA_, 0)?;
        self.write(AWY_, 0)?;
        self.write(AWZ_, 0xFFFF)?;
        self.write(AWX_, 0x10000)?;
        
        
        self.write(AXH_, 0)?;
        self.write(AXF_, 0)?;
        self.write(AXG_, 0)?;
        self.write(AXE_, 0x10000)?;
        
        
        self.write(AXR_, 0)?;
        self.write(AXP_, 0)?;
        self.write(AXQ_, 0x67)?;
        self.write(AXO_, 0x8B)?; 
        
        
        let ida = 0x80050033u64; 
        let idb = 0x2620u64;     
        self.write(AWC_, ida)?;
        self.write(AWD_, 0)?; 
        self.write(AWE_, idb)?;
        
        
        let idc = 0x500u64; 
        self.write(AXB_, idc)?;
        
        
        self.write(CBS_, 0x0007040600070406u64)?;
        
        
        self.write(FV_, entry_point)?;
        self.write(AXJ_, stack_ptr)?;
        self.write(AXI_, 0x2)?; 
        
        
        self.write(DEP_, 0xFFFFFFFF_FFFFFFFF)?;
        
        
        self.write(CBP_, 0)?;
        self.write(CBU_, 0)?;
        
        
        self.write(CCA_, 0)?;
        self.write(CCC_, 0)?;
        self.write(CCB_, 0)?;
        
        
        self.write(CBQ_, 0x400)?;
        
        
        self.write(CBR_, 0)?;
        
        
        self.write(CBW_, 0)?;
        
        
        self.write(AWV_, 0)?;
        self.write(AWW_, 0)?;
        self.write(AXC_, 0)?;
        self.write(AXD_, 0)?;
        
        crate::serial_println!("[VMCS] Guest state: RIP=0x{:X} RSP=0x{:X} CR0=0x{:X} CR4=0x{:X} EFER=0x{:X}",
                              entry_point, stack_ptr, ida, idb, idc);
        
        Ok(())
    }
    
    
    pub fn setup_host_state(&self, cjc: u64, dn: u64) -> Result<()> {
        use fields::*;
        use core::arch::asm;
        
        
        let cr0: u64;
        let cr3: u64;
        let cr4: u64;
        
        unsafe {
            asm!("mov {}, cr0", out(reg) cr0);
            asm!("mov {}, cr3", out(reg) cr3);
            asm!("mov {}, cr4", out(reg) cr4);
        }
        
        self.write(CCY_, cr0)?;
        self.write(CCZ_, cr3)?;
        self.write(CDA_, cr4)?;
        
        
        
        let cs: u16;
        let ss: u16;
        let ds: u16;
        let es: u16;
        let fs: u16;
        let gs: u16;
        let tr: u16;
        
        unsafe {
            asm!("mov {:x}, cs", out(reg) cs);
            asm!("mov {:x}, ss", out(reg) ss);
            asm!("mov {:x}, ds", out(reg) ds);
            asm!("mov {:x}, es", out(reg) es);
            asm!("mov {:x}, fs", out(reg) fs);
            asm!("mov {:x}, gs", out(reg) gs);
            asm!("str {:x}", out(reg) tr);
        }
        
        self.write(CDB_, (cs & !3) as u64)?;
        self.write(CDP_, (ss & !3) as u64)?;
        self.write(CDC_, (ds & !3) as u64)?;
        self.write(CDD_, (es & !3) as u64)?;
        self.write(CDF_, (fs & !3) as u64)?;
        self.write(CDI_, (gs & !3) as u64)?;
        self.write(CDU_, (tr & !3) as u64)?;
        
        
        let fxw = vmx::ach(0xC000_0100); 
        let gs_base = vmx::ach(0xC000_0101); 
        self.write(CDE_, fxw)?;
        self.write(CDH_, gs_base)?;
        
        
        #[repr(C, packed)]
        struct DescriptorTablePtr {
            jm: u16,
            base: u64,
        }
        
        let mut gdtr = DescriptorTablePtr { jm: 0, base: 0 };
        let mut idtr = DescriptorTablePtr { jm: 0, base: 0 };
        
        unsafe {
            asm!("sgdt [{}]", in(reg) &mut gdtr as *mut DescriptorTablePtr, options(nostack));
            asm!("sidt [{}]", in(reg) &mut idtr as *mut DescriptorTablePtr, options(nostack));
        }
        
        self.write(CDG_, gdtr.base)?;
        self.write(CDL_, idtr.base)?;
        
        
        let gzv = (tr >> 3) as usize;
        let gdt_ptr = gdtr.base as *const u64;
        let jof = if tr != 0 && gzv > 0 {
            unsafe {
                let low = *gdt_ptr.add(gzv);
                let high = *gdt_ptr.add(gzv + 1);
                
                let bxs = ((low >> 16) & 0xFFFF)
                             | (((low >> 32) & 0xFF) << 16)
                             | (((low >> 56) & 0xFF) << 24);
                let bxr = high & 0xFFFFFFFF;
                bxs | (bxr << 32)
            }
        } else {
            0u64
        };
        self.write(CDT_, jof)?;
        
        
        self.write(CDQ_, vmx::ach(0x174) as u64)?;  
        self.write(CDS_, vmx::ach(0x175))?; 
        self.write(CDR_, vmx::ach(0x176))?; 
        
        
        let ifa = vmx::ach(0xC000_0080); 
        let mmf = vmx::ach(0x277);        
        self.write(CDJ_, ifa)?;
        self.write(CDK_, mmf)?;
        
        
        self.write(CDM_, cjc)?;
        self.write(CDN_, dn)?;
        
        crate::serial_println!("[VMCS] Host state: CR0=0x{:X} CR3=0x{:X} CR4=0x{:X}", cr0, cr3, cr4);
        crate::serial_println!("[VMCS] Host state: CS=0x{:X} SS=0x{:X} TR=0x{:X} TR_BASE=0x{:X}",
                              cs & !3, ss & !3, tr & !3, jof);
        let gdt_base = unsafe { core::ptr::addr_of!(gdtr.base).read_unaligned() };
        let mnv = unsafe { core::ptr::addr_of!(idtr.base).read_unaligned() };
        crate::serial_println!("[VMCS] Host state: GDT_BASE=0x{:X} IDT_BASE=0x{:X}", gdt_base, mnv);
        crate::serial_println!("[VMCS] Host state: RIP=0x{:X} RSP=0x{:X} EFER=0x{:X}",
                              cjc, dn, ifa);
        
        Ok(())
    }
}

impl Drop for Vmcs {
    fn drop(&mut self) {
        
        let _ = hbu(self.phys_addr);
    }
}
