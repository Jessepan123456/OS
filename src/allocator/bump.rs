use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// A bump allocator that allocates memory sequentially from a fixed heap.
/// 
/// A bump allocator keeps track of the next available address in the heap.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,

    /// Number of currently active allocations
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new empty bump allocator
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    /// 
    /// This method is unsafe because the caller must ensure that the given 
    /// memory range is unused.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_size;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

/// Implements Rust's global allocation interface for the bump allocator.
/// 
/// Allocations are performed sequentially from the heap.
/// 
/// Deallocation does not immediately reclaim individual allocations. Instead,
/// the allocator resets the entire heap once the number reaches zero.
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /// Allocates a block of memory with requested layout.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        // Align the next available address
        let alloc_start = align_up(bump.next, layout.align());

        // calculate the end of the allocation
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    /// Deallocates a previously allocated block
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // get a mutable reference

        bump.allocations -= 1;

        // Reset the allocator when all allocations have been freed.
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
