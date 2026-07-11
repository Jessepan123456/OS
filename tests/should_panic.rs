//! Tests that the kernel correctly handles panics.
//! 
//! This test trigger a panic. The test succeeds when the 
//! panic handler is reached and exits QEMU with a success code.

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::{QemuExitCode, exit_qemu, serial_print, serial_println};

/// Entry Point for the should_panic test.
/// 
/// Runs code that is expected to panic. If execution continues, the test failed
/// because no panic occurred.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    should_fall();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// Handles the expected panic.
/// 
/// Reaching this function means the test succeeded because panic was triggered
#[panic_handler]
fn panic(_info: & PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop{}
}

/// Trigger a panic
/// 
/// This verifies that the kernel correctly handles failed assertions.
fn should_fall() {
    serial_print!("Should_panic::should_fail...\t");
    assert_eq!(0, 1);
}
