use alloc::boxed::Box;
use core::marker::PhantomData;
use async_channel::Sender;
use zcene_core::actor::{ActorMessage, ActorMessageSender, ActorSendError, ActorFuture};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel_derive::Constructor;
use crate::actor::channel::pt_message::PtMessage;
use crate::actor::channel::pt_message::PtMessage::Copy;

#[derive(Constructor)]
pub struct PtActorMessageChannelSender<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler
{
    sender: Sender<PtMessage<M, H>>,
    alloc: H::Allocator,
}

impl<M, H> Clone for PtActorMessageChannelSender<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler
{
    fn clone(&self) -> Self {
        Self::new(self.sender.clone(), self.alloc.clone())
    }
}

impl<M, H> ActorMessageSender<PtMessage<M, H>> for PtActorMessageChannelSender<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler
{
    fn send(&self, msg: PtMessage<M, H>) -> impl ActorFuture<'_, Result<(), ActorSendError>> {
        async move {
            self.sender.send(msg).await.map_err(|_| ActorSendError::Closed)
        }
    }
}

impl<M, H> ActorMessageSender<M> for PtActorMessageChannelSender<M, H>
where
    M: ActorMessage,
    H: FutureRuntimeHandler
{
    fn send(&self, msg: M) -> impl ActorFuture<'_, Result<(), ActorSendError>> {
        async move {
            self.sender.send(Copy(Box::new_in(msg, self.alloc.clone()))).await.map_err(|_| ActorSendError::Closed)
        }
    }
}