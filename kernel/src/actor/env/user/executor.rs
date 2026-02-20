use alloc::alloc::Global;
use crate::actor::env::root::environment::RootEnvironment;
use crate::actor::env::user::environment::UserEnvironment;
use crate::actor::env::user::executor_event::{IrqExecutorType, IrqType, SystemCallExecutorType, UserExecutorEvent};
use crate::actor::env::user::handler;
use crate::actor::env::user::message_handler::UserMessageHandler;
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::{kprintln, linker_symbols, log_dbg};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::arch::{asm, naked_asm};
use core::fmt::{Debug, Display};
use core::marker::PhantomData;
use core::num::{NonZero, NonZeroU64, NonZeroUsize};
use core::{slice};
use core::iter::Map;
use core::time::Duration;
use kernel_derive::Constructor;
use zcene_core::actor::{Actor, ActorEnvironmentAllocator, ActorMessageChannelAddress, ActorMessageChannelReceiver, ActorMessageChannelSender};
use zcene_core::future::runtime::FutureRuntimeHandler;
use zcene_core::future::r#yield;
use crate::actor::channel::{INBOX_VA_ADDR, OUTBOX_VA_ADDR};
use crate::actor::channel::pt_channel_receiver::PtActorMessageChannelReceiver;
use crate::actor::channel::pt_message::PtMessage;
use crate::drivers::gic400::GIC400;
use crate::drivers::mmu;
use crate::hal::driver::Driver;
use crate::isr::context::ISRContext;
use crate::isr::irq_ctx::El0IrqContext;
use crate::isr::svc_ctx::SyscallContext;
use crate::isr::SvcType;
use crate::platform::aarch64::cpu::cpuid;
use crate::save_callee_regs;

macro_rules! push_callee_saved_regs {
    () => {
        r#"
            sub sp, sp, #96
            stp x29, x30, [sp, #80]
            stp x27, x28, [sp, #64]
            stp x25, x26, [sp, #48]
            stp x23, x24, [sp, #32]
            stp x21, x22, [sp, #16]
            stp x19, x20, [sp, #0]
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
    receiver: PtActorMessageChannelReceiver<A::Message, H>,
    deadline_in_ms: Option<NonZeroU64>,
    message_handlers: Vec<
        Box<
            dyn UserMessageHandler<UserEnvironment>,
            <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
        >,
        <RootEnvironment<H> as ActorEnvironmentAllocator>::Allocator,
    >,
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
            kprintln!("USER GOT MESSAGE: {:?}", &message);

            let mut ptr: u64 = 0;

            match &message {
                PtMessage::Copy(b) => {
                    ptr = &**b as *const _ as u64;
                },
                PtMessage::Page(page_id, pa) => {
                    mmu::map_va_pa(INBOX_VA_ADDR, *pa as u64);
                    ptr = INBOX_VA_ADDR;
                }
            }

            debug_assert_ne!(ptr, 0);

            self.handle(move |actor, event, stack| {
                Self::execute_msg(
                    Box::as_mut_ptr(actor),
                    ptr as *const A::Message,
                    event,
                    stack,
                    handler::user_message_handler,
                )
            })
                .await;

            // Free page after it has been read!
            match message {
                PtMessage::Page(id, _) => {
                    RootEnvironment::get().message_frame_allocator().lock().free_frame(id);
                },
                _ => {}
            }
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

        kprintln!(
            "USER: DONE\n\
            ================================================================================================================"
        );
    }

    fn notify_local_end_of_irq(iar: u32) {
        GIC400::write_eoir(iar);
    }

    fn enable_deadline(&mut self) {
        if let Some(deadline) = self.deadline_in_ms {
            let mut timer = get_cpu_timer();

            timer.set_interval(Duration::from_millis(deadline.get()));
            timer.reset();
            timer.enable();
        }
    }

    fn setup_memory_mappings(&self, addr: usize) {
        mmu::map_va_pa(OUTBOX_VA_ADDR, addr as u64);
    }

    async fn handle<F>(&mut self, func: F)
    where
        F: FnOnce(&mut Box<A>, &mut Option<UserExecutorEvent>, u64),
    {
        let mut event: Option<UserExecutorEvent> = None;

        let r = RootEnvironment::get().user_stack_manager().lock().get_stack_addr().unwrap();
        let stack = r.1;

        let (mut page_id, mut addr) = RootEnvironment::get().message_frame_allocator().lock().alloc_frame_addr().unwrap();

        self.enable_deadline();
        self.setup_memory_mappings(addr);

        func(&mut self.actor, &mut event, stack as u64);

        loop {
            match event.take() {
                None => break,
                Some(UserExecutorEvent::SystemCall(ctx)) => {
                    kprintln!("{:?} args_hex: [{:#X}, {:#X}]", ctx, ctx.args[0], ctx.args[1]);

                    match ctx.svc_num {
                        SvcType::PrintMsg => {
                            let slice = unsafe {
                                slice::from_raw_parts(ctx.args[0] as *const u8, ctx.args[1] as usize)
                            };

                            match str::from_utf8(slice) {
                                Ok(s) => kprintln!("User: {}", s),
                                Err(_) => kprintln!("Invalid UTF-8 string"),
                            }

                            Self::continue_from_syscall(&mut event, &ctx.ctx);
                        },
                        SvcType::Test => {
                            Self::continue_from_syscall(&mut event, &ctx.ctx);
                        },
                        SvcType::SendMsg => {
                            // arg0 = target_actor_id
                            // arg1 = msg_ptr

                            match self.message_handlers.get(ctx.args[0] as usize) {
                                Some(handler) => {
                                    kprintln!("MSG_PTR: {:X}", ctx.args[1] as u64);
                                    let _result = handler.send(&Global, ctx.args[1] as *const ()).await;
                                },
                                None => todo!()
                            }

                            Self::continue_from_syscall(&mut event, &ctx.ctx);
                        },
                        SvcType::SendPt => {
                            // arg0 = target_actor_id

                            match self.message_handlers.get(ctx.args[0] as usize) {
                                Some(handler) => {
                                    kprintln!("Sending PAGE!! {:X} @ {:#X}", page_id, addr);
                                    let _result = handler.send_page(&Global, page_id, addr).await;
                                },
                                None => todo!()
                            }

                            (page_id, addr) = RootEnvironment::get().message_frame_allocator().lock().alloc_frame_addr().unwrap();
                            self.setup_memory_mappings(addr);

                            Self::continue_from_syscall(&mut event, &ctx.ctx);
                        },
                        SvcType::ReturnEl1 => {
                            break;
                        },
                        _ => unimplemented!()
                    }
                }
                Some(UserExecutorEvent::Irq(ctx)) => {
                    kprintln!("{:?}", ctx);

                    match ctx.irq_type {
                        IrqType::Preemption => {
                            get_cpu_timer().mask_irq();
                            Self::notify_local_end_of_irq(ctx.iar);

                            r#yield().await;

                            self.enable_deadline();

                            Self::continue_from_irq(&mut event, &ctx.ctx);
                        },
                        IrqType::UartRx => {
                            let char = RootEnvironment::get().logger().read_char();
                            if (char.is_some()) {
                                kprintln!("Received char: '{}'", char.unwrap() as char);
                            }

                            Self::notify_local_end_of_irq(ctx.iar);
                            Self::continue_from_irq(&mut event, &ctx.ctx);
                        },
                        IrqType::Unknown => {
                            unimplemented!()
                        }
                    }
                }
            }
        }

        RootEnvironment::get().message_frame_allocator().lock().free_frame(page_id);

        cpu::disable_irq();
        RootEnvironment::get().user_stack_manager().lock().free_stack(r.0);
    }

    #[inline(never)]
    extern "C" fn continue_from_irq(
        event: &mut Option<UserExecutorEvent>,
        ctx: &El0IrqContext
    ) {
        unsafe {
            asm!(
                r#"
                    mov x1, sp
                "#,
                    save_callee_regs!(),
                r#"
                    adr x0, 1f
                    sub sp, sp, #32
                    stp x12, xzr,   [sp, #16]
                    stp x0, x1,     [sp, #0]

                    ldp x0, x1, [x13, #0]
                    msr ELR_EL1, x0
                    msr SPSR_EL1, x1

                    add x13, x13, #16

                    ldp x30, x0, [x13, #240]
                    msr SP_EL0, x0

                    ldp x0, x1,   [x13, #0]
                    ldp x2, x3,   [x13, #16]
                    ldp x4, x5,   [x13, #32]
                    ldp x6, x7,   [x13, #48]
                    ldp x8, x9,   [x13, #64]
                    ldp x10, x11, [x13, #80]
                    ldp x14, x15, [x13, #112]
                    ldp x16, x17, [x13, #128]
                    ldp x18, x19, [x13, #144]
                    ldp x20, x21, [x13, #160]
                    ldp x22, x23, [x13, #176]
                    ldp x24, x25, [x13, #192]
                    ldp x26, x27, [x13, #208]
                    ldp x28, x29, [x13, #224]

                    // Load x13 last
                    ldp x12, x13, [x13, #96]

                    isb
                    eret
                    1:
                "#,
                in("x12") (event as *const _ as u64),
                in("x13") (ctx as *const _ as u64),
                clobber_abi("C")
            )
        }
    }

    #[inline(never)]
    extern "C" fn continue_from_syscall(
        event: &mut Option<UserExecutorEvent>,
        ctx: &SyscallContext,
    ) {
        unsafe {
            asm!(
                r#"
                    mov x1, sp
                "#,
                    save_callee_regs!(),
                r#"
                    adr x0, 1f
                    sub sp, sp, #32
                    stp x12, xzr, [sp, #16]
                    stp x0, x1, [sp, #0]

                    ldp x19, x20, [x13, #0]
                    ldp x21, x22, [x13, #16]
                    ldp x23, x24, [x13, #32]
                    ldp x25, x26, [x13, #48]
                    ldp x27, x28, [x13, #64]
                    ldp x29, x30, [x13, #80]

                    ldp x15, x16, [x13, #96]
                    msr ELR_EL1, x15
                    msr SPSR_EL1, x16

                    isb
                    eret
                    1:
                "#,
                in("x12") (event as *const _ as u64),
                in("x13") (ctx as *const _ as u64),
                clobber_abi("C")
            )
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

        unsafe {
            asm!(
                "mov x1, sp",                   // Save the current SP

                save_callee_regs!(),            // Save registers x19-x30 (AAPCS)

                "adr x0, 1f",
                "sub sp, sp, #32",
                "stp x12, xzr, [sp, #16]",    // Save event-ptr and padding (SP ~ 16 Byte)
                "stp x0, x1, [sp, #0]",      // Save return addr and SP

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
        message: *const A::Message,
        event: &mut Option<UserExecutorEvent>,
        stack: u64,
        function: extern "C" fn(*mut A, &A::Message) -> !,
    ) {
        unsafe {
            asm!(
                "mov x1, sp",                   // Save the current SP

                save_callee_regs!(),            // Save registers x19-x30 (AAPCS)

                "adr x0, 1f",
                "sub sp, sp, #32",
                "stp x12, xzr, [sp, #16]",    // Save event-ptr and padding (SP ~ 16 Byte)
                "stp x0, x1, [sp, #0]",      // Save return addr and SP

                "msr SP_EL0, x10",
                "msr ELR_EL1, x11",

                "msr SPSR_EL1, xzr",

                "mov x0, x13",
                "mov x1, x14",
                "isb",

                "eret",
                "1:",

                in("x10") stack as u64,
                in("x11") function as u64,
                in("x12") event as *mut _ as u64,
                in("x13") actor as u64,
                in("x14") message as *const _ as u64,

                options(preserves_flags),
                clobber_abi("C")
            )
        }
    }
}