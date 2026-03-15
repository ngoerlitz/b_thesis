use alloc::string::String;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::{kprintln, uprintln};
use zcene_core::actor::{Actor, ActorDestroyError, ActorEnvironment, ActorFuture, ActorHandleError};
use kernel::actor::env::user::environment::UserEnvironment;
use crate::tests::get_time;

#[derive(Default)]
pub struct ReceivingActor;

pub type ReceivingActorMessage = u64;

impl Actor<UserEnvironment> for ReceivingActor {
    type Message = ReceivingActorMessage;

    fn handle<'a>(
        &mut self,
        context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>,
    ) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        async move {
            let now = get_time();

            uprintln!("Received last: {} @ {} ({})", context.message, now.0, now.1);

            Ok(())
        }
    }
}
