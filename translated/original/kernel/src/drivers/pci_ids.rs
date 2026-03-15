//! PCI Device ID Database
//!
//! Common vendor and device IDs for driver matching.

use alloc::string::String;
use alloc::format;

/// Get human-readable device name
pub fn get_device_name(vendor_id: u16, device_id: u16) -> String {
    match vendor_id {
        // Intel
        0x8086 => match device_id {
            0x100E | 0x100F => String::from("Intel 82540EM (e1000)"),
            0x10D3 => String::from("Intel 82574L GigE"),
            0x1237 => String::from("Intel 440FX Host Bridge"),
            0x7000 => String::from("Intel PIIX3 ISA Bridge"),
            0x7010 => String::from("Intel PIIX3 IDE"),
            0x7020 => String::from("Intel PIIX3 USB UHCI"),
            0x7113 => String::from("Intel PIIX4 ACPI"),
            0x2918 => String::from("Intel ICH9 LPC"),
            0x2922 => String::from("Intel ICH9 AHCI"),
            0x2930 => String::from("Intel ICH9 SMBus"),
            0x29C0 => String::from("Intel 82G33 Host Bridge"),
            0x2934 => String::from("Intel ICH9 USB UHCI #1"),
            0x2935 => String::from("Intel ICH9 USB UHCI #2"),
            0x2936 => String::from("Intel ICH9 USB UHCI #3"),
            0x293A => String::from("Intel ICH9 USB EHCI"),
            0x293C => String::from("Intel ICH9 HD Audio"),
            _ => format!("Intel {:04X}", device_id),
        },
        
        // Red Hat (QEMU)
        0x1AF4 => match device_id {
            0x1000 => String::from("VirtIO Network"),
            0x1001 => String::from("VirtIO Block"),
            0x1002 => String::from("VirtIO Balloon"),
            0x1003 => String::from("VirtIO Console"),
            0x1004 => String::from("VirtIO SCSI"),
            0x1005 => String::from("VirtIO RNG"),
            0x1009 => String::from("VirtIO 9P Transport"),
            0x1041 => String::from("VirtIO Network (1.0)"),
            0x1042 => String::from("VirtIO Block (1.0)"),
            0x1050 => String::from("VirtIO GPU"),
            0x1052 => String::from("VirtIO Input"),
            _ => format!("VirtIO {:04X}", device_id),
        },
        
        // QEMU
        0x1234 => match device_id {
            0x1111 => String::from("QEMU VGA"),
            _ => format!("QEMU {:04X}", device_id),
        },
        
        // Realtek
        0x10EC => match device_id {
            0x8139 => String::from("Realtek RTL8139"),
            0x8168 => String::from("Realtek RTL8168/8111"),
            0x8169 => String::from("Realtek RTL8169"),
            _ => format!("Realtek {:04X}", device_id),
        },
        
        // AMD/ATI
        0x1002 => match device_id {
            0x4752 => String::from("AMD Rage XL PCI"),
            0x515E => String::from("AMD ES1000"),
            _ => format!("AMD {:04X}", device_id),
        },
        
        // NVIDIA
        0x10DE => format!("NVIDIA {:04X}", device_id),
        
        // VirtualBox
        0x80EE => match device_id {
            0xBEEF => String::from("VirtualBox Graphics"),
            0xCAFE => String::from("VirtualBox Guest"),
            _ => format!("VirtualBox {:04X}", device_id),
        },
        
        // VMware
        0x15AD => match device_id {
            0x0405 => String::from("VMware SVGA II"),
            0x0710 => String::from("VMware SVGA"),
            0x0720 => String::from("VMware VMXNET"),
            0x0740 => String::from("VMware Virtual Machine Bus"),
            0x0770 => String::from("VMware USB2 EHCI"),
            0x0774 => String::from("VMware USB1.1 UHCI"),
            0x0790 => String::from("VMware PCI Bridge"),
            0x07A0 => String::from("VMware PCI Express Root Port"),
            0x07B0 => String::from("VMware VMXNET3"),
            0x07C0 => String::from("VMware PVSCSI"),
            0x0801 => String::from("VMware Virtual Machine Interface"),
            _ => format!("VMware {:04X}", device_id),
        },
        
        // Broadcom
        0x14E4 => format!("Broadcom {:04X}", device_id),
        
        // Unknown vendor
        _ => format!("{:04X}:{:04X}", vendor_id, device_id),
    }
}

/// Get vendor name
pub fn get_vendor_name(vendor_id: u16) -> &'static str {
    match vendor_id {
        0x8086 => "Intel",
        0x1AF4 => "Red Hat (VirtIO)",
        0x1234 => "QEMU",
        0x10EC => "Realtek",
        0x1002 => "AMD/ATI",
        0x10DE => "NVIDIA",
        0x80EE => "VirtualBox",
        0x15AD => "VMware",
        0x14E4 => "Broadcom",
        0x168C => "Qualcomm Atheros",
        0x8087 => "Intel USB",
        _ => "Unknown",
    }
}
