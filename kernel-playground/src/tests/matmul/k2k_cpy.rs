use alloc::alloc::Global;
use alloc::boxed::Box;
use core::fmt::Debug;
use core::iter::zip;
use core::ptr;
use zcene_core::actor::{Actor, ActorEnvironment, ActorHandleError, ActorFuture, ActorCreateError, ActorEnvironmentSpawn, ActorMessageSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_message::PtMessage;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::kprintln;
use kernel_derive::Constructor;
use crate::tests::get_time;

type MatrixVector<const N: usize> = MatrixMessage<N>;
type MatrixMessage<const N: usize> = [u8; N];
type Matrix<const N: usize> = [[u8; N]; N];

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
            kprintln!("[CPY] <- {}", now);
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

impl<A, E, H, const N: usize> Actor<RootEnvironment> for MultiplyingActor<A, E, H, N>
where
    A: Actor<E, Message = ReceiveMessage>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>
{
    type Message = MultiplyMessage<N>;

    fn handle<'a>(&mut self, context: <RootEnvironment as ActorEnvironment>::HandleContext<'a, Self::Message>) -> impl ActorFuture<'a, Result<(), ActorHandleError>> {
        let mut vec = [0u8; N];

        for (i, item) in vec.iter_mut().enumerate() {
            *item = (i + 1) as u8;
        }

        let mut res: usize = 0;

        for (v, m) in zip(vec, context.message.0) {
            res += (v * m) as usize;
        }

        let target = self.target.clone();

        async move {
            let mut x = Box::<ReceiveMessage>::new((res, context.message.1));

            target.send_msg(PtMessage::Copy(x)).await;

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

        let mut counter: u8 = 1;

        counter = 0;

        for row in m.iter_mut() {
            for item in row.iter_mut() {
                *item = counter;

                counter += 1;
            }
        }

        // print_matrix(&m);

        let target = self.target.clone();
        let env = context.eref.clone();

        async move {
            let now = get_time().0;

            for (row_index, row) in m.iter().enumerate() {
                let row_target = target.clone();

                let mut x = Box::<MultiplyMessage<N>>::new_uninit();

                unsafe {
                    let x_ptr = x.as_mut_ptr();

                    ptr::copy_nonoverlapping(row.as_ptr(), (*x_ptr).0.as_mut_ptr(), N);
                    ptr::write(ptr::addr_of_mut!((*x_ptr).1), row_index);
                }

                let msg = unsafe {x.assume_init()};

                let addr = env.spawn(
                    MultiplyingActor::new(row_target)
                ).unwrap();

                addr.send_msg(PtMessage::Copy(msg)).await;
            }

            kprintln!("[CPY] -> {}", now);

            Ok(())
        }
    }
}