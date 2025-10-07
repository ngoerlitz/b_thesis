use alloc::alloc::Global;
use core::alloc::GlobalAlloc;
use zcene_core::future::runtime::{
    FutureRuntime, FutureRuntimeConcurrentQueue, FutureRuntimeContinueWaker, FutureRuntimeHandler,
    FutureRuntimeNoOperationYielder, FutureRuntimeReference,
};

pub type KernelFutureRuntime = FutureRuntime<KernelFutureRuntimeHandler>;
pub type KernelFutureRuntimeReference = FutureRuntimeReference<KernelFutureRuntimeHandler>;

pub struct KernelFutureRuntimeHandler {
    allocator: Global,
    queue: FutureRuntimeConcurrentQueue<Self>,
    yielder: FutureRuntimeNoOperationYielder,
    waker: FutureRuntimeContinueWaker,
}

impl Default for KernelFutureRuntimeHandler {
    fn default() -> Self {
        Self {
            allocator: Global::default(),
            queue: FutureRuntimeConcurrentQueue::default(),
            yielder: FutureRuntimeNoOperationYielder::default(),
            waker: FutureRuntimeContinueWaker::default(),
        }
    }
}

impl FutureRuntimeHandler for KernelFutureRuntimeHandler {
    type Allocator = Global;
    type Queue = FutureRuntimeConcurrentQueue<Self>;
    type Yielder = FutureRuntimeNoOperationYielder;
    type Waker = FutureRuntimeContinueWaker;
    type Data = ();
    type Specification = ();

    fn allocator(&self) -> &Self::Allocator {
        &self.allocator
    }

    fn queue(&self) -> &Self::Queue {
        &self.queue
    }

    fn yielder(&self) -> &Self::Yielder {
        &self.yielder
    }

    fn waker(&self) -> &Self::Waker {
        &self.waker
    }
}
