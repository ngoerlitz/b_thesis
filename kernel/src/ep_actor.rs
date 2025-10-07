use crate::UartSink;
use crate::actor::root::actor_root_environment::{ActorRootEnvironment, ActorSpawnSpecification};
use alloc::collections::btree_map::Entry;
use alloc::string::String;
use core::arch::asm;
use core::fmt::Write;
use zcene_core::actor::{
    Actor, ActorAddressReference, ActorContextMessageProvider, ActorCreateError, ActorEnvironment,
    ActorFuture, ActorHandleError, ActorMessageChannelAddress, ActorMessageSender,
};
use zcene_core::future::runtime::FutureRuntimeHandler;
use zcene_core::future::r#yield;

pub struct EntryPointActor<H>
where
    H: FutureRuntimeHandler,
{
    pub channel: ActorMessageChannelAddress<PrintActor, ActorRootEnvironment<H>>,
}

impl<H> Actor<ActorRootEnvironment<H>> for EntryPointActor<H>
where
    H: FutureRuntimeHandler,
{
    type Message = ();

    async fn create(
        &mut self,
        context: <ActorRootEnvironment<H> as ActorEnvironment>::CreateContext,
    ) -> Result<(), ActorCreateError> {
        let _ = writeln!(UartSink, "Spawned EntryPointActor").unwrap();

        loop {
            self.channel.send("Hello!").await;

            for _ in 0..400_000 {
                r#yield().await;
            }
        }

        Ok(())
    }
}

pub struct PrintActor;

impl<H> Actor<ActorRootEnvironment<H>> for PrintActor
where
    H: FutureRuntimeHandler,
{
    type Message = &'static str;

    async fn create(
        &mut self,
        context: <ActorRootEnvironment<H> as ActorEnvironment>::CreateContext,
    ) -> Result<(), ActorCreateError> {
        let _ = writeln!(UartSink, "Spawned PrintActor").unwrap();

        Ok(())
    }

    async fn handle(
        &mut self,
        context: <ActorRootEnvironment<H> as ActorEnvironment>::HandleContext<Self::Message>,
    ) -> Result<(), ActorHandleError> {
        let _ = writeln!(
            UartSink,
            "[CPU: {}] [PrintActor]: {:?}",
            crate::platform::aarch64::cpu::cpuid(),
            context.message()
        );

        Ok(())
    }
}
