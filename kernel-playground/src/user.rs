use alloc::string::String;
use core::arch::asm;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError, ActorMessageSender};
use kernel::{kprintln, uprintln};

#[derive(Default)]
pub struct UserActor {
}

pub type UserActorMessage = u64;

impl Actor<UserEnvironment> for UserActor {
    type Message = UserActorMessage;

    #[unsafe(link_section = ".user_text")]
    fn create<'a>(
        &'a mut self,
        context: <UserEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        async move {
            Ok(())
        }
    }

    #[unsafe(link_section = ".user_text")]
    fn handle<'a>(
        &mut self,
        context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>,
    ) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {

        async move {
            uprintln!("[I RECEIVED THE MESSAGE] -- \"{:?}\"", context.message);

            Ok(())
        }
    }
}
