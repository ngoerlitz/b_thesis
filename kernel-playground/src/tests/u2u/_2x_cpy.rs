use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::vec;
use core::marker::PhantomData;
use core::ptr;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentSpawn, ActorFuture, ActorHandleError, ActorMessageSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_message::PtMessage;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::address::UserViewAddress;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::{kprintln, uprintln};
use kernel_derive::Constructor;
use crate::tests::get_time;

const MESSAGE_SIZE: usize = 500;
type TMessage = [u8; MESSAGE_SIZE];

#[inline(always)]
pub fn register_tests() {
    let root_env = RootEnvironment::get();

    let receiver = root_env.spawn_user(
        ReceivingActor::default(),
        vec![]
    ).unwrap();

    root_env.spawn_user(
        SendingActor::new(UserViewAddress::new(0, PhantomData)),
        vec![Box::new(receiver)]
    ).unwrap();
}

#[derive(Default)]
struct ReceivingActor
{
}

impl Actor<UserEnvironment> for ReceivingActor
{
    type Message = TMessage;

    fn handle<'a>(&mut self, context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        let now = get_time();

        async move {
            uprintln!("[CPY] <- {}", now.0);

            Ok(())
        }
    }
}

#[derive(Constructor)]
struct SendingActor
{
    target: UserViewAddress<ReceivingActor>
}

impl Actor<UserEnvironment> for SendingActor
{
    type Message = ();

    fn create<'a>(&'a mut self, context: <UserEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        let target = self.target.clone();

        async move {
            let mut msg: [u8; MESSAGE_SIZE] = [0; MESSAGE_SIZE];

            for i in 0..MESSAGE_SIZE {
                msg[i] = i as u8;
            }

            let mut x = Box::<TMessage>::new_uninit();

            let now = get_time();

            unsafe {
                ptr::copy_nonoverlapping(msg.as_ptr(), x.as_mut_ptr() as *mut u8, MESSAGE_SIZE);
            }

            let mem = unsafe {x.assume_init()};

            target.send(*mem).await;

            Box::leak(mem);

            uprintln!("[CPY] -> {}", now.0);

            Ok(())
        }
    }
}