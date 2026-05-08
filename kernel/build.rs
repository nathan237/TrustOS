use std::path::PathBuf;
use std::process::Command;

fn main() {
    // ── CoreMark C compilation (when feature enabled) ──
    if std::env::var("CARGO_FEATURE_COREMARK").is_ok() {
        build_coremark();
    }

    // ── Linker script (architecture-aware) ──
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target = std::env::var("TARGET").unwrap_or_default();
    let ld_name = if target.starts_with("aarch64") {
        "linker-aarch64.ld"
    } else if target.starts_with("riscv64") {
        "linker-riscv64.ld"
    } else {
        "linker.ld"
    };
    let linker_script = PathBuf::from(&manifest_dir).join(ld_name);
    println!("cargo:rustc-link-arg=-T{}", linker_script.display());
    println!("cargo:rerun-if-changed={}", linker_script.display());

    // Get the path to the kernel binary
    let kernel_path = std::env::var("CARGO_BIN_FILE_TRUSTOS_KERNEL")
        .map(PathBuf::from)
        .ok();
    
    if let Some(path) = kernel_path {
        println!("cargo:rustc-env=KERNEL_PATH={}", path.display());
    }
    
    // Tell cargo to rerun if kernel changes
    println!("cargo:rerun-if-changed=src/");

    // Embed build timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            // Simple UTC date format
            let days = secs / 86400;
            let time_of_day = secs % 86400;
            let hours = time_of_day / 3600;
            let minutes = (time_of_day % 3600) / 60;
            let seconds = time_of_day % 60;
            // Approximate date from days since epoch
            let mut y = 1970i64;
            let mut remaining = days as i64;
            loop {
                let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
                let days_in_year: i64 = if leap { 366 } else { 365 };
                if remaining < days_in_year { break; }
                remaining -= days_in_year;
                y += 1;
            }
            let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
            let month_days: [i64; 12] = [31, if leap {29} else {28}, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut m = 0usize;
            for i in 0..12 {
                if remaining < month_days[i] { m = i; break; }
                remaining -= month_days[i];
            }
            format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", y, m + 1, remaining + 1, hours, minutes, seconds)
        })
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=TRUSTOS_BUILD_TIME={}", now);

    // ── ROM embedding: detect .nes and .gb files in roms/ directory ──
    let roms_dir = PathBuf::from(&manifest_dir).join("roms");
    println!("cargo:rerun-if-changed={}", roms_dir.display());
    
    if roms_dir.exists() {
        // Find .nes ROM
        if let Ok(entries) = std::fs::read_dir(&roms_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if ext == "nes" {
                        println!("cargo:rustc-cfg=has_nes_rom");
                        println!("cargo:rustc-env=NES_ROM_PATH={}", path.display());
                        println!("cargo:rerun-if-changed={}", path.display());
                        eprintln!("  [ROM] Found NES ROM: {}", path.display());
                    }
                    if ext == "gb" || ext == "gbc" {
                        println!("cargo:rustc-cfg=has_gb_rom");
                        println!("cargo:rustc-env=GB_ROM_PATH={}", path.display());
                        println!("cargo:rerun-if-changed={}", path.display());
                        eprintln!("  [ROM] Found GB ROM: {}", path.display());
                    }
                }
            }
        }
    }
}

fn build_coremark() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let coremark_root = PathBuf::from(&manifest_dir).join("coremark-src");
    let port_dir = coremark_root.join("trustos");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let c_files = [
        coremark_root.join("core_list_join.c"),
        coremark_root.join("core_main.c"),
        coremark_root.join("core_matrix.c"),
        coremark_root.join("core_state.c"),
        coremark_root.join("core_util.c"),
        port_dir.join("core_portme.c"),
        port_dir.join("ee_printf.c"),
    ];

    let compiler_flags = "-O2 -ffreestanding -nostdlib -fno-builtin -mcmodel=large -mno-red-zone -mno-sse";

    let common_args = &[
        "-O2",
        "-ffreestanding",
        "-nostdlib",
        "-fno-builtin",
        "-mcmodel=large",
        "-mno-red-zone",
        "-mno-sse",
        "-DPERFORMANCE_RUN=1",
        "-DITERATIONS=1500000",
        "-DHAS_PRINTF=0",
        "-DHAS_STDIO=0",
    ];

    let clang = if PathBuf::from("C:/Program Files/LLVM/bin/clang.exe").exists() {
        "C:/Program Files/LLVM/bin/clang.exe"
    } else {
        "clang"
    };

    let mut obj_files = Vec::new();

    for src in &c_files {
        let stem = src.file_stem().unwrap().to_str().unwrap();
        let obj = out_dir.join(format!("{}.o", stem));

        let status = Command::new(clang)
            .arg("--target=x86_64-unknown-none-elf")
            .args(common_args)
            .arg(&format!("-DFLAGS_STR=\"{}\"", compiler_flags))
            .arg("-I").arg(&coremark_root)
            .arg("-I").arg(&port_dir)
            .arg("-c")
            .arg(src)
            .arg("-o").arg(&obj)
            .status()
            .expect("Failed to run clang — install LLVM/clang");

        if !status.success() {
            panic!("gcc failed to compile {}", src.display());
        }
        obj_files.push(obj);
    }

    // Create static library (ELF archive via llvm-ar from Rust toolchain)
    let llvm_ar = find_llvm_ar();
    let lib_path = out_dir.join("libcoremark.a");
    let mut ar_cmd = Command::new(llvm_ar);
    ar_cmd.arg("rcs").arg(&lib_path);
    for obj in &obj_files {
        ar_cmd.arg(obj);
    }
    let status = ar_cmd.status().expect("Failed to run ar");
    if !status.success() {
        panic!("ar failed to create libcoremark.a");
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=coremark");
    println!("cargo:rerun-if-changed=coremark-src/");
}

fn find_llvm_ar() -> String {
    // Try LLVM install
    let llvm_path = "C:/Program Files/LLVM/bin/llvm-ar.exe";
    if PathBuf::from(llvm_path).exists() {
        return llvm_path.to_string();
    }
    // Try Rust toolchain sysroot
    if let Ok(output) = Command::new("rustc").arg("--print").arg("sysroot").output() {
        let sysroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Check gnu toolchain path
        let gnu_path = PathBuf::from(&sysroot)
            .join("lib/rustlib/x86_64-pc-windows-gnu/bin/llvm-ar.exe");
        if gnu_path.exists() {
            return gnu_path.to_string_lossy().to_string();
        }
        // Check msvc toolchain path
        let msvc_path = PathBuf::from(&sysroot)
            .join("lib/rustlib/x86_64-pc-windows-msvc/bin/llvm-ar.exe");
        if msvc_path.exists() {
            return msvc_path.to_string_lossy().to_string();
        }
    }
    // Fallback
    "llvm-ar".to_string()
}
