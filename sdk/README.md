# TrustOS SDK

Cross-compilation SDK for building TrustOS applications from Windows.

## Requirements

- Rust toolchain with `x86_64-unknown-none` target
- cargo (included with Rust)

## Quick Start

```powershell
# Install the target
rustup target add x86_64-unknown-none

# Build an example
cd examples/hello
cargo build --release

# The ELF will be at: target/x86_64-unknown-none/release/hello
```

## Project Structure

```
sdk/
├── README.md           # This file
├── trustos-rt/         # Runtime library (entry point, syscalls)
├── trustos-std/        # Standard library (print, alloc, etc.)
└── examples/
    ├── hello/          # Hello world example
    └── syscall-test/   # Syscall test program
```

## Creating a New App

1. Create a new Cargo project:
```powershell
cargo new --bin myapp
cd myapp
```

2. Edit `Cargo.toml`:
```toml
[package]
name = "myapp"
version = "0.1.0"
edition = "2021"

[dependencies]
trustos-rt = { path = "../sdk/trustos-rt" }

[profile.release]
panic = "abort"
lto = true
```

3. Edit `.cargo/config.toml`:
```toml
[build]
target = "x86_64-unknown-none"

[target.x86_64-unknown-none]
rustflags = ["-C", "link-arg=-nostartfiles"]
```

4. Write your app in `src/main.rs`:
```rust
#![no_std]
#![no_main]

use trustos_rt::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello from TrustOS!");
    exit(0);
}
```

5. Build:
```powershell
cargo build --release
```

## Syscalls

| Number | Name       | Description           |
|--------|------------|-----------------------|
| 0      | read       | Read from file        |
| 1      | write      | Write to file         |
| 2      | open       | Open file             |
| 3      | close      | Close file            |
| 60     | exit       | Exit process          |
| 0x1000 | debug_print| Print to serial       |

## Loading into TrustOS

Copy the ELF to the TrustOS filesystem and execute:

```bash
# In TrustOS shell
exec /bin/myapp
# or
./myapp
```
