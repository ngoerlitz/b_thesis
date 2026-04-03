use core::marker::PhantomData;
use async_channel::{bounded, unbounded};
use zcene_core::future::runtime::FutureRuntimeHandler;
use zcene_core::actor::ActorMessage;
use crate::actor::channel::pt_channel_receiver::PtActorMessageChannelReceiver;
use crate::actor::channel::pt_channel_sender::PtActorMessageChannelSender;

pub struct PtActorMessageChannel<M, H>(PhantomData<(M, H)>)
where
    M: ActorMessage,
    H: FutureRuntimeHandler;

impl<M, H> PtActorMessageChannel<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler,
{
    pub fn new_bounded(alloc: H::Allocator, n: usize) -> (PtActorMessageChannelSender<M, H>, PtActorMessageChannelReceiver<M, H>) {
        let (sender, receiver) = bounded(n);

        (
            PtActorMessageChannelSender::new(sender, alloc),
            PtActorMessageChannelReceiver::new(receiver),
        )
    }

    pub fn new_unbounded(alloc: H::Allocator) -> (PtActorMessageChannelSender<M, H>, PtActorMessageChannelReceiver<M, H>) {
        let (sender, receiver) = unbounded();

        (
            PtActorMessageChannelSender::new(sender, alloc),
            PtActorMessageChannelReceiver::new(receiver),
        )
    }
}