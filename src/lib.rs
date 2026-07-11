//! Core kernal library.
//! 
//! This module provides the basic function needed by the kernel,
//! including init, custom testing support, QEMU handling, and CPU 
//! hlt function.

#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;

/// Trait implemented by functions that can be executed as kernel tests.
/// 
/// The custom test framework uses this trait to run tests and print their
/// status through the serial port.
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where 
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// Runs all registered kernel tests.
/// 
/// Each test is executed individually and its result is printed through
/// the serial interface. After all tests complete successfully, QEMU is exited.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Handles panics during kernel tests.
/// 
/// The panic information is printed and QEMU exits with a failure code.
pub fn test_panic_handler(info: &PanicInfo) -> !{
    serial_println!("[Failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

/// Entry point used when running 'cargo test'
#[cfg(test)]
#[unsafe(no_mangle)] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    hlt_loop();
}

// /Our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
/// Exit codes used to communicate test results
/// 
/// Ther kernel cannot return normally, so it writes these values to a special
/// I/O port that QEMU montiors.
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Sends an exit code to QEMU
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// Initializes core kernel components
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

/// Puts the CPU into an idle loop
/// 
/// The HLT instruction stop CPU execution until an interrupt occurs,
/// preventing the kernel from wasting CPU resources while waiting.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}