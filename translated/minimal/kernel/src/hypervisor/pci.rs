


























#[derive(Debug, Clone)]
pub struct PciBus {
    
    pub config_addr: u32,
    
    pub host_bridge: [u8; 256],
    
    pub isa_bridge: [u8; 256],
    
    pub virtio_console: [u8; 256],
    
    pub virtio_blk: [u8; 256],
}

impl Default for PciBus {
    fn default() -> Self {
        let mut bus = Self {
            config_addr: 0,
            host_bridge: [0u8; 256],
            isa_bridge: [0u8; 256],
            virtio_console: [0u8; 256],
            virtio_blk: [0u8; 256],
        };
        bus.init_host_bridge();
        bus.init_isa_bridge();
        bus.init_virtio_console();
        bus.init_virtio_blk();
        bus
    }
}


fn zq(config: &mut [u8], offset: usize, val: u16) {
    let bytes = val.to_le_bytes();
    config[offset] = bytes[0];
    config[offset + 1] = bytes[1];
}


fn ffi(config: &mut [u8], offset: usize, val: u32) {
    let bytes = val.to_le_bytes();
    config[offset] = bytes[0];
    config[offset + 1] = bytes[1];
    config[offset + 2] = bytes[2];
    config[offset + 3] = bytes[3];
}


fn ock(config: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        config[offset],
        config[offset + 1],
        config[offset + 2],
        config[offset + 3],
    ])
}


mod regs {
    pub const ZJ_: usize      = 0x00; 
    pub const TX_: usize      = 0x02; 
    pub const Lg: usize        = 0x04; 
    pub const Fz: usize         = 0x06; 
    pub const XZ_: usize    = 0x08; 
    pub const XS_: usize        = 0x09; 
    pub const Ng: usize       = 0x0A; 
    pub const NC_: usize     = 0x0B; 
    pub const BOL_: usize     = 0x0C; 
    pub const CHB_: usize  = 0x0D; 
    pub const VC_: usize    = 0x0E; 
    pub const Asy: usize           = 0x0F; 
    pub const Wl: usize           = 0x10; 
    pub const Asw: usize           = 0x14; 
    pub const YU_: usize = 0x2C; 
    pub const YT_: usize   = 0x2E; 
    pub const Dl: usize   = 0x34; 
    pub const AFE_: usize = 0x3C; 
    pub const AZD_: usize  = 0x3D; 
}

impl PciBus {
    
    
    
    
    
    
    fn init_host_bridge(&mut self) {
        let c = &mut self.host_bridge;
        
        
        zq(c, regs::ZJ_, 0x8086);
        
        zq(c, regs::TX_, 0x1237);
        
        zq(c, regs::Lg, 0x0006);
        
        zq(c, regs::Fz, 0x0000);
        
        c[regs::XZ_] = 0x02;
        
        c[regs::XS_] = 0x00;
        c[regs::Ng] = 0x00;
        c[regs::NC_] = 0x06;
        
        c[regs::VC_] = 0x00;
        
        zq(c, regs::YU_, 0x8086);
        zq(c, regs::YT_, 0x1237);
    }
    
    
    
    
    
    
    
    fn init_isa_bridge(&mut self) {
        let c = &mut self.isa_bridge;
        
        
        zq(c, regs::ZJ_, 0x8086);
        
        zq(c, regs::TX_, 0x7000);
        
        zq(c, regs::Lg, 0x0007);
        
        zq(c, regs::Fz, 0x0200); 
        
        c[regs::XZ_] = 0x00;
        
        c[regs::XS_] = 0x00;
        c[regs::Ng] = 0x01;
        c[regs::NC_] = 0x06;
        
        c[regs::VC_] = 0x80;
        
        zq(c, regs::YU_, 0x8086);
        zq(c, regs::YT_, 0x7000);
    }
    
    
    
    
    
    
    
    
    
    fn init_virtio_console(&mut self) {
        let c = &mut self.virtio_console;
        
        
        zq(c, regs::ZJ_, 0x1AF4);
        
        zq(c, regs::TX_, 0x1003);
        
        zq(c, regs::Lg, 0x0005);
        
        zq(c, regs::Fz, 0x0010); 
        
        c[regs::XZ_] = 0x00;
        
        c[regs::XS_] = 0x00;
        c[regs::Ng] = 0x80;
        c[regs::NC_] = 0x07;
        
        c[regs::VC_] = 0x00;
        
        ffi(c, regs::Wl, 0xC001);
        
        zq(c, regs::YU_, 0x1AF4);
        zq(c, regs::YT_, 0x0003);
        
        c[regs::AFE_] = 17;
        c[regs::AZD_] = 0x01; 
    }
    
    
    
    
    
    
    
    
    
    fn init_virtio_blk(&mut self) {
        let c = &mut self.virtio_blk;
        
        
        zq(c, regs::ZJ_, 0x1AF4);
        
        zq(c, regs::TX_, 0x1001);
        
        zq(c, regs::Lg, 0x0005);
        
        zq(c, regs::Fz, 0x0010); 
        
        c[regs::XZ_] = 0x00;
        
        c[regs::XS_] = 0x00;
        c[regs::Ng] = 0x80;
        c[regs::NC_] = 0x01;
        
        c[regs::VC_] = 0x00;
        
        ffi(c, regs::Wl, 0xC041);
        
        zq(c, regs::YU_, 0x1AF4);
        zq(c, regs::YT_, 0x0002);
        
        c[regs::AFE_] = 18;
        c[regs::AZD_] = 0x01; 
    }
    
    
    pub fn write_config_address(&mut self, value: u32) {
        self.config_addr = value;
    }
    
    
    pub fn read_config_address(&self) -> u32 {
        self.config_addr
    }
    
    
    fn parse_address(&self) -> (bool, u8, u8, u8, u8) {
        let addr = self.config_addr;
        let enable = (addr >> 31) & 1 != 0;
        let bus = ((addr >> 16) & 0xFF) as u8;
        let device = ((addr >> 11) & 0x1F) as u8;
        let function = ((addr >> 8) & 0x7) as u8;
        let register = (addr & 0xFC) as u8; 
        (enable, bus, device, function, register)
    }
    
    
    fn get_config_space(&self, bus: u8, device: u8, function: u8) -> Option<&[u8; 256]> {
        match (bus, device, function) {
            (0, 0, 0) => Some(&self.host_bridge),
            (0, 1, 0) => Some(&self.isa_bridge),
            (0, 2, 0) => Some(&self.virtio_console),
            (0, 3, 0) => Some(&self.virtio_blk),
            _ => None,
        }
    }
    
    
    fn get_config_space_mut(&mut self, bus: u8, device: u8, function: u8) -> Option<&mut [u8; 256]> {
        match (bus, device, function) {
            (0, 0, 0) => Some(&mut self.host_bridge),
            (0, 1, 0) => Some(&mut self.isa_bridge),
            (0, 2, 0) => Some(&mut self.virtio_console),
            (0, 3, 0) => Some(&mut self.virtio_blk),
            _ => None,
        }
    }
    
    
    
    pub fn read_config_data(&self, offset: u8) -> u32 {
        let (enable, bus, device, function, register) = self.parse_address();
        
        if !enable {
            return 0xFFFF_FFFF;
        }
        
        if let Some(config) = self.get_config_space(bus, device, function) {
            let abg = (register as usize) + (offset as usize);
            if abg + 4 <= 256 {
                ock(config, abg & 0xFC) 
            } else {
                0xFFFF_FFFF
            }
        } else {
            
            0xFFFF_FFFF
        }
    }
    
    
    pub fn write_config_data(&mut self, offset: u8, value: u32) {
        let (enable, bus, device, function, register) = self.parse_address();
        
        if !enable {
            return;
        }
        
        if let Some(config) = self.get_config_space_mut(bus, device, function) {
            let abg = register as usize;
            if abg + 4 <= 256 {
                
                match abg {
                    
                    0x00 => {}
                    
                    0x04 => {
                        
                        zq(config, regs::Lg, value as u16);
                        
                        let nmw = u16::from_le_bytes([config[0x06], config[0x07]]);
                        let ptg = (value >> 16) as u16;
                        zq(config, regs::Fz, nmw & !ptg);
                    }
                    
                    0x08 => {}
                    
                    0x0C => {
                        config[regs::BOL_] = value as u8;
                        config[regs::CHB_] = (value >> 8) as u8;
                    }
                    
                    0x10..=0x27 => {
                        ffi(config, abg, value);
                    }
                    
                    0x3C => {
                        config[regs::AFE_] = value as u8;
                    }
                    
                    _ => {
                        if abg + 4 <= 256 {
                            ffi(config, abg, value);
                        }
                    }
                }
            }
        }
    }
    
    
    pub fn qcq(&self) -> &'static str {
        "PCI Bus 0: [0:0.0] Host Bridge (440FX), [0:1.0] ISA Bridge (PIIX3), [0:2.0] VirtIO Console, [0:3.0] VirtIO Block"
    }
    
    
    pub fn device_exists(&self, bus: u8, device: u8, function: u8) -> bool {
        self.get_config_space(bus, device, function).is_some()
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn jma() {
        let bus = PciBus::default();
        
        assert!(bus.device_exists(0, 0, 0));
        
        assert!(bus.device_exists(0, 1, 0));
        
        assert!(!bus.device_exists(0, 2, 0));
        assert!(!bus.device_exists(1, 0, 0));
    }
    
    #[test]
    fn qzl() {
        let bus = PciBus::default();
        let vendor = u16::from_le_bytes([bus.host_bridge[0], bus.host_bridge[1]]);
        let device = u16::from_le_bytes([bus.host_bridge[2], bus.host_bridge[3]]);
        assert_eq!(vendor, 0x8086);
        assert_eq!(device, 0x1237);
    }
    
    #[test]
    fn qzp() {
        let bus = PciBus::default();
        assert_eq!(bus.isa_bridge[regs::NC_], 0x06);
        assert_eq!(bus.isa_bridge[regs::Ng], 0x01);
    }
    
    #[test]
    fn qzc() {
        let mut bus = PciBus::default();
        
        bus.write_config_address(0x8000_F800);
        let val = bus.read_config_data(0);
        assert_eq!(val, 0xFFFF_FFFF);
    }
    
    #[test]
    fn qzb() {
        let mut bus = PciBus::default();
        
        bus.write_config_address(0x8000_0000);
        let val = bus.read_config_data(0);
        assert_eq!(val & 0xFFFF, 0x8086);       
        assert_eq!((val >> 16) & 0xFFFF, 0x1237); 
    }
    
    #[test]
    fn qza() {
        let mut bus = PciBus::default();
        
        bus.write_config_address(0x0000_0000);
        let val = bus.read_config_data(0);
        assert_eq!(val, 0xFFFF_FFFF);
    }
    
    #[test]
    fn qyy() {
        let mut bus = PciBus::default();
        
        bus.write_config_address(0x8000_0010);
        
        bus.write_config_data(0, 0xFFFF_FFFF);
        
        let val = bus.read_config_data(0);
        
        assert_eq!(val, 0xFFFF_FFFF);
    }
}
