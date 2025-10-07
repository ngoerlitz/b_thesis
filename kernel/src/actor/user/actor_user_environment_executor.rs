use crate::actor::root::actor_root_environment::ActorRootEnvironment;
use crate::actor::user::actor_user_environment::ActorUserEnvironment;
use crate::actor::user::actor_user_executor_event::ActorUserExecutorEvent;
use crate::actor::user::actor_user_message_handler::ActorUserMessageHandler;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::num::NonZero;
use zcene_core::actor::{Actor, ActorEnvironmentAllocator, ActorMessageChannelReceiver};
use zcene_core::future::runtime::FutureRuntimeHandler;

pub struct ActorUserExecutor<AI, AR, H>
where
    AI: Actor<ActorUserEnvironment>,
    AR: Actor<ActorRootEnvironment<H>, Message = AI::Message>,
    H: FutureRuntimeHandler,
{
    allocator: <ActorRootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
    actor: Box<AI>,
    receiver: ActorMessageChannelReceiver<AR::Message>,
    deadline_in_ms: Option<NonZero<usize>>,
    message_handlers: Vec<
        Box<
            dyn ActorUserMessageHandler<ActorRootEnvironment<H>>,
            <ActorRootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
        >,
    >,
    marker: PhantomData<(AR, H)>,
}

impl<AI, AR, H> ActorUserExecutor<AI, AR, H>
where
    AI: Actor<ActorUserEnvironment>,
    AR: Actor<ActorRootEnvironment<H>, Message = AI::Message>,
    H: FutureRuntimeHandler,
{
    pub fn new(
        allocator: <ActorRootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
        actor: Box<AI>,
        receiver: ActorMessageChannelReceiver<AR::Message>,
        deadline_in_ms: Option<NonZero<usize>>,
    ) -> Self {
        Self {
            allocator,
            actor,
            receiver,
            deadline_in_ms,
            message_handlers: Vec::new(),
            marker: PhantomData,
        }
    }

    pub async fn run(mut self) {
        self.handle(|actor, event, stack| todo!()).await;
    }

    async fn handle<F>(&mut self, execute_function: F)
    where
        F: FnOnce(&mut Box<AI>, &mut Option<ActorUserExecutorEvent>, u64),
    {
        let mut event: Option<ActorUserExecutorEvent> = None;

        let user_stack: u64 = 0u64; // TODO

        execute_function(&mut self.actor, &mut event, user_stack);

        loop {
            match event.take() {
                None => break,
                Some(ActorUserExecutorEvent::SystemCall(sc)) => {}
                Some(ActorUserExecutorEvent::DeadlinePreemption(d)) => {}
                Some(ActorUserExecutorEvent::Exception) => {
                    todo!()
                }
            }
        }
    }
}
