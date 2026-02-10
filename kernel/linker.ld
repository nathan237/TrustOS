/* Limine kernel linker script - based on official template */

OUTPUT_FORMAT(elf64-x86-64)
ENTRY(kmain)

PHDRS
{
    text    PT_LOAD FLAGS(5);   /* R-X */
    rodata  PT_LOAD FLAGS(4);   /* R-- */
    data    PT_LOAD FLAGS(6);   /* RW- */
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text : {
        __text_start = .;
        KEEP(*(.text.kmain))
        *(.text .text.*)
        __text_end = .;
    } :text

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        __rodata_start = .;
        *(.rodata .rodata.*)
    } :rodata
    
    /* Limine requests sections - must be in rodata for Limine to find them */
    .requests_start_marker : {
        KEEP(*(.requests_start_marker))
    } :rodata

    .requests : {
        KEEP(*(.requests))
    } :rodata

    .requests_end_marker : {
        KEEP(*(.requests_end_marker))
    } :rodata

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .data : {
        *(.data .data.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    .got : {
        *(.got .got.*)
    } :data

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}
