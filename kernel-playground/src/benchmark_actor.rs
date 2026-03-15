use alloc::alloc::Global;
use core::alloc::Allocator;
use core::marker::PhantomData;
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::pt_channel_sender::PtActorMessageChannelSender;
use kernel::actor::env::root::environment::RootEnvironment;
use zcene_core::actor::{Actor, ActorCreateError, ActorEnvironment, ActorFuture, ActorMessage, ActorMessageSender};
use kernel::actor::channel::OUTBOX_VA_ADDR;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_message::PtMessage::Page;
use kernel::drivers::mmu;
use kernel::kprintln;

type TargetMessageType = [u64; 262_144];

pub struct BenchmarkActor<A, E, H>
where
    A: Actor<E, Message = TargetMessageType>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>,
    H::Allocator: Allocator + Clone,
{
    target: PtActorMessageChannelAddress<A, E, H>,
}

impl<A, E, H> BenchmarkActor<A, E, H>
where
    A: Actor<E, Message = TargetMessageType>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>,
    H::Allocator: Allocator + Clone,
{
    pub fn new(target: PtActorMessageChannelAddress<A, E, H>) -> Self {
        Self { target }
    }
}

impl<A, E, H> Actor<RootEnvironment> for BenchmarkActor<A, E, H>
where
    A: Actor<E, Message = TargetMessageType>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>,
    H::Allocator: Allocator + Clone,{
    type Message = u64;

    fn create<'a>(&'a mut self, context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        async move {
            let mut x: [u64; 100_000] = [0; 100_000];

            for i in 0..x.len() {
                x[i] = i as u64;
            }

            // kprintln!("{:?}", x);


            // self.target.send(x).await;

            // let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
            // mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);
            //
            // unsafe {
            //     let mut y = &mut *(OUTBOX_VA_ADDR as *mut [u64; 262_144]);
            //
            //     for i in 0..y.len() {
            //         y[i] = i as u64;
            //     }
            // }
            //
            // self.target.send_msg(Page(page_id, addr)).await;
            Ok(())
        }
    }

}