/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann <pspwizard@gmx.de>
 * License: Apache License 2.0 / MIT
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-test-framework/||VERSION||")]
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(test_runner)]

//! # RusPiRo Test Framework
//! 
//! Thanks to the preparation work and initial investigations of [@André Richter](https://github.com/andre-richter) and
//! [@Phil Opp](https://github.com/phil-opp) this crate provides the core test framework to enable unit and integration 
//! testing for the *ruspiro-* family of crates to be executed using QEMU.
//!

extern crate ruspiro_boot;
extern crate ruspiro_allocator;
use core::panic::PanicInfo;
use qemu_exit::QEMUExit;
use ruspiro_console::*;
use ruspiro_mailbox::Mailbox;
use ruspiro_uart::Uart1; 
use ruspiro_mmu as mmu;
pub use ruspiro_test_macros::*;

#[cfg(target_arch = "aarch64")]
const QEMU_EXIT: qemu_exit::AArch64 = qemu_exit::AArch64::new();

/// Unit test container structure
pub struct UnitTest {
    /// Name of the test.
    pub name: &'static str,

    /// Function pointer to the test.
    pub test_func: fn(),
}

/// Default test runner for unit tests
pub fn test_runner(tests: &[&UnitTest]) {
    println!("Running {} tests", tests.len());
    println!("-------------------------------------------------------------------\r\n");
    
    for (i, test) in tests.iter().enumerate() {
        print!("{:>3}. {:.<58}", i + 1, test.name);

        // Run the actual test.
        (test.test_func)();

        // Failed tests call panic!(). Execution reaches here only if the test has passed.
        println!("[ok]")
    }

    println!();
}

#[no_mangle]
pub unsafe fn __rust_entry(core: u32) -> ! {
    let mut mailbox = Mailbox::new();
    let (vc_mem_start, vc_mem_size) = mailbox
        .get_vc_memory()
        .expect("Fatal issue getting VC memory split");

    mmu::initialize(core, vc_mem_start, vc_mem_size);

    let mut uart = Uart1::new();
    if uart.initialize(250_000_000, 115_200).is_ok() {
        CONSOLE.take_for(|cons| cons.replace(uart));
    }

    // now execute the tests
    run_test_main();

    QEMU_EXIT.exit_success()
}

#[panic_handler]
pub fn _panic_exit(_info: &PanicInfo) -> ! {
    QEMU_EXIT.exit_failure()
}

#[cfg(feature = "test")]
extern "C" {
    pub fn run_test_main();
}

#[cfg(not(feature = "test"))]
fn run_test_main() {
    #[cfg(test)]
    test_main();
}


#[cfg(test)]
mod tests {
    use super::*;
    use ruspiro_test_macros::ruspiro_test;

    #[ruspiro_test]
    fn simple_unittest() {
        assert_eq!(1, 1);
    }
}
