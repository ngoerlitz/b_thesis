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
use crate::save_callee_regs;

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
    STACK_EL0_TOP = __stack_el0_top;
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

        while let Some(message) = self.receiver.receive().await {
            kprintln!("USER: Received message: {:?}", message);

            // self.handle(move |actor, event, stack| {
            //     Self::execute_msg(
            //         Box::as_mut_ptr(actor),
            //         message,
            //         event,
            //         stack,
            //         handler::user_message_handler,
            //     )
            // })
            // .await;
        }

        kprintln!("USER: AFTER MESSAGE");

        self.handle(|actor, event, stack| {
            Self::execute(
                Box::as_mut_ptr(actor),
                event,
                stack,
                handler::user_destroy_handler,
            )
        })
        .await;

        kprintln!("USER: DONE");
    }

    async fn handle<F>(&mut self, func: F)
    where
        F: FnOnce(&mut Box<A>, &mut Option<UserExecutorEvent>, u64),
    {
        let mut event: Option<UserExecutorEvent> = None;

        // Todo: User stack (this should be from a proper KernelMemoryManager)
        let stack = STACK_EL0_TOP();

        self.enable_deadline();

        // Todo: execute function
        func(&mut self.actor, &mut event, stack as u64);

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
        #[cfg(feature = "log_debug")]
        kprintln!("actor: {}, fp: {}, event: {}, sp: {stack}", actor as u64, function as u64, event as *mut _ as u64);

        // TODO - THIS NEEDS TO BE CHECKED
        // Missing rust registers (in(reg) actor, ...)
        // Missing restore after label 1:
        // Missing sanity check to see if sp and reg vals are clobbered / restored correctly by llvm
        unsafe {
            asm!(
                "msr DAIFSet, #0b1111",
                "isb",

                "mov x9, sp",                   // Save the current SP

                save_callee_regs!(),            // Save registers x19-x30 (AAPCS)

                "adr x0, 1f",
                "stp x0, x9, [sp, #-16]!",      // Save return addr and SP

                "stp x12, xzr, [sp, #-16]!",    // Save xptr and padding (SP ~ 16 Byte)

                "msr SP_EL0, x10",
                "msr ELR_EL1, x11",

                "msr SPSR_EL1, xzr",

                "mov x0, x13",
                "isb",

                "eret",
                "1:",

                in("x10") stack as u64,
                in("x11") function as u64,
                in("x12") event as *mut _ as u64,
                in("x13") actor as u64,

                options(preserves_flags),
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
        loop {}

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
