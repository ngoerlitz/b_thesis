use alloc::string::String;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::kprintln;
use zcene_core::actor::{Actor, ActorEnvironment, ActorFuture, ActorHandleError};

#[derive(Default)]
pub struct ReceivingActor;

pub type ReceivingActorMessage = u64;

impl Actor<RootEnvironment> for ReceivingActor {
    type Message = ReceivingActorMessage;

    fn handle<'a>(
        &mut self,
        context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>,
    ) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        async move {
            kprintln!("ReceivingActor received message: \"{:?}\"", context.message);

            Ok(())
        }
    }
}
