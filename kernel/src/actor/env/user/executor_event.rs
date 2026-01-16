use crate::isr::context::ISRContext;
use crate::isr::SvcType;
use crate::isr::svc_ctx::SyscallContext;

#[derive(Debug)]
pub struct SystemCallExecutorType {
    pub ctx: SyscallContext,
    pub args: [u64; 2],
    pub svc_num: SvcType
}

pub enum UserExecutorEvent {
    SystemCall(SystemCallExecutorType),
    Preemption(/*todo*/),
}
