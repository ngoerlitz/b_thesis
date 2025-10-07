use core::task::Poll;

#[repr(C, packed)]
pub struct ActorUserExecutorDeadlinePreemptionContext {
    // TODO: This needs to be changed (obviously :D)
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,

    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

pub struct ActorUserExecutorSystemCallContext {
    // TODO: This needs to be changed (obviously :D)
    rsp: u64,
    rip: u64,
    rflags: u64,
}

#[derive(Debug)]
pub enum ActorUserExecutorSystemCallType {
    Continue,
    Preempt,
    Poll(Poll<()>),
    SendMessageCopy(usize, usize),
    Unknown(usize),
}

pub struct ActorUserExecutorDeadlinePreemptionEvent {
    context: ActorUserExecutorDeadlinePreemptionContext,
}

pub struct ActorUserExecutorSystemCallEvent {
    r#type: ActorUserExecutorSystemCallType,
    context: ActorUserExecutorSystemCallContext,
}

pub enum ActorUserExecutorEvent {
    SystemCall(ActorUserExecutorSystemCallEvent),
    DeadlinePreemption(ActorUserExecutorDeadlinePreemptionEvent),
    Exception,
}
