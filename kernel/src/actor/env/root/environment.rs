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
use crate::{getter, kprintln, linker_symbols, log_dbg};
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
use spin::{Mutex, RwLock};
use zcene_core::actor::{Actor, ActorEnterError, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentReference, ActorEnvironmentSpawn, ActorMessage, ActorMessageChannel, ActorMessageChannelAddress, ActorMessageChannelSender, ActorSpawnError};
use zcene_core::future::runtime::{FutureRuntimeHandler, FutureRuntimeReference};
use crate::actor::channel::INBOX_VA_ADDR;
use crate::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use crate::actor::channel::pt_message::PtMessage::{Copy, Page};
use crate::actor::channel::pt_message_channel::PtActorMessageChannel;
use crate::actor::env::root::service::message_frame_allocator_service::MessageFrameAllocatorService;
use crate::actor::env::user::message_handler::UserMessageHandler;
use crate::drivers::mmu;
use crate::memory::bitmap_stack_allocator::StackAllocator;

linker_symbols! {
    STACK_EL0_TOP = __stack_el0_top;
    USTACK_SIZE = __ustack_size;
}
const USER_STACK_SIZE: usize = 4096;

pub struct RootEnvironment<H = RuntimeHandler>
where
    H: FutureRuntimeHandler,
{
    future_runtime: FutureRuntimeReference<H>,
    logger: ActorRootLoggerService<PL011>,
    irq_manager: RwLock<IrqManagerService<GIC400, 216>>,
    user_stack_manager: Mutex<StackAllocator>,
    message_frame_allocator: Mutex<MessageFrameAllocatorService>,
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
            user_stack_manager: Mutex::new(StackAllocator::new(STACK_EL0_TOP(), USTACK_SIZE())),
            message_frame_allocator: Mutex::new(MessageFrameAllocatorService::new(0x4000_0000))
        }
    }

    getter!(future_runtime: FutureRuntimeReference<H> as runtime);
    getter!(logger: ActorRootLoggerService<PL011>);
    getter!(irq_manager: RwLock<IrqManagerService<GIC400, 216>>);
    getter!(user_stack_manager: Mutex<StackAllocator>);
    getter!(message_frame_allocator: Mutex<MessageFrameAllocatorService>);

    pub fn enter(&self) -> Result<(), ActorEnterError> {
        log_dbg!("Entered future runtime...");

        self.future_runtime.run();

        Ok(())
    }

    pub fn spawn_user<A: Actor<UserEnvironment>>(
        self: &ActorEnvironmentReference<Self>,
        mut actor: A,
        message_handlers: Vec<
            Box<
                dyn UserMessageHandler<UserEnvironment>,
                <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
            >,
            <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
        >,
    ) -> Result<PtActorMessageChannelAddress<A, UserEnvironment, H>, ActorSpawnError>
    {
        let allocator = self.allocator().clone();
        let (sender, receiver) = PtActorMessageChannel::<A::Message, H>::new_bounded(allocator.clone(), 50);

        let executor_allocator = allocator.clone();
        self.future_runtime.spawn(async move {
            UserExecutor::<A, H>::new(
                executor_allocator,
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
            PtActorMessageChannelAddress::new(
                sender,
                allocator,
                PhantomData
            )
        )
    }
}

impl<H: FutureRuntimeHandler> ActorEnvironment for RootEnvironment<H> {
    type Address<A: Actor<Self>> = PtActorMessageChannelAddress<A, Self, H>;
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
        let alloc = self.allocator().clone();
        let (sender, receiver) = PtActorMessageChannel::<A::Message, H>::new_bounded(alloc.clone(), 50);

        self.future_runtime.spawn({
            let environment = self.clone();

            async move {
                actor.create(Self::CreateContext::new(&*environment.clone(), &environment)).await;

                while let Some(message) = receiver.receive().await {
                    // log_dbg!("Received message: {:?}", &message);

                    let mut forwarded: bool = false;

                    match message {
                        Copy(msg_box) => {
                            actor.handle(Self::HandleContext::new(&*environment, &*msg_box, None, &mut forwarded)).await;
                        },
                        Page(id, pa) => {
                            mmu::map_va_pa(INBOX_VA_ADDR, pa as u64);

                            // log_dbg!("Reading message...");

                            let msg = unsafe {
                                &*(INBOX_VA_ADDR as *const A::Message)
                            };

                            actor.handle(Self::HandleContext::new(&*environment, msg, Some((id, pa)), &mut forwarded)).await;

                            // TODO: This shouldn't free the fame, if we decided to forward it!
                            if !forwarded {
                                environment.message_frame_allocator().lock().free_frame(id);
                            }
                        }
                    }
                }

                actor
                    .destroy(Self::DestroyContext::new(&*environment))
                    .await;
            }
        })?;

        Ok(<RootEnvironment<H> as ActorEnvironment>::Address::<A>::new(
            sender,
            alloc,
            PhantomData
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
