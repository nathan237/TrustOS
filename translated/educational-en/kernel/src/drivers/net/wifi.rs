//! WiFi Driver Interface
//!
//! Provides the WiFi-specific trait extending NetworkDriver
//! with scanning, authentication, and 802.11 management.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInformation, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// ============================================================================
// WiFi Types
// ============================================================================

/// Security type of a WiFi network
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Enumeration — a type that can be one of several variants.
pub enum WifiSecurity {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    Unknown,
}

// Implementation block — defines methods for the type above.
impl WifiSecurity {
        // Public function — callable from other modules.
pub fn as_str(&self) -> &'static str {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            WifiSecurity::Open => "Open",
            WifiSecurity::WEP => "WEP",
            WifiSecurity::WPA => "WPA",
            WifiSecurity::WPA2 => "WPA2",
            WifiSecurity::WPA3 => "WPA3",
            WifiSecurity::Unknown => "???",
        }
    }
}

/// A scanned WiFi network (BSS)
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct WifiNetwork {
    /// SSID (network name, max 32 bytes)
    pub ssid: String,
    /// BSSID (AP MAC address)
    pub bssid: [u8; 6],
    /// Channel number (1-14 for 2.4GHz, 36-165 for 5GHz)
    pub channel: u8,
    /// Signal strength in dBm (negative, e.g. -40 = strong, -80 = weak)
    pub signal_dbm: i8,
    /// Security type
    pub security: WifiSecurity,
    /// Frequency in MHz
    pub frequency_mhz: u16,
}

// Implementation block — defines methods for the type above.
impl WifiNetwork {
    /// Signal quality as percentage (0-100)
    pub fn signal_quality(&self) -> u8 {
        // Convert dBm to percentage: -30 dBm = 100%, -90 dBm = 0%
        if self.signal_dbm >= -30 { return 100; }
        if self.signal_dbm <= -90 { return 0; }
        ((self.signal_dbm as i16 + 90) * 100 / 60) as u8
    }

    /// Signal bars (0-4)
    pub fn signal_bars(&self) -> u8 {
                // Pattern matching — Rust's exhaustive branching construct.
match self.signal_quality() {
            80..=100 => 4,
            60..=79 => 3,
            40..=59 => 2,
            20..=39 => 1,
            _ => 0,
        }
    }
}

/// WiFi connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Enumeration — a type that can be one of several variants.
pub enum WifiState {
    /// No WiFi hardware detected
    NoHardware,
    /// Hardware detected but not initialized
    Disabled,
    /// Ready, not connected (idle)
    Disconnected,
    /// Currently scanning for networks
    Scanning,
    /// Attempting to connect
    Connecting,
    /// Authenticating (WPA handshake)
    Authenticating,
    /// Connected and associated
    Connected,
    /// Connection failed
    Failed,
}

/// WiFi driver trait — extends NetworkDriver with wireless capabilities
pub trait WifiDriver: NetworkDriver {
    /// Get current WiFi state
    fn wifi_state(&self) -> WifiState;

    /// Start scanning for available networks
    fn scan(&mut self) -> Result<(), &'static str>;

    /// Get scan results (call after scan completes)
    fn scan_results(&self) -> Vec<WifiNetwork>;

    /// Connect to a network
    fn connect(&mut self, ssid: &str, password: &str) -> Result<(), &'static str>;

    /// Disconnect from current network
    fn disconnect(&mut self) -> Result<(), &'static str>;

    /// Get currently connected SSID (if any)
    fn connected_ssid(&self) -> Option<String>;

    /// Get current channel
    fn current_channel(&self) -> Option<u8>;

    /// Get current signal strength in dBm
    fn signal_strength(&self) -> Option<i8>;
}

// ============================================================================
// Global WiFi State
// ============================================================================

/// Active WiFi driver
static WIFI_DRIVER: Mutex<Option<Box<dyn WifiDriver>>> = Mutex::new(None);
// Atomic variable — provides lock-free thread-safe access.
static WIFI_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Last scan results (cached for UI)
static SCAN_RESULTS: Mutex<Vec<WifiNetwork>> = Mutex::new(Vec::new());

/// Current connection state
static CONNECTION_STATE: Mutex<WifiState> = Mutex::new(WifiState::NoHardware);

/// Currently connected SSID
static CONNECTED_SSID: Mutex<Option<String>> = Mutex::new(None);

/// Pending connection request
static CONNECT_REQUEST: Mutex<Option<(String, String)>> = Mutex::new(None);

// ============================================================================
// Public API (called from desktop/shell)
// ============================================================================

/// Check if WiFi hardware is available
pub fn has_wifi() -> bool {
    WIFI_ACTIVE.load(Ordering::Relaxed)
}

/// Get current WiFi state
pub fn state() -> WifiState {
    *CONNECTION_STATE.lock()
}

/// Check if connected
pub fn is_connected() -> bool {
    *CONNECTION_STATE.lock() == WifiState::Connected
}

/// Get connected SSID
pub fn connected_ssid() -> Option<String> {
    CONNECTED_SSID.lock().clone()
}

/// Get signal strength of current connection
pub fn signal_strength() -> Option<i8> {
    WIFI_DRIVER.lock().as_ref().and_then(|d| d.signal_strength())
}

/// Start a scan for available networks
pub fn start_scan() -> Result<(), &'static str> {
    let mut guard = WIFI_DRIVER.lock();
    let driver = guard.as_mut().ok_or("No WiFi driver")?;
    *CONNECTION_STATE.lock() = WifiState::Scanning;
    driver.scan()
}

/// Get cached scan results
pub fn get_scan_results() -> Vec<WifiNetwork> {
    SCAN_RESULTS.lock().clone()
}

/// Poll WiFi driver (called from desktop tick)
pub fn poll() {
    let mut guard = WIFI_DRIVER.lock();
    if let Some(driver) = guard.as_mut() {
        driver.poll();

        // Update cached state
        let new_state = driver.wifi_state();
        let old_state = *CONNECTION_STATE.lock();

        if new_state != old_state {
            *CONNECTION_STATE.lock() = new_state;
            crate::serial_println!("[WIFI] State: {:?} -> {:?}", old_state, new_state);
        }

        // Update scan results when scan completes
        if old_state == WifiState::Scanning && new_state != WifiState::Scanning {
            let results = driver.scan_results();
            crate::serial_println!("[WIFI] Scan complete: {} networks found", results.len());
            *SCAN_RESULTS.lock() = results;
        }

        // Update connected SSID
        *CONNECTED_SSID.lock() = driver.connected_ssid();

        // Process pending connect request
        let request = CONNECT_REQUEST.lock().take();
        if let Some((ssid, password)) = request {
            crate::serial_println!("[WIFI] Connecting to '{}'...", ssid);
                        // Pattern matching — Rust's exhaustive branching construct.
match driver.connect(&ssid, &password) {
                Ok(()) => {
                    *CONNECTION_STATE.lock() = WifiState::Connecting;
                }
                Err(e) => {
                    crate::serial_println!("[WIFI] Connect failed: {}", e);
                    *CONNECTION_STATE.lock() = WifiState::Failed;
                }
            }
        }
    }
}

/// Request connection to a network
pub fn request_connect(ssid: &str, password: &str) {
    *CONNECT_REQUEST.lock() = Some((String::from(ssid), String::from(password)));
}

/// Disconnect from current network
pub fn disconnect() -> Result<(), &'static str> {
    let mut guard = WIFI_DRIVER.lock();
    let driver = guard.as_mut().ok_or("No WiFi driver")?;
    driver.disconnect()?;
    *CONNECTION_STATE.lock() = WifiState::Disconnected;
    *CONNECTED_SSID.lock() = None;
    Ok(())
}

/// Set the active WiFi driver (called during PCI probe)
pub fn set_driver(driver: Box<dyn WifiDriver>) {
    crate::log!("[WIFI] WiFi driver active: {}", driver.information().name);
    *WIFI_DRIVER.lock() = Some(driver);
    WIFI_ACTIVE.store(true, Ordering::SeqCst);
    *CONNECTION_STATE.lock() = WifiState::Disconnected;
}

/// Probe a PCI device for WiFi capability
pub fn probe_pci(pci_device: &PciDevice) -> bool {
    // Intel WiFi devices: class 0x02 (Network) subclass 0x80 (Other)
    // or class 0x0D (Wireless)
    let is_wireless = pci_device.class_code == crate::pci::class::WIRELESS
        || (pci_device.class_code == crate::pci::class::NETWORK && pci_device.subclass == 0x80);

    if !is_wireless {
        return false;
    }

    crate::serial_println!("[WIFI] Found wireless device: {:04X}:{:04X} at {}.{}.{}",
        pci_device.vendor_id, pci_device.device_id,
        pci_device.bus, pci_device.device, pci_device.function);

    // Try Intel WiFi Link 4965AGN (T61)
    if pci_device.vendor_id == 0x8086 {
        if let Some(driver) = super::iwl4965::probe(pci_device) {
            set_driver(driver);
            return true;
        }
    }

    false
}
