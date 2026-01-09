use crate::actor::env::user::executor_event::UserExecutorEvent;

#[repr(C)]
pub struct ISRContext {
    pub x: [u64; 30],

    pub elr_el1: u64,
    pub spsr_el1: u64,

    pub x30: u64,
    pub _pad: u64,
}

#[repr(C)]
pub struct EL1Context {
    pub event: *mut Option<UserExecutorEvent>,
    pub pad0: u64, // xzr (always 0)

    pub ret_addr: u64, // adr x0, 1f  (saved return address into kernel)
    pub saved_sp: u64, // x9          (kernel SP value from before your pushes)

    // Callee-saved GPRs as pushed by save_callee_regs!()
    pub x29: u64,
    pub x30: u64,

    pub x27: u64,
    pub x28: u64,

    pub x25: u64,
    pub x26: u64,

    pub x23: u64,
    pub x24: u64,

    pub x21: u64,
    pub x22: u64,

    pub x19: u64,
    pub x20: u64,
}

impl core::fmt::Debug for ISRContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("ExceptionFrame");

        for i in 0..30 {
            let name = match i {
                0 => "x0",
                1 => "x1",
                2 => "x2",
                3 => "x3",
                4 => "x4",
                5 => "x5",
                6 => "x6",
                7 => "x7",
                8 => "x8",
                9 => "x9",
                10 => "x10",
                11 => "x11",
                12 => "x12",
                13 => "x13",
                14 => "x14",
                15 => "x15",
                16 => "x16",
                17 => "x17",
                18 => "x18",
                19 => "x19",
                20 => "x20",
                21 => "x21",
                22 => "x22",
                23 => "x23",
                24 => "x24",
                25 => "x25",
                26 => "x26",
                27 => "x27",
                28 => "x28",
                29 => "x29",
                30 => "x30",
                _ => unreachable!(),
            };

            ds.field(name, &self.x[i]);
        }

        ds.field("elr_el1", &format_args!("{:#0x}", self.elr_el1));
        ds.field("spsr_el1", &format_args!("{:#0x}", self.spsr_el1));

        ds.field("x30", &format_args!("{:#0x}", self.x30));
        ds.field("_pad", &format_args!("{:#0x}", self._pad));

        ds.finish()
    }
}

impl core::fmt::Debug for EL1Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("EL1Context");

        // Metadata / frame header (lowest addresses first, as laid out)
        ds.field("event", &format_args!("{:#018x}", self.event as u64));
        ds.field("pad0", &format_args!("{:#018x}", self.pad0));
        ds.field("ret_addr", &format_args!("{:#018x}", self.ret_addr));
        ds.field("saved_sp", &format_args!("{:#018x}", self.saved_sp));

        // Callee-saved regs (print in architectural order)
        ds.field("x19", &format_args!("{:#018x}", self.x19));
        ds.field("x20", &format_args!("{:#018x}", self.x20));
        ds.field("x21", &format_args!("{:#018x}", self.x21));
        ds.field("x22", &format_args!("{:#018x}", self.x22));
        ds.field("x23", &format_args!("{:#018x}", self.x23));
        ds.field("x24", &format_args!("{:#018x}", self.x24));
        ds.field("x25", &format_args!("{:#018x}", self.x25));
        ds.field("x26", &format_args!("{:#018x}", self.x26));
        ds.field("x27", &format_args!("{:#018x}", self.x27));
        ds.field("x28", &format_args!("{:#018x}", self.x28));
        ds.field("x29", &format_args!("{:#018x}", self.x29));
        ds.field("x30", &format_args!("{:#018x}", self.x30));

        ds.finish()
    }
}
