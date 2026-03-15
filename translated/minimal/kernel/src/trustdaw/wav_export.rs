




use alloc::vec::Vec;
use alloc::vec;


const AJV_: usize = 44;





pub fn tcs(un: &[i16], auy: u32, lq: u16) -> Vec<u8> {
    let cpv = un.len() * 2; 
    let yy = AJV_ + cpv;

    let mut bi = vec![0u8; yy];

    
    
    bi[0] = b'R';
    bi[1] = b'I';
    bi[2] = b'F';
    bi[3] = b'F';
    
    let aiw = (yy - 8) as u32;
    bi[4..8].dg(&aiw.ho());
    
    bi[8] = b'W';
    bi[9] = b'A';
    bi[10] = b'V';
    bi[11] = b'E';

    
    
    bi[12] = b'f';
    bi[13] = b'm';
    bi[14] = b't';
    bi[15] = b' ';
    
    bi[16..20].dg(&16u32.ho());
    
    bi[20..22].dg(&1u16.ho());
    
    bi[22..24].dg(&lq.ho());
    
    bi[24..28].dg(&auy.ho());
    
    let quy = auy * lq as u32 * 2; 
    bi[28..32].dg(&quy.ho());
    
    let qqh = lq * 2;
    bi[32..34].dg(&qqh.ho());
    
    bi[34..36].dg(&16u16.ho());

    
    
    bi[36] = b'd';
    bi[37] = b'a';
    bi[38] = b't';
    bi[39] = b'a';
    
    bi[40..44].dg(&(cpv as u32).ho());

    
    for (a, &yr) in un.iter().cf() {
        let l = AJV_ + a * 2;
        bi[l..l + 2].dg(&yr.ho());
    }

    bi
}


pub fn hio(path: &str, un: &[i16], auy: u32, lq: u16) -> Result<usize, &'static str> {
    if un.is_empty() {
        return Err("No audio data to export");
    }

    let bxv = tcs(un, auy, lq);
    let aw = bxv.len();

    
    crate::vfs::ns(path, &bxv)
        .jd(|_| "Failed to write WAV file to VFS")?;

    crate::serial_println!("[TRUSTDAW] Exported WAV: {} ({} bytes, {} samples, {}Hz {}ch)",
        path, aw, un.len(), auy, lq);

    Ok(aw)
}


pub fn ypp(uwm: usize) -> usize {
    AJV_ + uwm * 2 
}


pub fn shk(evo: usize, auy: u32, lq: u16) -> (u32, u32) {
    
    let vj = evo / lq as usize;
    let alu = (vj as u64 * 1000) / auy as u64;
    let dvm = (alu / 1000) as u32;
    let jn = (alu % 1000) as u32;
    (dvm, jn)
}
