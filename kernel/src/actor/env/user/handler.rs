use crate::actor::env::user::ctx::UserEnvironmentHandleCtx;
use crate::actor::env::user::environment::UserEnvironment;
use crate::utils::memory::leaking_heap_memory_alloc::LeakingHeapMemoryAllocator;
use alloc::boxed::Box;
use core::arch::asm;
use core::pin::pin;
use core::task::{Context, Poll, Waker};
use zcene_core::actor::Actor;

pub(crate) extern "C" fn user_create_handler<A: Actor<UserEnvironment>>(actor: *mut A) -> ! {
    let mut actor = unsafe { Box::from_raw_in(actor, LeakingHeapMemoryAllocator) };

    let mut future_ctx = Context::from_waker(Waker::noop());
    let mut pinned = pin!(actor.create(()));

    let _result = match pinned.as_mut().poll(&mut future_ctx) {
        Poll::Pending => todo!(),
        Poll::Ready(result) => result,
    };

    unsafe {
        asm!(
            // TODO
            "wfe",
            options(noreturn)
        )
    }
}

pub(crate) extern "C" fn user_message_handler<A: Actor<UserEnvironment>>(
    actor: *mut A,
    msg: &A::Message,
) -> ! {
    let mut actor = unsafe { Box::from_raw_in(actor, LeakingHeapMemoryAllocator) };

    let mut future_ctx = Context::from_waker(Waker::noop());
    let mut pinned = pin!(actor.handle(UserEnvironmentHandleCtx::new(msg.clone())));

    let _result = match pinned.as_mut().poll(&mut future_ctx) {
        Poll::Pending => todo!(),
        Poll::Ready(result) => result,
    };

    unsafe {
        asm!(
            // TODO
            "wfe",
            options(noreturn)
        )
    }
}

pub(crate) extern "C" fn user_destroy_handler<A: Actor<UserEnvironment>>(actor: *mut A) -> ! {
    let mut actor = unsafe { Box::from_raw_in(actor, LeakingHeapMemoryAllocator) };

    let mut future_ctx = Context::from_waker(Waker::noop());
    let mut pinned = pin!(actor.destroy(()));

    let _result = match pinned.as_mut().poll(&mut future_ctx) {
        Poll::Pending => todo!(),
        Poll::Ready(result) => result,
    };

    unsafe {
        asm!(
            // TODO
            "wfe",
            options(noreturn)
        )
    }
}
