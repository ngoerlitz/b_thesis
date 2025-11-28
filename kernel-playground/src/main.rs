#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(arbitrary_self_types)]
#![no_std]
#![no_main]

use zcene_core::actor::{Actor, ActorEnvironment, ActorCreateError, ActorDestroyError};
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::{bootstrap_system, kprintln};

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
        kprintln!("This is a test of the new actor system. Please tell me this thing works!");

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

bootstrap_system!(RootActor::default());