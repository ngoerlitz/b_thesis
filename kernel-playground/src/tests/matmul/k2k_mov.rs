use alloc::alloc::Global;
use alloc::boxed::Box;
use core::fmt::Debug;
use core::iter::zip;
use core::ptr;
use zcene_core::actor::{Actor, ActorEnvironment, ActorHandleError, ActorFuture, ActorCreateError, ActorEnvironmentSpawn, ActorMessageSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::OUTBOX_VA_ADDR;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_message::PtMessage;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::drivers::mmu;
use kernel::kprintln;
use kernel_derive::Constructor;
use crate::tests::get_time;

type MatrixVector<const N: usize> = MatrixMessage<N>;
type MatrixMessage<const N: usize> = [u32; N];
type Matrix<const N: usize> = [[u32; N]; N];

type MultiplyMessage<const N: usize> = (MatrixMessage<N>, usize);

type ReceiveMessage = (usize, usize);

fn print_matrix<const N: usize>(mat: &Matrix<N>) {
    for row in mat {
        kprintln!("{:?}", row);
    }
}

#[inline(always)]
pub fn register_tests<const N: usize>() {
    let root_env = RootEnvironment::get();

    let receiver = root_env.spawn(ReceivingActor::<N>::new()).unwrap();

    root_env.spawn(
        StartingActor::<_, _, _, N>::new(receiver)
    ).unwrap();
}

struct ReceivingActor<const N: usize>
{
    result: [usize; N],
    received: usize,
}

impl<const N: usize> ReceivingActor<N> {
    pub fn new() -> Self {
        Self {
            result: [0; N],
            received: 0,
        }
    }
}

impl<const N: usize> Actor<RootEnvironment> for ReceivingActor<N> {
    type Message = ReceiveMessage;

    fn handle<'a>(&mut self, context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        self.result[context.message.1] = context.message.0;
        self.received += 1;

        if self.received == N {
            let now = get_time().0;
            kprintln!("[MOV] <- {}", now);

            // kprintln!("{:?}", self.result);
        }

        async move {
            Ok(())
        }
    }
}

#[derive(Constructor)]
struct MultiplyingActor<A, E, H, const N: usize>
where
    A: Actor<E, Message = ReceiveMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    target: PtActorMessageChannelAddress<A, E, H>
}

impl<A, E, H, const N: usize> MultiplyingActor<A, E, H, N>
where
    A: Actor<E, Message = ReceiveMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    const fn gen_vec() -> [u32; N] {
        let mut vec = [0; N];

        let mut i = 0;
        while i < N {
            vec[i] = (i + 1) as u32;
            i += 1;
        }

        vec
    }
}

impl<A, E, H, const N: usize> Actor<RootEnvironment> for MultiplyingActor<A, E, H, N>
where
    A: Actor<E, Message = ReceiveMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    type Message = MultiplyMessage<N>;

    fn handle<'a>(&mut self, context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        // kprintln!("Received: {:?} -- {}", context.message.0, context.message.1);

        let mut vec = Self::gen_vec();

        let mut res: usize = 0;

        for (v, m) in zip(vec, context.message.0) {
            let vu = v as usize;
            let mu = m as usize;

            res += (vu * mu);
        }

        let target = self.target.clone();

        async move {
            let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
            mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

            unsafe {
                let out = &mut *(OUTBOX_VA_ADDR as *mut ReceiveMessage);

                (*out).0 = res;
                (*out).1 = context.message.1;
            }

            target.send_msg(PtMessage::Page(page_id, addr)).await;

            Ok(())
        }
    }
}

#[derive(Constructor)]
struct StartingActor<A, E, H, const N: usize>
where
    A: Actor<E, Message = ReceiveMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    target: PtActorMessageChannelAddress<A, E, H>
}

impl<A, E, H, const N: usize> Actor<RootEnvironment> for StartingActor<A, E, H, N>
where
    A: Actor<E, Message = ReceiveMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    type Message = ();

    fn create<'a>(&'a mut self, context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>) -> impl ActorFuture<'a, Result<(), ActorCreateError>> {
        let mut m: Matrix<N> = [[0; N]; N];

        let mut counter: usize = 1;

        counter = 0;

        for row in m.iter_mut() {
            for item in row.iter_mut() {
                *item = counter as u32;

                counter += 1;
            }
        }

        //print_matrix(&m);

        let target = self.target.clone();
        let env = context.eref.clone();

        async move {
            let now = get_time().0;

            for (row_index, row) in m.iter().enumerate() {
                let row_target = target.clone();

                let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
                mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

                unsafe {
                    let out = &mut *(OUTBOX_VA_ADDR as *mut MultiplyMessage<N>);

                    ptr::copy_nonoverlapping(row.as_ptr(), (*out).0.as_mut_ptr(), N);
                    (*out).1 = row_index;
                }

                let target_addr = env.spawn(
                    MultiplyingActor::<A, E, H, N>::new(row_target)
                ).unwrap();

                target_addr.send_msg(PtMessage::Page(page_id, addr)).await;
            }

            kprintln!("[MOV] -> {}", now);

            Ok(())
        }
    }
}