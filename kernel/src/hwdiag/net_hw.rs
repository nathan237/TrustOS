//! Network Hardware Diagnostics — detect NICs, WiFi, Ethernet via PCI
//!
//! Identifies network controllers, reports link capabilities,
//! MAC address (if accessible), and driver compatibility.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Run network hardware diagnostics
pub fn run() {
    dbg_out!("[NET] === Network Interface Diagnostics ===");

    let devices = crate::pci::scan();

    // Find network controllers (class 0x02) and wireless (class 0x0D)
    let net_devs: Vec<_> = devices.iter()
        .filter(|d| d.class_code == 0x02 || d.class_code == 0x0D)
        .collect();

    if net_devs.is_empty() {
        dbg_out!("[NET] No network controllers found via PCI!");
        // Check for USB-based NICs as a hint
        let usb_devs: Vec<_> = devices.iter().filter(|d| d.class_code == 0x0C && d.subclass == 0x03).collect();
        if !usb_devs.is_empty() {
            dbg_out!("[NET] Note: {} USB controller(s) found — NIC may be USB-based", usb_devs.len());
        }
        return;
    }

    dbg_out!("[NET] Found {} network device(s)", net_devs.len());
    dbg_out!("");

    for dev in &net_devs {
        let bdf = alloc::format!("{:02x}:{:02x}.{}", dev.bus, dev.device, dev.function);

        let nic_type = identify_nic(dev.vendor_id, dev.device_id);
        let interface_type = match (dev.class_code, dev.subclass) {
            (0x02, 0x00) => "Ethernet",
            (0x02, 0x80) => "Other Network",
            (0x0D, 0x00) => "iRDA",
            (0x0D, 0x01) => "Consumer IR",
            (0x0D, 0x10) => "RF Controller",
            (0x0D, 0x11) => "Bluetooth",
            (0x0D, 0x12) => "Broadband",
            (0x0D, 0x20) => "WiFi 802.11a",
            (0x0D, 0x21) => "WiFi 802.11b",
            (0x0D, 0x80) => "Wireless (other)",
            _ => "Unknown",
        };

        dbg_out!("[NET] ┌─ {} [{:04x}:{:04x}] {} ──────────────────",
            bdf, dev.vendor_id, dev.device_id, interface_type);
        dbg_out!("[NET] │ Vendor: {} (0x{:04X})", dev.vendor_name(), dev.vendor_id);
        dbg_out!("[NET] │ Chip:   {}", nic_type);

        // Check BARs for MMIO
        for bar_idx in 0..6 {
            if let Some(addr) = dev.bar_address(bar_idx) {
                if addr != 0 {
                    let is_mem = dev.bar_is_memory(bar_idx);
                    dbg_out!("[NET] │ BAR{}: {} 0x{:016X}",
                        bar_idx,
                        if is_mem { "MEM" } else { "I/O" },
                        addr);
                }
            }
        }

        // Check if device has MSI/MSI-X
        let status = crate::pci::config_read16(dev.bus, dev.device, dev.function, 0x06);
        if status & (1 << 4) != 0 {
            check_interrupt_caps(dev);
        }

        // Driver compatibility assessment
        assess_driver_support(dev);

        dbg_out!("[NET] └──────────────────────────────────────────────────");
        dbg_out!("");
    }
}

fn identify_nic(vendor: u16, device: u16) -> &'static str {
    match (vendor, device) {
        // Intel Ethernet
        (0x8086, 0x100E) => "Intel 82540EM (e1000)",
        (0x8086, 0x100F) => "Intel 82545EM (e1000)",
        (0x8086, 0x10D3) => "Intel 82574L (e1000e)",
        (0x8086, 0x1533) => "Intel I210 (igb)",
        (0x8086, 0x1539) => "Intel I211 (igb)",
        (0x8086, 0x15B8) => "Intel I219-V (e1000e)",
        (0x8086, 0x153A) => "Intel I217-LM (e1000e)",
        (0x8086, 0x15BC) => "Intel I219-V (CNP)",
        (0x8086, 0x15BD) => "Intel I219-LM (CNP)",
        (0x8086, 0x15BE) => "Intel I219-V (CNP-H)",
        (0x8086, 0x0D4F) => "Intel Ethernet (Ice Lake)",
        (0x8086, 0x125B) => "Intel I226-V (2.5GbE)",
        (0x8086, 0x125C) => "Intel I226-LM (2.5GbE)",

        // Intel WiFi
        (0x8086, 0x4222) => "Intel WiFi 4965AGN (iwl4965)",
        (0x8086, 0x4227) => "Intel WiFi 4965AGN (iwl4965)",
        (0x8086, 0x4232) => "Intel WiFi 5100AGN (iwl5000)",
        (0x8086, 0x0085) => "Intel WiFi 6205 (iwlwifi)",
        (0x8086, 0x0082) => "Intel WiFi 6205 (iwlwifi)",
        (0x8086, 0x24F3) => "Intel WiFi 8260 (iwlwifi)",
        (0x8086, 0x24FD) => "Intel WiFi 8265 (iwlwifi)",
        (0x8086, 0x2526) => "Intel WiFi 9260 (iwlwifi)",
        (0x8086, 0x2723) => "Intel WiFi 6 AX200 (iwlwifi)",
        (0x8086, 0x2725) => "Intel WiFi 6E AX210 (iwlwifi)",
        (0x8086, 0x7A70) => "Intel WiFi 7 BE200 (iwlwifi)",
        (0x8086, 0x272B) => "Intel WiFi 6E AX211 (iwlwifi)",

        // Realtek
        (0x10EC, 0x8139) => "Realtek RTL8139 (100M)",
        (0x10EC, 0x8168) => "Realtek RTL8111/8168 (GbE)",
        (0x10EC, 0x8169) => "Realtek RTL8169 (GbE)",
        (0x10EC, 0x8125) => "Realtek RTL8125 (2.5GbE)",
        (0x10EC, 0xB852) => "Realtek RTL8852BE (WiFi 6)",
        (0x10EC, 0xC852) => "Realtek RTL8852CE (WiFi 6E)",
        (0x10EC, 0xC822) => "Realtek RTL8822CE (WiFi 5)",
        (0x10EC, 0x8821) => "Realtek RTL8821CE (WiFi 5)",

        // Qualcomm / Atheros
        (0x168C, 0x003E) => "Qualcomm Atheros QCA6174 (WiFi 5)",
        (0x168C, 0x0046) => "Qualcomm Atheros QCA9377 (WiFi 5)",
        (0x168C, 0x0042) => "Qualcomm Atheros QCA9882 (WiFi 5)",
        (0x17CB, 0x1103) => "Qualcomm WCN785x (WiFi 7)",

        // Broadcom
        (0x14E4, 0x1677) => "Broadcom BCM5751 (GbE)",
        (0x14E4, 0x16B5) => "Broadcom BCM57311 (10GbE)",
        (0x14E4, 0x4365) => "Broadcom BCM43142 (WiFi 4)",
        (0x14E4, 0x43A0) => "Broadcom BCM4360 (WiFi 5)",

        // MediaTek
        (0x14C3, 0x7961) => "MediaTek MT7921 (WiFi 6)",
        (0x14C3, 0x7922) => "MediaTek MT7922 (WiFi 6E)",
        (0x14C3, 0x0616) => "MediaTek MT7925 (WiFi 7)",

        // Virtio
        (0x1AF4, 0x1000) => "Virtio Network (legacy)",
        (0x1AF4, 0x1041) => "Virtio Network (modern)",

        // Generic fallback by vendor
        (0x8086, _) => "Intel (unknown model)",
        (0x10EC, _) => "Realtek (unknown model)",
        (0x168C, _) => "Atheros (unknown model)",
        (0x14E4, _) => "Broadcom (unknown model)",
        (0x14C3, _) => "MediaTek (unknown model)",
        (0x1AF4, _) => "Virtio (Red Hat)",

        _ => "Unknown NIC",
    }
}

fn check_interrupt_caps(dev: &crate::pci::PciDevice) {
    let mut cap_ptr = crate::pci::config_read8(dev.bus, dev.device, dev.function, 0x34) & 0xFC;
    let mut count = 0;

    while cap_ptr != 0 && count < 32 {
        let cap_id = crate::pci::config_read8(dev.bus, dev.device, dev.function, cap_ptr);
        let next = crate::pci::config_read8(dev.bus, dev.device, dev.function, cap_ptr + 1) & 0xFC;

        match cap_id {
            0x05 => {
                let ctrl = crate::pci::config_read16(dev.bus, dev.device, dev.function, cap_ptr + 2);
                let vectors = 1 << ((ctrl >> 1) & 0x7);
                dbg_out!("[NET] │ MSI: {} vector(s)", vectors);
            }
            0x11 => {
                let ctrl = crate::pci::config_read16(dev.bus, dev.device, dev.function, cap_ptr + 2);
                let vectors = (ctrl & 0x7FF) + 1;
                dbg_out!("[NET] │ MSI-X: {} vector(s)", vectors);
            }
            _ => {}
        }

        cap_ptr = next;
        count += 1;
    }
}

fn assess_driver_support(dev: &crate::pci::PciDevice) {
    let support = match (dev.vendor_id, dev.device_id) {
        // Virtio — fully supported by TrustOS
        (0x1AF4, 0x1000) | (0x1AF4, 0x1041) => "FULL (virtio_net driver)",

        // Intel WiFi 4965 — supported via iwl4965 driver
        (0x8086, 0x4222) | (0x8086, 0x4227) => "SUPPORTED (iwl4965 firmware loaded)",

        // Intel Ethernet e1000-family — basic support
        (0x8086, 0x100E) | (0x8086, 0x100F) => "BASIC (e1000 via virtio layer)",

        // Common WiFi chips — detection only, no full driver yet
        (0x8086, 0x2723) | (0x8086, 0x2725) | (0x8086, 0x272B) |
        (0x8086, 0x7A70) => "DETECTED (WiFi 6/6E/7 — firmware not yet bundled)",

        // Others
        _ => "DETECTED (no TrustOS driver yet)",
    };

    dbg_out!("[NET] │ TrustOS driver: {}", support);
}
