use crate::UartSink;
use crate::actor::root::actor_root_environment::{ActorRootEnvironment, ActorSpawnSpecification};
use crate::ep_actor::{EntryPointActor, PrintActor};
use core::fmt::Write;
use zcene_core::actor::{
    Actor, ActorCreateError, ActorEnvironment, ActorHandleError, ActorMessageSender,
};
use zcene_core::future::runtime::FutureRuntimeHandler;

pub struct RootActor;

impl<H> Actor<ActorRootEnvironment<H>> for RootActor
where
    H: FutureRuntimeHandler,
{
    type Message = ();

    async fn create(
        &mut self,
        context: <ActorRootEnvironment<H> as ActorEnvironment>::CreateContext,
    ) -> Result<(), ActorCreateError> {
        let _ = writeln!(UartSink, "Spawned RootActor").unwrap();

        let print_addr = context
            .system
            .clone()
            .spawn(ActorSpawnSpecification::new(PrintActor))
            .unwrap();

        let addr = context
            .system
            .clone()
            .spawn(ActorSpawnSpecification::new(EntryPointActor {
                channel: print_addr,
            }))
            .unwrap();

        Ok(())
    }
}
