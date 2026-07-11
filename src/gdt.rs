//! Global Descriptor Table (GDT) and Task State Segment (TSS) setup.
//! 
//! The GDT defines CPU segments used by the kernel, including the kernel
//! code segment and TSS segment. The TSS provides special stacks for CPU
//! exceptions, such as the double fault handler.
//!
//! A separate Interrupt Stack Table (IST) stack is used so that the CPU can
//! handle critical exceptions even if the normal kernel stack is corrupted.

use x86_64::VirtAddr;
use x86_64::structures::{tss::TaskStateSegment, gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector}};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static!{
    /// Task State Segment used by the CPU for special exception handling.
    /// 
    /// The TSS stores and IST. The double fault handler uses a decdicated stack.
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // Reserve memory for the double fault stack.
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static!{
    /// Global Descriptor Table containing kernel segments and the TSS entry.
    /// 
    /// GDT are used later to load 
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

/// Stores references to entries inside the GDT.
/// 
/// Segment selectors are offsets that allow the CPU to locate specific descriptors
/// inside the GDT.
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

/// Loads the GDT and initializes the CPU segmentation registers.
/// 
/// Sets the code segment register and loads the TSS so the CPU can use it.
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

