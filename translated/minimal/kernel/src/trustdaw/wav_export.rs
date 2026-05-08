




use alloc::vec::Vec;
use alloc::vec;


const ALQ_: usize = 44;





pub fn mcn(jo: &[i16], sample_rate: u32, channels: u16) -> Vec<u8> {
    let data_size = jo.len() * 2; 
    let file_size = ALQ_ + data_size;

    let mut buffer = vec![0u8; file_size];

    
    
    buffer[0] = b'R';
    buffer[1] = b'I';
    buffer[2] = b'F';
    buffer[3] = b'F';
    
    let rs = (file_size - 8) as u32;
    buffer[4..8].copy_from_slice(&rs.to_le_bytes());
    
    buffer[8] = b'W';
    buffer[9] = b'A';
    buffer[10] = b'V';
    buffer[11] = b'E';

    
    
    buffer[12] = b'f';
    buffer[13] = b'm';
    buffer[14] = b't';
    buffer[15] = b' ';
    
    buffer[16..20].copy_from_slice(&16u32.to_le_bytes());
    
    buffer[20..22].copy_from_slice(&1u16.to_le_bytes());
    
    buffer[22..24].copy_from_slice(&channels.to_le_bytes());
    
    buffer[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    
    let kgn = sample_rate * channels as u32 * 2; 
    buffer[28..32].copy_from_slice(&kgn.to_le_bytes());
    
    let kcp = channels * 2;
    buffer[32..34].copy_from_slice(&kcp.to_le_bytes());
    
    buffer[34..36].copy_from_slice(&16u16.to_le_bytes());

    
    
    buffer[36] = b'd';
    buffer[37] = b'a';
    buffer[38] = b't';
    buffer[39] = b'a';
    
    buffer[40..44].copy_from_slice(&(data_size as u32).to_le_bytes());

    
    for (i, &sample) in jo.iter().enumerate() {
        let offset = ALQ_ + i * 2;
        buffer[offset..offset + 2].copy_from_slice(&sample.to_le_bytes());
    }

    buffer
}


pub fn dpb(path: &str, jo: &[i16], sample_rate: u32, channels: u16) -> Result<usize, &'static str> {
    if jo.is_empty() {
        return Err("No audio data to export");
    }

    let anf = mcn(jo, sample_rate, channels);
    let size = anf.len();

    
    crate::vfs::write_file(path, &anf)
        .map_err(|_| "Failed to write WAV file to VFS")?;

    crate::serial_println!("[TRUSTDAW] Exported WAV: {} ({} bytes, {} samples, {}Hz {}ch)",
        path, size, jo.len(), sample_rate, channels);

    Ok(size)
}


pub fn qff(num_stereo_samples: usize) -> usize {
    ALQ_ + num_stereo_samples * 2 
}


pub fn lmr(cbz: usize, sample_rate: u32, channels: u16) -> (u32, u32) {
    
    let frames = cbz / channels as usize;
    let total_ms = (frames as u64 * 1000) / sample_rate as u64;
    let abi = (total_ms / 1000) as u32;
    let dh = (total_ms % 1000) as u32;
    (abi, dh)
}
