use std::path::PathBuf;

fn main() {
    // Path to the kernel binary
    let kernel_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/x86_64-unknown-none/release/trustos_kernel");
    
    // Create UEFI disk image
    let uefi_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/trustos-uefi.img");
    
    // Create BIOS disk image
    let bios_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("target/trustos-bios.img");

    println!("Kernel: {:?}", kernel_path);
    println!("Creating boot images...");

    // Use bootloader to create disk images
    let uefi_builder = bootloader::UefiBoot::new(&kernel_path);
    uefi_builder.create_disk_image(&uefi_path).expect("Failed to create UEFI disk image");
    println!("UEFI image: {:?}", uefi_path);

    let bios_builder = bootloader::BiosBoot::new(&kernel_path);
    bios_builder.create_disk_image(&bios_path).expect("Failed to create BIOS disk image");
    println!("BIOS image: {:?}", bios_path);
    
    println!("\nDone! Run with:");
    println!("  qemu-system-x86_64 -drive format=raw,file={}", bios_path.display());
}
