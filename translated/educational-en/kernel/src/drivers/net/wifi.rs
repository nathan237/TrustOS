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
pub(crate) // Global shared state guarded by a Mutex (mutual exclusion lock).
static WIFI_DRIVER: Mutex<Option<Box<dyn WifiDriver>>> = Mutex::new(None);
// Atomic variable — provides lock-free thread-safe access.
static WIFI_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Deferred WiFi PCI location (bus, device, function) — set during boot, probed on first use
static DEFERRED_PCI: Mutex<Option<(u8, u8, u8)>> = Mutex::new(None);

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

/// Store PCI location for deferred WiFi probe (called during boot — no MMIO access)
pub fn set_deferred_pci(bus: u8, device: u8, function: u8) {
    *DEFERRED_PCI.lock() = Some((bus, device, function));
    crate::serial_println!("[WIFI] Deferred PCI probe stored: {}.{}.{}", bus, device, function);
}

/// Check if WiFi hardware is available (detected or deferred)
pub fn has_wifi() -> bool {
    WIFI_ACTIVE.load(Ordering::Relaxed) || DEFERRED_PCI.lock().is_some()
}

/// Check if WiFi hardware has been fully probed and driver is active
pub fn has_active_driver() -> bool {
    WIFI_ACTIVE.load(Ordering::Relaxed)
}

/// Lazy probe: actually touch the hardware for the first time.
/// Called on first WiFi command, NOT during boot.
pub fn lazy_probe() -> Result<(), &'static str> {
    // Already probed?
    if WIFI_ACTIVE.load(Ordering::Relaxed) {
        return Ok(());
    }

    let pci_loc = DEFERRED_PCI.lock().take();
    let (bus, dev, func) = pci_loc.ok_or("No WiFi hardware detected during boot")?;

    crate::println!("  WiFi lazy probe: PCI {}.{}.{}", bus, dev, func);
    crate::serial_println!("[WIFI] Lazy probe: {}.{}.{}", bus, dev, func);

    // Find the PCI device by bus/device/function
    let devices = crate::pci::get_devices();
    let pci_dev = devices.iter()
        .find(|d| d.bus == bus && d.device == dev && d.function == func)
        .ok_or("WiFi PCI device not found")?;

    // Now actually probe (map_bar0 + bus master)
    if probe_pci(pci_dev) {
        crate::println!("  WiFi probe OK: iwl4965");
        Ok(())
    } else {
        // Put the deferred info back so user can retry
        *DEFERRED_PCI.lock() = Some((bus, dev, func));
        Err("WiFi hardware probe failed")
    }
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

/// Ensure the WiFi driver is started (hw_init + firmware load).
/// Called automatically on first use (scan, connect).
pub fn ensure_started() -> Result<(), &'static str> {
    // Lazy probe if not yet done (deferred from boot)
    if !WIFI_ACTIVE.load(Ordering::Relaxed) {
        lazy_probe()?;
    }

    let mut guard = WIFI_DRIVER.lock();
    let driver = guard.as_mut().ok_or("No WiFi driver")?;
    if driver.status() == crate::drivers::DriverStatus::Running {
        return Ok(());
    }
    crate::serial_println!("[WIFI] Auto-starting driver (hw_init + firmware)...");
    crate::println!("  Starting WiFi hardware...");
        // Pattern matching — Rust's exhaustive branching construct.
match driver.start() {
        Ok(()) => {
            crate::serial_println!("[WIFI] Driver started successfully");
            crate::println!("  WiFi hardware initialized");
            Ok(())
        }
        Err(e) => {
            crate::serial_println!("[WIFI] Driver start failed: {}", e);
            crate::println!("  WiFi start failed: {}", e);
            Err(e)
        }
    }
}

/// Start a scan for available networks
pub fn start_scan() -> Result<(), &'static str> {
    // Lazy probe + auto-start
    ensure_started()?;

    {
        let mut guard = WIFI_DRIVER.lock();
        let driver = guard.as_mut().ok_or("No WiFi driver")?;
        *CONNECTION_STATE.lock() = WifiState::Scanning;
        driver.scan()
    }
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
    // Auto-start driver if needed
    if let Err(e) = ensure_started() {
        crate::serial_println!("[WIFI] Cannot connect — start failed: {}", e);
        return;
    }
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
    crate::log!("[WIFI] WiFi driver active: {}", driver.info().name);
    *WIFI_DRIVER.lock() = Some(driver);
    WIFI_ACTIVE.store(true, Ordering::SeqCst);
    *CONNECTION_STATE.lock() = WifiState::Disconnected;
}

/// Probe a PCI device for WiFi capability
pub fn probe_pci(pci_dev: &PciDevice) -> bool {
    // Debug: log every device we check
    crate::serial_println!("[WIFI-PROBE] Checking {:04X}:{:04X} class={:02X} sub={:02X} at {}.{}.{}",
        pci_dev.vendor_id, pci_dev.device_id,
        pci_dev.class_code, pci_dev.subclass,
        pci_dev.bus, pci_dev.device, pci_dev.function);

    // Intel WiFi devices: class 0x02 (Network) subclass 0x80 (Other)
    // or class 0x0D (Wireless)
    // or Intel vendor with known WiFi device IDs
    let is_wireless = pci_dev.class_code == crate::pci::class::WIRELESS
        || (pci_dev.class_code == crate::pci::class::NETWORK && pci_dev.subclass == 0x80)
        || (pci_dev.vendor_id == 0x8086 && super::iwl4965::IWL4965_DEVICE_IDS.contains(&pci_dev.device_id));

    if !is_wireless {
        crate::serial_println!("[WIFI-PROBE] -> Not wireless (class={:02X} sub={:02X} devid={:04X})",
            pci_dev.class_code, pci_dev.subclass, pci_dev.device_id);
        return false;
    }

    crate::serial_println!("[WIFI] Found wireless device: {:04X}:{:04X} at {}.{}.{}",
        pci_dev.vendor_id, pci_dev.device_id,
        pci_dev.bus, pci_dev.device, pci_dev.function);

    // Try Intel WiFi Link 4965AGN
    if pci_dev.vendor_id == 0x8086 {
        if let Some(driver) = super::iwl4965::probe(pci_dev) {
            set_driver(driver);
            return true;
        }
    }

    false
}
