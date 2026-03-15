use alloc::string::String;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorHandleError, ActorMessageSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::address::UserViewAddress;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::{kprintln, uprintln};
use kernel::platform::aarch64::get_cpu_timer;
use crate::pingActor::PingActor;
use kernel::hal::timer::SystemTimerDriver;
use crate::tests::get_time;

pub struct StartActor
{
    target: UserViewAddress<PingActor>,
}

impl StartActor
{
    pub fn new(target: UserViewAddress<PingActor>) -> Self {
        Self { target }
    }
}

impl Actor<UserEnvironment> for StartActor
{
    type Message = u64;

    fn create<'a>(&'a mut self, _context: <UserEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        let target = self.target.clone();
        async move {
            let now = get_time();

            for i in 0..1 {
                target.send(1).await;
            }

            uprintln!("-> {}", now.0);

            Ok(())
        }
    }
}