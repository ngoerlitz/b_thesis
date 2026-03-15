use alloc::string::String;
use zcene_core::actor::{Actor, ActorEnvironment, ActorFuture, ActorHandleError, ActorMessageSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::address::UserViewAddress;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::{kprintln, uprintln};
use crate::receiver::ReceivingActor;

pub struct PingActor
{
    target: UserViewAddress<ReceivingActor>,
}

impl PingActor
{
    pub fn new(target: UserViewAddress<ReceivingActor>) -> Self {
        Self { target }
    }
}

impl Actor<UserEnvironment> for PingActor
{
    type Message = u64;

    fn handle<'a>(&mut self, context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        let target = self.target.clone();

        uprintln!("Received : {}", context.message);

        async move {
            target.send(123).await;

            Ok(())
        }
    }
}