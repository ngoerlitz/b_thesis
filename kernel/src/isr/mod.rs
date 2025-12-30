use core::fmt::Debug;

pub(crate) mod context;
pub(crate) mod el;
pub(crate) mod handlers;

#[repr(u16)]
pub enum Svc {
    PrintMsg = 0x2,
    SendMsg = 0x3,
    ReturnEl1 = 0x4,
}

impl From<u16> for Svc {
    fn from(val: u16) -> Svc {
        match val {
            0x2 => Svc::PrintMsg,
            0x3 => Svc::SendMsg,
            0x4 => Svc::ReturnEl1,
            _ => unimplemented!(),
        }
    }
}
