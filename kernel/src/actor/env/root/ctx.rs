use crate::actor::env::root::environment::RootEnvironment;
use kernel_derive::Constructor;
use zcene_core::actor::ActorMessage;
use zcene_core::future::runtime::FutureRuntimeHandler;

#[derive(Constructor)]
pub struct RootEnvironmentCreateCtx<'a, H: FutureRuntimeHandler> {
    pub environment: &'a RootEnvironment<H>,
}

#[derive(Constructor)]
pub struct RootEnvironmentHandleCtx<'a, H: FutureRuntimeHandler, M: ActorMessage> {
    pub environment: &'a RootEnvironment<H>,
    pub message: M,
}

#[derive(Constructor)]
pub struct RootEnvironmentDestroyCtx<'a, H: FutureRuntimeHandler> {
    pub environment: &'a RootEnvironment<H>,
}
