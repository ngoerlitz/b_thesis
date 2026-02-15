use alloc::string::String;
use core::arch::asm;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError, ActorMessageSender};
use kernel::{kprintln, uprintln};
use kernel::actor::channel::OUTBOX_VA_ADDR;

pub struct UserActor {
    id: usize,
}

impl UserActor {
    pub fn new(id: usize) -> UserActor {
        Self {
            id
        }
    }
}

impl Actor<UserEnvironment> for UserActor {
    type Message = u64;

    #[unsafe(link_section = ".user_text")]
    fn handle<'a>(
        &mut self,
        context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>,
    ) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {

        uprintln!("[I RECEIVED THE MESSAGE -- {}] -- \"{:?}\"", self.id, context.message);

        async move {

            Ok(())
        }
    }
}
