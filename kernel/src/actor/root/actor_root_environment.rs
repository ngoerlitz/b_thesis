use crate::actor::root::actor_root_environment_executor::ActorRootEnvironmentExecutor;
use zcene_core::actor::{
    Actor, ActorCommonHandleContext, ActorEnterError, ActorEnvironment, ActorEnvironmentAllocator,
    ActorEnvironmentEnterable, ActorEnvironmentSpawnable, ActorMessage, ActorMessageChannel,
    ActorMessageChannelAddress, ActorMessageChannelSender, ActorSpawnError, ActorSystem,
    ActorSystemReference,
};
use zcene_core::future::runtime::{FutureRuntimeHandler, FutureRuntimeReference};

pub struct ActorRootEnvironmentCreateContext<H>
where
    H: FutureRuntimeHandler,
{
    pub system: ActorSystemReference<ActorRootEnvironment<H>>,
}

impl<H> ActorRootEnvironmentCreateContext<H>
where
    H: FutureRuntimeHandler,
{
    pub(crate) fn new(system: ActorSystemReference<ActorRootEnvironment<H>>) -> Self {
        Self { system }
    }
}

pub struct ActorRootEnvironmentHandleContext<H, M>
where
    H: FutureRuntimeHandler,
{
    system: ActorSystemReference<ActorRootEnvironment<H>>,
    message: M,
}

pub struct ActorRootEnvironmentDestroyContext<H>
where
    H: FutureRuntimeHandler,
{
    system: ActorSystemReference<ActorRootEnvironment<H>>,
}

pub struct ActorRootEnvironment<H>
where
    H: FutureRuntimeHandler,
{
    runtime: FutureRuntimeReference<H>,
}

impl<H> ActorRootEnvironment<H>
where
    H: FutureRuntimeHandler,
{
    pub(crate) fn new(runtime: FutureRuntimeReference<H>) -> Self {
        Self { runtime }
    }
}

impl<H> ActorEnvironment for ActorRootEnvironment<H>
where
    H: FutureRuntimeHandler,
{
    type Address<A>
        = ActorMessageChannelAddress<A, Self>
    where
        A: Actor<Self>;

    type CreateContext = ActorRootEnvironmentCreateContext<H>;
    type HandleContext<M>
        = ActorCommonHandleContext<M>
    where
        M: ActorMessage;
    type DestroyContext = ();
}

impl<H> ActorEnvironmentAllocator for ActorRootEnvironment<H>
where
    H: FutureRuntimeHandler,
{
    type Allocator = <H as FutureRuntimeHandler>::Allocator;

    fn allocator(&self) -> &Self::Allocator {
        self.runtime.handler().allocator()
    }
}

impl<H> ActorEnvironmentEnterable<ActorRootEnvironment<H>> for ()
where
    H: FutureRuntimeHandler,
{
    fn enter(
        self,
        system: &ActorSystemReference<ActorRootEnvironment<H>>,
    ) -> Result<(), ActorEnterError> {
        system.environment().runtime.run();

        Ok(())
    }
}

pub struct ActorSpawnSpecification<A> {
    actor: A,
}

impl<A> ActorSpawnSpecification<A> {
    pub fn new(actor: A) -> Self {
        Self { actor }
    }
}

impl<A, H> ActorEnvironmentSpawnable<A, ActorRootEnvironment<H>> for ActorSpawnSpecification<A>
where
    A: Actor<ActorRootEnvironment<H>>,
    H: FutureRuntimeHandler,
{
    fn spawn(
        self,
        system: &ActorSystemReference<ActorRootEnvironment<H>>,
    ) -> Result<Self::Address, ActorSpawnError> {
        let (sender, receiver) = ActorMessageChannel::<A::Message>::new_unbounded();

        let cloned = system.clone();

        system
            .environment()
            .runtime
            .spawn(ActorRootEnvironmentExecutor::new(cloned.into(), self.actor, receiver).run())?;

        Ok(<ActorRootEnvironment<H> as ActorEnvironment>::Address::new(
            sender,
        ))
    }
}
