/* TrustOS Kernel Linker Script - riscv64 (RISC-V 64-bit)
 * Limine bootloader, higher-half kernel
 */

OUTPUT_FORMAT(elf64-littleriscv)
OUTPUT_ARCH(riscv)
ENTRY(kmain)

PHDRS
{
    text    PT_LOAD FLAGS(5);   /* R-X */
    rodata  PT_LOAD FLAGS(4);   /* R-- */
    data    PT_LOAD FLAGS(6);   /* RW- */
}

SECTIONS
{
    /* Kernel virtual base — Sv48 higher half */
    . = 0xffffffc000000000;

    .text : {
        __text_start = .;
        KEEP(*(.text.kmain))
        *(.text .text.*)
        __text_end = .;
    } :text

    . = ALIGN(4096);

    .rodata : {
        __rodata_start = .;
        *(.rodata .rodata.*)
    } :rodata
    
    /* Limine requests sections */
    .requests_start_marker : {
        KEEP(*(.requests_start_marker))
    } :rodata

    .requests : {
        KEEP(*(.requests))
    } :rodata

    .requests_end_marker : {
        KEEP(*(.requests_end_marker))
    } :rodata

    __rodata_end = .;

    . = ALIGN(4096);

    .data : {
        *(.data .data.*)
    } :data

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } :data

    .got : {
        *(.got .got.*)
    } :data

    /* Discard .note sections */
    /DISCARD/ : {
        *(.comment)
        *(.note .note.*)
        *(.eh_frame)
        *(.eh_frame_hdr)
    }

    _end = .;
}
