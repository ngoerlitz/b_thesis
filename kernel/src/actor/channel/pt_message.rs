use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use zcene_core::future::runtime::{FutureRuntimeHandler, FutureRuntimeAllocator};
use zcene_core::actor::{ActorMessage};

pub enum PtMessage<M: ActorMessage, H: FutureRuntimeHandler> {
    Copy(Box<M, H::Allocator>),
    Page(usize, usize)
}

impl<M, H> Debug for PtMessage<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            PtMessage::Copy(m) => {
                let ptr: *const M = &**m; // pointer to heap allocation
                f.debug_tuple("Copy")
                    .field(&format_args!("{:p}", ptr))
                    .finish()
            }
            PtMessage::Page(id, addr) => f.debug_tuple("Page").field(id).field(addr).finish(),
        }
    }
}

impl<M, H> Clone for PtMessage<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler,
    H::Allocator: FutureRuntimeAllocator,
{
    fn clone(&self) -> Self {
        match self {
            PtMessage::Copy(b) => PtMessage::Copy(Box::clone(b)),
            PtMessage::Page(page_id, pa) => PtMessage::Page(*page_id, *pa),
        }
    }
}