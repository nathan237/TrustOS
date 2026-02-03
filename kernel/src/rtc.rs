//! Real-Time Clock Driver
//! 
//! Reads date and time from the CMOS RTC chip.

use x86_64::instructions::port::Port;
use spin::Mutex;

/// CMOS address port
const CMOS_ADDRESS: u16 = 0x70;
/// CMOS data port
const CMOS_DATA: u16 = 0x71;

/// RTC register addresses
const RTC_SECONDS: u8 = 0x00;
const RTC_MINUTES: u8 = 0x02;
const RTC_HOURS: u8 = 0x04;
const RTC_DAY: u8 = 0x07;
const RTC_MONTH: u8 = 0x08;
const RTC_YEAR: u8 = 0x09;
const RTC_CENTURY: u8 = 0x32; // May not exist on all systems
const RTC_STATUS_A: u8 = 0x0A;
const RTC_STATUS_B: u8 = 0x0B;

/// Global RTC lock
static RTC_LOCK: Mutex<()> = Mutex::new(());

/// Date and time structure
#[derive(Clone, Copy, Debug)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl DateTime {
    /// Format as ISO-like string: YYYY-MM-DD HH:MM:SS
    pub fn format(&self) -> alloc::string::String {
        alloc::format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day,
            self.hour, self.minute, self.second
        )
    }
    
    /// Format date only: YYYY-MM-DD
    pub fn format_date(&self) -> alloc::string::String {
        alloc::format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
    
    /// Format time only: HH:MM:SS
    pub fn format_time(&self) -> alloc::string::String {
        alloc::format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

/// Read a byte from CMOS
fn cmos_read(reg: u8) -> u8 {
    let mut addr_port = Port::<u8>::new(CMOS_ADDRESS);
    let mut data_port = Port::<u8>::new(CMOS_DATA);
    
    unsafe {
        // Disable NMI (bit 7) and select register
        addr_port.write(0x80 | reg);
        data_port.read()
    }
}

/// Check if RTC update is in progress
fn is_update_in_progress() -> bool {
    cmos_read(RTC_STATUS_A) & 0x80 != 0
}

/// Convert BCD to binary
fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd >> 4) * 10) + (bcd & 0x0F)
}

/// Try to read current date and time from RTC.
fn try_read_rtc() -> Option<DateTime> {
    let _lock = RTC_LOCK.lock();

    // Wait for update to complete (with timeout to avoid boot hangs)
    let mut spins: u32 = 0;
    while is_update_in_progress() {
        core::hint::spin_loop();
        spins = spins.wrapping_add(1);
        if spins >= 1_000_000 {
            return None;
        }
    }

    // Read values
    let mut second = cmos_read(RTC_SECONDS);
    let mut minute = cmos_read(RTC_MINUTES);
    let mut hour = cmos_read(RTC_HOURS);
    let mut day = cmos_read(RTC_DAY);
    let mut month = cmos_read(RTC_MONTH);
    let mut year = cmos_read(RTC_YEAR);

    // Read status register B to check format
    let status_b = cmos_read(RTC_STATUS_B);

    // Convert BCD to binary if needed (bit 2 of status B)
    if status_b & 0x04 == 0 {
        second = bcd_to_binary(second);
        minute = bcd_to_binary(minute);
        hour = bcd_to_binary(hour & 0x7F) | (hour & 0x80);
        day = bcd_to_binary(day);
        month = bcd_to_binary(month);
        year = bcd_to_binary(year);
    }

    // Convert 12-hour to 24-hour if needed (bit 1 of status B)
    if status_b & 0x02 == 0 && hour & 0x80 != 0 {
        hour = ((hour & 0x7F) + 12) % 24;
    }

    // Calculate full year (assume 2000s)
    let full_year = 2000u16 + year as u16;

    Some(DateTime {
        year: full_year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

/// Read current date and time from RTC (falls back if unavailable).
pub fn read_rtc() -> DateTime {
    try_read_rtc().unwrap_or(DateTime {
        year: 2000,
        month: 1,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
    })
}

/// Get current timestamp as seconds since midnight
pub fn get_time_seconds() -> u32 {
    let dt = read_rtc();
    dt.hour as u32 * 3600 + dt.minute as u32 * 60 + dt.second as u32
}

/// Initialize RTC (just verify it works)
pub fn try_init() -> bool {
    if let Some(dt) = try_read_rtc() {
        crate::serial_println!("[RTC] Initialized: {}", dt.format());
        true
    } else {
        crate::serial_println!("[RTC] Init skipped (no RTC)");
        false
    }
}
