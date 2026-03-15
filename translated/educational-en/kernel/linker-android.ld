/* TrustOS Kernel Linker Script - Android Boot (aarch64)
 *
 * For booting TrustOS as a bare-metal kernel via Android bootloader.
 * The Android bootloader (ABL/fastboot) loads the kernel at a physical
 * address (typically 0x80080000 on Qualcomm, varies by SoC).
 *
 * Unlike the Limine higher-half layout, this uses identity-mapped
 * physical addresses since we start with MMU off.
 *
 * Boot flow: Bootrom → BL1 → BL2 → BL31 → ABL → THIS KERNEL (EL1)
 *
 * On entry from Android bootloader:
 *   x0 = DTB physical address (Device Tree Blob)
 *   x1 = 0 (reserved)
 *   CPU in EL1, MMU off, caches off
 */

OUTPUT_FORMAT(elf64-littleaarch64)
OUTPUT_ARCH(aarch64)
ENTRY(_android_entry)

/* Android kernel load address — standard for most Qualcomm/MediaTek SoCs.
 * Can be overridden at link time with --defsym KERNEL_BASE=0x... */
KERNEL_PHYS = DEFINED(KERNEL_PHYS) ? KERNEL_PHYS : 0x80080000;

SECTIONS
{
    . = KERNEL_PHYS;

    /* Boot stub must be at the very start (first instruction executed) */
    .boot : ALIGN(4096) {
        KEEP(*(.text.android_entry))
        KEEP(*(.text.android_boot))
    }

    .text : ALIGN(4096) {
        __text_start = .;
        KEEP(*(.text.kmain))
        *(.text .text.*)
        __text_end = .;
    }

    . = ALIGN(4096);

    .rodata : {
        __rodata_start = .;
        *(.rodata .rodata.*)
    }

    /* Limine request sections — kept for compatibility but unused on Android */
    .requests_start_marker : {
        KEEP(*(.requests_start_marker))
    }
    .requests : {
        KEEP(*(.requests))
    }
    .requests_end_marker : {
        KEEP(*(.requests_end_marker))
    }

    __rodata_end = .;

    . = ALIGN(4096);

    .data : {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    }

    .bss (NOLOAD) : ALIGN(4096) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    /* Boot stack — 64 KB, placed after BSS */
    . = ALIGN(4096);
    __stack_bottom = .;
    . += 0x10000;
    __stack_top = .;

    /* End of kernel image */
    __kernel_end = .;

    .got : {
        *(.got .got.*)
    }

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
        *(.comment)
    }
}

/* Export symbols for runtime use */
PROVIDE(__kernel_size = __kernel_end - KERNEL_PHYS);
