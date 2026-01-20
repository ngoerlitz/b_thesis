use alloc::string::String;
use core::arch::asm;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError, ActorMessageSender};
use kernel::{kprintln, uprintln};

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
        let mut counter = 0;

        async move {
            loop {
                let mut sp: u64;
                unsafe {
                    asm!("mov {}, sp", out(reg) sp);
                }
                uprintln!("USER_SP: {sp:X}");

                uprintln!("[I RECEIVED THE MESSAGE [{}]] -- {}", &counter, context.message);

                for _ in 0..1_000_000 {
                    unsafe {
                        asm!("nop")
                    }
                }

                counter += 1;
                if counter == 100 {
                    break;
                }
            }

            Ok(())
        }
    }
}
