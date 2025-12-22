use crate::actor::env::root::environment::RootEnvironment;
use crate::actor::env::user::environment::UserEnvironment;
use crate::actor::env::user::executor_event::UserExecutorEvent;
use crate::actor::env::user::message_handler::UserMessageHandler;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::num::NonZero;
use kernel_derive::Constructor;
use zcene_core::actor::{Actor, ActorEnvironmentAllocator, ActorMessageChannelReceiver};
use zcene_core::future::runtime::FutureRuntimeHandler;

#[derive(Constructor)]
pub struct UserExecutor<A, AR, H>
where
    A: Actor<UserEnvironment>,
    AR: Actor<RootEnvironment<H>, Message = A::Message>,
    H: FutureRuntimeHandler,
{
    allocator: <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
    actor: Box<A>,
    receiver: ActorMessageChannelReceiver<AR::Message>,
    deadline_in_ms: Option<NonZero<usize>>,
    message_handlers: Vec<
        Box<
            dyn UserMessageHandler<RootEnvironment<H>>,
            <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
        >,
        <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
    >,
    marker_ar: PhantomData<AR>,
    marker_h: PhantomData<H>,
}

impl<A, AR, H> UserExecutor<A, AR, H>
where
    A: Actor<UserEnvironment>,
    AR: Actor<RootEnvironment<H>, Message = A::Message>,
    H: FutureRuntimeHandler,
{
    pub async fn run(mut self) {
        // TODO

        loop {
            let message = match self.receiver.receive().await {
                None => break,
                Some(m) => m,
            };

            // TODO
        }

        // TODO
    }

    async fn handle<F>(&mut self, func: F)
    where
        F: FnOnce(&mut Box<A>, &mut Option<UserExecutorEvent>, u64),
    {
        let mut event: Option<UserExecutorEvent> = None;

        // Todo: User stack

        // Todo: enable deadline

        // Todo: execute function

        loop {
            match event.take() {
                None => break,
                Some(UserExecutorEvent::SystemCall()) => {}
                Some(UserExecutorEvent::Preemption()) => {}
            }
        }
    }
}
