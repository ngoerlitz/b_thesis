use alloc::boxed::Box;
use core::marker::PhantomData;
use zcene_core::actor::{Actor, ActorAddress, ActorEnvironment, ActorFuture, ActorMessageChannelAddress, ActorMessageSender, ActorSendError};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel_derive::Constructor;
use crate::actor::channel::pt_channel_sender::PtActorMessageChannelSender;
use crate::actor::channel::pt_message::PtMessage;
use crate::kprintln;

#[derive(Constructor)]
pub struct PtActorMessageChannelAddress<A, E, H>
where
    A: Actor<E>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    sender: PtActorMessageChannelSender<A::Message, H>,
    alloc: H::Allocator,
    handler_type: PhantomData<E>,
}

impl<A, E, H> Clone for PtActorMessageChannelAddress<A, E, H>
where
    A: Actor<E>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    fn clone(&self) -> Self {
        Self::new(self.sender.clone(), self.alloc.clone(), PhantomData)
    }
}

impl<A, E, H> ActorMessageSender<A::Message> for PtActorMessageChannelAddress<A, E, H>
where
    A: Actor<E>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    fn send(&self, message: A::Message) -> impl ActorFuture<'_, Result<(), ActorSendError>> {
        async move {
            self.sender.send(PtMessage::Copy(Box::new_in(message, self.alloc.clone()))).await
        }
    }
}

impl<A, E, H> PtActorMessageChannelAddress<A, E, H>
where
    A: Actor<E>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    pub fn send_msg(&self, message: PtMessage<A::Message, H>) -> impl ActorFuture<'_, Result<(), ActorSendError>> {
        async move {
            self.sender.send(message).await
        }
    }
}

impl<A, E, H> ActorAddress<A, E> for PtActorMessageChannelAddress<A, E, H>
where
    A: Actor<E>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
}