




use core::mem::size_of;


pub mod control_offsets {
    pub const BSK_: usize = 0x000;
    pub const BSL_: usize = 0x004;
    pub const DOE_: usize = 0x008;
    pub const DOF_: usize = 0x00C;
    pub const BXL_: usize = 0x010;
    pub const CFE_: usize = 0x014;
    pub const CFF_: usize = 0x018;
    pub const ECS_: usize = 0x03C;
    pub const ECR_: usize = 0x03E;
    pub const CFR_: usize = 0x040;
    pub const CKL_: usize = 0x048;
    pub const DCQ_: usize = 0x050;
    pub const AWB_: usize = 0x058;
    pub const BJK_: usize = 0x05C;
    pub const DFC_: usize = 0x060;
    pub const ENY_: usize = 0x061;
    pub const ENV_: usize = 0x062;
    pub const ENW_: usize = 0x063;
    pub const ENX_: usize = 0x064;
    pub const DUZ_: usize = 0x068;
    pub const Lv: usize = 0x070;
    pub const Lx: usize = 0x078;
    pub const Ly: usize = 0x080;
    pub const Avf: usize = 0x088;
    pub const XD_: usize = 0x090;
    pub const DGW_: usize = 0x098;
    pub const DQY_: usize = 0x0A0;
    pub const HA_: usize = 0x0A8;
    pub const BDX_: usize = 0x0B0;
    pub const DWD_: usize = 0x0B8;
    pub const BKT_: usize = 0x0C0;
    pub const AHS_: usize = 0x0C8;
    pub const BOG_: usize = 0x0D0;
    pub const CBT_: usize = 0x0D1;
    pub const DGV_: usize = 0x0E0;
    pub const DGX_: usize = 0x0F0;
    pub const DGY_: usize = 0x0F8;
    pub const ENP_: usize = 0x108;
}


pub mod state_offsets {
    pub const M: usize = 0x400;
    
    pub const UF_: usize = M + 0x000;
    pub const BXF_: usize = M + 0x002;
    pub const BXH_: usize = M + 0x004;
    pub const BXG_: usize = M + 0x008;
    
    pub const KO_: usize = M + 0x010;
    pub const ACN_: usize = M + 0x012;
    pub const ACO_: usize = M + 0x014;
    pub const TP_: usize = M + 0x018;
    
    pub const YO_: usize = M + 0x020;
    pub const AJY_: usize = M + 0x022;
    pub const AKA_: usize = M + 0x024;
    pub const AJZ_: usize = M + 0x028;
    
    pub const NT_: usize = M + 0x030;
    pub const BVB_: usize = M + 0x032;
    pub const BVD_: usize = M + 0x034;
    pub const BVC_: usize = M + 0x038;
    
    pub const AEB_: usize = M + 0x040;
    pub const BYU_: usize = M + 0x042;
    pub const BYV_: usize = M + 0x044;
    pub const AEA_: usize = M + 0x048;
    
    pub const AEV_: usize = M + 0x050;
    pub const CBN_: usize = M + 0x052;
    pub const CBO_: usize = M + 0x054;
    pub const AEU_: usize = M + 0x058;
    
    pub const DQK_: usize = M + 0x060;
    pub const DQH_: usize = M + 0x062;
    pub const BZW_: usize = M + 0x064;
    pub const BZV_: usize = M + 0x068;
    
    pub const CHS_: usize = M + 0x070;
    pub const CHP_: usize = M + 0x072;
    pub const CHR_: usize = M + 0x074;
    pub const CHQ_: usize = M + 0x078;
    
    pub const DUI_: usize = M + 0x080;
    pub const DUG_: usize = M + 0x082;
    pub const CEX_: usize = M + 0x084;
    pub const CEW_: usize = M + 0x088;
    
    pub const DCP_: usize = M + 0x090;
    pub const DCM_: usize = M + 0x092;
    pub const DCO_: usize = M + 0x094;
    pub const DCN_: usize = M + 0x098;
    
    pub const Of: usize = M + 0x0CB;
    pub const Eu: usize = M + 0x0D0;
    pub const Jp: usize = M + 0x148;
    pub const Jo: usize = M + 0x150;
    pub const Jn: usize = M + 0x158;
    pub const Aip: usize = M + 0x160;
    pub const Aio: usize = M + 0x168;
    pub const Ek: usize = M + 0x170;
    pub const Af: usize = M + 0x178;
    pub const De: usize = M + 0x1D8;
    pub const Fa: usize = M + 0x1F8;
    pub const Aek: usize = M + 0x200;
    pub const Aar: usize = M + 0x208;
    pub const Xb: usize = M + 0x210;
    pub const Adq: usize = M + 0x218;
    pub const AZU_: usize = M + 0x220;
    pub const BIX_: usize = M + 0x228;
    pub const BIZ_: usize = M + 0x230;
    pub const BIY_: usize = M + 0x238;
    pub const Og: usize = M + 0x240;
    pub const Uf: usize = M + 0x268;
    pub const Atu: usize = M + 0x270;
    pub const DHQ_: usize = M + 0x278;
    pub const DHR_: usize = M + 0x280;
    pub const Ayi: usize = M + 0x288;
    pub const Ayj: usize = M + 0x290;
}


pub mod intercepts {
    pub const Axg: u32 = 1 << 0;
    pub const Azp: u32 = 1 << 1;
    pub const Bcs: u32 = 1 << 2;
    pub const Bm: u32 = 1 << 3;
    pub const Bev: u32 = 1 << 4;
    pub const DLC_: u32 = 1 << 5;
    pub const DUH_: u32 = 1 << 6;
    pub const DQJ_: u32 = 1 << 7;
    pub const DWE_: u32 = 1 << 8;
    pub const EMM_: u32 = 1 << 9;
    pub const DUJ_: u32 = 1 << 10;
    pub const DQL_: u32 = 1 << 11;
    pub const DWG_: u32 = 1 << 12;
    pub const EMN_: u32 = 1 << 13;
    pub const Anp: u32 = 1 << 14;
    pub const Ano: u32 = 1 << 15;
    pub const Bap: u32 = 1 << 16;
    pub const Bag: u32 = 1 << 17;
    pub const Rh: u32 = 1 << 18;
    pub const Aok: u32 = 1 << 19;
    pub const Axs: u32 = 1 << 20;
    pub const Bdf: u32 = 1 << 21;
    pub const Zx: u32 = 1 << 22;
    pub const Acb: u32 = 1 << 23;
    pub const Su: u32 = 1 << 24;
    pub const Alm: u32 = 1 << 25;
    pub const Axj: u32 = 1 << 26;
    pub const Alo: u32 = 1 << 27;
    pub const Amq: u32 = 1 << 28;
    pub const DBE_: u32 = 1 << 29;
    pub const DPG_: u32 = 1 << 30;
    pub const Iu: u32 = 1 << 31;
}


pub mod intercepts2 {
    pub const Arq: u32 = 1 << 0;
    pub const Aro: u32 = 1 << 1;
    pub const Bez: u32 = 1 << 2;
    pub const Bfe: u32 = 1 << 3;
    pub const Bda: u32 = 1 << 4;
    pub const Ati: u32 = 1 << 5;
    pub const Bcr: u32 = 1 << 6;
    pub const Anq: u32 = 1 << 7;
    pub const Axa: u32 = 1 << 8;
    pub const Agh: u32 = 1 << 9;
    pub const Abb: u32 = 1 << 10;
    pub const Abe: u32 = 1 << 11;
    pub const DYR_: u32 = 1 << 12;
    pub const Wb: u32 = 1 << 13;
}


pub mod clean_bits {
    pub const Axf: u32 = 1 << 0;
    pub const Alp: u32 = 1 << 1;
    pub const Agx: u32 = 1 << 2;
    pub const Bdy: u32 = 1 << 3;
    pub const Amw: u32 = 1 << 4;
    pub const Atm: u32 = 1 << 5;
    pub const Atx: u32 = 1 << 6;
    pub const Aty: u32 = 1 << 7;
    pub const Bbo: u32 = 1 << 8;
    pub const Og: u32 = 1 << 9;
    pub const Ame: u32 = 1 << 10;
    pub const Agz: u32 = 1 << 11;
}


pub mod tlb_control {
    pub const DNH_: u32 = 0;
    pub const DPW_: u32 = 1;
    pub const DPY_: u32 = 3;
    pub const DPX_: u32 = 7;
}


#[repr(C, align(4096))]
pub struct Vmcb {
    data: [u8; 4096],
}

impl Vmcb {
    
    pub const fn new() -> Self {
        Self { data: [0; 4096] }
    }
    
    
    #[inline]
    pub fn read_u8(&self, offset: usize) -> u8 {
        self.data[offset]
    }
    
    
    
    
    
    pub fn guest_insn_bytes(&self) -> (usize, [u8; 15]) {
        let count = self.data[control_offsets::BOG_] as usize;
        let mut buf = [0u8; 15];
        let ae = count.min(15);
        for i in 0..ae {
            buf[i] = self.data[control_offsets::CBT_ + i];
        }
        (ae, buf)
    }
    
    
    #[inline]
    pub fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.data[offset], self.data[offset + 1]])
    }
    
    
    #[inline]
    pub fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }
    
    
    #[inline]
    pub fn read_u64(&self, offset: usize) -> u64 {
        u64::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
            self.data[offset + 4],
            self.data[offset + 5],
            self.data[offset + 6],
            self.data[offset + 7],
        ])
    }
    
    
    #[inline]
    pub fn write_u16(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.data[offset] = bytes[0];
        self.data[offset + 1] = bytes[1];
    }
    
    
    #[inline]
    pub fn write_u32(&mut self, offset: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.data[offset] = bytes[0];
        self.data[offset + 1] = bytes[1];
        self.data[offset + 2] = bytes[2];
        self.data[offset + 3] = bytes[3];
    }
    
    
    #[inline]
    pub fn write_u64(&mut self, offset: usize, value: u64) {
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            self.data[offset + i] = bytes[i];
        }
    }
    
    
    pub fn phys_addr(&self) -> u64 {
        let virt = self as *const _ as u64;
        virt.wrapping_sub(crate::memory::hhdm_offset())
    }
    
    
    
    
    
    
    #[inline]
    pub fn read_control(&self, offset: usize) -> u64 {
        self.read_u64(offset)
    }
    
    
    #[inline]
    pub fn write_control(&mut self, offset: usize, value: u64) {
        self.write_u64(offset, value);
    }
    
    
    #[inline]
    pub fn read_state(&self, offset: usize) -> u64 {
        self.read_u64(offset)
    }
    
    
    #[inline]
    pub fn write_state(&mut self, offset: usize, value: u64) {
        self.write_u64(offset, value);
    }
    
    
    
    
    
    
    pub fn qvp(&mut self, mask: u32) {
        self.write_u32(control_offsets::BSK_, mask);
    }
    
    
    pub fn set_cr_write_intercepts(&mut self, mask: u32) {
        self.write_u32(control_offsets::BSL_, mask);
    }
    
    
    pub fn qvu(&mut self, mask: u32) {
        self.write_u32(control_offsets::BXL_, mask);
    }
    
    
    pub fn set_intercepts1(&mut self, mask: u32) {
        self.write_u32(control_offsets::CFE_, mask);
    }
    
    
    pub fn set_intercepts2(&mut self, mask: u32) {
        self.write_u32(control_offsets::CFF_, mask);
    }
    
    
    pub fn set_iopm_base(&mut self, phys: u64) {
        self.write_u64(control_offsets::CFR_, phys);
    }
    
    
    pub fn set_msrpm_base(&mut self, phys: u64) {
        self.write_u64(control_offsets::CKL_, phys);
    }
    
    
    pub fn qwo(&mut self, offset: u64) {
        self.write_u64(control_offsets::DCQ_, offset);
    }
    
    
    pub fn qvz(&mut self, asid: u32) {
        self.write_u32(control_offsets::AWB_, asid);
    }
    
    
    pub fn qwn(&mut self, control: u32) {
        self.write_u32(control_offsets::BJK_, control);
    }
    
    
    pub fn exit_code(&self) -> u64 {
        self.read_u64(control_offsets::Lv)
    }
    
    
    pub fn bmb(&self) -> u64 {
        self.read_u64(control_offsets::Lx)
    }
    
    
    pub fn fvp(&self) -> u64 {
        self.read_u64(control_offsets::Ly)
    }
    
    
    pub fn qez(&mut self, ncr3: u64) {
        self.write_u64(control_offsets::XD_, 1);
        self.write_u64(control_offsets::BDX_, ncr3);
    }
    
    
    pub fn qdb(&mut self) {
        self.write_u64(control_offsets::XD_, 0);
    }
    
    
    pub fn clean_bits(&self) -> u32 {
        self.read_u32(control_offsets::BKT_)
    }
    
    
    pub fn set_clean_bits(&mut self, bits: u32) {
        self.write_u32(control_offsets::BKT_, bits);
    }
    
    
    pub fn vo(&self) -> u64 {
        self.read_u64(control_offsets::AHS_)
    }
    
    
    pub fn inject_event(&mut self, vector: u8, event_type: u8, error_code: Option<u32>) {
        let mut event: u64 = (vector as u64) | ((event_type as u64) << 8) | (1u64 << 31); 
        if let Some(ec) = error_code {
            event |= 1u64 << 11; 
            event |= (ec as u64) << 32;
        }
        self.write_u64(control_offsets::HA_, event);
    }
    
    
    
    
    
    
    pub fn set_rip(&mut self, rip: u64) {
        self.write_u64(state_offsets::Af, rip);
    }
    
    
    pub fn rip(&self) -> u64 {
        self.read_u64(state_offsets::Af)
    }
    
    
    pub fn set_rsp(&mut self, rsp: u64) {
        self.write_u64(state_offsets::De, rsp);
    }
    
    
    pub fn rsp(&self) -> u64 {
        self.read_u64(state_offsets::De)
    }
    
    
    pub fn qwl(&mut self, rax: u64) {
        self.write_u64(state_offsets::Fa, rax);
    }
    
    
    pub fn rax(&self) -> u64 {
        self.read_u64(state_offsets::Fa)
    }
    
    
    pub fn set_rflags(&mut self, rflags: u64) {
        self.write_u64(state_offsets::Ek, rflags);
    }
    
    
    pub fn rflags(&self) -> u64 {
        self.read_u64(state_offsets::Ek)
    }
    
    
    pub fn set_cr0(&mut self, cr0: u64) {
        self.write_u64(state_offsets::Jn, cr0);
    }
    
    
    pub fn cr0(&self) -> u64 {
        self.read_u64(state_offsets::Jn)
    }
    
    
    pub fn set_cr3(&mut self, cr3: u64) {
        self.write_u64(state_offsets::Jo, cr3);
    }
    
    
    pub fn cr3(&self) -> u64 {
        self.read_u64(state_offsets::Jo)
    }
    
    
    pub fn set_cr4(&mut self, cr4: u64) {
        self.write_u64(state_offsets::Jp, cr4);
    }
    
    
    pub fn cr4(&self) -> u64 {
        self.read_u64(state_offsets::Jp)
    }
    
    
    pub fn set_efer(&mut self, efer: u64) {
        self.write_u64(state_offsets::Eu, efer);
    }
    
    
    pub fn efer(&self) -> u64 {
        self.read_u64(state_offsets::Eu)
    }
    
    
    pub fn set_cs(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::KO_, selector);
        self.write_u16(state_offsets::ACN_, aga);
        self.write_u32(state_offsets::ACO_, jm);
        self.write_u64(state_offsets::TP_, base);
    }
    
    
    pub fn set_ds(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::NT_, selector);
        self.write_u16(state_offsets::BVB_, aga);
        self.write_u32(state_offsets::BVD_, jm);
        self.write_u64(state_offsets::BVC_, base);
    }
    
    
    pub fn set_es(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::UF_, selector);
        self.write_u16(state_offsets::BXF_, aga);
        self.write_u32(state_offsets::BXH_, jm);
        self.write_u64(state_offsets::BXG_, base);
    }
    
    
    pub fn set_ss(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::YO_, selector);
        self.write_u16(state_offsets::AJY_, aga);
        self.write_u32(state_offsets::AKA_, jm);
        self.write_u64(state_offsets::AJZ_, base);
    }
    
    
    pub fn set_fs(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::AEB_, selector);
        self.write_u16(state_offsets::BYU_, aga);
        self.write_u32(state_offsets::BYV_, jm);
        self.write_u64(state_offsets::AEA_, base);
    }
    
    
    pub fn set_gs(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::AEV_, selector);
        self.write_u16(state_offsets::CBN_, aga);
        self.write_u32(state_offsets::CBO_, jm);
        self.write_u64(state_offsets::AEU_, base);
    }
    
    
    pub fn set_gdtr(&mut self, jm: u32, base: u64) {
        self.write_u32(state_offsets::BZW_, jm);
        self.write_u64(state_offsets::BZV_, base);
    }
    
    
    pub fn set_idtr(&mut self, jm: u32, base: u64) {
        self.write_u32(state_offsets::CEX_, jm);
        self.write_u64(state_offsets::CEW_, base);
    }
    
    
    pub fn set_tr(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::DCP_, selector);
        self.write_u16(state_offsets::DCM_, aga);
        self.write_u32(state_offsets::DCO_, jm);
        self.write_u64(state_offsets::DCN_, base);
    }
    
    
    pub fn set_ldtr(&mut self, selector: u16, aga: u16, jm: u32, base: u64) {
        self.write_u16(state_offsets::CHS_, selector);
        self.write_u16(state_offsets::CHP_, aga);
        self.write_u32(state_offsets::CHR_, jm);
        self.write_u64(state_offsets::CHQ_, base);
    }
    
    
    pub fn set_cpl(&mut self, eiz: u8) {
        self.data[state_offsets::Of] = eiz;
    }
    
    
    pub fn eiz(&self) -> u8 {
        self.data[state_offsets::Of]
    }
    
    
    pub fn set_dr6(&mut self, dr6: u64) {
        self.write_u64(state_offsets::Aio, dr6);
    }
    
    
    pub fn set_dr7(&mut self, dr7: u64) {
        self.write_u64(state_offsets::Aip, dr7);
    }
    
    
    pub fn set_pat(&mut self, pat: u64) {
        self.write_u64(state_offsets::Uf, pat);
    }
    
    
    
    
    
    
    pub fn setup_real_mode(&mut self) {
        
        let bfu: u16 = 0x0093; 
        let fno: u16 = 0x009B; 
        
        
        self.set_cs(0xF000, fno, 0xFFFF, 0xF0000);
        
        
        self.set_ds(0, bfu, 0xFFFF, 0);
        self.set_es(0, bfu, 0xFFFF, 0);
        self.set_fs(0, bfu, 0xFFFF, 0);
        self.set_gs(0, bfu, 0xFFFF, 0);
        self.set_ss(0, bfu, 0xFFFF, 0);
        
        
        self.set_tr(0, 0x008B, 0xFFFF, 0);
        
        
        self.set_ldtr(0, 0x0082, 0xFFFF, 0);
        
        
        self.set_gdtr(0xFFFF, 0);
        self.set_idtr(0x3FF, 0);  
        
        
        self.set_cr0(0x60000010);  
        
        
        self.set_rflags(0x00000002);
        
        
        self.set_rip(0xFFF0);
        
        
        self.set_dr6(0xFFFF0FF0);
        self.set_dr7(0x00000400);
        
        
        self.set_efer(0);
        
        
        self.set_pat(0x0007040600070406);
    }
    
    
    pub fn setup_protected_mode(&mut self, entry_point: u64) {
        let bfu: u16 = 0x00C3; 
        let fno: u16 = 0x00CB; 
        
        self.set_cs(0x08, fno, 0xFFFFFFFF, 0);
        self.set_ds(0x10, bfu, 0xFFFFFFFF, 0);
        self.set_es(0x10, bfu, 0xFFFFFFFF, 0);
        self.set_fs(0x10, bfu, 0xFFFFFFFF, 0);
        self.set_gs(0x10, bfu, 0xFFFFFFFF, 0);
        self.set_ss(0x10, bfu, 0xFFFFFFFF, 0);
        
        self.set_tr(0x18, 0x008B, 0xFFFF, 0);
        self.set_ldtr(0, 0x0082, 0xFFFF, 0);
        
        
        self.set_gdtr(23, 0x1000);
        
        self.set_idtr(0, 0);
        
        self.set_cr0(0x60000011);  
        self.set_rflags(0x00000002);
        self.set_rip(entry_point);
        self.set_efer(0);
    }
    
    
    
    
    
    
    
    
    
    
    
    
    pub fn setup_long_mode(&mut self, entry_point: u64, guest_cr3: u64) {
        
        
        let fnn: u16 = 0x0A9B;
        
        let bft: u16 = 0x0C93;
        
        self.set_cs(0x08, fnn, 0xFFFFFFFF, 0);
        self.set_ds(0x10, bft, 0xFFFFFFFF, 0);
        self.set_es(0x10, bft, 0xFFFFFFFF, 0);
        self.set_fs(0x10, bft, 0xFFFFFFFF, 0);
        self.set_gs(0x10, bft, 0xFFFFFFFF, 0);
        self.set_ss(0x10, bft, 0xFFFFFFFF, 0);
        
        
        self.set_tr(0, 0x008B, 0xFFFF, 0);
        
        self.set_ldtr(0, 0x0082, 0xFFFF, 0);
        
        
        self.set_gdtr(23, 0x1000);
        self.set_idtr(0, 0);
        
        
        self.set_cr0(0x8001_0031); 
        
        
        self.set_cr3(guest_cr3);
        
        
        self.set_cr4(0x00000620); 
        
        
        self.set_efer(0x00001501); 
        
        self.set_rflags(0x00000002);
        self.set_rip(entry_point);
        
        
        self.set_cpl(0);
        
        self.set_dr6(0xFFFF0FF0);
        self.set_dr7(0x00000400);
        self.set_pat(0x0007040600070406);
    }
    
    
    
    pub fn setup_long_mode_for_linux(
        &mut self,
        entry_point: u64,
        stack_ptr: u64,
        guest_cr3: u64,
        gdt_base: u64,
        fyf: u32,
    ) {
        
        let fnn: u16 = 0x0A9B;
        
        let bft: u16 = 0x0C93;
        
        self.set_cs(0x08, fnn, 0xFFFFFFFF, 0);
        self.set_ds(0x10, bft, 0xFFFFFFFF, 0);
        self.set_es(0x10, bft, 0xFFFFFFFF, 0);
        self.set_fs(0x10, bft, 0xFFFFFFFF, 0);
        self.set_gs(0x10, bft, 0xFFFFFFFF, 0);
        self.set_ss(0x10, bft, 0xFFFFFFFF, 0);
        
        
        self.set_tr(0, 0x008B, 0x67, 0);
        
        self.set_ldtr(0, 0x0082, 0, 0);
        
        
        self.set_gdtr(fyf, gdt_base);
        self.set_idtr(0, 0);
        
        
        self.set_cr0(0x8001_0033);
        
        
        self.set_cr3(guest_cr3);
        
        
        self.set_cr4(0x00000620);
        
        
        self.set_efer(0x00001501);
        
        self.set_rflags(0x00000002);
        self.set_rip(entry_point);
        self.set_rsp(stack_ptr);
        self.set_cpl(0);
        
        self.set_dr6(0xFFFF0FF0);
        self.set_dr7(0x00000400);
        self.set_pat(0x0007040600070406);
    }
    
    
    pub fn setup_basic_intercepts(&mut self) {
        
        let mqx = intercepts::Rh 
            | intercepts::Su 
            | intercepts::Alo
            | intercepts::Amq 
            | intercepts::Iu
            | intercepts::Zx;  
        
        let intercepts2 = intercepts2::Aro 
            | intercepts2::Arq
            | intercepts2::Wb    
            | intercepts2::Agh    
            | intercepts2::Abb   
            | intercepts2::Abe;
        
        self.set_intercepts1(mqx);
        self.set_intercepts2(intercepts2);
        
        
        self.set_cr_write_intercepts(0x19);  
    }
}


pub mod seg_attrib {
    
    pub fn qgj(access: u16, granularity: bool, fu: bool, long: bool) -> u16 {
        let mut aga = access & 0xFF;
        if granularity { aga |= 1 << 8; }
        if fu { aga |= 1 << 10; }
        if long { aga |= 1 << 9; }
        aga
    }
    
    pub const Bg: u16 = 1 << 7;
    pub const DNI_: u16 = 0 << 5;
    pub const DNJ_: u16 = 3 << 5;
    pub const EKW_: u16 = 1 << 4;
    pub const EMP_: u16 = 0x0A;
    pub const EMQ_: u16 = 0x02;
    pub const Aws: u16 = 1 << 8;
    pub const Att: u16 = 1 << 10;
    pub const Ayn: u16 = 1 << 9;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn rab() {
        assert_eq!(size_of::<Vmcb>(), 4096);
    }
}
