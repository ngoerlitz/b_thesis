use core::fmt::{Display, Formatter};

pub enum ExceptionLevel {
    EL3,
    EL2,
    EL1,
    EL0,
}

impl Display for ExceptionLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ExceptionLevel::EL3 => write!(f, "EL3"),
            ExceptionLevel::EL2 => write!(f, "EL2"),
            ExceptionLevel::EL1 => write!(f, "EL1"),
            ExceptionLevel::EL0 => write!(f, "EL0"),
        }
    }
}
