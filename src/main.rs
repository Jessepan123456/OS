#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
use core::panic::PanicInfo;
use os::println;
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::{Page};

entry_point!(kernal_main);

/// Kernel entry point.
/// 
/// Initializes the kernel, runs tests when compiled in test mode, and the netners
/// the hlt loop to the CPU from wasting rescoures.
// don't mangle the name of this function
fn kernal_main(boot_info: &'static BootInfo) -> ! {
    use os::memory;
    use os::allocator;
    use os::memory::{BootInfoFrameAllocator};
    use x86_64::{VirtAddr};

    //this function is the entry point
    //_start is the default name
    println!("Hello World{}", "!");
    os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    //allocate a number on the heap
    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    //Runs our tests
    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    os::hlt_loop();
}

/// Handles kernel panics during normal execution.
/// 
/// Prints panic information and stops the CPU because there is no operating
/// system to return control to.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();
}

/// Handles panics during kernel tests.
/// 
/// A panic during testing is reported through the test panic handler so the
/// test framework can mark the test as failed.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}

//Into bytes
// static HELLO: &[u8] = b"Hello World";

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