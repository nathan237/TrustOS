// TrustOS Android Boot Entry Point (aarch64)
//
// This is the first code executed when the Android bootloader (ABL/U-Boot)
// hands off to TrustOS. The CPU is in EL1, MMU off, caches off.
//
// On entry:
//   x0 = DTB physical address (Device Tree Blob from bootloader)
//   x1 = 0 (reserved)
//   x2 = 0 (reserved)  
//   x3 = 0 (reserved)
//   MMU: OFF, D-cache: OFF, I-cache: OFF
//   Exception level: EL1 (non-secure)
//   BL31 running in EL3 (available via SMC calls)

.section .text.android_entry, "ax"
.global _android_entry
.type _android_entry, @function

_android_entry:
    // ═══════════════════════════════════════════════════════════════
    // Step 0: Save DTB pointer (passed by bootloader in x0)
    // ═══════════════════════════════════════════════════════════════
    mov     x20, x0             // x20 = DTB address (callee-saved)

    // ═══════════════════════════════════════════════════════════════
    // Step 1: Ensure we're in EL1 (drop from EL2 if needed)
    // ═══════════════════════════════════════════════════════════════
    mrs     x0, CurrentEL
    lsr     x0, x0, #2
    cmp     x0, #2
    b.ne    .Lskip_el2_drop

    // We're in EL2 — configure EL1 and drop down
    // Set HCR_EL2.RW = 1 (AArch64 for EL1)
    mov     x0, #(1 << 31)     // RW bit
    msr     HCR_EL2, x0

    // Set SPSR_EL2 to return to EL1h (SP_EL1)
    mov     x0, #0x3C5         // EL1h, DAIF masked
    msr     SPSR_EL2, x0

    // Set ELR_EL2 to continue at .Lskip_el2_drop
    adr     x0, .Lskip_el2_drop
    msr     ELR_EL2, x0
    eret

.Lskip_el2_drop:
    // ═══════════════════════════════════════════════════════════════
    // Step 2: Disable interrupts during init (DAIF = all masked)
    // ═══════════════════════════════════════════════════════════════
    msr     DAIFSet, #0xF

    // ═══════════════════════════════════════════════════════════════
    // Step 3: Set up stack pointer (from linker script)
    // ═══════════════════════════════════════════════════════════════
    adrp    x0, __stack_top
    add     x0, x0, :lo12:__stack_top
    mov     sp, x0

    // ═══════════════════════════════════════════════════════════════
    // Step 4: Zero BSS section
    // ═══════════════════════════════════════════════════════════════
    adrp    x0, __bss_start
    add     x0, x0, :lo12:__bss_start
    adrp    x1, __bss_end
    add     x1, x1, :lo12:__bss_end
.Lbss_loop:
    cmp     x0, x1
    b.ge    .Lbss_done
    stp     xzr, xzr, [x0], #16
    b       .Lbss_loop
.Lbss_done:

    // ═══════════════════════════════════════════════════════════════
    // Step 5: Enable caches + MMU setup for flat mapping
    // ═══════════════════════════════════════════════════════════════

    // Enable I-cache and D-cache (no MMU yet — identity mapping later)
    mrs     x0, SCTLR_EL1
    orr     x0, x0, #(1 << 2)      // C — D-cache
    orr     x0, x0, #(1 << 12)     // I — I-cache
    bic     x0, x0, #(1 << 1)      // A — disable alignment check
    bic     x0, x0, #(1 << 0)      // M — ensure MMU stays off for now
    msr     SCTLR_EL1, x0
    isb

    // Set MAIR_EL1 (Memory Attribute Indirection Register)
    // Attr0 = Normal, Write-Back (0xFF)
    // Attr1 = Device-nGnRnE (0x00)
    ldr     x0, =0x00000000000000FF
    msr     MAIR_EL1, x0

    // Set TCR_EL1 for 4KB pages, 48-bit VA
    // T0SZ = 16 (48-bit), TG0 = 4KB, SH0 = Inner Shareable
    ldr     x0, =0x00000000B5193519
    msr     TCR_EL1, x0
    isb

    // ═══════════════════════════════════════════════════════════════
    // Step 6: Initialize UART for early debug output
    // ═══════════════════════════════════════════════════════════════
    // Try PL011 at 0x09000000 (QEMU virt) as initial default.
    // The Rust code will detect the real UART from DTB later.
    ldr     x0, =0x09000000
    adrp    x1, __uart_base
    add     x1, x1, :lo12:__uart_base
    str     x0, [x1]

    // Print banner: "TrustOS\r\n"
    mov     x1, #'T'
    str     w1, [x0]
    mov     x1, #'r'
    str     w1, [x0]
    mov     x1, #'u'
    str     w1, [x0]
    mov     x1, #'s'
    str     w1, [x0]
    mov     x1, #'t'
    str     w1, [x0]
    mov     x1, #'O'
    str     w1, [x0]
    mov     x1, #'S'
    str     w1, [x0]
    mov     x1, #'\r'
    str     w1, [x0]
    mov     x1, #'\n'
    str     w1, [x0]

    // ═══════════════════════════════════════════════════════════════
    // Step 7: Jump to Rust android_main(dtb_ptr)
    // ═══════════════════════════════════════════════════════════════
    mov     x0, x20             // x0 = DTB pointer (restored)
    bl      android_main

    // Should never return, but halt if it does
.Lhalt:
    wfe
    b       .Lhalt

// ═══════════════════════════════════════════════════════════════════
// Data section
// ═══════════════════════════════════════════════════════════════════
.section .data
.global __uart_base
__uart_base:
    .quad   0x09000000          // Default UART base (QEMU PL011)

.section .text.android_boot, "ax"

// SMC helper — call ARM Trusted Firmware from EL1
// x0 = function ID, x1-x3 = args
// Returns: x0 = result
.global android_smc_call
.type android_smc_call, @function
android_smc_call:
    smc     #0
    ret

// PSCI system off — clean shutdown via BL31
.global android_psci_off
.type android_psci_off, @function
android_psci_off:
    ldr     x0, =0x84000008    // PSCI_SYSTEM_OFF (32-bit SMC)
    smc     #0
    // Should not return
    wfe
    b       android_psci_off

// PSCI system reset — reboot via BL31
.global android_psci_reset
.type android_psci_reset, @function
android_psci_reset:
    ldr     x0, =0x84000009    // PSCI_SYSTEM_RESET (32-bit SMC)
    smc     #0
    wfe
    b       android_psci_reset
