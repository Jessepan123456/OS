//! Tests stack overflow handling.
//! 
//! This test overflows the stack to trigger a double fault.
//! The double fault handler uses a IST (Interrupt Stack Table)
//! stack from the TSS so it can run even when the original stack is corrupted.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use os::serial_print;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use os::{exit_qemu, QemuExitCode, serial_println};

lazy_static! {
    /// Interrupt Descriptor Table used for execption handling tests.
    /// 
    /// IDT overrides the default double fault handler and uses 
    /// the TSS-defined IST stack to safely handle stack overflow
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(os::gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

// This function loads the test IDT
pub fn init_test_idt() {
    TEST_IDT.load();
}

/// Runs when the stack overflow causes a double fault.
/// 
/// Reaching this handler means the CPU sucessfullly switched to a 
/// double fault stack
extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

/// Entry Point for the stack overflow test
/// 
/// Initializes the GDT and IDT before the stack overflow
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    os::gdt::init();
    init_test_idt();

    // trigger a stack overflow
    stack_overflow();

    panic!("Execution continued after stack overflow");
}


/// Panic Handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

/// Recursively calls itself forever for the stack
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}