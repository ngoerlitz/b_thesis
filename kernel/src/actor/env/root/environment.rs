use crate::actor::env::root::ctx::{
    RootEnvironmentCreateCtx, RootEnvironmentDestroyCtx, RootEnvironmentHandleCtx,
};
use crate::actor::env::root::service::actor_root_logger_service::ActorRootLoggerService;
use crate::actor::runtime::handler::RuntimeHandler;
use crate::boot::global;
use crate::drivers::pl011::PL011;
use crate::getter;
use crate::hal::serial::SerialDriver;
use alloc::alloc::Global;
use alloc::sync::Arc;
use zcene_core::actor::{
    Actor, ActorEnterError, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentReference,
    ActorEnvironmentSpawn, ActorMessage, ActorMessageChannel, ActorMessageChannelAddress,
    ActorSpawnError,
};
use zcene_core::future::runtime::{FutureRuntimeHandler, FutureRuntimeReference};

pub struct RootEnvironment<H = RuntimeHandler, L = PL011>
where
    H: FutureRuntimeHandler,
    L: SerialDriver,
{
    future_runtime: FutureRuntimeReference<H>,
    logger: ActorRootLoggerService<L>,
}

impl<H: FutureRuntimeHandler, L: SerialDriver> RootEnvironment<H, L> {
    pub fn new(
        future_runtime: FutureRuntimeReference<H>,
        logger: ActorRootLoggerService<L>,
    ) -> Self {
        Self {
            future_runtime,
            logger,
        }
    }

    getter!(future_runtime: FutureRuntimeReference<H> as runtime);
    getter!(logger: ActorRootLoggerService<L>);

    pub fn enter(&self) -> Result<(), ActorEnterError> {
        self.future_runtime.run();

        Ok(())
    }
}

impl<H: FutureRuntimeHandler> ActorEnvironment for RootEnvironment<H> {
    type Address<A: Actor<Self>> = ActorMessageChannelAddress<A, Self>;
    type CreateContext<'a> = RootEnvironmentCreateCtx<'a, H>;
    type HandleContext<'a, M: ActorMessage> = RootEnvironmentHandleCtx<'a, H, M>;
    type DestroyContext<'a> = RootEnvironmentDestroyCtx<'a, H>;
}

impl<H: FutureRuntimeHandler> ActorEnvironmentAllocator for RootEnvironment<H> {
    type Allocator = H::Allocator;

    fn allocator(&self) -> &Self::Allocator {
        self.future_runtime.handler().allocator()
    }
}

impl<A: Actor<Self>, H: FutureRuntimeHandler> ActorEnvironmentSpawn<A> for RootEnvironment<H> {
    fn spawn(
        self: &ActorEnvironmentReference<Self>,
        mut actor: A,
    ) -> Result<<Self as ActorEnvironment>::Address<A>, ActorSpawnError> {
        let (sender, receiver) = ActorMessageChannel::<A::Message>::new_unbounded();

        self.future_runtime.spawn({
            let environment = self.clone();

            async move {
                actor.create(Self::CreateContext::new(&*environment)).await;

                while let Some(message) = receiver.receive().await {
                    actor
                        .handle(Self::HandleContext::new(&*environment, message))
                        .await;
                }

                actor
                    .destroy(Self::DestroyContext::new(&*environment))
                    .await;
            }
        })?;

        Ok(<RootEnvironment<H> as ActorEnvironment>::Address::<A>::new(
            sender,
        ))
    }
}

impl RootEnvironment<RuntimeHandler, PL011> {
    pub fn get() -> &'static ActorEnvironmentReference<Self> {
        unsafe {
            // SAFETY
            // The ::get() static method should only be called once the static `ACTOR_ROOT_ENVIRONMENT`
            // has been initialized on the primary core - this happens as one of the first operations
            // when the core boots. Calling this on any of the secondary cores is therefore inherently
            // safe, since these are brought up after the primary core has done its initialization.

            global::ACTOR_ROOT_ENVIRONMENT
                .get()
                .as_ref()
                .unwrap()
                .assume_init_ref()
        }
    }
}
