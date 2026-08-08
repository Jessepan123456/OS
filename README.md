# Rust Operating System
A small operating system in Rust, built up from operating-system concepts, low-level programming, memory management, and asynchronous task execution.

## Overview
The project uses x86_64 systems; it was developed to gain experience in how an operating system works and is made.

Implementation includes:
- Memory Allocation
- Paging
- Task management
- Asynchronous execution

## Features
- Custom heap memory allocation + Multiple heap allocator implementations
- Virtual memory and paging
- Task Management
- Asynchronous task executor
- x86_64 support
- Hardware-level interaction
- Kernel debugging and testing through QEMU

## Running
Built and run in Rust and uses the QEMU emulator 

## What I Learned
- Low-level Rust programming
- Operating system architecture
- Virtual memory and page tables
- Heap allocation
- Memory safety without a standard library
- Asynchronous task execution
- Debugging kernel-level code
- Working in a no_std
- Booting and developing a custom kernel
- Testing and running an OS through QEMU
- Interrupt handling

## Credits
This project was made while following Philipp Oppermann's Writing an OS in Rust series.

The tutorial focuses on building an operating system in Rust, including concepts like booting, memory management, paging, heap allocation, interrupts, and asynchronous task execution.

https://os.phil-opp.com 
