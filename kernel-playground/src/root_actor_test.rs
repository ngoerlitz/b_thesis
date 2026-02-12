use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError};
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::kprintln;

#[derive(Default)]
pub struct RootActorTest {}

impl Actor<RootEnvironment> for RootActorTest {
    type Message = u64;

    async fn create<'a>(&'a mut self, context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>
    ) -> Result<(), ActorCreateError> {

        Ok(())
    }

    fn handle<'a>(&mut self, context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        kprintln!("[ROOT] Handle message --> {}", context.message);

        async move {
            Ok(())
        }
    }
}