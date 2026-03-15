


























#[derive(Debug, Clone)]
pub struct PciBus {
    
    pub dfe: u32,
    
    pub ery: [u8; 256],
    
    pub gkk: [u8; 256],
    
    pub virtio_console: [u8; 256],
    
    pub virtio_blk: [u8; 256],
}

impl Default for PciBus {
    fn default() -> Self {
        let mut aq = Self {
            dfe: 0,
            ery: [0u8; 256],
            gkk: [0u8; 256],
            virtio_console: [0u8; 256],
            virtio_blk: [0u8; 256],
        };
        aq.ttn();
        aq.ttr();
        aq.tuc();
        aq.tub();
        aq
    }
}


fn axk(config: &mut [u8], l: usize, ap: u16) {
    let bf = ap.ho();
    config[l] = bf[0];
    config[l + 1] = bf[1];
}


fn jxf(config: &mut [u8], l: usize, ap: u32) {
    let bf = ap.ho();
    config[l] = bf[0];
    config[l + 1] = bf[1];
    config[l + 2] = bf[2];
    config[l + 3] = bf[3];
}


fn vrk(config: &[u8], l: usize) -> u32 {
    u32::dj([
        config[l],
        config[l + 1],
        config[l + 2],
        config[l + 3],
    ])
}


mod regs {
    pub const YF_: usize      = 0x00; 
    pub const SQ_: usize      = 0x02; 
    pub const Aah: usize        = 0x04; 
    pub const Nz: usize         = 0x06; 
    pub const WS_: usize    = 0x08; 
    pub const WJ_: usize        = 0x09; 
    pub const Aeo: usize       = 0x0A; 
    pub const ME_: usize     = 0x0B; 
    pub const BLS_: usize     = 0x0C; 
    pub const CDS_: usize  = 0x0D; 
    pub const TU_: usize    = 0x0E; 
    pub const Crs: usize           = 0x0F; 
    pub const Bcg: usize           = 0x10; 
    pub const Crq: usize           = 0x14; 
    pub const XN_: usize = 0x2C; 
    pub const XM_: usize   = 0x2E; 
    pub const Ig: usize   = 0x34; 
    pub const ADO_: usize = 0x3C; 
    pub const AXC_: usize  = 0x3D; 
}

impl PciBus {
    
    
    
    
    
    
    fn ttn(&mut self) {
        let r = &mut self.ery;
        
        
        axk(r, regs::YF_, 0x8086);
        
        axk(r, regs::SQ_, 0x1237);
        
        axk(r, regs::Aah, 0x0006);
        
        axk(r, regs::Nz, 0x0000);
        
        r[regs::WS_] = 0x02;
        
        r[regs::WJ_] = 0x00;
        r[regs::Aeo] = 0x00;
        r[regs::ME_] = 0x06;
        
        r[regs::TU_] = 0x00;
        
        axk(r, regs::XN_, 0x8086);
        axk(r, regs::XM_, 0x1237);
    }
    
    
    
    
    
    
    
    fn ttr(&mut self) {
        let r = &mut self.gkk;
        
        
        axk(r, regs::YF_, 0x8086);
        
        axk(r, regs::SQ_, 0x7000);
        
        axk(r, regs::Aah, 0x0007);
        
        axk(r, regs::Nz, 0x0200); 
        
        r[regs::WS_] = 0x00;
        
        r[regs::WJ_] = 0x00;
        r[regs::Aeo] = 0x01;
        r[regs::ME_] = 0x06;
        
        r[regs::TU_] = 0x80;
        
        axk(r, regs::XN_, 0x8086);
        axk(r, regs::XM_, 0x7000);
    }
    
    
    
    
    
    
    
    
    
    fn tuc(&mut self) {
        let r = &mut self.virtio_console;
        
        
        axk(r, regs::YF_, 0x1AF4);
        
        axk(r, regs::SQ_, 0x1003);
        
        axk(r, regs::Aah, 0x0005);
        
        axk(r, regs::Nz, 0x0010); 
        
        r[regs::WS_] = 0x00;
        
        r[regs::WJ_] = 0x00;
        r[regs::Aeo] = 0x80;
        r[regs::ME_] = 0x07;
        
        r[regs::TU_] = 0x00;
        
        jxf(r, regs::Bcg, 0xC001);
        
        axk(r, regs::XN_, 0x1AF4);
        axk(r, regs::XM_, 0x0003);
        
        r[regs::ADO_] = 17;
        r[regs::AXC_] = 0x01; 
    }
    
    
    
    
    
    
    
    
    
    fn tub(&mut self) {
        let r = &mut self.virtio_blk;
        
        
        axk(r, regs::YF_, 0x1AF4);
        
        axk(r, regs::SQ_, 0x1001);
        
        axk(r, regs::Aah, 0x0005);
        
        axk(r, regs::Nz, 0x0010); 
        
        r[regs::WS_] = 0x00;
        
        r[regs::WJ_] = 0x00;
        r[regs::Aeo] = 0x80;
        r[regs::ME_] = 0x01;
        
        r[regs::TU_] = 0x00;
        
        jxf(r, regs::Bcg, 0xC041);
        
        axk(r, regs::XN_, 0x1AF4);
        axk(r, regs::XM_, 0x0002);
        
        r[regs::ADO_] = 18;
        r[regs::AXC_] = 0x01; 
    }
    
    
    pub fn dni(&mut self, bn: u32) {
        self.dfe = bn;
    }
    
    
    pub fn ozx(&self) -> u32 {
        self.dfe
    }
    
    
    fn otw(&self) -> (bool, u8, u8, u8, u8) {
        let ag = self.dfe;
        let aiy = (ag >> 31) & 1 != 0;
        let aq = ((ag >> 16) & 0xFF) as u8;
        let de = ((ag >> 11) & 0x1F) as u8;
        let gw = ((ag >> 8) & 0x7) as u8;
        let nw = (ag & 0xFC) as u8; 
        (aiy, aq, de, gw, nw)
    }
    
    
    fn nxx(&self, aq: u8, de: u8, gw: u8) -> Option<&[u8; 256]> {
        match (aq, de, gw) {
            (0, 0, 0) => Some(&self.ery),
            (0, 1, 0) => Some(&self.gkk),
            (0, 2, 0) => Some(&self.virtio_console),
            (0, 3, 0) => Some(&self.virtio_blk),
            _ => None,
        }
    }
    
    
    fn tdd(&mut self, aq: u8, de: u8, gw: u8) -> Option<&mut [u8; 256]> {
        match (aq, de, gw) {
            (0, 0, 0) => Some(&mut self.ery),
            (0, 1, 0) => Some(&mut self.gkk),
            (0, 2, 0) => Some(&mut self.virtio_console),
            (0, 3, 0) => Some(&mut self.virtio_blk),
            _ => None,
        }
    }
    
    
    
    pub fn duw(&self, l: u8) -> u32 {
        let (aiy, aq, de, gw, nw) = self.otw();
        
        if !aiy {
            return 0xFFFF_FFFF;
        }
        
        if let Some(config) = self.nxx(aq, de, gw) {
            let ban = (nw as usize) + (l as usize);
            if ban + 4 <= 256 {
                vrk(config, ban & 0xFC) 
            } else {
                0xFFFF_FFFF
            }
        } else {
            
            0xFFFF_FFFF
        }
    }
    
    
    pub fn mra(&mut self, l: u8, bn: u32) {
        let (aiy, aq, de, gw, nw) = self.otw();
        
        if !aiy {
            return;
        }
        
        if let Some(config) = self.tdd(aq, de, gw) {
            let ban = nw as usize;
            if ban + 4 <= 256 {
                
                match ban {
                    
                    0x00 => {}
                    
                    0x04 => {
                        
                        axk(config, regs::Aah, bn as u16);
                        
                        let uxv = u16::dj([config[0x06], config[0x07]]);
                        let xtg = (bn >> 16) as u16;
                        axk(config, regs::Nz, uxv & !xtg);
                    }
                    
                    0x08 => {}
                    
                    0x0C => {
                        config[regs::BLS_] = bn as u8;
                        config[regs::CDS_] = (bn >> 8) as u8;
                    }
                    
                    0x10..=0x27 => {
                        jxf(config, ban, bn);
                    }
                    
                    0x3C => {
                        config[regs::ADO_] = bn as u8;
                    }
                    
                    _ => {
                        if ban + 4 <= 256 {
                            jxf(config, ban, bn);
                        }
                    }
                }
            }
        }
    }
    
    
    pub fn ylr(&self) -> &'static str {
        "PCI Bus 0: [0:0.0] Host Bridge (440FX), [0:1.0] ISA Bridge (PIIX3), [0:2.0] VirtIO Console, [0:3.0] VirtIO Block"
    }
    
    
    pub fn dpx(&self, aq: u8, de: u8, gw: u8) -> bool {
        self.nxx(aq, de, gw).is_some()
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn psm() {
        let aq = PciBus::default();
        
        assert!(aq.dpx(0, 0, 0));
        
        assert!(aq.dpx(0, 1, 0));
        
        assert!(!aq.dpx(0, 2, 0));
        assert!(!aq.dpx(1, 0, 0));
    }
    
    #[test]
    fn zrr() {
        let aq = PciBus::default();
        let acs = u16::dj([aq.ery[0], aq.ery[1]]);
        let de = u16::dj([aq.ery[2], aq.ery[3]]);
        assert_eq!(acs, 0x8086);
        assert_eq!(de, 0x1237);
    }
    
    #[test]
    fn zrv() {
        let aq = PciBus::default();
        assert_eq!(aq.gkk[regs::ME_], 0x06);
        assert_eq!(aq.gkk[regs::Aeo], 0x01);
    }
    
    #[test]
    fn zri() {
        let mut aq = PciBus::default();
        
        aq.dni(0x8000_F800);
        let ap = aq.duw(0);
        assert_eq!(ap, 0xFFFF_FFFF);
    }
    
    #[test]
    fn zrh() {
        let mut aq = PciBus::default();
        
        aq.dni(0x8000_0000);
        let ap = aq.duw(0);
        assert_eq!(ap & 0xFFFF, 0x8086);       
        assert_eq!((ap >> 16) & 0xFFFF, 0x1237); 
    }
    
    #[test]
    fn zrg() {
        let mut aq = PciBus::default();
        
        aq.dni(0x0000_0000);
        let ap = aq.duw(0);
        assert_eq!(ap, 0xFFFF_FFFF);
    }
    
    #[test]
    fn zre() {
        let mut aq = PciBus::default();
        
        aq.dni(0x8000_0010);
        
        aq.mra(0, 0xFFFF_FFFF);
        
        let ap = aq.duw(0);
        
        assert_eq!(ap, 0xFFFF_FFFF);
    }
}
