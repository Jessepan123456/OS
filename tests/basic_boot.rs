//! Basic boot test.
//! 
//! Verifies that the kernel can successfully boot and run the test framework. 

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::println;

/// Kernal entry point
/// 
/// This function is called when the kernel starts. Since the normal Rust
/// runtime is disabled.
#[unsafe(no_mangle)] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    // Keeps the CPU alive when something is return to
    loop {}
}

/// Handles panics during kernel tests.
/// 
/// Since there is no operating system to return control to, the kernel
/// reports the panic and stops execution.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
} 

/// Verifies that the kernal println macro works correctly.
#[test_case]
fn test_println() {
    println!("test_println output");
}