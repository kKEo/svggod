//! The `svggod` command line program. Everything it does lives in the library; what
//! is here is the one thing a library must not do for its users: choose the allocator.

use std::alloc::{GlobalAlloc, Layout, System};

use svggod::memory;

/// The system allocator, held to the program's memory limit (`--max-memory`). An
/// allocation that would pass the limit ends the program with the `memory-limit`
/// error, so that no input, whatever it holds, can take the machine's memory.
struct Limited;

// SAFETY: every request goes to the system allocator unchanged, with the layout it
// came with; this type only counts bytes before and after. When the count would
// pass the limit, `memory::exceeded` ends the process and never returns, so no
// caller is ever handed memory that was not allocated.
unsafe impl GlobalAlloc for Limited {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if !memory::charge(layout.size()) {
            memory::exceeded()
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if !memory::charge(layout.size()) {
            memory::exceeded()
        }
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        memory::release(layout.size());
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if new_size > layout.size() {
            if !memory::charge(new_size - layout.size()) {
                memory::exceeded()
            }
        } else {
            memory::release(layout.size() - new_size);
        }
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: Limited = Limited;

fn main() {
    svggod::app::main()
}
