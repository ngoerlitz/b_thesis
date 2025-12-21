use core::fmt::Debug;

pub mod el;
mod handlers;

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
    pub x: [u64; 30],

    pub elr_el1: u64,  // offset 240
    pub spsr_el1: u64, // offset 248

    pub resume_elr: u64, // offset 256  (address of label 1:)
    pub x30: u64,        // offset 264  (LR)
    pub sp: u64,         // offset 272  (exact pre-frame SP)
    pub xptr: u64,       // offset 280  (your saved pointer)
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

        ds.field("resome_elr", &format_args!("{:#0x}", self.resume_elr));
        ds.field("x30", &format_args!("{:#0x}", self.x30));
        ds.field("sp", &format_args!("{:#0x}", self.sp));
        ds.field("xptr", &format_args!("{:#0x}", self.xptr));

        ds.finish()
    }
}
