use crate::{kprintln, linker_symbols};
use linked_list_allocator::LockedHeap;

linker_symbols! {
    HEAP_START = __heap_start;
    HEAP_END = __heap_end;
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    unsafe {
        let start = HEAP_START();
        let heap_size = HEAP_END() - HEAP_START();

        ALLOCATOR.lock().init(start as *mut u8, heap_size);
    }
}
