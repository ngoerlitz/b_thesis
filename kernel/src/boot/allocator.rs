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
        let start = HEAP_START();
        let end = HEAP_END();

        let heap_size = end - start;
        ALLOCATOR.lock().init(start as *mut u8, heap_size);
    }
}
