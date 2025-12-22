use alloc::boxed::Box;
use core::fmt::Debug;
use zcene_core::actor::{
    Actor, ActorBoxFuture, ActorCommonBounds, ActorEnvironment, ActorEnvironmentAllocator,
    ActorMessageChannelAddress, ActorMessageSender, ActorSendError,
};

pub trait UserMessageHandler<E: ActorEnvironment + ActorEnvironmentAllocator>:
    ActorCommonBounds
{
    fn send(
        &self,
        allocator: &E::Allocator,
        message: *const (),
    ) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E>;
}

impl<A, E> UserMessageHandler<E> for ActorMessageChannelAddress<A, E>
where
    A: Actor<E>,
    A::Message: Debug,
    E: ActorEnvironment + ActorEnvironmentAllocator,
{
    fn send(
        &self,
        allocator: &E::Allocator,
        message: *const (),
    ) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E> {
        let sender = self.clone();

        let message = unsafe { message.cast::<A::Message>().as_ref().unwrap() }.clone();

        Box::pin_in(
            async move { <Self as ActorMessageSender<_>>::send(&sender, message).await },
            allocator.clone(),
        )
    }
}
