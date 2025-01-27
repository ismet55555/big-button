/* memory.x - Tell linker how much memory is available and where it is */
/* Note that these values are specific to our microcontroller */
/* Source: https://github.com/embassy-rs/embassy/blob/main/examples/rp/memory.x */

MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 264K
}
