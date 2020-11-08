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
#[macro_use]
extern crate ruspiro_boot;
extern crate alloc;
extern crate ruspiro_allocator;
use alloc::{boxed::Box, sync::Arc};
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicU16, Ordering},
};
use qemu_exit::QEMUExit;
use ruspiro_channel::mpmc;
pub use ruspiro_console::*;
use ruspiro_mailbox::Mailbox;
use ruspiro_mmu as mmu;
pub use ruspiro_test_macros::*;
use ruspiro_uart::Uart1;

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

/// Run an arbitrary closure on a specific core to allow concurrent execution
pub fn run_on_core<F>(core: u32, f: F) -> Result<(), ()>
where
    F: FnOnce() + Send + 'static,
{
    if core == 0 {
        return Err(()); // only allow running on cores other then 0!
    }

    // send the closure to the channel of the core that should execute this one
    unsafe {
        if let Some(ref core_channel) = CORE_CHANNEL {
            let sender = &core_channel[core as usize].channel.0;
            // as soon as we send something to the core to be processed we incree the counter of things that are in
            // flight for this core
            core_channel[core as usize]
                .closure_inflight
                .fetch_add(1, Ordering::AcqRel);
            sender.send(Box::new(f));
            Ok(())
        } else {
            Err(())
        }
    }
}

/// wait for execution on a specific core has been finished
pub fn wait_for_core(core: u32) {
    if core > 0 {
        // send the closure to the channel of the core that should execute this one
        unsafe {
            if let Some(ref core_channel) = CORE_CHANNEL {
                // wait until nothing is inflight for this core anymore
                while core_channel[core as usize]
                    .closure_inflight
                    .load(Ordering::Relaxed)
                    > 0
                {}
            }
        }
    }
}

struct CoreExecution {
    channel: (
        mpmc::Sender<Box<dyn FnOnce() + Send + 'static>>,
        mpmc::Receiver<Box<dyn FnOnce() + Send + 'static>>,
    ),
    closure_inflight: AtomicU16,
}

static mut CORE_CHANNEL: Option<[CoreExecution; 4]> = None;

come_alive_with!(prepare_test_runner);
run_with!(execute_test_runner);

fn prepare_test_runner(core: u32) {
    if core == 0 {
        let mut mailbox = Mailbox::new();
        let (vc_mem_start, vc_mem_size) = mailbox
            .get_vc_memory()
            .expect("Fatal issue getting VC memory split");

        unsafe { mmu::initialize(core, vc_mem_start, vc_mem_size) };

        let mut uart = Uart1::new();
        if uart.initialize(250_000_000, 115_200).is_ok() {
            CONSOLE.take_for(|cons| cons.replace(uart));
        }

        unsafe {
            for core in 0..4 {}
            CORE_CHANNEL.replace([
                CoreExecution {
                    channel: mpmc::channel(),
                    closure_inflight: AtomicU16::new(0),
                },
                CoreExecution {
                    channel: mpmc::channel(),
                    closure_inflight: AtomicU16::new(0),
                },
                CoreExecution {
                    channel: mpmc::channel(),
                    closure_inflight: AtomicU16::new(0),
                },
                CoreExecution {
                    channel: mpmc::channel(),
                    closure_inflight: AtomicU16::new(0),
                },
            ]);
        }
    }
}

fn execute_test_runner(core: u32) -> ! {
    if core == 0 {
        // now execute the tests
        unsafe { run_test_main() };

        for core in 1..4 {
            wait_for_core(core);
        }

        QEMU_EXIT.exit_success()
    } else {
        // check if the channel of this core has any work to do and if so
        // process the same
        unsafe {
            if let Some(ref core_channel) = CORE_CHANNEL {
                let receiver = &core_channel[core as usize].channel.1;
                loop {
                    while let Ok(closure) = receiver.recv() {
                        closure();
                        // as soon as the execution of the closure has finished we can decreas the things in flight at
                        // this core
                        core_channel[core as usize]
                            .closure_inflight
                            .fetch_sub(1, Ordering::AcqRel);
                    }
                }
            } else {
                loop {}
            }
        }
    }
}

#[panic_handler]
pub fn _panic_exit(_info: &PanicInfo) -> ! {
    QEMU_EXIT.exit_failure()
}

#[cfg(not(test))]//feature = "test")]
extern "C" {
    pub fn run_test_main();
}

#[cfg(test)] //not(feature = "test"))]
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

        // explicitly run something on core 1
        run_on_core(1, || {
            println!("test runs something on core 1");
        });

        // explicitly run something on core 2
        run_on_core(2, || {
            println!("test runs something on core 2");
        });

        // explicitly run something on core 3
        run_on_core(3, || {
            println!("test runs something on core 3");
        });

        // explicitly run something else on core 1
        run_on_core(1, || {
            println!("test runs something else on core 1");
        });

        // finally we wait until the work is done on all cores to finishe this test case
        // in case further processing might expect the other cores to have finished their tasks
        wait_for_core(1);
        wait_for_core(2);
        wait_for_core(3);
    }
}
