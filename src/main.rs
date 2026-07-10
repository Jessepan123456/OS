#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os::println;

//Into bytes
// static HELLO: &[u8] = b"Hello World";

// don't mangle the name of this function
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    //this function is the entry point
    //_start is the default name
    println!("Hello World{}", "!");

    os::init();

    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();
    
    //Runs our tests
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}

/// This function is called on panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

// Version 1
// let vga_buffer = 0xb8000 as *mut u8;

// for (i, &byte) in HELLO.iter().enumerate() {
//     //Unsafe because it tells it that it valid
//     unsafe {
//         //Position
//         *vga_buffer.offset(i as isize * 2) = byte;
//         //Color
//         *vga_buffer.offset(i as isize * 2 + 1) = 0xF;
//     }
// }

// Version 2
// use core::fmt::Write;
// vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
// write!(vga_buffer::WRITER.lock(), ", some numbers: {} {}", 42, 1.337).unwrap();

// Version 3
//println!("Hello World{}", "!");