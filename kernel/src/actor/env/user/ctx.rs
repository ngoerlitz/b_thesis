use core::marker::PhantomData;
use crate::actor::env::root::environment::RootEnvironment;
use kernel_derive::Constructor;
use zcene_core::actor::ActorMessage;
use zcene_core::future::runtime::FutureRuntimeHandler;

#[derive(Constructor)]
pub struct UserEnvironmentHandleCtx<M: ActorMessage> {
    message_ptr: u64,
    phantom: PhantomData<M>,
}

impl<A: ActorMessage> UserEnvironmentHandleCtx<A> {
    pub fn get_message(&self) -> &A {
        unsafe {
            &*(self.message_ptr as *const A)
        }
    }
}