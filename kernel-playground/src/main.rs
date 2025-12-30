#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(arbitrary_self_types)]
#![no_std]
#![no_main]

mod receiver;
mod user;

extern crate alloc;

use crate::receiver::ReceivingActor;
use crate::user::UserActor;
use alloc::format;
use alloc::string::String;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::boot::global::ACTOR_ROOT_ENVIRONMENT;
use kernel::{bootstrap_system, kprintln};
use zcene_core::actor::{
    Actor, ActorCreateError, ActorDestroyError, ActorEnvironment, ActorEnvironmentSpawn,
    ActorFuture, ActorHandleError, ActorMessageSender,
};

#[derive(Default)]
pub struct RootActor {
    id: usize,
}

impl RootActor {
    fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Clone, Debug)]
pub enum RootActorMessage {
    String(String),
}

impl Actor<RootEnvironment> for RootActor {
    type Message = RootActorMessage;

    async fn create<'a>(
        &'a mut self,
        context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {
        let new_actor = ReceivingActor::default();

        let user_addr = unsafe {
            RootEnvironment::get()
                .spawn_user(UserActor::default())
                .unwrap()
        };

        user_addr.send(3);

        let addr = unsafe { RootEnvironment::get().spawn(new_actor).unwrap() };

        addr.send(format!("Hello World, this is a message from {}!", self.id).into())
            .await;

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
