use crate::actor::env::user::ctx::UserEnvironmentHandleCtx;
use crate::actor::env::user::environment::UserEnvironment;
use crate::utils::memory::leaking_heap_memory_alloc::NoOpMemoryAllocator;
use alloc::boxed::Box;
use core::arch::asm;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use zcene_core::actor::Actor;
use crate::{svc_call, uprintln};
use crate::isr::SvcType;

#[unsafe(link_section = ".user_text")]
pub(crate) extern "C" fn user_create_handler<A: Actor<UserEnvironment>>(actor: *mut A) -> ! {
    uprintln!("[USER_HANDLER 1/2] CREATE HANDLER!!! A: {:#X}", actor as u64);
    svc_call!(SvcType::Test);
    uprintln!("[USER_HANDLER 2/2] CREATE HANDLER!!! A: {:#X}", actor as u64);


    let mut actor = unsafe { Box::from_raw_in(actor, NoOpMemoryAllocator) };

    let mut future_ctx = Context::from_waker(Waker::noop());
    let mut pinned = pin!(actor.create(()));

    let _result = match pinned.as_mut().poll(&mut future_ctx) {
        Poll::Pending => todo!(),
        Poll::Ready(result) => result,
    };

    svc_call!(SvcType::ReturnEl1);
    unreachable!()
}

#[unsafe(link_section = ".user_text")]
pub(crate) extern "C" fn user_message_handler<A: Actor<UserEnvironment>>(
    actor: *mut A,
    msg: &A::Message,
) -> ! {
    // TODO: Adding this svc_call breaks things! Some regs may not be put back to their
    // TODO: correct values. This needs to be checked (with GDB)
    // svc_call!(SvcType::Test);
    // uprintln!("[USER_HANDLER] MESSAGE HANDLER!!! A: {:#X}. Message: {:?}", actor as u64, msg);

    let mut actor = unsafe { Box::from_raw_in(actor, NoOpMemoryAllocator) };

    let mut future_ctx = Context::from_waker(Waker::noop());
    let mut pinned = pin!(actor.handle(UserEnvironmentHandleCtx::new(msg.clone())));

    let _result = match pinned.as_mut().poll(&mut future_ctx) {
        Poll::Pending => todo!(),
        Poll::Ready(result) => result,
    };

    svc_call!(SvcType::ReturnEl1);
    unreachable!()
}

#[unsafe(link_section = ".user_text")]
pub(crate) extern "C" fn user_destroy_handler<A: Actor<UserEnvironment>>(actor: *mut A) -> ! {
    uprintln!("[USER_HANDLER] DESTROY HANDLER!!! A: {:#X}", actor as u64);

    let mut actor = unsafe { Box::from_raw_in(actor, NoOpMemoryAllocator) };

    let mut future_ctx = Context::from_waker(Waker::noop());
    let mut pinned = pin!(actor.destroy(()));

    let _result = match pinned.as_mut().poll(&mut future_ctx) {
        Poll::Pending => todo!(),
        Poll::Ready(result) => result,
    };

    svc_call!(SvcType::ReturnEl1);
    unreachable!()
}
