use crate::actor::env::root::service::message_frame_allocator_service::MessageFrameAllocatorService;
use crate::kprintln;

#[cfg(feature = "test")]
pub fn test_all() {
    let mut svc = MessageFrameAllocatorService::new(0x1000);
    let ok = svc.self_test();
    kprintln!("[BOOT] MessageAllocatorService self_test: {}", ok);

    loop {}
}