use crate::actor::env::root::ctx::{
    RootEnvironmentCreateCtx, RootEnvironmentDestroyCtx, RootEnvironmentHandleCtx,
};
use crate::actor::env::root::service::actor_root_logger_service::ActorRootLoggerService;
use crate::actor::env::user::address::UserViewAddress;
use crate::actor::env::user::environment::UserEnvironment;
use crate::actor::env::user::executor::UserExecutor;
use crate::actor::runtime::handler::RuntimeHandler;
use crate::boot::global;
use crate::drivers::gic400::GIC400;
use crate::drivers::pl011::PL011;
use crate::getter;
use crate::hal::serial::SerialDriver;
use crate::services::irq_manager::IrqManagerService;
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::fmt::Debug;
use core::iter::Map;
use core::marker::PhantomData;
use core::num::{NonZero, NonZeroU64, NonZeroUsize};
use spin::RwLock;
use zcene_core::actor::{Actor, ActorEnterError, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentReference, ActorEnvironmentSpawn, ActorMessage, ActorMessageChannel, ActorMessageChannelAddress, ActorMessageChannelSender, ActorSpawnError};
use zcene_core::future::runtime::{FutureRuntimeHandler, FutureRuntimeReference};
use crate::actor::env::user::message_handler::UserMessageHandler;

pub struct RootEnvironment<H = RuntimeHandler>
where
    H: FutureRuntimeHandler,
{
    future_runtime: FutureRuntimeReference<H>,
    logger: ActorRootLoggerService<PL011>,
    irq_manager: RwLock<IrqManagerService<GIC400, 216>>,
}

impl<H: FutureRuntimeHandler> RootEnvironment<H> {
    pub fn new(
        future_runtime: FutureRuntimeReference<H>,
        logger: ActorRootLoggerService<PL011>,
    ) -> Self {
        Self {
            future_runtime,
            logger,
            irq_manager: RwLock::new(IrqManagerService::new(GIC400::new())),
        }
    }

    getter!(future_runtime: FutureRuntimeReference<H> as runtime);
    getter!(logger: ActorRootLoggerService<PL011>);
    getter!(irq_manager: RwLock<IrqManagerService<GIC400, 216>>);

    pub fn enter(&self) -> Result<(), ActorEnterError> {
        self.future_runtime.run();

        Ok(())
    }

    pub fn spawn_user<A: Actor<UserEnvironment>>(
        self: &ActorEnvironmentReference<Self>,
        offset: usize,
        mut actor: A,
        message_handlers: Vec<
            Box<
                dyn UserMessageHandler<UserEnvironment>,
                <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
            >,
            <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
        >,
    ) -> Result<ActorMessageChannelAddress<A, UserEnvironment>, ActorSpawnError>
    {
        let (sender, receiver) = ActorMessageChannel::<A::Message>::new_unbounded();
        let allocator = self.allocator().clone();

        self.future_runtime.spawn(async move {
            UserExecutor::<_, H>::new(
                offset,
                allocator,
                Box::new(actor),
                receiver,
                NonZeroU64::new(100),
                message_handlers,
                PhantomData,
            )
            .run()
            .await
        });

        Ok(
            ActorMessageChannelAddress::new(
                sender,
            )
        )
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

impl RootEnvironment<RuntimeHandler> {
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
