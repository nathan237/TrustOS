




use alloc::string::String;
use alloc::vec::Vec;






pub mod wl_display {
    pub const Aqc: u16 = 0;
    pub const DQS_: u16 = 1;
    
    
    pub const Hr: u16 = 0;
    pub const DMP_: u16 = 1;
}


pub mod wl_registry {
    pub const Wm: u16 = 0;
    
    
    pub const Zd: u16 = 0;
    pub const DRD_: u16 = 1;
}


pub mod wl_compositor {
    pub const DLH_: u16 = 0;
    pub const DLG_: u16 = 1;
}


pub mod wl_surface {
    pub const Lj: u16 = 0;
    pub const Asr: u16 = 1;
    pub const Ats: u16 = 2;
    pub const Ajy: u16 = 3;
    pub const EJT_: u16 = 4;
    pub const EJM_: u16 = 5;
    pub const Atj: u16 = 6;
    pub const EJI_: u16 = 7;
    pub const EJH_: u16 = 8;
    pub const DMB_: u16 = 9;
    pub const Azu: u16 = 10;
    
    
    pub const Yc: u16 = 0;
    pub const Aan: u16 = 1;
    pub const EER_: u16 = 2;
    pub const EES_: u16 = 3;
}


pub mod wl_shm {
    pub const DLE_: u16 = 0;
    
    
    pub const Avz: u16 = 0;
}


pub mod wl_shm_pool {
    pub const DLD_: u16 = 0;
    pub const Lj: u16 = 1;
    pub const Aob: u16 = 2;
}


pub mod wl_buffer {
    pub const Lj: u16 = 0;
    
    
    pub const Uo: u16 = 0;
}


pub mod wl_seat {
    pub const DQP_: u16 = 0;
    pub const DQN_: u16 = 1;
    pub const DQW_: u16 = 2;
    pub const Uo: u16 = 3;
    
    
    pub const Dl: u16 = 0;
    pub const Azo: u16 = 1;
}


pub mod wl_pointer {
    pub const EJK_: u16 = 0;
    pub const Uo: u16 = 1;
    
    
    pub const Yc: u16 = 0;
    pub const Aan: u16 = 1;
    pub const Azd: u16 = 2;
    pub const Atb: u16 = 3;
    pub const Ass: u16 = 4;
    pub const Ajy: u16 = 5;
    pub const DHA_: u16 = 6;
    pub const DHB_: u16 = 7;
    pub const DGZ_: u16 = 8;
}


pub mod wl_keyboard {
    pub const Uo: u16 = 0;
    
    
    pub const Ayf: u16 = 0;
    pub const Yc: u16 = 1;
    pub const Aan: u16 = 2;
    pub const Ayc: u16 = 3;
    pub const Azc: u16 = 4;
    pub const EGA_: u16 = 5;
}


pub mod xdg_wm_base {
    pub const Lj: u16 = 0;
    pub const DLF_: u16 = 1;
    pub const DQX_: u16 = 2;
    pub const Baf: u16 = 3;
    
    
    pub const Bac: u16 = 0;
}


pub mod xdg_surface {
    pub const Lj: u16 = 0;
    pub const DQV_: u16 = 1;
    pub const DQQ_: u16 = 2;
    pub const EJX_: u16 = 3;
    pub const DFX_: u16 = 4;
    
    
    pub const Aht: u16 = 0;
}


pub mod xdg_toplevel {
    pub const Lj: u16 = 0;
    pub const EJU_: u16 = 1;
    pub const EJW_: u16 = 2;
    pub const EJG_: u16 = 3;
    pub const EKB_: u16 = 4;
    pub const Azf: u16 = 5;
    pub const Aob: u16 = 6;
    pub const EJO_: u16 = 7;
    pub const EJR_: u16 = 8;
    pub const EJN_: u16 = 9;
    pub const ENC_: u16 = 10;
    pub const EJL_: u16 = 11;
    pub const ENB_: u16 = 12;
    pub const EJQ_: u16 = 13;
    
    
    pub const Aht: u16 = 0;
    pub const Rf: u16 = 1;
    pub const DKE_: u16 = 2;
}






#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WlMessageHeader {
    
    pub object_id: u32,
    
    pub opcode_size: u32,
}

impl WlMessageHeader {
    pub fn new(object_id: u32, opcode: u16, size: u16) -> Self {
        Self {
            object_id,
            opcode_size: (size as u32) << 16 | opcode as u32,
        }
    }
    
    pub fn opcode(&self) -> u16 {
        (self.opcode_size & 0xFFFF) as u16
    }
    
    pub fn size(&self) -> u16 {
        (self.opcode_size >> 16) as u16
    }
}


#[derive(Debug, Clone)]
pub struct Ny {
    pub header: WlMessageHeader,
    pub payload: Vec<u8>,
}

impl Ny {
    pub fn new(object_id: u32, opcode: u16) -> Self {
        Self {
            header: WlMessageHeader::new(object_id, opcode, 8),
            payload: Vec::new(),
        }
    }
    
    pub fn rcr(object_id: u32, opcode: u16, payload: Vec<u8>) -> Self {
        let size = 8 + payload.len() as u16;
        Self {
            header: WlMessageHeader::new(object_id, opcode, size),
            payload,
        }
    }
    
    
    pub fn push_u32(&mut self, value: u32) {
        self.payload.extend_from_slice(&value.to_ne_bytes());
        self.update_size();
    }
    
    
    pub fn qrj(&mut self, value: i32) {
        self.payload.extend_from_slice(&value.to_ne_bytes());
        self.update_size();
    }
    
    
    pub fn qrl(&mut self, j: &str) {
        let len = j.len() as u32 + 1; 
        self.push_u32(len);
        self.payload.extend_from_slice(j.as_bytes());
        self.payload.push(0); 
        
        while self.payload.len() % 4 != 0 {
            self.payload.push(0);
        }
        self.update_size();
    }
    
    
    pub fn qrk(&mut self, id: u32) {
        self.push_u32(id);
    }
    
    fn update_size(&mut self) {
        let size = 8 + self.payload.len() as u16;
        self.header.opcode_size = (size as u32) << 16 | (self.header.opcode_size & 0xFFFF);
    }
    
    
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8 + self.payload.len());
        bytes.extend_from_slice(&self.header.object_id.to_ne_bytes());
        bytes.extend_from_slice(&self.header.opcode_size.to_ne_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}






#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bfn {
    Argb8888 = 0,
    Xrgb8888 = 1,
    
}






pub mod SeatCapability {
    pub const Bae: u32 = 1;
    pub const Ayd: u32 = 2;
    pub const Bdx: u32 = 4;
}






#[derive(Debug, Clone)]
pub struct Gd {
    pub name: u32,
    pub interface: String,
    pub version: u32,
}


pub fn fys() -> Vec<Gd> {
    alloc::vec![
        Gd { name: 1, interface: String::from("wl_compositor"), version: 5 },
        Gd { name: 2, interface: String::from("wl_shm"), version: 1 },
        Gd { name: 3, interface: String::from("wl_seat"), version: 8 },
        Gd { name: 4, interface: String::from("wl_output"), version: 4 },
        Gd { name: 5, interface: String::from("xdg_wm_base"), version: 5 },
    ]
}
