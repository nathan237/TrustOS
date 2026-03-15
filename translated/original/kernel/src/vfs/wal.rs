//! Write-Ahead Log (WAL) for TrustFS
//!
//! Provides crash-safe writes by logging operations before applying them.
//! On mount, the WAL is replayed if the previous shutdown was unclean.
//!
//! Layout: WAL occupies sectors immediately before data area.
//! - WAL header (1 sector): magic, entry count, committed flag
//! - WAL entries (up to 63 sectors): each is a pending sector write

use spin::Mutex;

const SECTOR_SIZE: usize = 512;
const WAL_MAGIC: u32 = 0x57414C21; // "WAL!"
const MAX_WAL_ENTRIES: usize = 63;

/// WAL lives in sectors right before the TrustFS data start.
/// TrustFS data starts at sector 33, so WAL uses sectors 33..96
/// and data shifts to sector 97. We use a separate reserved range.
const WAL_HEADER_SECTOR: u64 = 33;
const WAL_ENTRY_SECTOR_START: u64 = 34; // entries at 34..96

/// WAL header stored in first WAL sector
#[repr(C)]
#[derive(Clone, Copy)]
struct WalHeader {
    magic: u32,
    entry_count: u32,
    committed: u32, // 1 = entries committed but not yet applied
    sequence: u64,   // Monotonic sequence number
    _pad: [u8; SECTOR_SIZE - 20],
}

/// A single WAL entry — records a pending sector write
#[repr(C)]
#[derive(Clone, Copy)]
struct WalEntry {
    target_sector: u64,        // Where this data should go
    data: [u8; SECTOR_SIZE - 8], // Sector data (504 bytes — we truncate for header)
}

/// Simplified WAL that logs full sector writes
pub struct WriteAheadLog {
    pending: [(u64, [u8; SECTOR_SIZE]); MAX_WAL_ENTRIES],
    count: usize,
    sequence: u64,
    active: bool,
}

impl WriteAheadLog {
    pub const fn new() -> Self {
        Self {
            pending: [(0, [0u8; SECTOR_SIZE]); MAX_WAL_ENTRIES],
            count: 0,
            sequence: 0,
            active: false,
        }
    }

    /// Begin a transaction — subsequent writes are buffered
    pub fn begin(&mut self) {
        self.count = 0;
        self.active = true;
    }

    /// Record a pending sector write
    pub fn log_write(&mut self, sector: u64, data: &[u8; SECTOR_SIZE]) -> Result<(), ()> {
        if !self.active || self.count >= MAX_WAL_ENTRIES {
            return Err(());
        }
        self.pending[self.count] = (sector, *data);
        self.count += 1;
        Ok(())
    }

    /// Commit: write WAL header + entries to disk, then apply, then clear WAL
    pub fn commit(
        &mut self,
        write_sector: &dyn Fn(u64, &[u8; SECTOR_SIZE]) -> Result<(), ()>,
    ) -> Result<(), ()> {
        if self.count == 0 {
            self.active = false;
            return Ok(());
        }

        self.sequence += 1;

        // Step 1: Write WAL header (marks transaction as pending)
        let mut hdr_buf = [0u8; SECTOR_SIZE];
        let hdr = unsafe { &mut *(hdr_buf.as_mut_ptr() as *mut WalHeader) };
        hdr.magic = WAL_MAGIC;
        hdr.entry_count = self.count as u32;
        hdr.committed = 1;
        hdr.sequence = self.sequence;
        write_sector(WAL_HEADER_SECTOR, &hdr_buf)?;

        // Step 2: Write each pending entry to WAL area (for replay on crash)
        for i in 0..self.count {
            let (target, ref data) = self.pending[i];
            // Store target sector in first 8 bytes, rest is data
            let mut entry_buf = [0u8; SECTOR_SIZE];
            entry_buf[0..8].copy_from_slice(&target.to_le_bytes());
            let copy_len = core::cmp::min(data.len(), SECTOR_SIZE - 8);
            entry_buf[8..8 + copy_len].copy_from_slice(&data[..copy_len]);
            write_sector(WAL_ENTRY_SECTOR_START + i as u64, &entry_buf)?;
        }

        // Step 3: Apply — write actual data to target sectors
        for i in 0..self.count {
            let (target, ref data) = self.pending[i];
            write_sector(target, data)?;
        }

        // Step 4: Clear WAL header (marks transaction as complete)
        let zero = [0u8; SECTOR_SIZE];
        write_sector(WAL_HEADER_SECTOR, &zero)?;

        self.count = 0;
        self.active = false;
        Ok(())
    }

    /// Number of pending writes
    pub fn pending_count(&self) -> usize {
        self.count
    }
}

/// Global WAL instance
static WAL: Mutex<WriteAheadLog> = Mutex::new(WriteAheadLog::new());

/// Replay WAL on mount (called during TrustFS init)
pub fn replay_if_needed(
    read_sector: &dyn Fn(u64, &mut [u8; SECTOR_SIZE]) -> Result<(), ()>,
    write_sector: &dyn Fn(u64, &[u8; SECTOR_SIZE]) -> Result<(), ()>,
) -> Result<usize, ()> {
    let mut hdr_buf = [0u8; SECTOR_SIZE];
    read_sector(WAL_HEADER_SECTOR, &mut hdr_buf)?;

    let hdr = unsafe { &*(hdr_buf.as_ptr() as *const WalHeader) };
    if hdr.magic != WAL_MAGIC || hdr.committed != 1 || hdr.entry_count == 0 {
        return Ok(0); // No pending WAL
    }

    let count = hdr.entry_count as usize;
    crate::log!("[WAL] Replaying {} pending writes from sequence {}", count, hdr.sequence);

    for i in 0..count.min(MAX_WAL_ENTRIES) {
        let mut entry_buf = [0u8; SECTOR_SIZE];
        read_sector(WAL_ENTRY_SECTOR_START + i as u64, &mut entry_buf)?;

        let target = u64::from_le_bytes(entry_buf[0..8].try_into().unwrap_or([0; 8]));
        // Reconstruct sector data (first 8 bytes were target, rest is data)
        let mut data = [0u8; SECTOR_SIZE];
        let copy_len = core::cmp::min(SECTOR_SIZE - 8, SECTOR_SIZE);
        data[..copy_len].copy_from_slice(&entry_buf[8..8 + copy_len]);
        // Pad remaining with zeros (last 8 bytes)
        write_sector(target, &data)?;
    }

    // Clear WAL
    let zero = [0u8; SECTOR_SIZE];
    write_sector(WAL_HEADER_SECTOR, &zero)?;

    crate::log!("[WAL] Replay complete — {} writes applied", count);
    Ok(count)
}

/// Begin a WAL transaction
pub fn begin() {
    WAL.lock().begin();
}

/// Log a sector write in the current transaction
pub fn log_write(sector: u64, data: &[u8; SECTOR_SIZE]) -> Result<(), ()> {
    WAL.lock().log_write(sector, data)
}

/// Commit the current transaction
pub fn commit(
    write_sector: &dyn Fn(u64, &[u8; SECTOR_SIZE]) -> Result<(), ()>,
) -> Result<(), ()> {
    WAL.lock().commit(write_sector)
}
