use crate::actor::env::root::environment::RootEnvironment;
use crate::actor::env::user::environment::UserEnvironment;
use crate::actor::env::user::executor_event::UserExecutorEvent;
use crate::actor::env::user::handler;
use crate::actor::env::user::message_handler::UserMessageHandler;
use crate::platform::aarch64::get_cpu_timer;
use crate::{kprintln, linker_symbols};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::{asm, naked_asm};
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::num::NonZero;
use core::time::Duration;
use kernel_derive::Constructor;
use zcene_core::actor::{Actor, ActorEnvironmentAllocator, ActorMessageChannelReceiver};
use zcene_core::future::runtime::FutureRuntimeHandler;
use zcene_core::future::r#yield;

macro_rules! push_kernel_data {
    () => {
        r#"
            mov x0, sp
            adr x1, 1f
            stp x0, x1, [sp, #-16]!
        "#
    };
}
macro_rules! push_system_regs {
    () => {
        r#"
            mrs x0, ELR_EL1
            mrs x1, SPSR_EL1
            stp x0, x1, [sp, #-16]!
        "#
    };
}

macro_rules! push_callee_saved_regs {
    () => {
        r#"
            stp x29, x30, [sp, #-16]!
            stp x27, x28, [sp, #-16]!
            stp x25, x26, [sp, #-16]!
            stp x23, x24, [sp, #-16]!
            stp x21, x22, [sp, #-16]!
            stp x19, x20, [sp, #-16]!
        "#
    };
}

#[derive(Constructor)]
pub struct UserExecutor<A, H>
where
    A: Actor<UserEnvironment>,
    H: FutureRuntimeHandler,
{
    allocator: <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
    actor: Box<A>,
    receiver: ActorMessageChannelReceiver<A::Message>,
    deadline_in_ms: Option<NonZero<usize>>,
    // message_handlers: Vec<
    //     Box<
    //         dyn UserMessageHandler<RootEnvironment<H>>,
    //         <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
    //     >,
    //     <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
    // >,
    marker_h: PhantomData<H>,
}

linker_symbols! {
    USER_END = __user_end;
}

impl<A, H> UserExecutor<A, H>
where
    A: Actor<UserEnvironment>,
    H: FutureRuntimeHandler,
{
    pub async fn run(mut self)
    where
        A::Message: Debug,
    {
        kprintln!("USER: Inside Run function!");

        self.handle(|actor, event, stack| {
            Self::execute(
                Box::as_mut_ptr(actor),
                event,
                stack,
                handler::user_create_handler,
            );
        })
        .await;

        kprintln!("USER: AFTER HANDLE");

        // TODO: The receiver here should be a "normal" message CHANNEL - not a USER address. Since
        // TODO: the user address requires to be in EL0, but all of this code is handled in EL1.
        while let Some(message) = self.receiver.receive().await {
            kprintln!("USER: Received message: {:?}", message);

            self.handle(move |actor, event, stack| {
                Self::execute_msg(
                    Box::as_mut_ptr(actor),
                    message,
                    event,
                    stack,
                    handler::user_message_handler,
                )
            })
            .await;
        }

        self.handle(|actor, event, stack| {
            Self::execute(
                Box::as_mut_ptr(actor),
                event,
                stack,
                handler::user_destroy_handler,
            )
        })
        .await;
    }

    async fn handle<F>(&mut self, func: F)
    where
        F: FnOnce(&mut Box<A>, &mut Option<UserExecutorEvent>, u64),
    {
        let mut event: Option<UserExecutorEvent> = None;

        // Todo: User stack (this should be from a proper KernelMemoryManager)
        let stack = USER_END() - 16;

        self.enable_deadline();

        // Todo: execute function

        loop {
            match event.take() {
                None => break,
                Some(UserExecutorEvent::SystemCall()) => {}
                Some(UserExecutorEvent::Preemption()) => {
                    r#yield().await;

                    self.enable_deadline();

                    todo!("Continue from deadline preemption")
                }
            }
        }
    }

    fn enable_deadline(&mut self) {
        if let Some(deadline) = self.deadline_in_ms {
            let mut timer = get_cpu_timer();

            timer.set_interval(Duration::from_millis(deadline.get() as u64));
            timer.reset();
        }
    }

    #[inline(never)]
    #[rustfmt::skip]
    extern "C" fn execute(
        actor: *mut A,
        event: &mut Option<UserExecutorEvent>,
        stack: u64,
        function: extern "C" fn(*mut A) -> !,
    ) {
        // TODO - THIS NEEDS TO BE CHECKED
        // Missing rust registers (in(reg) actor, ...)
        // Missing restore after label 1:
        // Missing sanity check to see if sp and reg vals are clobbered / restored correctly by llvm
        unsafe {
            asm!(
                push_kernel_data!(),
                push_system_regs!(),
                push_callee_saved_regs!(),
                // "str {event}, [sp, #-16]!",

                // "msr SP_EL0, {stack}",
                // "msr ELR_EL1, {function}",
                // "msr SPSR_EL1, xzr",
                //
                // "mov x0, {actor}",

                "isb",
                "eret",

                "1:",
                options(noreturn),
                clobber_abi("C")
            )
        }
    }

    #[inline(never)]
    extern "C" fn execute_msg(
        actor: *mut A,
        message: A::Message,
        event: &mut Option<UserExecutorEvent>,
        stack: u64,
        function: extern "C" fn(*mut A, &A::Message) -> !,
    ) {
        // TODO
        unsafe { asm!("wfe", options(noreturn)) }
    }
}

// TODO: Verify if this struct is correct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StackState {
    // +0x00 (SP)
    pub event: u64,
    pub _pad_event: u64,

    // x19 - x30
    pub regs: [u64; 12],

    // +0x70
    pub elr_el1: u64,
    pub spsr_el1: u64,

    // +0x80
    pub saved_kernel_sp: u64,
    pub inline_return_addr: u64,
}

const _: () = {
    assert!(core::mem::size_of::<StackState>() == 144);
};
