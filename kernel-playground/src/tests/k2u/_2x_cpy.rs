use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::vec;
use core::ptr;
use zcene_core::actor::{Actor, ActorMessage, ActorCreateError, ActorDestroyError, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentSpawn, ActorFuture, ActorHandleError, ActorMessageChannelSender, ActorMessageSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_message::PtMessage;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::{kprintln, uprintln};
use kernel_derive::Constructor;
use crate::tests::get_time;

type TMessage<const N: usize> = [u8; N];

#[inline(always)]
pub fn register_tests<const N: usize>() {
    let root_env = RootEnvironment::get();

    let receiver = root_env.spawn_user(
        UserReceiver::<N>::default(),
        vec![]
    ).unwrap();

    root_env.spawn(
        KernelSender::<_, _, _, N>::new(receiver)
    ).unwrap();
}

#[derive(Default)]
struct UserReceiver<const N: usize> {}

impl<const N: usize> Actor<UserEnvironment> for UserReceiver<N> {
    type Message = TMessage<N>;

    fn handle<'a>(&mut self, context: <UserEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        let now = get_time();

        async move {
            uprintln!("[CPY] <- {}", now.0);

            Ok(())
        }
    }
}

#[derive(Constructor)]
struct KernelSender<A, E, H, const N: usize>
where
    A: Actor<E, Message = TMessage<N>>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    target: PtActorMessageChannelAddress<A, E, H>
}

impl<A, E, H, const N: usize> Actor<RootEnvironment> for KernelSender<A, E, H, N>
where
    A: Actor<E, Message = TMessage<N>>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    type Message = ();

    fn create<'a>(&'a mut self, context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        let target = self.target.clone();

        async move {
            let mut msg: [u8; N] = [0; N];

            for i in 0..N {
                msg[i] = i as u8;
            }

            let mut x = Box::<TMessage<N>>::new_uninit();

            let now = get_time();

            unsafe {
                ptr::copy_nonoverlapping(msg.as_ptr(), x.as_mut_ptr() as *mut u8, N);
            }

            let mem = unsafe {x.assume_init()};

            target.send_msg(PtMessage::Copy(mem)).await;

            kprintln!("[CPY] -> {}", now.0);

            Ok(())
        }
    }
}