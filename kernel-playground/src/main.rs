#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(arbitrary_self_types)]
#![no_std]
#![no_main]

mod receiver;
mod user;
mod user_sender;
mod root_actor_test;

extern crate alloc;

use crate::receiver::ReceivingActor;
use crate::user::UserActor;
use alloc::{format, vec};
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::string::String;
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
use crate::root_actor_test::RootActorTest;
use crate::user_sender::UserSender;

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
        *(OUTBOX_VA_ADDR as *mut u64) = 123;
    }

    actor_addr.send_msg(Page(page_id, addr)).await;
}

impl Actor<RootEnvironment> for RootActor {
    type Message = ();

    async fn create<'a>(
        &'a mut self,
        context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {
        let user_addr2 = unsafe {
            RootEnvironment::get()
                .spawn_user(UserActor::new(10), vec![])
                .unwrap()
        };


        unsafe {
            RootEnvironment::get()
                .spawn_user(UserSender::new(UserViewAddress::new(0, PhantomData)), vec![Box::new(user_addr2.clone())])
                .unwrap();
        }

        unsafe {
            RootEnvironment::get()
                .spawn_user(UserSender::new(UserViewAddress::new(0, PhantomData)), vec![Box::new(user_addr2.clone())])
                .unwrap();
        }

        unsafe {
            RootEnvironment::get()
                .spawn_user(UserSender::new(UserViewAddress::new(0, PhantomData)), vec![Box::new(user_addr2.clone())])
                .unwrap();
        }

        unsafe {
            RootEnvironment::get()
                .spawn_user(UserSender::new(UserViewAddress::new(0, PhantomData)), vec![Box::new(user_addr2.clone())])
                .unwrap();
        }

        unsafe {
            RootEnvironment::get()
                .spawn_user(UserSender::new(UserViewAddress::new(0, PhantomData)), vec![Box::new(user_addr2.clone())])
                .unwrap();
        }

        Ok(())
    }

    async fn destroy<'a>(
        self,
        _context: <RootEnvironment as ActorEnvironment>::DestroyContext<'a>,
    ) -> Result<(), ActorDestroyError> {
        kprintln!("Destroyed!");
        Ok(())
    }
}

bootstrap_system!(RootActor::default());
