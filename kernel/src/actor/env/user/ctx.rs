use crate::actor::env::root::environment::RootEnvironment;
use kernel_derive::Constructor;
use zcene_core::actor::ActorMessage;
use zcene_core::future::runtime::FutureRuntimeHandler;

#[derive(Constructor)]
pub struct UserEnvironmentHandleCtx<M: ActorMessage> {
    pub message: M,
}
