use crate::linker_symbols;
use linked_list_allocator::LockedHeap;

linker_symbols! {
    HEAP_START = __heap_start;
    HEAP_END = __heap_end;
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    unsafe {
        let start = 0x40004700usize;
        let heap_size = 0x7d000;

        ALLOCATOR.lock().init(start as *mut u8, heap_size);
    }
}
