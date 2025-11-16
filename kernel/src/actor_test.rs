use crate::actor::env::root::environment::RootEnvironment;
use crate::kprintln;
use core::fmt::Write;
use zcene_core::actor::{
    Actor, ActorCreateError, ActorDestroyError, ActorEnvironment, ActorFuture,
};

#[derive(Default)]
pub struct RootActor;

#[derive(Clone)]
pub enum RootActorMessage {}

impl Actor<RootEnvironment> for RootActor {
    type Message = RootActorMessage;

    async fn create<'a>(
        &'a mut self,
        context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {
        kprintln!("Hello World!");

        Ok(())
    }

    async fn destroy<'a>(
        self,
        _context: <RootEnvironment as ActorEnvironment>::DestroyContext<'a>,
    ) -> Result<(), ActorDestroyError> {
        kprintln!("Destroyed!");
        Ok(())
    }
}
