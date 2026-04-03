use alloc::alloc::Global;
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
    H: FutureRuntimeHandler<Allocator = Global>
{
    fn send(
        &self,
        allocator: &E::Allocator,
        message: *const (),
    ) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E> {
        let sender = self.clone();

        let msg_box = unsafe { Box::from_raw(message as *mut A::Message) };

        Box::pin_in(
            async move { sender.send_msg(PtMessage::Copy(msg_box)).await },
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
