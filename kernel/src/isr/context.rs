use crate::actor::env::user::executor_event::UserExecutorEvent;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct ISRContext {
    pub elr_el1: u64,
    pub spsr_el1: u64,

    pub x: [u64; 31],
    pub _pad: u64,
}

#[repr(C, align(16))]
#[derive(Debug)]
pub struct EL1Context {
    pub ret_addr: u64,              // +0
    pub saved_sp: u64,              // +8

    pub event: *mut Option<UserExecutorEvent>,      // +16
    pub pad_: u64,  // +24

    pub xn: [u64; 12], // +32 [x19 - x30]
}
