PHDRS
{
    text    PT_LOAD FLAGS (0x1);
    rodata  PT_LOAD FLAGS (0x4);
    data    PT_LOAD FLAGS (0x6);
    limine  PT_LOAD FLAGS (0x6);
    dynamic PT_DYNAMIC FLAGS (0x6);
}

SECTIONS
{
    . = 0xffffffff80000000;
    PROVIDE(ktext_start = .);


    .text : {
        *(.text*)
    } :text


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    PROVIDE(ktext_end = .);
    PROVIDE(krodata_start = .);


    .rodata : {
        *(.rodata*)
    } :rodata

    .rela.dyn : {
        *(.rela*)
        *(.rel*)
    } :rodata
    .dynsym : {
        *(.dynsym*)
    } :rodata
    .dynstr : {
        *(.dynstr*)
    } :rodata
    .gnu.hash : { *(.gnu.hash*) } :rodata
    .hash     : { *(.hash*) } :rodata


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    PROVIDE(krodata_end = .);
    PROVIDE(kdata_start = .);


    .data : {
        *(.data*)
    } :data

    .got : {
        *(.got*)
    } :data

    .dynamic : {
        *(.dynamic*)
    } :data :dynamic

    .bss (NOLOAD) : {
        *(.bss*)
        *(COMMON)
    } :data


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    PROVIDE(kdata_end = .);
    PROVIDE(klimine_start = .);


    .limine : {
        KEEP(*(.limine*))
    } :limine


    . = ALIGN(CONSTANT(MAXPAGESIZE));
    PROVIDE(klimine_end = .);


    /DISCARD/ : {
        *(.eh_frame*)
        *(.note*)
    }
}
