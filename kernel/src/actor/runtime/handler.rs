use crate::getter_assoc;
use alloc::alloc::Global;
use zcene_core::future::runtime::{
    FutureRuntimeConcurrentQueue, FutureRuntimeContinueWaker, FutureRuntimeHandler,
    FutureRuntimeNoOperationYielder, FutureRuntimeQueue,
};

#[derive(Default)]
pub struct RuntimeHandler {
    allocator: Global,
    queue: FutureRuntimeConcurrentQueue<Self>,
    yielder: FutureRuntimeNoOperationYielder,
    waker: FutureRuntimeContinueWaker,
}

impl FutureRuntimeHandler for RuntimeHandler {
    type Allocator = Global;
    type Queue = FutureRuntimeConcurrentQueue<Self>;
    type Yielder = FutureRuntimeNoOperationYielder;
    type Waker = FutureRuntimeContinueWaker;
    type Data = ();
    type Specification = ();

    getter_assoc!(allocator);
    getter_assoc!(queue);
    getter_assoc!(yielder);
    getter_assoc!(waker);
}
