ENTRY(_start)

SECTIONS {
    . = 0x80000; /* AArch64 boot address is 0x80000 */

    .text : {
        *(.entry)
        *(.text*)
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        *(COMMON*)
        *(.bss*)
    }

    /DISCARD/ : {
        *(.comment .note* .dtors)
    }
}
