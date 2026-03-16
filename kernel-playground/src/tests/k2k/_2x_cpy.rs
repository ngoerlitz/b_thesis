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
use kernel::kprintln;
use kernel_derive::Constructor;
use crate::tests::get_time;

const MESSAGE_SIZE: usize = 50_000;
type TMessage = [u8; MESSAGE_SIZE];

#[inline(always)]
pub fn register_tests() {
    let root_env = RootEnvironment::get();

    let receiver = root_env.spawn(
        ReceivingActor::default()
    ).unwrap();

    root_env.spawn(
        SendingActor::new(receiver)
    ).unwrap();
}

#[derive(Default)]
struct ReceivingActor
{
}

impl Actor<RootEnvironment> for ReceivingActor
{
    type Message = TMessage;

    fn handle<'a>(&mut self, context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        let now = get_time();

        async move {
            kprintln!("[CPY] <- {}", now.0);

            Ok(())
        }
    }
}

#[derive(Constructor)]
struct SendingActor<A, E, H>
where
    A: Actor<E, Message = TMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    target: PtActorMessageChannelAddress<A, E, H>
}

impl<A, E, H> Actor<RootEnvironment> for SendingActor<A, E, H>
where
    A: Actor<E, Message = TMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    type Message = ();

    fn create<'a>(&'a mut self, context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
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

            target.send_msg(PtMessage::Copy(mem)).await;

            kprintln!("[CPY] -> {}", now.0);

            Ok(())
        }
    }
}