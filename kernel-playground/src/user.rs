use alloc::string::String;
use core::arch::asm;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError, ActorMessageSender};
use kernel::uprintln;

#[derive(Default)]
pub struct UserActor {
}

pub type UserActorMessage = &'static str;

#[unsafe(link_section = ".user_text")]
static MESSAGE: &'static str = "User Text Message";

impl Actor<UserEnvironment> for UserActor {
    #[unsafe(link_section = ".user_text")]
    type Message = UserActorMessage;

    #[unsafe(link_section = ".user_text")]
    async fn create<'a>(
        &'a mut self,
        context: <UserEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {
        Ok(())
    }

    #[unsafe(link_section = ".user_text")]
    fn handle<'a>(
        &mut self,
        context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>,
    ) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        async move {
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            uprintln!("[I RECEIVED THE MESSAGE] -- {}", context.message);
            Ok(())
        }
    }
}
