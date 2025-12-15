use core::fmt::Debug;

pub mod el;
mod handlers;

#[repr(C)]
pub struct ISRContext {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,

    pub elr_el1: u64,
    pub spsr_el1: u64,

    pub x30: u64,
    pub _pad: u64,
}

#[repr(C)]
pub struct EL1Context {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,

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

        ds.field("x0", &format_args!("{:#0x}", self.x0));
        ds.field("x1", &format_args!("{:#0x}", self.x1));
        ds.field("x2", &format_args!("{:#0x}", self.x2));
        ds.field("x3", &format_args!("{:#0x}", self.x3));
        ds.field("x4", &format_args!("{:#0x}", self.x4));
        ds.field("x5", &format_args!("{:#0x}", self.x5));
        ds.field("x6", &format_args!("{:#0x}", self.x6));
        ds.field("x7", &format_args!("{:#0x}", self.x7));
        ds.field("x8", &format_args!("{:#0x}", self.x8));
        ds.field("x9", &format_args!("{:#0x}", self.x9));
        ds.field("x10", &format_args!("{:#0x}", self.x10));
        ds.field("x11", &format_args!("{:#0x}", self.x11));
        ds.field("x12", &format_args!("{:#0x}", self.x12));
        ds.field("x13", &format_args!("{:#0x}", self.x13));
        ds.field("x14", &format_args!("{:#0x}", self.x14));
        ds.field("x15", &format_args!("{:#0x}", self.x15));
        ds.field("x16", &format_args!("{:#0x}", self.x16));
        ds.field("x17", &format_args!("{:#0x}", self.x17));
        ds.field("x18", &format_args!("{:#0x}", self.x18));
        ds.field("x19", &format_args!("{:#0x}", self.x19));
        ds.field("x20", &format_args!("{:#0x}", self.x20));
        ds.field("x21", &format_args!("{:#0x}", self.x21));
        ds.field("x22", &format_args!("{:#0x}", self.x22));
        ds.field("x23", &format_args!("{:#0x}", self.x23));
        ds.field("x24", &format_args!("{:#0x}", self.x24));
        ds.field("x25", &format_args!("{:#0x}", self.x25));
        ds.field("x26", &format_args!("{:#0x}", self.x26));
        ds.field("x27", &format_args!("{:#0x}", self.x27));
        ds.field("x28", &format_args!("{:#0x}", self.x28));
        ds.field("x29", &format_args!("{:#0x}", self.x29));

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

        ds.field("x0", &format_args!("{:#0x}", self.x0));
        ds.field("x1", &format_args!("{:#0x}", self.x1));
        ds.field("x2", &format_args!("{:#0x}", self.x2));
        ds.field("x3", &format_args!("{:#0x}", self.x3));
        ds.field("x4", &format_args!("{:#0x}", self.x4));
        ds.field("x5", &format_args!("{:#0x}", self.x5));
        ds.field("x6", &format_args!("{:#0x}", self.x6));
        ds.field("x7", &format_args!("{:#0x}", self.x7));
        ds.field("x8", &format_args!("{:#0x}", self.x8));
        ds.field("x9", &format_args!("{:#0x}", self.x9));
        ds.field("x10", &format_args!("{:#0x}", self.x10));
        ds.field("x11", &format_args!("{:#0x}", self.x11));
        ds.field("x12", &format_args!("{:#0x}", self.x12));
        ds.field("x13", &format_args!("{:#0x}", self.x13));
        ds.field("x14", &format_args!("{:#0x}", self.x14));
        ds.field("x15", &format_args!("{:#0x}", self.x15));
        ds.field("x16", &format_args!("{:#0x}", self.x16));
        ds.field("x17", &format_args!("{:#0x}", self.x17));
        ds.field("x18", &format_args!("{:#0x}", self.x18));
        ds.field("x19", &format_args!("{:#0x}", self.x19));
        ds.field("x20", &format_args!("{:#0x}", self.x20));
        ds.field("x21", &format_args!("{:#0x}", self.x21));
        ds.field("x22", &format_args!("{:#0x}", self.x22));
        ds.field("x23", &format_args!("{:#0x}", self.x23));
        ds.field("x24", &format_args!("{:#0x}", self.x24));
        ds.field("x25", &format_args!("{:#0x}", self.x25));
        ds.field("x26", &format_args!("{:#0x}", self.x26));
        ds.field("x27", &format_args!("{:#0x}", self.x27));
        ds.field("x28", &format_args!("{:#0x}", self.x28));
        ds.field("x29", &format_args!("{:#0x}", self.x29));

        ds.field("elr_el1", &format_args!("{:#0x}", self.elr_el1));
        ds.field("spsr_el1", &format_args!("{:#0x}", self.spsr_el1));

        ds.field("resome_elr", &format_args!("{:#0x}", self.resume_elr));
        ds.field("x30", &format_args!("{:#0x}", self.x30));
        ds.field("sp", &format_args!("{:#0x}", self.sp));
        ds.field("xptr", &format_args!("{:#0x}", self.xptr));

        ds.finish()
    }
}
