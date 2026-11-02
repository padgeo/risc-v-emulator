/* Memory layout for bare-metal RV32 guests.
 * ORIGIN and LENGTH must match rv_core::memmap::{RAM_BASE, RAM_SIZE}. */

OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY
{
    RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 16M
}

SECTIONS
{
    .text : ALIGN(4)
    {
        KEEP(*(.text.init))
        *(.text .text.*)
    } > RAM

    .rodata : ALIGN(4)
    {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > RAM

    .sdata : ALIGN(4)
    {
        *(.sdata .sdata.*)
    } > RAM

    .data : ALIGN(4)
    {
        *(.data .data.*)
    } > RAM

    /* Zero-filled by the ELF loader (memsz > filesz), so _start does not clear it. */
    .bss (NOLOAD) : ALIGN(4)
    {
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4);
    } > RAM

    /* The stack grows down from the top of RAM. */
    __stack_top = ORIGIN(RAM) + LENGTH(RAM);

    /* Conventional global pointer, 0x800 into RAM so that gp-relative addressing (+-2 KiB) covers
     * the first 4 KiB. Defined here because .sdata is empty in small guests, and the linker's own
     * default for __global_pointer$ is meaningless then. */
    __global_pointer$ = ORIGIN(RAM) + 0x800;

    /DISCARD/ : { *(.eh_frame) *(.eh_frame_hdr) }
}
