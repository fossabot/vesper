ENTRY(kstart)
OUTPUT_ARCH(aarch64)
OUTPUT_FORMAT(elf64-aarch64) # Should be binary really

START_ADDRESS = 0x8000;

SECTIONS {
    . = START_ADDRESS;

    .text ALIGN (4) : {
        aarch64.o(*) # Vectors and entry point
        *(.text*)
    }

    .rodate ALIGN (4) : {
        *(.rodata*)
        FILL(0x00)
    }

    .data ALIGN (4) : {
        *(.data*)
        FILL(0x00)
    }

    .bss ALIGN (4) : {
        *(COMMON*)
        *(.bss*)
    }

    /DISCARD/ : {
        *(.comment .note* .dtors)
    }
}
