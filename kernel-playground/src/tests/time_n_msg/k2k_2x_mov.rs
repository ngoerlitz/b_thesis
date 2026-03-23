use alloc::alloc::Global;
use zcene_core::actor::{Actor, ActorEnvironment, ActorHandleError, ActorFuture, ActorCreateError, ActorEnvironmentSpawn};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::OUTBOX_VA_ADDR;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_message::PtMessage::Page;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::drivers::mmu;
use kernel::kprintln;
use kernel_derive::Constructor;
use crate::tests::get_time;

const MESSAGE_SIZE: usize = 1000;
type TMessage = [u8; MESSAGE_SIZE];

pub fn register_tests<const N: usize>() {
    let env = RootEnvironment::get();

    let receiver = env.spawn(
        ReceivingActor::<N>::new()
    ).unwrap();

    env.spawn(
        SendingActor::<_, _, _, N>::new(receiver)
    ).unwrap();
}

struct ReceivingActor<const N: usize> {
    counter: usize,
}

impl<const N: usize> ReceivingActor<N>
{
    pub fn new() -> Self {
        Self { counter: 0 }
    }
}

impl<const N: usize> Actor<RootEnvironment> for ReceivingActor<N> {
    type Message = TMessage;

    fn handle<'a>(&mut self, context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        self.counter += 1;

        kprintln!("counter: {}", self.counter);

        if self.counter == N {
            let now = get_time();
            kprintln!("[MOV] <- {}", now.0);
        }

        async move {
            Ok(())
        }
    }
}

#[derive(Constructor)]
struct SendingActor<A, E, H, const N: usize>
where
    A: Actor<E, Message = TMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    target: PtActorMessageChannelAddress<A, E, H>,
}

fn _send_msg<'a, A, E, H>(target: &PtActorMessageChannelAddress<A, E, H>, ctx: &<RootEnvironment as ActorEnvironment>::CreateContext<'a>) -> (usize, usize)
where
    A: Actor<E, Message = TMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    let (page_id, addr) = ctx.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
    mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

    unsafe {
        let out = &mut *(OUTBOX_VA_ADDR as *mut TMessage);

        for i in 0..MESSAGE_SIZE {
            out[i] = i as u8;
        }
    }

    (page_id, addr)
}

impl<A, E, H, const N: usize> Actor<RootEnvironment> for SendingActor<A, E, H, N>
where
    A: Actor<E, Message = TMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    type Message = ();

    fn create<'a>(&'a mut self, context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        let target = self.target.clone();

        async move {
            let mut sent = 0;
            let mut now: u64 = 0;

            for _ in 0..N {
                let (page_id, addr) = _send_msg(&target, &context);

                if sent == 0 {
                    now = get_time().0;
                    target.send_msg(Page(page_id, addr)).await;
                } else
                {
                    target.send_msg(Page(page_id, addr)).await;
                }

                sent += 1;
            }

            kprintln!("[MOV] -> {}", now);

            Ok(())
        }
    }
}
