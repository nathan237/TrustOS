




use alloc::string::String;
use alloc::vec::Vec;






pub mod wl_display {
    pub const Cmo: u16 = 0;
    pub const DMY_: u16 = 1;
    
    
    pub const Sf: u16 = 0;
    pub const DJA_: u16 = 1;
}


pub mod wl_registry {
    pub const Bch: u16 = 0;
    
    
    pub const Bhr: u16 = 0;
    pub const DNJ_: u16 = 1;
}


pub mod wl_compositor {
    pub const DHO_: u16 = 0;
    pub const DHN_: u16 = 1;
}


pub mod wl_surface {
    pub const Aau: u16 = 0;
    pub const Crg: u16 = 1;
    pub const Ctw: u16 = 2;
    pub const Cde: u16 = 3;
    pub const EGA_: u16 = 4;
    pub const EFT_: u16 = 5;
    pub const Csl: u16 = 6;
    pub const EFP_: u16 = 7;
    pub const EFO_: u16 = 8;
    pub const DIM_: u16 = 9;
    pub const Ddh: u16 = 10;
    
    
    pub const Bfj: u16 = 0;
    pub const Bkk: u16 = 1;
    pub const EBA_: u16 = 2;
    pub const EBB_: u16 = 3;
}


pub mod wl_shm {
    pub const DHL_: u16 = 0;
    
    
    pub const Cwx: u16 = 0;
}


pub mod wl_shm_pool {
    pub const DHK_: u16 = 0;
    pub const Aau: u16 = 1;
    pub const Cjt: u16 = 2;
}


pub mod wl_buffer {
    pub const Aau: u16 = 0;
    
    
    pub const Axj: u16 = 0;
}


pub mod wl_seat {
    pub const DMV_: u16 = 0;
    pub const DMT_: u16 = 1;
    pub const DNC_: u16 = 2;
    pub const Axj: u16 = 3;
    
    
    pub const Ig: u16 = 0;
    pub const Dcr: u16 = 1;
}


pub mod wl_pointer {
    pub const EFR_: u16 = 0;
    pub const Axj: u16 = 1;
    
    
    pub const Bfj: u16 = 0;
    pub const Bkk: u16 = 1;
    pub const Dby: u16 = 2;
    pub const Crv: u16 = 3;
    pub const Crh: u16 = 4;
    pub const Cde: u16 = 5;
    pub const DDG_: u16 = 6;
    pub const DDH_: u16 = 7;
    pub const DDF_: u16 = 8;
}


pub mod wl_keyboard {
    pub const Axj: u16 = 0;
    
    
    pub const Dar: u16 = 0;
    pub const Bfj: u16 = 1;
    pub const Bkk: u16 = 2;
    pub const Dao: u16 = 3;
    pub const Dbx: u16 = 4;
    pub const ECK_: u16 = 5;
}


pub mod xdg_wm_base {
    pub const Aau: u16 = 0;
    pub const DHM_: u16 = 1;
    pub const DND_: u16 = 2;
    pub const Ddv: u16 = 3;
    
    
    pub const Dds: u16 = 0;
}


pub mod xdg_surface {
    pub const Aau: u16 = 0;
    pub const DNB_: u16 = 1;
    pub const DMW_: u16 = 2;
    pub const EGE_: u16 = 3;
    pub const DCC_: u16 = 4;
    
    
    pub const Bza: u16 = 0;
}


pub mod xdg_toplevel {
    pub const Aau: u16 = 0;
    pub const EGB_: u16 = 1;
    pub const EGD_: u16 = 2;
    pub const EFN_: u16 = 3;
    pub const EGI_: u16 = 4;
    pub const Dca: u16 = 5;
    pub const Cjt: u16 = 6;
    pub const EFV_: u16 = 7;
    pub const EFY_: u16 = 8;
    pub const EFU_: u16 = 9;
    pub const EJN_: u16 = 10;
    pub const EFS_: u16 = 11;
    pub const EJM_: u16 = 12;
    pub const EFX_: u16 = 13;
    
    
    pub const Bza: u16 = 0;
    pub const App: u16 = 1;
    pub const DGL_: u16 = 2;
}






#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WlMessageHeader {
    
    pub fpl: u32,
    
    pub gok: u32,
}

impl WlMessageHeader {
    pub fn new(fpl: u32, opcode: u16, aw: u16) -> Self {
        Self {
            fpl,
            gok: (aw as u32) << 16 | opcode as u32,
        }
    }
    
    pub fn opcode(&self) -> u16 {
        (self.gok & 0xFFFF) as u16
    }
    
    pub fn aw(&self) -> u16 {
        (self.gok >> 16) as u16
    }
}


#[derive(Debug, Clone)]
pub struct Afu {
    pub dh: WlMessageHeader,
    pub ew: Vec<u8>,
}

impl Afu {
    pub fn new(fpl: u32, opcode: u16) -> Self {
        Self {
            dh: WlMessageHeader::new(fpl, opcode, 8),
            ew: Vec::new(),
        }
    }
    
    pub fn zwf(fpl: u32, opcode: u16, ew: Vec<u8>) -> Self {
        let aw = 8 + ew.len() as u16;
        Self {
            dh: WlMessageHeader::new(fpl, opcode, aw),
            ew,
        }
    }
    
    
    pub fn oyl(&mut self, bn: u32) {
        self.ew.bk(&bn.mlj());
        self.moe();
    }
    
    
    pub fn zgx(&mut self, bn: i32) {
        self.ew.bk(&bn.mlj());
        self.moe();
    }
    
    
    pub fn zgz(&mut self, e: &str) {
        let len = e.len() as u32 + 1; 
        self.oyl(len);
        self.ew.bk(e.as_bytes());
        self.ew.push(0); 
        
        while self.ew.len() % 4 != 0 {
            self.ew.push(0);
        }
        self.moe();
    }
    
    
    pub fn zgy(&mut self, ad: u32) {
        self.oyl(ad);
    }
    
    fn moe(&mut self) {
        let aw = 8 + self.ew.len() as u16;
        self.dh.gok = (aw as u32) << 16 | (self.dh.gok & 0xFFFF);
    }
    
    
    pub fn pts(&self) -> Vec<u8> {
        let mut bf = Vec::fc(8 + self.ew.len());
        bf.bk(&self.dh.fpl.mlj());
        bf.bk(&self.dh.gok.mlj());
        bf.bk(&self.ew);
        bf
    }
}






#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dlz {
    Apd = 0,
    Aor = 1,
    
}






pub mod SeatCapability {
    pub const Ddu: u32 = 1;
    pub const Dap: u32 = 2;
    pub const Djn: u32 = 4;
}






#[derive(Debug, Clone)]
pub struct Op {
    pub j: u32,
    pub akf: String,
    pub dk: u32,
}


pub fn kys() -> Vec<Op> {
    alloc::vec![
        Op { j: 1, akf: String::from("wl_compositor"), dk: 5 },
        Op { j: 2, akf: String::from("wl_shm"), dk: 1 },
        Op { j: 3, akf: String::from("wl_seat"), dk: 8 },
        Op { j: 4, akf: String::from("wl_output"), dk: 4 },
        Op { j: 5, akf: String::from("xdg_wm_base"), dk: 5 },
    ]
}
