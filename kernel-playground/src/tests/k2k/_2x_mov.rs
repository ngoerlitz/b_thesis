use alloc::boxed::Box;
use alloc::vec;
use core::marker::PhantomData;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorEnvironmentSpawn, ActorFuture, ActorHandleError, ActorMessageSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::OUTBOX_VA_ADDR;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_message::PtMessage::Page;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::address::{MsgOf, UserViewAddress};
use kernel::drivers::mmu;
use kernel::kprintln;
use kernel_derive::Constructor;
use crate::tests::get_time;

type TMessage<const N: usize> = [u8; N];

#[inline(always)]
pub fn register_tests<const N: usize>() {
    let root_env = RootEnvironment::get();

    let receiver = root_env.spawn(
        ReceivingActor::<N>::default()
    ).unwrap();

    root_env.spawn(
        SendingActor::<_, _, _, N>::new(receiver)
    ).unwrap();
}

#[derive(Default)]
struct ReceivingActor<const N: usize>
{
}

impl<const N: usize> Actor<RootEnvironment> for ReceivingActor<N>
{
    type Message = TMessage<N>;

    fn handle<'a>(&mut self, context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        let now = get_time();

        async move {
            kprintln!("[MOV] <- {}", now.0);

            Ok(())
        }
    }
}

#[derive(Constructor)]
struct SendingActor<A, E, H, const N: usize>
where
    A: Actor<E, Message = TMessage<N>>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    target: PtActorMessageChannelAddress<A, E, H>
}

impl<A, E, H, const N: usize> Actor<RootEnvironment> for SendingActor<A, E, H, N>
where
    A: Actor<E, Message = TMessage<N>>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler
{
    type Message = ();

    fn create<'a>(&'a mut self, context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        let target = self.target.clone();

        async move {
            let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
            mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

            unsafe {
                let out = &mut *(OUTBOX_VA_ADDR as *mut TMessage<N>);

                for i in 0..N {
                    out[i] = i as u8;
                }
            }

            let now = get_time();
            target.send_msg(Page(page_id, addr)).await;

            kprintln!("[MOV] -> {}", now.0);

            Ok(())
        }
    }
}