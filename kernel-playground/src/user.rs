use alloc::string::String;
use core::arch::asm;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError, ActorMessageSender};
use kernel::uprintln;

pub struct UserActor {
    id: usize,
}

impl Default for UserActor {
    fn default() -> Self {
        Self {
            id: 25,
        }
    }
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
        uprintln!("[{}] -- {}", &self.id, MESSAGE);

        Ok(())
    }

    #[unsafe(link_section = ".user_text")]
    fn handle<'a>(
        &mut self,
        context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>,
    ) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        let id = self.id.clone();

        async move {
            uprintln!("[{}] -- {}", id, MESSAGE);
            Ok(())
        }
    }
}
