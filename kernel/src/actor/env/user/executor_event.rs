use crate::isr::context::ISRContext;
use crate::isr::irq_ctx::El0IrqContext;
use crate::isr::SvcType;
use crate::isr::svc_ctx::SyscallContext;

#[derive(Debug)]
pub struct SystemCallExecutorType {
    pub ctx: SyscallContext,
    pub args: [u64; 2],
    pub svc_num: SvcType
}

#[derive(Debug)]
pub enum IrqType {
    Preemption,
    UartRx,
    Unknown
}

#[derive(Debug)]
pub struct IrqExecutorType {
    pub ctx: El0IrqContext,
    pub irq_type: IrqType,
    pub iar: u32,
}

pub enum UserExecutorEvent {
    SystemCall(SystemCallExecutorType),
    Irq(IrqExecutorType),
}
