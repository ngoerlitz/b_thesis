use alloc::boxed::Box;
use core::fmt::Debug;
use zcene_core::actor::{
    Actor, ActorBoxFuture, ActorCommonBounds, ActorEnvironment, ActorEnvironmentAllocator,
    ActorMessageChannelAddress, ActorMessageSender, ActorSendError,
};
use zcene_core::future::runtime::FutureRuntimeHandler;
use crate::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use crate::actor::channel::pt_message::PtMessage;

pub trait UserMessageHandler<E: ActorEnvironment + ActorEnvironmentAllocator>:
    ActorCommonBounds
{
    fn send(
        &self,
        allocator: &E::Allocator,
        message: *const (),
    ) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E>;

    fn send_page(
        &self,
        allocator: &E::Allocator,
        page_id: usize,
        pa: usize,
    ) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E>;
}

impl<A, E, H> UserMessageHandler<E> for PtActorMessageChannelAddress<A, E, H>
where
    A: Actor<E>,
    A::Message: Debug,
    E: ActorEnvironment + ActorEnvironmentAllocator,
    H: FutureRuntimeHandler
{
    fn send(
        &self,
        allocator: &E::Allocator,
        message: *const (),
    ) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E> {
        let sender = self.clone();

        let message: <A as Actor<E>>::Message = unsafe { message.cast::<A::Message>().as_ref().unwrap() }.clone();

        Box::pin_in(
            async move { <Self as ActorMessageSender<_>>::send(&sender, message).await },
            allocator.clone(),
        )
    }

    fn send_page(&self, allocator: &E::Allocator, page_id: usize, pa: usize) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E> {
        let sender = self.clone();

        Box::pin_in(
            async move {
                sender.send_msg(PtMessage::Page(page_id, pa)).await
            },
            allocator.clone(),
        )
    }
}
