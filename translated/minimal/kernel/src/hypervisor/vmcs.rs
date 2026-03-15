









use super::{HypervisorError, Result};
use super::vmx::{self, mps, pyp, igs, pyr};
use alloc::boxed::Box;


pub mod fields {
    
    pub const Cpi: u64 = 0x0000;
    
    
    pub const AUM_: u64 = 0x0800;
    pub const AUE_: u64 = 0x0802;
    pub const AVJ_: u64 = 0x0804;
    pub const AUI_: u64 = 0x0806;
    pub const AUQ_: u64 = 0x0808;
    pub const AUW_: u64 = 0x080A;
    pub const AVD_: u64 = 0x080C;
    pub const AVN_: u64 = 0x080E;
    
    
    pub const BZS_: u64 = 0x0C00;
    pub const BZQ_: u64 = 0x0C02;
    pub const CAE_: u64 = 0x0C04;
    pub const BZR_: u64 = 0x0C06;
    pub const BZU_: u64 = 0x0C08;
    pub const BZX_: u64 = 0x0C0A;
    pub const CAJ_: u64 = 0x0C0C;
    
    
    pub const DRM_: u64 = 0x2000;
    pub const DRN_: u64 = 0x2002;
    pub const DUY_: u64 = 0x2004;
    pub const BUH_: u64 = 0x201A;
    
    
    pub const DAX_: u64 = 0x2800;
    pub const BYL_: u64 = 0x2802;
    pub const BYM_: u64 = 0x2804;
    pub const AUX_: u64 = 0x2806;
    
    
    pub const BZZ_: u64 = 0x2C00;
    pub const BZY_: u64 = 0x2C02;
    
    
    pub const CJR_: u64 = 0x4000;
    pub const BOX_: u64 = 0x4002;
    pub const BUO_: u64 = 0x4004;
    pub const DYN_: u64 = 0x4006;
    pub const DYO_: u64 = 0x4008;
    pub const BPO_: u64 = 0x400A;
    pub const DBC_: u64 = 0x400C;
    pub const DBF_: u64 = 0x400E;
    pub const DBE_: u64 = 0x4010;
    pub const DAZ_: u64 = 0x4012;
    pub const DBB_: u64 = 0x4014;
    pub const DBA_: u64 = 0x4016;
    pub const EKC_: u64 = 0x4018;
    pub const EKD_: u64 = 0x401A;
    pub const CSA_: u64 = 0x401E;
    
    
    pub const AUL_: u64 = 0x4800;
    pub const AUD_: u64 = 0x4802;
    pub const AVI_: u64 = 0x4804;
    pub const AUH_: u64 = 0x4806;
    pub const AUP_: u64 = 0x4808;
    pub const AUV_: u64 = 0x480A;
    pub const AVC_: u64 = 0x480C;
    pub const AVM_: u64 = 0x480E;
    pub const AUS_: u64 = 0x4810;
    pub const AUZ_: u64 = 0x4812;
    pub const AUJ_: u64 = 0x4814;
    pub const AUB_: u64 = 0x4816;
    pub const AVG_: u64 = 0x4818;
    pub const AUF_: u64 = 0x481A;
    pub const AUN_: u64 = 0x481C;
    pub const AUT_: u64 = 0x481E;
    pub const AVA_: u64 = 0x4820;
    pub const AVK_: u64 = 0x4822;
    pub const BYO_: u64 = 0x4824;
    pub const BYJ_: u64 = 0x4826;
    pub const BYU_: u64 = 0x482A;
    
    
    pub const CAF_: u64 = 0x4C00;
    
    
    pub const BPM_: u64 = 0x6000;
    pub const BPP_: u64 = 0x6002;
    pub const BPN_: u64 = 0x6004;
    pub const BPQ_: u64 = 0x6006;
    
    
    pub const ATY_: u64 = 0x6800;
    pub const ATZ_: u64 = 0x6802;
    pub const AUA_: u64 = 0x6804;
    pub const AUK_: u64 = 0x6806;
    pub const AUC_: u64 = 0x6808;
    pub const AVH_: u64 = 0x680A;
    pub const AUG_: u64 = 0x680C;
    pub const AUO_: u64 = 0x680E;
    pub const AUU_: u64 = 0x6810;
    pub const AVB_: u64 = 0x6812;
    pub const AVL_: u64 = 0x6814;
    pub const AUR_: u64 = 0x6816;
    pub const AUY_: u64 = 0x6818;
    pub const BYK_: u64 = 0x681A;
    pub const AVF_: u64 = 0x681C;
    pub const FG_: u64 = 0x681E;
    pub const AVE_: u64 = 0x6820;
    pub const BYQ_: u64 = 0x6822;
    pub const BYW_: u64 = 0x6824;
    pub const BYV_: u64 = 0x6826;
    
    
    pub const BZN_: u64 = 0x6C00;
    pub const BZO_: u64 = 0x6C02;
    pub const BZP_: u64 = 0x6C04;
    pub const BZT_: u64 = 0x6C06;
    pub const BZW_: u64 = 0x6C08;
    pub const CAI_: u64 = 0x6C0A;
    pub const BZV_: u64 = 0x6C0C;
    pub const CAA_: u64 = 0x6C0E;
    pub const CAH_: u64 = 0x6C10;
    pub const CAG_: u64 = 0x6C12;
    pub const CAC_: u64 = 0x6C14;
    pub const CAB_: u64 = 0x6C16;
    
    
    pub const DBI_: u64 = 0x4400;
    pub const DBG_: u64 = 0x4402;
    pub const EKG_: u64 = 0x4404;
    pub const EKF_: u64 = 0x4406;
    pub const DQR_: u64 = 0x4408;
    pub const DQQ_: u64 = 0x440A;
    pub const DBD_: u64 = 0x440C;
    pub const EKE_: u64 = 0x440E;
    pub const BUR_: u64 = 0x6400;
    pub const BYP_: u64 = 0x640A;
    pub const BYR_: u64 = 0x2400;
}


pub mod exit_reason {
    pub const DLH_: u32 = 0;
    pub const DLM_: u32 = 1;
    pub const CYQ_: u32 = 2;
    pub const DRA_: u32 = 3;
    pub const Dho: u32 = 4;
    pub const DRS_: u32 = 5;
    pub const DYJ_: u32 = 6;
    pub const DRH_: u32 = 7;
    pub const DVD_: u32 = 8;
    pub const CXM_: u32 = 9;
    pub const Apr: u32 = 10;
    pub const Cxt: u32 = 11;
    pub const Atl: u32 = 12;
    pub const Bje: u32 = 13;
    pub const Cfn: u32 = 14;
    pub const Cji: u32 = 15;
    pub const Cjj: u32 = 16;
    pub const Ckc: u32 = 17;
    pub const Cpd: u32 = 18;
    pub const Dky: u32 = 19;
    pub const Dla: u32 = 20;
    pub const Dlc: u32 = 21;
    pub const Dld: u32 = 22;
    pub const Dle: u32 = 23;
    pub const Dlf: u32 = 24;
    pub const Dli: u32 = 25;
    pub const Dlj: u32 = 26;
    pub const Dlk: u32 = 27;
    pub const DHP_: u32 = 28;
    pub const DKP_: u32 = 29;
    pub const CCI_: u32 = 30;
    pub const Cjh: u32 = 31;
    pub const Bwl: u32 = 32;
    pub const CBZ_: u32 = 33;
    pub const DUZ_: u32 = 34;
    pub const Bly: u32 = 36;
    pub const DUJ_: u32 = 37;
    pub const Blv: u32 = 39;
    pub const Boi: u32 = 40;
    pub const DTP_: u32 = 41;
    pub const EIN_: u32 = 43;
    pub const DCQ_: u32 = 44;
    pub const EKA_: u32 = 45;
    pub const DMO_: u32 = 46;
    pub const DSM_: u32 = 47;
    pub const BUI_: u32 = 48;
    pub const DLG_: u32 = 49;
    pub const Cyx: u32 = 50;
    pub const Cjk: u32 = 51;
    pub const EAZ_: u32 = 52;
    pub const Cza: u32 = 53;
    pub const Bwf: u32 = 54;
    pub const Bbf: u32 = 55;
    pub const DCR_: u32 = 56;
    pub const Dfg: u32 = 57;
    pub const Cyz: u32 = 58;
    pub const Dkz: u32 = 59;
    pub const Cux: u32 = 60;
    pub const Dfh: u32 = 61;
    pub const EAR_: u32 = 62;
    pub const Dme: u32 = 63;
    pub const Dmd: u32 = 64;
}


#[repr(C, align(4096))]
pub struct VmcsRegion {
    pub cty: u32,
    pub qek: u32,
    pub f: [u8; 4088],
}

impl VmcsRegion {
    pub fn new(cty: u32) -> Self {
        VmcsRegion {
            cty,
            qek: 0,
            f: [0; 4088],
        }
    }
}


pub struct Vmcs {
    aoz: Box<VmcsRegion>,
    ki: u64,
    afb: bool,
}

impl Vmcs {
    
    pub fn new(cty: u32) -> Result<Self> {
        let aoz = Box::new(VmcsRegion::new(cty));
        let vd = aoz.as_ref() as *const VmcsRegion as u64;
        let ki = vmx::dmy(vd);
        
        crate::serial_println!("[VMCS] Allocated virt=0x{:016X} phys=0x{:016X} rev=0x{:08X}",
                              vd, ki, cty);
        
        
        mps(ki)?;
        
        Ok(Vmcs {
            aoz,
            ki,
            afb: false,
        })
    }
    
    
    pub fn load(&mut self) -> Result<()> {
        pyp(self.ki)?;
        self.afb = true;
        Ok(())
    }
    
    
    pub fn write(&self, buj: u64, bn: u64) -> Result<()> {
        if !self.afb {
            return Err(HypervisorError::Xd);
        }
        pyr(buj, bn)
    }
    
    
    pub fn read(&self, buj: u64) -> Result<u64> {
        if !self.afb {
            return Err(HypervisorError::Xd);
        }
        igs(buj)
    }
    
    
    pub fn wks(&self) -> Result<()> {
        use fields::*;
        
        
        let vhx = 0u32; 
        let ovr = vmx::gyb(vmx::vhz(), vhx);
        self.write(CJR_, ovr as u64)?;
        crate::serial_println!("[VMCS] Pin-based controls: 0x{:08X}", ovr);
        
        
        let rpn = (1u32 << 7)   
                        | (1u32 << 24)  
                        | (1u32 << 31); 
        let klb = vmx::gyb(vmx::vmk(), rpn);
        self.write(BOX_, klb as u64)?;
        crate::serial_println!("[VMCS] Primary proc-based controls: 0x{:08X}", klb);
        
        
        if klb & (1 << 31) != 0 {
            let mut mcy = 0u32;
            
            
            mcy |= 1 << 1;
            
            
            mcy |= super::vpid::tep() as u32;
            
            
            
            
            let phc = vmx::gyb(vmx::AWQ_, mcy);
            self.write(CSA_, phc as u64)?;
            crate::serial_println!("[VMCS] Secondary controls: 0x{:08X}", phc);
        }
        
        
        self.write(BUO_, 0)?;
        
        
        self.write(BPM_, 0)?;
        self.write(BPP_, 0)?;
        self.write(BPN_, 0)?;
        self.write(BPQ_, 0)?;
        
        
        self.write(BPO_, 0)?;
        
        
        
        
        
        
        
        Ok(())
    }
    
    
    pub fn wlm(&self, vpid: Option<u16>) -> Result<()> {
        use fields::*;
        
        let pyv = super::vpid::tfc(vpid);
        self.write(Cpi, pyv)?;
        
        if vpid.is_some() {
            crate::serial_println!("[VMCS] VPID set to {}", pyv);
        }
        
        Ok(())
    }
    
    
    pub fn wkt(&self) -> Result<()> {
        use fields::*;
        
        let soz = (1u32 << 9)   
                         | (1u32 << 15)  
                         | (1u32 << 20)  
                         | (1u32 << 21); 
        let nrt = vmx::gyb(vmx::soy(), soz);
        self.write(DBC_, nrt as u64)?;
        crate::serial_println!("[VMCS] Exit controls: 0x{:08X}", nrt);
        
        
        self.write(DBF_, 0)?;
        self.write(DBE_, 0)?;
        
        Ok(())
    }
    
    
    pub fn wkq(&self) -> Result<()> {
        use fields::*;
        
        let smb = (1u32 << 9)   
                          | (1u32 << 14)  
                          ;
        let nqp = vmx::gyb(vmx::sma(), smb);
        self.write(DAZ_, nqp as u64)?;
        crate::serial_println!("[VMCS] Entry controls: 0x{:08X}", nqp);
        
        
        self.write(DBB_, 0)?;
        
        
        self.write(DBA_, 0)?;
        
        Ok(())
    }
    
    
    pub fn wla(&self, mi: u64, ahu: u64) -> Result<()> {
        use fields::*;
        
        
        
        self.write(AUE_, 0x08)?;
        self.write(AUC_, 0)?;
        self.write(AUD_, 0xFFFFFFFF)?;
        self.write(AUB_, 0xA09B)?; 
        
        
        for (fua, ar, ul, gyw) in [
            (AUI_, AUG_, AUH_, AUF_),
            (AUM_, AUK_, AUL_, AUJ_),
            (AVJ_, AVH_, AVI_, AVG_),
        ] {
            self.write(fua, 0x10)?;
            self.write(ar, 0)?;
            self.write(ul, 0xFFFFFFFF)?;
            self.write(gyw, 0xC093)?; 
        }
        
        
        self.write(AUQ_, 0)?;
        self.write(AUO_, 0)?;
        self.write(AUP_, 0xFFFF)?;
        self.write(AUN_, 0x10000)?; 
        
        self.write(AUW_, 0)?;
        self.write(AUU_, 0)?;
        self.write(AUV_, 0xFFFF)?;
        self.write(AUT_, 0x10000)?;
        
        
        self.write(AVD_, 0)?;
        self.write(AVB_, 0)?;
        self.write(AVC_, 0)?;
        self.write(AVA_, 0x10000)?;
        
        
        self.write(AVN_, 0)?;
        self.write(AVL_, 0)?;
        self.write(AVM_, 0x67)?;
        self.write(AVK_, 0x8B)?; 
        
        
        let nzw = 0x80050033u64; 
        let nzx = 0x2620u64;     
        self.write(ATY_, nzw)?;
        self.write(ATZ_, 0)?; 
        self.write(AUA_, nzx)?;
        
        
        let nzy = 0x500u64; 
        self.write(AUX_, nzy)?;
        
        
        self.write(BYM_, 0x0007040600070406u64)?;
        
        
        self.write(FG_, mi)?;
        self.write(AVF_, ahu)?;
        self.write(AVE_, 0x2)?; 
        
        
        self.write(DAX_, 0xFFFFFFFF_FFFFFFFF)?;
        
        
        self.write(BYJ_, 0)?;
        self.write(BYO_, 0)?;
        
        
        self.write(BYU_, 0)?;
        self.write(BYW_, 0)?;
        self.write(BYV_, 0)?;
        
        
        self.write(BYK_, 0x400)?;
        
        
        self.write(BYL_, 0)?;
        
        
        self.write(BYQ_, 0)?;
        
        
        self.write(AUR_, 0)?;
        self.write(AUS_, 0)?;
        self.write(AUY_, 0)?;
        self.write(AUZ_, 0)?;
        
        crate::serial_println!("[VMCS] Guest state: RIP=0x{:X} RSP=0x{:X} CR0=0x{:X} CR4=0x{:X} EFER=0x{:X}",
                              mi, ahu, nzw, nzx, nzy);
        
        Ok(())
    }
    
    
    pub fn pjo(&self, fic: u64, jo: u64) -> Result<()> {
        use fields::*;
        use core::arch::asm;
        
        
        let akb: u64;
        let jm: u64;
        let cr4: u64;
        
        unsafe {
            asm!("mov {}, cr0", bd(reg) akb);
            asm!("mov {}, cr3", bd(reg) jm);
            asm!("mov {}, cr4", bd(reg) cr4);
        }
        
        self.write(BZN_, akb)?;
        self.write(BZO_, jm)?;
        self.write(BZP_, cr4)?;
        
        
        
        let aap: u16;
        let rv: u16;
        let bjw: u16;
        let cqf: u16;
        let fs: u16;
        let ckx: u16;
        let agd: u16;
        
        unsafe {
            asm!("mov {:x}, cs", bd(reg) aap);
            asm!("mov {:x}, ss", bd(reg) rv);
            asm!("mov {:x}, ds", bd(reg) bjw);
            asm!("mov {:x}, es", bd(reg) cqf);
            asm!("mov {:x}, fs", bd(reg) fs);
            asm!("mov {:x}, gs", bd(reg) ckx);
            asm!("str {:x}", bd(reg) agd);
        }
        
        self.write(BZQ_, (aap & !3) as u64)?;
        self.write(CAE_, (rv & !3) as u64)?;
        self.write(BZR_, (bjw & !3) as u64)?;
        self.write(BZS_, (cqf & !3) as u64)?;
        self.write(BZU_, (fs & !3) as u64)?;
        self.write(BZX_, (ckx & !3) as u64)?;
        self.write(CAJ_, (agd & !3) as u64)?;
        
        
        let kxh = vmx::bcg(0xC000_0100); 
        let fjy = vmx::bcg(0xC000_0101); 
        self.write(BZT_, kxh)?;
        self.write(BZW_, fjy)?;
        
        
        #[repr(C, packed)]
        struct Ahj {
            ul: u16,
            ar: u64,
        }
        
        let mut iwk = Ahj { ul: 0, ar: 0 };
        let mut ldf = Ahj { ul: 0, ar: 0 };
        
        unsafe {
            asm!("sgdt [{}]", in(reg) &mut iwk as *mut Ahj, options(nostack));
            asm!("sidt [{}]", in(reg) &mut ldf as *mut Ahj, options(nostack));
        }
        
        self.write(BZV_, iwk.ar)?;
        self.write(CAA_, ldf.ar)?;
        
        
        let mmp = (agd >> 3) as usize;
        let ghv = iwk.ar as *const u64;
        let pvm = if agd != 0 && mmp > 0 {
            unsafe {
                let ail = *ghv.add(mmp);
                let afq = *ghv.add(mmp + 1);
                
                let emp = ((ail >> 16) & 0xFFFF)
                             | (((ail >> 32) & 0xFF) << 16)
                             | (((ail >> 56) & 0xFF) << 24);
                let emo = afq & 0xFFFFFFFF;
                emp | (emo << 32)
            }
        } else {
            0u64
        };
        self.write(CAI_, pvm)?;
        
        
        self.write(CAF_, vmx::bcg(0x174) as u64)?;  
        self.write(CAH_, vmx::bcg(0x175))?; 
        self.write(CAG_, vmx::bcg(0x176))?; 
        
        
        let ocb = vmx::bcg(0xC000_0080); 
        let tpv = vmx::bcg(0x277);        
        self.write(BZY_, ocb)?;
        self.write(BZZ_, tpv)?;
        
        
        self.write(CAB_, fic)?;
        self.write(CAC_, jo)?;
        
        crate::serial_println!("[VMCS] Host state: CR0=0x{:X} CR3=0x{:X} CR4=0x{:X}", akb, jm, cr4);
        crate::serial_println!("[VMCS] Host state: CS=0x{:X} SS=0x{:X} TR=0x{:X} TR_BASE=0x{:X}",
                              aap & !3, rv & !3, agd & !3, pvm);
        let bun = unsafe { core::ptr::vf!(iwk.ar).md() };
        let trs = unsafe { core::ptr::vf!(ldf.ar).md() };
        crate::serial_println!("[VMCS] Host state: GDT_BASE=0x{:X} IDT_BASE=0x{:X}", bun, trs);
        crate::serial_println!("[VMCS] Host state: RIP=0x{:X} RSP=0x{:X} EFER=0x{:X}",
                              fic, jo, ocb);
        
        Ok(())
    }
}

impl Drop for Vmcs {
    fn drop(&mut self) {
        
        let _ = mps(self.ki);
    }
}
