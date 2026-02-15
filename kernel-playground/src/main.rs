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
use alloc::boxed::Box;
use alloc::string::String;
use core::arch::asm;
use core::iter::Map;
use core::marker::PhantomData;
use kernel::actor::env::root::environment::RootEnvironment;
use kernel::actor::env::user::environment::UserEnvironment;
use kernel::boot::global::ACTOR_ROOT_ENVIRONMENT;
use kernel::{bootstrap_system, kprintln, test};
use zcene_core::actor::{Actor, ActorCreateError, ActorDestroyError, ActorEnvironment, ActorEnvironmentAllocator, ActorEnvironmentSpawn, ActorFuture, ActorHandleError, ActorMessageChannelSender, ActorMessageSender};
use zcene_core::future::r#yield;
use kernel::actor::channel::OUTBOX_VA_ADDR;
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


impl Actor<RootEnvironment> for RootActor {
    type Message = ();

    async fn create<'a>(
        &'a mut self,
        context: <RootEnvironment as ActorEnvironment>::CreateContext<'a>,
    ) -> Result<(), ActorCreateError> {
        let ping_addr = unsafe {
            RootEnvironment::get().spawn(RootActorTest::default()).unwrap()
        };

        ping_addr.send_msg(Copy(Box::new(25))).await;

        let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
        mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

        unsafe {
            *(OUTBOX_VA_ADDR as *mut u64) = 567;
        }

        ping_addr.send_msg(Page(page_id, addr)).await;

        let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
        mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

        unsafe {
            *(OUTBOX_VA_ADDR as *mut u64) = 888;
        }

        ping_addr.send_msg(Page(page_id, addr)).await;

        let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
        mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

        unsafe {
            *(OUTBOX_VA_ADDR as *mut u64) = 999;
        }

        ping_addr.send_msg(Page(page_id, addr)).await;

        let user_addr2 = unsafe {
            RootEnvironment::get()
                .spawn_user(UserActor::new(10), vec![])
                .unwrap()
        };

        let (page_id, addr) = context.environment.message_frame_allocator().lock().alloc_frame_addr().unwrap();
        mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);

        unsafe {
            *(OUTBOX_VA_ADDR as *mut u64) = 1024;
        }

        user_addr2.send_msg(Page(page_id, addr)).await;

        //
        // let user_addr1 = unsafe {
        //     RootEnvironment::get()
        //         .spawn_user(UserActor::new(20), vec![])
        //         .unwrap()
        // };
        //
        // user_addr2.send(77u64).await;
        // user_addr2.send(90u64).await;
        // user_addr2.send(140u64).await;
        // user_addr2.send(832u64).await;
        //
        // user_addr1.send(77u64).await;
        // user_addr1.send(90u64).await;
        // user_addr1.send(140u64).await;
        // user_addr1.send(832u64).await;



        //
        // let user_addr3 = unsafe {
        //     RootEnvironment::get()
        //         .spawn_user(UserActor::default(), vec![])
        //         .unwrap()
        // };
        //
        let user_addr4 = unsafe {
            RootEnvironment::get()
                .spawn_user(UserSender::new(
                    UserViewAddress::new(0, PhantomData)
                ), vec![Box::new(user_addr2.clone())])
                .unwrap()
        };
        //
        // ActorMessageSender::send(&user_addr, 25).await;
        // ActorMessageSender::send(&user_addr, 50).await;
        // ActorMessageSender::send(&user_addr, 75).await;
        //
        // ActorMessageSender::send(&user_addr3, 25).await;
        // ActorMessageSender::send(&user_addr3, 50).await;
        // ActorMessageSender::send(&user_addr3, 75).await;
        //
        // let new_actor = ReceivingActor::default();

        kprintln!("[ROOT] Sent message to `user_addr` channel");
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
