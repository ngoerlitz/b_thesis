use crate::actor::env::root::environment::RootEnvironment;
use kernel_derive::Constructor;
use zcene_core::actor::{ActorEnvironmentAllocator, ActorEnvironmentReference, ActorMessage};
use zcene_core::future::runtime::FutureRuntimeHandler;
use crate::actor::runtime::handler::RuntimeHandler;

#[derive(Constructor)]
pub struct RootEnvironmentCreateCtx<'a, H: FutureRuntimeHandler>
{
    pub environment: &'a RootEnvironment<H>,
    pub eref: &'a ActorEnvironmentReference<RootEnvironment<H>>,
}

#[derive(Constructor)]
pub struct RootEnvironmentHandleCtx<'a, H: FutureRuntimeHandler, M: ActorMessage> {
    pub environment: &'a RootEnvironment<H>,
    pub message: &'a M,
    pub page: Option<(usize, usize)>,
    pub forwarded: &'a mut bool,
}

#[derive(Constructor)]
pub struct RootEnvironmentDestroyCtx<'a, H: FutureRuntimeHandler> {
    pub environment: &'a RootEnvironment<H>,
}
