use alloc::boxed::Box;
use core::fmt::Debug;
use zcene_core::actor::{
    Actor, ActorBoxFuture, ActorCommonBounds, ActorEnvironment, ActorEnvironmentAllocator,
    ActorMessageChannelAddress, ActorMessageSender, ActorSendError,
};

pub trait ActorUserMessageHandler<E>: ActorCommonBounds
where
    E: ActorEnvironment + ActorEnvironmentAllocator,
{
    fn send(
        &self,
        allocator: &E::Allocator,
        message: *const (),
    ) -> ActorBoxFuture<'static, Result<(), ActorSendError>, E>;
}

impl<A, E> ActorUserMessageHandler<E> for ActorMessageChannelAddress<A, E>
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

        // TODO: This is where the actual copying happens! This should then be replaced with a syscall / batching strategy.
        let message = unsafe { message.cast::<A::Message>().as_ref().unwrap() }.clone();

        Box::pin_in(
            async move { <Self as ActorMessageSender<_>>::send(&sender, message).await },
            allocator.clone(),
        )
    }
}
