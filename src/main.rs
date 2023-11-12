#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use flash_algorithm::*;
use rtt_target::{rprintln, rtt_init_print};

struct Algorithm;

algorithm!(Algorithm, {
    target_name: "stm32wle5",
    flash_address: 0x08000000,
    flash_size: 0x40000,
    page_size: 0x400,
    empty_value: 0xFF,
    ram_start_addr: 0x20000000,
    ram_end_addr: 0x20010000,
    sectors: [{
        size: 0x400,
        address: 0x0,
    }],
    self_tests: [
        {
            test_id: 1,
            test_name: "test",
        }
    ],
});

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        rtt_init_print!();
        rprintln!("Init");
        // TODO: Add setup code for the flash algorithm.
        Ok(Self)
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        rprintln!("Erase All");
        // TODO: Add code here that erases the entire flash.
        Err(ErrorCode::new(0x70d0).unwrap())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        rprintln!("Erase sector addr:{}", addr);
        // TODO: Add code here that erases a page to flash.
        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        rprintln!("Program Page addr:{} size:{}", addr, data.len());
        // TODO: Add code here that writes a page to flash.
        Ok(())
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        // TODO: Add code here to uninitialize the flash algorithm.
    }
}
