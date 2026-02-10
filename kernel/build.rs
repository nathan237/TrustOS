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
}
