use std::path::PathBuf;

fn main() {
    // Get the path to the kernel binary
    let kernel_path = std::env::var("CARGO_BIN_FILE_TRUSTOS_KERNEL")
        .map(PathBuf::from)
        .ok();
    
    if let Some(path) = kernel_path {
        println!("cargo:rustc-env=KERNEL_PATH={}", path.display());
    }
    
    // Tell cargo to rerun if kernel changes
    println!("cargo:rerun-if-changed=src/");
}
