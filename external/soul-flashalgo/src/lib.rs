//! Implement a [CMSIS-Pack] flash algorithm in Rust
//!
//! [CMSIS-Pack]: https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/flashAlgorithm.html
//!
//! # Feature flags
//!
//! - `panic-handler` this is enabled by default and includes a simple abort-on-panic
//!   panic handler. Disable this feature flag if you would prefer to use a different
//!   handler.

#![no_std]
#![no_main]
#![macro_use]

#[cfg(all(not(test), feature = "panic-handler"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("udf #0");
        core::hint::unreachable_unchecked();
    }
}

pub const FUNCTION_ERASE: u32 = 1;
pub const FUNCTION_PROGRAM: u32 = 2;
pub const FUNCTION_VERIFY: u32 = 3;

pub type ErrorCode = core::num::NonZeroU32;

pub const fn assign_name<const OPT_LEN: usize>(name: &str) -> [u8; OPT_LEN] {
    let name_len = name.len();
    let arr = name.as_bytes();
    assert!(name_len <= OPT_LEN);
    let mut ans = [0; OPT_LEN];
    let mut i = 0;
    while i < name_len {
        ans[i] = arr[i];
        i += 1;
    }
    ans
}

pub trait FlashAlgorithm: Sized + 'static {
    /// Initialize the flash algorithm.
    ///
    /// It can happen that the flash algorithm does not need any specific initialization
    /// for the function to be executed or no initialization at all. It is up to the implementor
    /// to decide this.
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash region to program.
    /// * `clock` - The clock speed in Hertz for programming the device.
    /// * `function` - The function for which this initialization is for.
    fn new(address: u32, clock: u32, function: Function) -> Result<Self, ErrorCode>;

    /// Erase entire chip. Will only be called after [`FlashAlgorithm::new()`] with [`Function::Erase`].
    #[cfg(feature = "erase-chip")]
    fn erase_all(&mut self) -> Result<(), ErrorCode>;

    /// Erase sector. Will only be called after [`FlashAlgorithm::new()`] with [`Function::Erase`].
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash sector to erase.
    fn erase_sector(&mut self, address: u32) -> Result<(), ErrorCode>;

    /// Program bytes. Will only be called after [`FlashAlgorithm::new()`] with [`Function::Program`].
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash page to program.
    /// * `data` - The data to be written to the page.
    fn program_page(&mut self, address: u32, data: &[u8]) -> Result<(), ErrorCode>;

    /// Verify the firmware that has been programmed.  Will only be called after [`FlashAlgorithm::new()`] with [`Function::Verify`].
    ///
    /// # Arguments
    ///
    /// * `address` - The start address of the flash to verify.
    /// * `size` - The length of the data to verify.
    /// * `data` - The data to compare with.
    #[cfg(feature = "verify")]
    fn verify(&mut self, address: u32, size: u32, data: Option<&[u8]>) -> Result<(), ErrorCode>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Function {
    Erase = 1,
    Program = 2,
    Verify = 3,
}

/// A macro to define a new flash algoritm.
///
/// It takes care of placing the functions in the correct linker sections
/// and checking the flash algorithm initialization status.
#[macro_export]
macro_rules! algorithm {
    ($type:ty, {
        target_name: $target_name:expr,
        flash_address: $flash_address:expr,
        flash_size: $flash_size:expr,
        page_size: $page_size:expr,
        empty_value: $empty_value:expr,
        ram_start_addr: $ram_start_addr:expr,
        ram_end_addr: $ram_end_addr:expr,
        sectors: [$({
            size: $size:expr,
            address: $address:expr,
        }),+],
        self_tests: [$({
            test_id: $test_id:expr,
            test_name: $test_name:expr,
        }),+],
    }) => {
        static mut _IS_INIT: bool = false;
        static mut _ALGO_INSTANCE: core::mem::MaybeUninit<$type> = core::mem::MaybeUninit::uninit();

        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn Init(addr: u32, clock: u32, function: u32) -> u32 {
            if _IS_INIT {
                UnInit();
            }
            _IS_INIT = true;
            let function = match function {
                1 => $crate::Function::Erase,
                2 => $crate::Function::Program,
                3 => $crate::Function::Verify,
                _ => panic!("This branch can only be reached if the host library sent an unknown function code.")
            };
            match <$type as FlashAlgorithm>::new(addr, clock, function) {
                Ok(inst) => {
                    _ALGO_INSTANCE.as_mut_ptr().write(inst);
                    _IS_INIT = true;
                    0
                }
                Err(e) => e.get(),
            }
        }
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn UnInit() -> u32 {
            if !_IS_INIT {
                return 1;
            }
            _ALGO_INSTANCE.as_mut_ptr().drop_in_place();
            _IS_INIT = false;
            0
        }
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn EraseSector(addr: u32) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgorithm>::erase_sector(this, addr) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn ProgramPage(addr: u32, size: u32, data: *const u8) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            let data_slice: &[u8] = unsafe { core::slice::from_raw_parts(data, size as usize) };
            match <$type as FlashAlgorithm>::program_page(this, addr, data_slice) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
        $crate::erase_chip!($type);
        $crate::verify!($type);

        #[allow(non_upper_case_globals)]
        #[no_mangle]
        #[used]
        #[link_section = "DeviceData"]
        pub static FlashDevice: FlashDeviceDescription = FlashDeviceDescription {
            // The version is never read by probe-rs and can be fixed.
            vers: 0x0,
            // Device (target uC) name, may be reused on some generic flash algo
            dev_name: assign_name($target_name),
            // The specification does not specify the values that can go here,
            // but this value means internal flash device.
            dev_type: 5,
            dev_addr: $flash_address,
            device_size: $flash_size,
            page_size: $page_size,
            _reserved: 0,
            // The empty state of a byte in flash.
            empty: $empty_value,
            // This value can be used to estimate the amount of time the flashing procedure takes worst case.
            program_time_out: 1000,
            // This value can be used to estimate the amount of time the erasing procedure takes worst case.
            erase_time_out: 2000,
            flash_sectors: [
                $(
                    FlashSector {
                        size: $size,
                        address: $address,
                    }
                ),+,
                // This marks the end of the flash sector list.
                FlashSector {
                    size: 0xffff_ffff,
                    address: 0xffff_ffff,
                }
            ],
        };

        #[allow(non_upper_case_globals)]
        #[no_mangle]
        #[used]
        #[link_section = "SelfTestInfo"]
        pub static SelfTestMetadata: SelfTestDescription = SelfTestDescription {
            magic: 0x536f_756c, // "Soul"
            test_cnt: $crate::count!($($test_id)*) as u32,
            ram_start_addr: $ram_start_addr,
            ram_end_addr: $ram_end_addr,
            test_items: [
                $(
                    SelfTestItem {
                        id: $test_id,
                        name: assign_name($test_name),
                    }
                ),+,
                // This marks the end of the flash sector list.
                SelfTestItem {
                    id: 0xffff_ffff,
                    name: [0xff; 32],
                }
            ]
        };

        #[repr(C)]
        pub struct FlashDeviceDescription {
            vers: u16,
            dev_name: [u8; 128],
            dev_type: u16,
            dev_addr: u32,
            device_size: u32,
            page_size: u32,
            _reserved: u32,
            empty: u8,
            program_time_out: u32,
            erase_time_out: u32,
            flash_sectors: [FlashSector; $crate::count!($($size)*) + 1],
        }

        #[repr(C, packed(1))]
        pub struct SelfTestItem {
            id: u32,
            name: [u8; 32],
        }

        #[repr(C, packed(1))]
        pub struct SelfTestDescription {
            magic: u32,
            ram_start_addr: u32,
            ram_end_addr: u32,
            test_cnt: u32,
            test_items: [SelfTestItem; $crate::count!($($test_id)*) + 1],
        }


        #[repr(C)]
        #[derive(Copy, Clone)]
        pub struct FlashSector {
            size: u32,
            address: u32,
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "erase-chip"))]
macro_rules! erase_chip {
    ($type:ty) => {};
}
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "erase-chip")]
macro_rules! erase_chip {
    ($type:ty) => {
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn EraseChip() -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();
            match <$type as FlashAlgorithm>::erase_all(this) {
                Ok(()) => 0,
                Err(e) => e.get(),
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "verify"))]
macro_rules! verify {
    ($type:ty) => {};
}
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "verify")]
macro_rules! verify {
    ($type:ty) => {
        #[no_mangle]
        #[link_section = ".entry"]
        pub unsafe extern "C" fn Verify(addr: u32, size: u32, data: *const u8) -> u32 {
            if !_IS_INIT {
                return 1;
            }
            let this = &mut *_ALGO_INSTANCE.as_mut_ptr();

            if data.is_null() {
                match <$type as FlashAlgorithm>::verify(this, addr, size, None) {
                    Ok(()) => 0,
                    Err(e) => e.get(),
                }
            } else {
                let data_slice: &[u8] = unsafe { core::slice::from_raw_parts(data, size as usize) };
                match <$type as FlashAlgorithm>::verify(this, addr, size, Some(data_slice)) {
                    Ok(()) => 0,
                    Err(e) => e.get(),
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}
