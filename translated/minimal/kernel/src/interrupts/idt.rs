



use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;


pub const BSB_: u16 = 0;


const IZ_: usize = 4096 * 5; 

lazy_static! {
    
    static ref Za: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        
        
        tss.yyj[BSB_ as usize] = {
            static mut Cme: [u8; IZ_] = [0; IZ_];
            let ibo = VirtAddr::nwg(unsafe { &Cme });
            let ibm = ibo + IZ_ as u64;
            ibm
        };
        
        tss
    };
}


pub fn yub() -> &'static TaskStateSegment {
    &Za
}
