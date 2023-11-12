SECTIONS {
    . = DEFINED(ALGO_PLACEMENT_START_ADDRESS) ? ALGO_PLACEMENT_START_ADDRESS : 0x0;

    /*
     * The PrgCode output section name comes from the CMSIS-Pack flash algorithms
     * templates and armlink. It is used here because several tools that work
     * with these flash algorithms expect this section name.
     *
     * All input sections are combined into PrgCode because RWPI using R9 is not
     * currently stable in Rust, thus having separate PrgData sections that the
     * debug host might locate at a different offset from PrgCode is not safe.
     */
    PrgCode : {
        KEEP(*(.entry))
        KEEP(*(.entry.*))

        *(.text)
        *(.text.*)

        *(.rodata)
        *(.rodata.*)

        *(.data)
        *(.data.*)

        *(.sdata)
        *(.sdata.*)
        
        *(.bss)
        *(.bss.*)

        *(.uninit)
        *(.uninit.*)

        . = ALIGN(4);
    }

    /* Section for data, specified by flashloader standard. */
    PrgData : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    PrgData : {
        /* Zero-initialized data */
        *(.bss .bss.*)
        *(.sbss .sbss.*)

        *(COMMON)
    }

    /* Description of the flash algorithm */
    DeviceData . : {
        /* The device data content is only for external tools,
         * and usually not referenced by the code.
         *
         * The KEEP statement ensures it's not removed by accident.
         */
        KEEP(*(DeviceData))
    }

    /* Description of the self tests */
    SelfTestInfo . : {
        KEEP(*(SelfTestInfo))
    }

    /DISCARD/ : {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}
