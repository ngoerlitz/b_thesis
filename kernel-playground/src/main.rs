#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(arbitrary_self_types)]
#![no_std]
#![no_main]

mod tests;

extern crate alloc;

use alloc::{format, vec};
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::alloc::{Allocator, GlobalAlloc};
use core::arch::asm;
use core::iter::Map;
use core::marker::PhantomData;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::boot::global::ACTOR_ROOT_ENVIRONMENT;
use kernel::{bootstrap_system, kprintln, test};
use zcene_core::actor::{Actor, ActorMessage, ActorCreateError, ActorDestroyError, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentSpawn, ActorFuture, ActorHandleError, ActorMessageChannelSender, ActorMessageSender};
use zcene_core::future::r#yield;
use zcene_core::future::runtime::FutureRuntimeHandler;
use kernel::actor::channel::OUTBOX_VA_ADDR;
use kernel::actor::channel::pt_channel_address::PtActorMessageChannelAddress;
use kernel::actor::channel::pt_channel_sender::PtActorMessageChannelSender;
use kernel::actor::channel::pt_message::PtMessage::{Copy, Page};
use kernel::actor::env::root::service::message_frame_allocator_service::MessageFrameAllocatorService;
use kernel::actor::env::user::address::UserViewAddress;
use kernel::actor::env::user::message_handler::UserMessageHandler;
use kernel::drivers::mmu;
use crate::tests::sleep;

#[derive(Default)]
pub struct RootActor {
    id: usize,
}

impl RootActor {
    fn new(id: usize) -> Self {
        Self { id }
    }
}

async fn send<A, E, H>(i: u64, actor_addr: &PtActorMessageChannelAddress<A, E, H>, context: &<RootEnvironment as ActorEnvironment>::CreateContext<'_>)
where
    A: Actor<E, Message = u64>,
    E: ActorEnvironment,
    H: FutureRuntimeHandler<Allocator = Global>,
    H::Allocator: Allocator + Clone,
{
    let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
    mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

    unsafe {
        *(OUTBOX_VA_ADDR as *mut u64) = i;
    }

    actor_addr.send_msg(Page(page_id, addr)).await;
}

impl Actor<RootEnvironment> for RootActor {
    type Message = ();

    async fn create<'a>(
        &'a mut self,
        context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {

        // tests::u2u::_2x_cpy::register_tests();

        for i in 0..5 {
            tests::u2u::_2x_mov::register_tests();
            sleep(1_000_000);
        }

        sleep(50_000_000);

        for i in 0..5 {
            tests::u2u::_2x_cpy::register_tests();
            sleep(1_000_000);
        }

        // tests::_1_2x_k2k_100_bytes_move::register_tests();
        //
        // sleep(1_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_move::register_tests();
        //
        // sleep(1_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_move::register_tests();
        //
        // sleep(1_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_move::register_tests();
        //
        // sleep(1_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_move::register_tests();
        //
        // sleep(1_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_move::register_tests();
        //
        // sleep(50_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_copy::register_tests();
        //
        // sleep(1_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_copy::register_tests();
        //
        // sleep(1_000_000);
        // tests::_1_2x_k2k_100_bytes_copy::register_tests();
        //
        // sleep(1_000_000);
        // tests::_1_2x_k2k_100_bytes_copy::register_tests();
        //
        // sleep(1_000_000);
        //
        // tests::_1_2x_k2k_100_bytes_copy::register_tests();
        //
        // sleep(1_000_000);
        // tests::_1_2x_k2k_100_bytes_copy::register_tests();
        //
        // sleep(1_000_000);
        // tests::_1_2x_k2k_100_bytes_copy::register_tests();

        Ok(())
    }

    async fn destroy<'a>(
        self,
        _context: <RootEnvironment as ActorEnvironment>::DestroyContext<'a>,
    ) -> Result<(), ActorDestroyError> {
        // kprintln!("Destroyed!");
        Ok(())
    }
}

bootstrap_system!(RootActor::default());
