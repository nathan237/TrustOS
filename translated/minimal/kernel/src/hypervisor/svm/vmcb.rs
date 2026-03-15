




use core::mem::size_of;


pub mod control_offsets {
    pub const BPT_: usize = 0x000;
    pub const BPU_: usize = 0x004;
    pub const DKQ_: usize = 0x008;
    pub const DKR_: usize = 0x00C;
    pub const BUP_: usize = 0x010;
    pub const CBT_: usize = 0x014;
    pub const CBU_: usize = 0x018;
    pub const DZB_: usize = 0x03C;
    pub const DZA_: usize = 0x03E;
    pub const CCG_: usize = 0x040;
    pub const CHB_: usize = 0x048;
    pub const CYY_: usize = 0x050;
    pub const ATX_: usize = 0x058;
    pub const BHG_: usize = 0x05C;
    pub const DBK_: usize = 0x060;
    pub const EKK_: usize = 0x061;
    pub const EKH_: usize = 0x062;
    pub const EKI_: usize = 0x063;
    pub const EKJ_: usize = 0x064;
    pub const DRG_: usize = 0x068;
    pub const Abm: usize = 0x070;
    pub const Abn: usize = 0x078;
    pub const Abo: usize = 0x080;
    pub const Cvt: usize = 0x088;
    pub const VU_: usize = 0x090;
    pub const DDC_: usize = 0x098;
    pub const DNE_: usize = 0x0A0;
    pub const GJ_: usize = 0x0A8;
    pub const BBU_: usize = 0x0B0;
    pub const DSK_: usize = 0x0B8;
    pub const BIM_: usize = 0x0C0;
    pub const AFY_: usize = 0x0C8;
    pub const BLN_: usize = 0x0D0;
    pub const BYN_: usize = 0x0D1;
    pub const DDB_: usize = 0x0E0;
    pub const DDD_: usize = 0x0F0;
    pub const DDE_: usize = 0x0F8;
    pub const EKB_: usize = 0x108;
}


pub mod state_offsets {
    pub const P: usize = 0x400;
    
    pub const SZ_: usize = P + 0x000;
    pub const BUJ_: usize = P + 0x002;
    pub const BUL_: usize = P + 0x004;
    pub const BUK_: usize = P + 0x008;
    
    pub const JU_: usize = P + 0x010;
    pub const AAX_: usize = P + 0x012;
    pub const AAY_: usize = P + 0x014;
    pub const SJ_: usize = P + 0x018;
    
    pub const XH_: usize = P + 0x020;
    pub const AIC_: usize = P + 0x022;
    pub const AIE_: usize = P + 0x024;
    pub const AID_: usize = P + 0x028;
    
    pub const MV_: usize = P + 0x030;
    pub const BSF_: usize = P + 0x032;
    pub const BSH_: usize = P + 0x034;
    pub const BSG_: usize = P + 0x038;
    
    pub const ACL_: usize = P + 0x040;
    pub const BVO_: usize = P + 0x042;
    pub const BVP_: usize = P + 0x044;
    pub const ACK_: usize = P + 0x048;
    
    pub const ADF_: usize = P + 0x050;
    pub const BYH_: usize = P + 0x052;
    pub const BYI_: usize = P + 0x054;
    pub const ADE_: usize = P + 0x058;
    
    pub const DMQ_: usize = P + 0x060;
    pub const DMN_: usize = P + 0x062;
    pub const BWQ_: usize = P + 0x064;
    pub const BWP_: usize = P + 0x068;
    
    pub const CEJ_: usize = P + 0x070;
    pub const CEG_: usize = P + 0x072;
    pub const CEI_: usize = P + 0x074;
    pub const CEH_: usize = P + 0x078;
    
    pub const DQO_: usize = P + 0x080;
    pub const DQM_: usize = P + 0x082;
    pub const CBM_: usize = P + 0x084;
    pub const CBL_: usize = P + 0x088;
    
    pub const CYX_: usize = P + 0x090;
    pub const CYU_: usize = P + 0x092;
    pub const CYW_: usize = P + 0x094;
    pub const CYV_: usize = P + 0x098;
    
    pub const Agx: usize = P + 0x0CB;
    pub const Lh: usize = P + 0x0D0;
    pub const Vw: usize = P + 0x148;
    pub const Vv: usize = P + 0x150;
    pub const Vu: usize = P + 0x158;
    pub const Caq: usize = P + 0x160;
    pub const Cap: usize = P + 0x168;
    pub const Kv: usize = P + 0x170;
    pub const Aw: usize = P + 0x178;
    pub const Hc: usize = P + 0x1D8;
    pub const Me: usize = P + 0x1F8;
    pub const Bsb: usize = P + 0x200;
    pub const Bko: usize = P + 0x208;
    pub const Bde: usize = P + 0x210;
    pub const Brh: usize = P + 0x218;
    pub const AXR_: usize = P + 0x220;
    pub const BGT_: usize = P + 0x228;
    pub const BGV_: usize = P + 0x230;
    pub const BGU_: usize = P + 0x238;
    pub const Agy: usize = P + 0x240;
    pub const Awo: usize = P + 0x268;
    pub const Cty: usize = P + 0x270;
    pub const DDW_: usize = P + 0x278;
    pub const DDX_: usize = P + 0x280;
    pub const Dax: usize = P + 0x288;
    pub const Day: usize = P + 0x290;
}


pub mod intercepts {
    pub const Cyv: u32 = 1 << 0;
    pub const Dcs: u32 = 1 << 1;
    pub const Dhq: u32 = 1 << 2;
    pub const Dm: u32 = 1 << 3;
    pub const Dkx: u32 = 1 << 4;
    pub const DHJ_: u32 = 1 << 5;
    pub const DQN_: u32 = 1 << 6;
    pub const DMP_: u32 = 1 << 7;
    pub const DSL_: u32 = 1 << 8;
    pub const EIY_: u32 = 1 << 9;
    pub const DQP_: u32 = 1 << 10;
    pub const DMR_: u32 = 1 << 11;
    pub const DSN_: u32 = 1 << 12;
    pub const EIZ_: u32 = 1 << 13;
    pub const Cjj: u32 = 1 << 14;
    pub const Cji: u32 = 1 << 15;
    pub const Def: u32 = 1 << 16;
    pub const Ddw: u32 = 1 << 17;
    pub const Apr: u32 = 1 << 18;
    pub const Ckc: u32 = 1 << 19;
    pub const Czh: u32 = 1 << 20;
    pub const Did: u32 = 1 << 21;
    pub const Bje: u32 = 1 << 22;
    pub const Boi: u32 = 1 << 23;
    pub const Atl: u32 = 1 << 24;
    pub const Cfn: u32 = 1 << 25;
    pub const Cyy: u32 = 1 << 26;
    pub const Cfp: u32 = 1 << 27;
    pub const Chd: u32 = 1 << 28;
    pub const CXM_: u32 = 1 << 29;
    pub const DLR_: u32 = 1 << 30;
    pub const Uf: u32 = 1 << 31;
}


pub mod intercepts2 {
    pub const Cpg: u32 = 1 << 0;
    pub const Cpf: u32 = 1 << 1;
    pub const Dlb: u32 = 1 << 2;
    pub const Dlg: u32 = 1 << 3;
    pub const Dhy: u32 = 1 << 4;
    pub const Csk: u32 = 1 << 5;
    pub const Dhp: u32 = 1 << 6;
    pub const Cjk: u32 = 1 << 7;
    pub const Cyp: u32 = 1 << 8;
    pub const Bwf: u32 = 1 << 9;
    pub const Blv: u32 = 1 << 10;
    pub const Bly: u32 = 1 << 11;
    pub const DVA_: u32 = 1 << 12;
    pub const Bbf: u32 = 1 << 13;
}


pub mod clean_bits {
    pub const Cyu: u32 = 1 << 0;
    pub const Cfq: u32 = 1 << 1;
    pub const Bxk: u32 = 1 << 2;
    pub const Djo: u32 = 1 << 3;
    pub const Chs: u32 = 1 << 4;
    pub const Cso: u32 = 1 << 5;
    pub const Cub: u32 = 1 << 6;
    pub const Cuc: u32 = 1 << 7;
    pub const Dgm: u32 = 1 << 8;
    pub const Agy: u32 = 1 << 9;
    pub const Cgk: u32 = 1 << 10;
    pub const Bxm: u32 = 1 << 11;
}


pub mod tlb_control {
    pub const DJT_: u32 = 0;
    pub const DMA_: u32 = 1;
    pub const DMC_: u32 = 3;
    pub const DMB_: u32 = 7;
}


#[repr(C, align(4096))]
pub struct Vmcb {
    f: [u8; 4096],
}

impl Vmcb {
    
    pub const fn new() -> Self {
        Self { f: [0; 4096] }
    }
    
    
    #[inline]
    pub fn ady(&self, l: usize) -> u8 {
        self.f[l]
    }
    
    
    
    
    
    pub fn thy(&self) -> (usize, [u8; 15]) {
        let az = self.f[control_offsets::BLN_] as usize;
        let mut k = [0u8; 15];
        let bo = az.v(15);
        for a in 0..bo {
            k[a] = self.f[control_offsets::BYN_ + a];
        }
        (bo, k)
    }
    
    
    #[inline]
    pub fn alp(&self, l: usize) -> u16 {
        u16::dj([self.f[l], self.f[l + 1]])
    }
    
    
    #[inline]
    pub fn za(&self, l: usize) -> u32 {
        u32::dj([
            self.f[l],
            self.f[l + 1],
            self.f[l + 2],
            self.f[l + 3],
        ])
    }
    
    
    #[inline]
    pub fn aqi(&self, l: usize) -> u64 {
        u64::dj([
            self.f[l],
            self.f[l + 1],
            self.f[l + 2],
            self.f[l + 3],
            self.f[l + 4],
            self.f[l + 5],
            self.f[l + 6],
            self.f[l + 7],
        ])
    }
    
    
    #[inline]
    pub fn aqr(&mut self, l: usize, bn: u16) {
        let bf = bn.ho();
        self.f[l] = bf[0];
        self.f[l + 1] = bf[1];
    }
    
    
    #[inline]
    pub fn sx(&mut self, l: usize, bn: u32) {
        let bf = bn.ho();
        self.f[l] = bf[0];
        self.f[l + 1] = bf[1];
        self.f[l + 2] = bf[2];
        self.f[l + 3] = bf[3];
    }
    
    
    #[inline]
    pub fn tw(&mut self, l: usize, bn: u64) {
        let bf = bn.ho();
        for a in 0..8 {
            self.f[l + a] = bf[a];
        }
    }
    
    
    pub fn ki(&self) -> u64 {
        let ju = self as *const _ as u64;
        ju.nj(crate::memory::lr())
    }
    
    
    
    
    
    
    #[inline]
    pub fn cgx(&self, l: usize) -> u64 {
        self.aqi(l)
    }
    
    
    #[inline]
    pub fn elc(&mut self, l: usize, bn: u64) {
        self.tw(l, bn);
    }
    
    
    #[inline]
    pub fn xs(&self, l: usize) -> u64 {
        self.aqi(l)
    }
    
    
    #[inline]
    pub fn abz(&mut self, l: usize, bn: u64) {
        self.tw(l, bn);
    }
    
    
    
    
    
    
    pub fn zms(&mut self, hs: u32) {
        self.sx(control_offsets::BPT_, hs);
    }
    
    
    pub fn wim(&mut self, hs: u32) {
        self.sx(control_offsets::BPU_, hs);
    }
    
    
    pub fn zmx(&mut self, hs: u32) {
        self.sx(control_offsets::BUP_, hs);
    }
    
    
    pub fn wja(&mut self, hs: u32) {
        self.sx(control_offsets::CBT_, hs);
    }
    
    
    pub fn wjb(&mut self, hs: u32) {
        self.sx(control_offsets::CBU_, hs);
    }
    
    
    pub fn wjd(&mut self, ht: u64) {
        self.tw(control_offsets::CCG_, ht);
    }
    
    
    pub fn wjg(&mut self, ht: u64) {
        self.tw(control_offsets::CHB_, ht);
    }
    
    
    pub fn zns(&mut self, l: u64) {
        self.tw(control_offsets::CYY_, l);
    }
    
    
    pub fn znc(&mut self, ajv: u32) {
        self.sx(control_offsets::ATX_, ajv);
    }
    
    
    pub fn znr(&mut self, control: u32) {
        self.sx(control_offsets::BHG_, control);
    }
    
    
    pub fn nz(&self) -> u64 {
        self.aqi(control_offsets::Abm)
    }
    
    
    pub fn dqp(&self) -> u64 {
        self.aqi(control_offsets::Abn)
    }
    
    
    pub fn kum(&self) -> u64 {
        self.aqi(control_offsets::Abo)
    }
    
    
    pub fn yph(&mut self, lnp: u64) {
        self.tw(control_offsets::VU_, 1);
        self.tw(control_offsets::BBU_, lnp);
    }
    
    
    pub fn ymf(&mut self) {
        self.tw(control_offsets::VU_, 0);
    }
    
    
    pub fn clean_bits(&self) -> u32 {
        self.za(control_offsets::BIM_)
    }
    
    
    pub fn wik(&mut self, fs: u32) {
        self.sx(control_offsets::BIM_, fs);
    }
    
    
    pub fn aqa(&self) -> u64 {
        self.aqi(control_offsets::AFY_)
    }
    
    
    pub fn hnz(&mut self, wj: u8, bqo: u8, error_code: Option<u32>) {
        let mut id: u64 = (wj as u64) | ((bqo as u64) << 8) | (1u64 << 31); 
        if let Some(ec) = error_code {
            id |= 1u64 << 11; 
            id |= (ec as u64) << 32;
        }
        self.tw(control_offsets::GJ_, id);
    }
    
    
    
    
    
    
    pub fn jpg(&mut self, pc: u64) {
        self.tw(state_offsets::Aw, pc);
    }
    
    
    pub fn pc(&self) -> u64 {
        self.aqi(state_offsets::Aw)
    }
    
    
    pub fn wjn(&mut self, rsp: u64) {
        self.tw(state_offsets::Hc, rsp);
    }
    
    
    pub fn rsp(&self) -> u64 {
        self.aqi(state_offsets::Hc)
    }
    
    
    pub fn znp(&mut self, rax: u64) {
        self.tw(state_offsets::Me, rax);
    }
    
    
    pub fn rax(&self) -> u64 {
        self.aqi(state_offsets::Me)
    }
    
    
    pub fn jpf(&mut self, rflags: u64) {
        self.tw(state_offsets::Kv, rflags);
    }
    
    
    pub fn rflags(&self) -> u64 {
        self.aqi(state_offsets::Kv)
    }
    
    
    pub fn hzw(&mut self, akb: u64) {
        self.tw(state_offsets::Vu, akb);
    }
    
    
    pub fn akb(&self) -> u64 {
        self.aqi(state_offsets::Vu)
    }
    
    
    pub fn mei(&mut self, jm: u64) {
        self.tw(state_offsets::Vv, jm);
    }
    
    
    pub fn jm(&self) -> u64 {
        self.aqi(state_offsets::Vv)
    }
    
    
    pub fn mej(&mut self, cr4: u64) {
        self.tw(state_offsets::Vw, cr4);
    }
    
    
    pub fn cr4(&self) -> u64 {
        self.aqi(state_offsets::Vw)
    }
    
    
    pub fn jov(&mut self, efer: u64) {
        self.tw(state_offsets::Lh, efer);
    }
    
    
    pub fn efer(&self) -> u64 {
        self.aqi(state_offsets::Lh)
    }
    
    
    pub fn jor(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::JU_, bof);
        self.aqr(state_offsets::AAX_, bjg);
        self.sx(state_offsets::AAY_, ul);
        self.tw(state_offsets::SJ_, ar);
    }
    
    
    pub fn jou(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::MV_, bof);
        self.aqr(state_offsets::BSF_, bjg);
        self.sx(state_offsets::BSH_, ul);
        self.tw(state_offsets::BSG_, ar);
    }
    
    
    pub fn jow(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::SZ_, bof);
        self.aqr(state_offsets::BUJ_, bjg);
        self.sx(state_offsets::BUL_, ul);
        self.tw(state_offsets::BUK_, ar);
    }
    
    
    pub fn jpi(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::XH_, bof);
        self.aqr(state_offsets::AIC_, bjg);
        self.sx(state_offsets::AIE_, ul);
        self.tw(state_offsets::AID_, ar);
    }
    
    
    pub fn jox(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::ACL_, bof);
        self.aqr(state_offsets::BVO_, bjg);
        self.sx(state_offsets::BVP_, ul);
        self.tw(state_offsets::ACK_, ar);
    }
    
    
    pub fn joz(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::ADF_, bof);
        self.aqr(state_offsets::BYH_, bjg);
        self.sx(state_offsets::BYI_, ul);
        self.tw(state_offsets::ADE_, ar);
    }
    
    
    pub fn joy(&mut self, ul: u32, ar: u64) {
        self.sx(state_offsets::BWQ_, ul);
        self.tw(state_offsets::BWP_, ar);
    }
    
    
    pub fn jpa(&mut self, ul: u32, ar: u64) {
        self.sx(state_offsets::CBM_, ul);
        self.tw(state_offsets::CBL_, ar);
    }
    
    
    pub fn jpj(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::CYX_, bof);
        self.aqr(state_offsets::CYU_, bjg);
        self.sx(state_offsets::CYW_, ul);
        self.tw(state_offsets::CYV_, ar);
    }
    
    
    pub fn jpc(&mut self, bof: u16, bjg: u16, ul: u32, ar: u64) {
        self.aqr(state_offsets::CEJ_, bof);
        self.aqr(state_offsets::CEG_, bjg);
        self.sx(state_offsets::CEI_, ul);
        self.tw(state_offsets::CEH_, ar);
    }
    
    
    pub fn piu(&mut self, ipj: u8) {
        self.f[state_offsets::Agx] = ipj;
    }
    
    
    pub fn ipj(&self) -> u8 {
        self.f[state_offsets::Agx]
    }
    
    
    pub fn mel(&mut self, sav: u64) {
        self.tw(state_offsets::Cap, sav);
    }
    
    
    pub fn men(&mut self, saw: u64) {
        self.tw(state_offsets::Caq, saw);
    }
    
    
    pub fn mer(&mut self, pat: u64) {
        self.tw(state_offsets::Awo, pat);
    }
    
    
    
    
    
    
    pub fn jpk(&mut self) {
        
        let dgb: u16 = 0x0093; 
        let kjq: u16 = 0x009B; 
        
        
        self.jor(0xF000, kjq, 0xFFFF, 0xF0000);
        
        
        self.jou(0, dgb, 0xFFFF, 0);
        self.jow(0, dgb, 0xFFFF, 0);
        self.jox(0, dgb, 0xFFFF, 0);
        self.joz(0, dgb, 0xFFFF, 0);
        self.jpi(0, dgb, 0xFFFF, 0);
        
        
        self.jpj(0, 0x008B, 0xFFFF, 0);
        
        
        self.jpc(0, 0x0082, 0xFFFF, 0);
        
        
        self.joy(0xFFFF, 0);
        self.jpa(0x3FF, 0);  
        
        
        self.hzw(0x60000010);  
        
        
        self.jpf(0x00000002);
        
        
        self.jpg(0xFFF0);
        
        
        self.mel(0xFFFF0FF0);
        self.men(0x00000400);
        
        
        self.jov(0);
        
        
        self.mer(0x0007040600070406);
    }
    
    
    pub fn iab(&mut self, mi: u64) {
        let dgb: u16 = 0x00C3; 
        let kjq: u16 = 0x00CB; 
        
        self.jor(0x08, kjq, 0xFFFFFFFF, 0);
        self.jou(0x10, dgb, 0xFFFFFFFF, 0);
        self.jow(0x10, dgb, 0xFFFFFFFF, 0);
        self.jox(0x10, dgb, 0xFFFFFFFF, 0);
        self.joz(0x10, dgb, 0xFFFFFFFF, 0);
        self.jpi(0x10, dgb, 0xFFFFFFFF, 0);
        
        self.jpj(0x18, 0x008B, 0xFFFF, 0);
        self.jpc(0, 0x0082, 0xFFFF, 0);
        
        
        self.joy(23, 0x1000);
        
        self.jpa(0, 0);
        
        self.hzw(0x60000011);  
        self.jpf(0x00000002);
        self.jpg(mi);
        self.jov(0);
    }
    
    
    
    
    
    
    
    
    
    
    
    
    pub fn mfb(&mut self, mi: u64, bnd: u64) {
        
        
        let kjp: u16 = 0x0A9B;
        
        let dga: u16 = 0x0C93;
        
        self.jor(0x08, kjp, 0xFFFFFFFF, 0);
        self.jou(0x10, dga, 0xFFFFFFFF, 0);
        self.jow(0x10, dga, 0xFFFFFFFF, 0);
        self.jox(0x10, dga, 0xFFFFFFFF, 0);
        self.joz(0x10, dga, 0xFFFFFFFF, 0);
        self.jpi(0x10, dga, 0xFFFFFFFF, 0);
        
        
        self.jpj(0, 0x008B, 0xFFFF, 0);
        
        self.jpc(0, 0x0082, 0xFFFF, 0);
        
        
        self.joy(23, 0x1000);
        self.jpa(0, 0);
        
        
        self.hzw(0x8001_0031); 
        
        
        self.mei(bnd);
        
        
        self.mej(0x00000620); 
        
        
        self.jov(0x00001501); 
        
        self.jpf(0x00000002);
        self.jpg(mi);
        
        
        self.piu(0);
        
        self.mel(0xFFFF0FF0);
        self.men(0x00000400);
        self.mer(0x0007040600070406);
    }
    
    
    
    pub fn wld(
        &mut self,
        mi: u64,
        ahu: u64,
        bnd: u64,
        bun: u64,
        kxw: u32,
    ) {
        
        let kjp: u16 = 0x0A9B;
        
        let dga: u16 = 0x0C93;
        
        self.jor(0x08, kjp, 0xFFFFFFFF, 0);
        self.jou(0x10, dga, 0xFFFFFFFF, 0);
        self.jow(0x10, dga, 0xFFFFFFFF, 0);
        self.jox(0x10, dga, 0xFFFFFFFF, 0);
        self.joz(0x10, dga, 0xFFFFFFFF, 0);
        self.jpi(0x10, dga, 0xFFFFFFFF, 0);
        
        
        self.jpj(0, 0x008B, 0x67, 0);
        
        self.jpc(0, 0x0082, 0, 0);
        
        
        self.joy(kxw, bun);
        self.jpa(0, 0);
        
        
        self.hzw(0x8001_0033);
        
        
        self.mei(bnd);
        
        
        self.mej(0x00000620);
        
        
        self.jov(0x00001501);
        
        self.jpf(0x00000002);
        self.jpg(mi);
        self.wjn(ahu);
        self.piu(0);
        
        self.mel(0xFFFF0FF0);
        self.men(0x00000400);
        self.mer(0x0007040600070406);
    }
    
    
    pub fn wki(&mut self) {
        
        let tvo = intercepts::Apr 
            | intercepts::Atl 
            | intercepts::Cfp
            | intercepts::Chd 
            | intercepts::Uf
            | intercepts::Bje;  
        
        let intercepts2 = intercepts2::Cpf 
            | intercepts2::Cpg
            | intercepts2::Bbf    
            | intercepts2::Bwf    
            | intercepts2::Blv   
            | intercepts2::Bly;
        
        self.wja(tvo);
        self.wjb(intercepts2);
        
        
        self.wim(0x19);  
    }
}


pub mod seg_attrib {
    
    pub fn yro(vz: u16, hlw: bool, ng: bool, uie: bool) -> u16 {
        let mut bjg = vz & 0xFF;
        if hlw { bjg |= 1 << 8; }
        if ng { bjg |= 1 << 10; }
        if uie { bjg |= 1 << 9; }
        bjg
    }
    
    pub const Cz: u16 = 1 << 7;
    pub const DJU_: u16 = 0 << 5;
    pub const DJV_: u16 = 3 << 5;
    pub const EHF_: u16 = 1 << 4;
    pub const EJA_: u16 = 0x0A;
    pub const EJB_: u16 = 0x02;
    pub const Cxw: u16 = 1 << 8;
    pub const Ctx: u16 = 1 << 10;
    pub const Dbc: u16 = 1 << 9;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn zsh() {
        assert_eq!(size_of::<Vmcb>(), 4096);
    }
}
