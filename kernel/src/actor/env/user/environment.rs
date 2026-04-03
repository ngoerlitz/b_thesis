use alloc::alloc::Global;
use alloc::boxed::Box;
use crate::actor::env::user::address::UserViewAddress;
use crate::actor::env::user::ctx::UserEnvironmentHandleCtx;
use core::fmt::Debug;
use core::marker::PhantomData;
use zcene_core::actor::{Actor, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentReference, ActorEnvironmentSpawn, ActorMessage, ActorMessageChannel, ActorSpawnError};
use zcene_core::future::runtime::FutureRuntimeHandler;
use crate::actor::env::root::environment::RootEnvironment;
use crate::actor::env::user::executor::UserExecutor;

pub struct UserEnvironment;

impl ActorEnvironment for UserEnvironment {
    type Address<A: Actor<Self>> = UserViewAddress<A>;
    type CreateContext<'a> = ();
    type HandleContext<'a, M: ActorMessage> = UserEnvironmentHandleCtx<M>;
    type DestroyContext<'a> = ();
}

impl ActorEnvironmentAllocator for UserEnvironment {
    type Allocator = Global;

    fn allocator(&self) -> &<Self as ActorEnvironmentAllocator>::Allocator {
        &Global
    }
}

// impl<A: Actor<Self>, H: FutureRuntimeHandler> ActorEnvironmentSpawn<A> for UserEnvironment {
//     fn spawn(self: &ActorEnvironmentReference<Self>, actor: A) -> Result<Self::Address<A>, ActorSpawnError> {
//         // TODO
//
//         Err(ActorSpawnError::Unknown)
//     }
// }