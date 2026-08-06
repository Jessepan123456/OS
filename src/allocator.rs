//! Heap memory allocation support for the kernel
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr
};
use bump::BumpAllocator;
use linked_list::LinkedListAllocator;
use fixed_size::FixedSizeBlockAllocator;

pub mod bump;
pub mod linked_list;
pub mod fixed_size;

/// A placeholder allocator when no allocator is initialized.
pub struct Dummy;

/// A thread-safe wrapper around an allocator
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

/// Starting virtual address of the kernel heap
pub const HEAP_START: usize = 0x_4444_4444_0000;

//// Size of the kernel heap in bytes
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
/// The kernel's active heap allocator
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
// static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(
    FixedSizeBlockAllocator::new()
);

impl<A> Locked<A> {
    /// Creates a new locked allocator
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    /// Locks the allocator and reutns mutable access to it
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

unsafe impl GlobalAlloc for Dummy {
    /// Attempts to allocate memory
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    /// Deallocates memory
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("deadlloc should be never called")
    }
}

/// Initializes the kernel heap.
/// 
/// This function maps the virtual memory pages to physical memory frame
/// and initializes the global allocator
pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

/// Align the given address 'addr' upwards to alignment 'align'.
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
}