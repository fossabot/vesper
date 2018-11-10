.globl _start
.section .entry, "x"
_start:
    orr sp, xzr, #0x80000 // Set sp to 0x80000 (just before kernel start)
    b =karch_start

// Crazy initial pagetables here
// Identity map first Gigabyte minus 16 meg for physical devices bus
// Identity map physical devices as well (0x3f000000-..)
/*section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb 64
stack_top:
*/
