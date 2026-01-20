use core::fmt::Debug;

pub(crate) mod context;
pub(crate) mod el;
pub(crate) mod handlers;
pub(crate) mod svc_ctx;
pub(crate) mod irq_ctx;

#[repr(u16)]
#[derive(Debug)]
pub enum SvcType {
    PrintMsg = 0x2,
    SendMsg = 0x3,
    ReturnEl1 = 0x4,
    Test = 0x5,
}

impl From<u16> for SvcType {
    fn from(val: u16) -> SvcType {
        match val {
            0x2 => SvcType::PrintMsg,
            0x3 => SvcType::SendMsg,
            0x4 => SvcType::ReturnEl1,
            0x5 => SvcType::Test,
            _ => unimplemented!(),
        }
    }
}