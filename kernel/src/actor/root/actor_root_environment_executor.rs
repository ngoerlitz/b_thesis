use crate::UartSink;
use crate::actor::root::actor_root_environment::{
    ActorRootEnvironment, ActorRootEnvironmentCreateContext, ActorRootEnvironmentHandleContext,
};
use core::fmt::Write;
use core::marker::PhantomData;
use core::ptr::write;
use zcene_core::actor::{
    Actor, ActorCommonHandleContext, ActorMessageChannelReceiver, ActorSystemReference,
};
use zcene_core::future::runtime::FutureRuntimeHandler;

pub struct ActorRootEnvironmentExecutor<A, H>
where
    A: Actor<ActorRootEnvironment<H>>,
    H: FutureRuntimeHandler,
{
    system: ActorSystemReference<ActorRootEnvironment<H>>,
    actor: A,
    receiver: ActorMessageChannelReceiver<A::Message>,
    marker: PhantomData<H>,
}

impl<A, H> ActorRootEnvironmentExecutor<A, H>
where
    A: Actor<ActorRootEnvironment<H>>,
    H: FutureRuntimeHandler,
{
    pub(crate) fn new(
        system: ActorSystemReference<ActorRootEnvironment<H>>,
        actor: A,
        receiver: ActorMessageChannelReceiver<A::Message>,
    ) -> Self {
        Self {
            system,
            actor,
            receiver,
            marker: PhantomData,
        }
    }

    pub async fn run(mut self) {
        let _result = self
            .actor
            .create(ActorRootEnvironmentCreateContext::new(self.system.clone()))
            .await;

        while let Some(message) = self.receiver.receive().await {
            let _ = self
                .actor
                .handle(ActorCommonHandleContext::new(message))
                .await;
        }

        let _result = self.actor.destroy(()).await;
    }
}
