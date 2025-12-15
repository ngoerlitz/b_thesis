use crate::drivers::common::readonly::ReadOnly;
use crate::drivers::common::register::RegU32;
use crate::drivers::common::writeonly::WriteOnly;
use crate::hal::driver::Driver;
use crate::hal::irq::InterruptController;
use crate::hal::irq_driver::{CpuTarget, InterruptGroup, IrqDriver, IrqType};
use crate::{bsp, kprintln};
use bitflags::{Flags, bitflags};
use core::cell::UnsafeCell;
use core::fmt::{Debug, Display, Formatter, Write};
use core::mem::offset_of;
use core::ops::Sub;
use core::ptr::{NonNull, addr_of, write_volatile};

const GICD_CTLR_ENABLE_GRP0: u32 = 1 << 0;
const GICD_CTLR_ENABLE_GRP1: u32 = 1 << 1;

const GICC_CTLR_ENABLE_GRP0: u32 = 1 << 0;
const GICC_CTLR_ENABLE_GRP1: u32 = 1 << 1;
const GICC_CTLR_ACKCTL: u32 = 1 << 2; // make NS CPU-IF self-contained
const GICC_CTLR_FIQEN: u32 = 1 << 3; // route Grp1 to FIQ if set (we keep 0)
const GICC_CTLR_EOIMODENS: u32 = 1 << 9; // 0: EOIR also deactivates; 1: DIR needed

const PPI30_CNTPNSIRQ: u32 = 30;

const GICD_OFFSET: usize = 0x1000;
const GICC_OFFSET: usize = 0x2000;

/// GIC-400 GICD (Generic Interrupt Controller Distributor)
/// Source: https://developer.arm.com/documentation/ddi0471/b/programmers-model/distributor-register-summary
#[repr(C)]
#[rustfmt::skip]
pub struct GIC400GICDRegisters {
    // 0x000 - 0x00C
    ctlr: RegU32,                        // 0x000 - GICD_CTLR
    typer: ReadOnly<RegU32>,             // 0x004 - GICD_TYPER
    iidr: ReadOnly<RegU32>,              // 0x008 - GICD_IIDR
    _r0: [u32; 29],                      // 0x00C - 0x07C

    igroupr: [RegU32; 16],               // 0x080 - 0x0BC (16 regs)
    _r1: [u32; 16],                      // 0x0C0 - 0x0FC

    isenabler: [RegU32; 32],             // 0x100 - 0x17C (32 regs)
    icenabler: [RegU32; 32],             // 0x180 - 0x1FC
    ispendr: [RegU32; 32],               // 0x200 - 0x27C
    icpendr: [RegU32; 32],               // 0x280 - 0x2FC
    isactiver: [RegU32; 32],             // 0x300 - 0x37C
    icactiver: [RegU32; 32],             // 0x380 - 0x3FC

    ipriorityr: [RegU32; 256],           // 0x400 - 0x7FC (256 regs)

    itargetsr_ro: [ReadOnly<RegU32>; 8], // 0x800 - 0x81C (RO)
    itargetsr: [RegU32; 120],            // 0x820 - 0x9FC
    _r2: [u32; 128],                     // 0xA00 - 0xBFC

    icfgr_sgi: ReadOnly<RegU32>,         // 0xC00
    icfgr_ppi: ReadOnly<RegU32>,         // 0xC04
    icfgr_spi: [RegU32; 30],             // 0xC08 - 0xC7C (30 regs)
    _r3: [u32; 32],                      // 0xC80 - 0xCFC (reserved gap)

    ppisr: ReadOnly<RegU32>,             // 0xD00 - GICD_PPISR
    spisr: [ReadOnly<RegU32>; 15],       // 0xD04 - 0xD3C
    _r4: [u32; 112],                     // 0xD40 - 0xEFC

    sgir: RegU32,                        // 0xF00
    _r5: [u32; 3],                       // 0xF04 - 0xF0C
    cpendsgir: [RegU32; 4],              // 0xF10 - 0xF1C
    spendsgir: [RegU32; 4],              // 0xF20 - 0xF2C
    _r6: [u32; 40],                      // 0xF30 - 0xFCC

    pidr4: ReadOnly<RegU32>,             // 0xFD0
    pidr5: ReadOnly<RegU32>,             // 0xFD4
    pidr6: ReadOnly<RegU32>,             // 0xFD8
    pidr7: ReadOnly<RegU32>,             // 0xFDC
    pidr0: ReadOnly<RegU32>,             // 0xFE0
    pidr1: ReadOnly<RegU32>,             // 0xFE4
    pidr2: ReadOnly<RegU32>,             // 0xFE8
    pidr3: ReadOnly<RegU32>,             // 0xFEC
    cidr0: ReadOnly<RegU32>,             // 0xFF0
    cidr1: ReadOnly<RegU32>,             // 0xFF4
    cidr2: ReadOnly<RegU32>,             // 0xFF8
    cidr3: ReadOnly<RegU32>,             // 0xFFC
}

/// GIC-400 GICC (Generic Interrupt Controller CPU Interface)
/// Source: https://developer.arm.com/documentation/ddi0471/b/programmers-model/cpu-interface-register-summary
#[repr(C)]
pub struct GIC400GICCRegisters {
    // 0x0000 - 0x001C
    ctlr: RegU32,             // 0x0000 - GICC_CTLR (RW)
    pmr: RegU32,              // 0x0004 - GICC_PMR  (RW)
    bpr: RegU32,              // 0x0008 - GICC_BPR  (RW)
    iar: ReadOnly<RegU32>,    // 0x000C - GICC_IAR  (RO)
    eoir: WriteOnly<RegU32>,  // 0x0010 - GICC_EOIR (WO)
    rpr: ReadOnly<RegU32>,    // 0x0014 - GICC_RPR  (RO)
    hppir: ReadOnly<RegU32>,  // 0x0018 - GICC_HPPIR (RO)
    abpr: RegU32,             // 0x001C - GICC_ABPR (RW)
    aiar: ReadOnly<RegU32>,   // 0x0020 - GICC_AIAR  (RO)
    aeoir: WriteOnly<RegU32>, // 0x0024 - GICC_AEOIR (WO)
    ahppir: ReadOnly<RegU32>, // 0x0028 - GICC_AHPPIR (RO)
    _reserved_0: [u32; 41],   // 0x002C..0x00CC
    apr0: RegU32,             // 0x00D0 - GICC_APR0 (RW)
    _reserved_1: [u32; 3],    // 0x00D4..0x00DC
    nsapr0: RegU32,           // 0x00E0 - GICC_NSAPR0 (RW)
    _reserved_2: [u32; 6],    // 0x00E4..0x00F8
    iidr: ReadOnly<RegU32>,   // 0x00FC - GICC_IIDR (RO)
    _reserved_3: [u32; 960],  // 0x0100..0x0FFC
    dir: WriteOnly<RegU32>,   // 0x1000 - GICC_DIR (WO)
}

///
/// Source: https://developer.arm.com/documentation/ddi0471/b/programmers-model/gic-400-register-map
pub struct GIC400 {
    gicd: NonNull<GIC400GICDRegisters>,
    gicc: NonNull<GIC400GICCRegisters>,
}

unsafe impl Send for GIC400 {}
unsafe impl Sync for GIC400 {}

macro_rules! gicc_reg_access {
    ($t: ty, $field: ident) => {
        (bsp::constants::GIC400_BASE + GICC_OFFSET + offset_of!($t, $field)) as *mut _
    };
}

impl GIC400 {
    pub const fn new() -> Self {
        unsafe {
            Self {
                gicd: NonNull::new_unchecked(
                    (bsp::constants::GIC400_BASE + GICD_OFFSET) as *mut GIC400GICDRegisters,
                ),
                gicc: NonNull::new_unchecked(
                    (bsp::constants::GIC400_BASE + GICC_OFFSET) as *mut GIC400GICCRegisters,
                ),
            }
        }
    }

    pub fn gicd_unchecked() -> *mut GIC400GICDRegisters {
        (bsp::constants::GIC400_BASE + GICD_OFFSET) as *mut _
    }

    pub fn gicc_unchecked() -> *mut GIC400GICCRegisters {
        (bsp::constants::GIC400_BASE + GICC_OFFSET) as *mut _
    }

    pub fn read_iar() -> u32 {
        let iar: *const ReadOnly<RegU32> = gicc_reg_access!(GIC400GICCRegisters, iar);

        unsafe { (*iar).read() }
    }

    pub fn irq_info() -> (u32, u32) {
        let iar = Self::read_iar();

        (iar, iar & 0x3FF)
    }

    pub fn write_eoir(val: u32) {
        let eoir: *mut WriteOnly<RegU32> = gicc_reg_access!(GIC400GICCRegisters, eoir);

        unsafe {
            /// SAFETY:
            /// The CPU-interface registers (IAR/EOIR/DIR/...) are banked per CPU core.
            /// Therefore, writing to this value does NOT need to be synchronized between
            /// cores, hence why shared references can concurrently access IAR and EOIR.
            (*eoir).write(val);
        }
    }
}

impl GIC400 {
    pub fn init(&mut self) {
        let gicd = self.gicd.as_ptr();
        let gicc = self.gicc.as_ptr();

        unsafe {
            (*gicc).ctlr.zero();
            (*gicd).ctlr.zero();

            (*gicd).ctlr.write(0x2);
        }
    }

    pub fn core_init(&self) {
        let gicd = self.gicd.as_ptr();
        let gicc = self.gicc.as_ptr();

        unsafe {
            (*gicd).igroupr[0].write(0xFFFF_FFFF);
            (*gicc).ctlr.write(0b110);

            (*gicc).pmr.write(0xFF);
            (*gicc).bpr.zero();

            (*gicd).isenabler[0].write(1 << PPI30_CNTPNSIRQ);
        }
    }

    fn get_idx_shift(&self, irq_type: IrqType) -> (usize, u32) {
        let irq_num: usize = irq_type.into();
        (irq_num / 32, (irq_num % 32) as u32)
    }

    fn clear_irq_pending_active_state(&mut self, irq_type: IrqType) {
        let gicd = self.gicd.as_ptr();
        let (idx, shift) = self.get_idx_shift(irq_type);

        unsafe {
            (*gicd).icpendr[idx].modify(|v| v | (1 << shift));
            (*gicd).icactiver[idx].modify(|v| v | (1 << shift));
        }
    }

    fn compute_itargetsr_offset(irq_num: usize) -> (usize, usize, u32) {
        let normalized_irq_num = irq_num - 32;
        let (index, offset) = (normalized_irq_num / 4, (normalized_irq_num % 4) * 8);
        (index, offset, 0xFF << offset)
    }

    fn compute_igroupr_offset(irq_num: usize) -> (usize, usize) {
        (irq_num / 32, irq_num % 32)
    }
}

impl Driver for GIC400 {
    const NAME: &'static str = "GIC-400 - Interrupt Distributor Driver";

    fn disable(&mut self) {
        todo!()
    }
}

impl IrqDriver for GIC400 {
    fn enable_irq(&mut self, irq_type: IrqType, cpu: Option<CpuTarget>) {
        let gicd = self.gicd.as_ptr();
        let gicc = self.gicc.as_ptr();

        match irq_type {
            IrqType::Sgi(n) => {
                unimplemented!()
            }
            IrqType::Ppi(n) => unsafe {
                let idx = n as usize;

                (*gicd).icpendr[0].enable_bit(idx);
                (*gicd).icactiver[0].enable_bit(idx);
                (*gicd).isenabler[0].enable_bit(idx);
            },
            IrqType::Spi(n) => unsafe {
                assert!(cpu.is_some());

                let idx = n as usize;
                let cpubits: u32 = cpu.unwrap().bits().into();

                let (n, offset, mask) = Self::compute_itargetsr_offset(idx);
                (*gicd).itargetsr[n].modify(|v| (v & !(mask)) | (cpubits << offset));

                let (n, offset) = Self::compute_igroupr_offset(idx);
                (*gicd).igroupr[n].enable_bit(offset);

                (*gicd).icpendr[n].enable_bit(offset);
                (*gicd).icactiver[n].enable_bit(offset);

                (*gicd).isenabler[n].enable_bit(offset);
            },
        }
    }

    fn disable_irq(&mut self, irq_type: IrqType) {
        let gicd = self.gicd.as_ptr();
        let (idx, shift) = self.get_idx_shift(irq_type);

        unsafe {
            (*gicd).icenabler[idx].modify(|v| v | (1 << shift));
        }

        self.set_irq_target(irq_type, CpuTarget::empty());
    }

    fn set_irq_target(&mut self, irq_type: IrqType, cpu: CpuTarget) {
        let gicd = self.gicd.as_ptr();
        let irq_num: usize = irq_type.into();

        let (idx, offset) = (((irq_num - 32) / 4) as usize, ((irq_num - 32) % 4) * 8);
        let cpu_bits: u32 = cpu.bits().into();
        let mask = 0xFF << offset;

        unsafe {
            (*gicd).itargetsr[idx].modify(|v| (v & !mask) | (cpu_bits << offset));
        }
    }

    fn set_irq_group(&mut self, irq_type: IrqType, group: InterruptGroup) {
        let gicd = self.gicd.as_ptr();
        let (idx, shift) = self.get_idx_shift(irq_type);
        let mask = 0b1 << shift;

        match group {
            InterruptGroup::Zero => unsafe {
                (*gicd).igroupr[idx].modify(|v| (v & !mask));
            },
            InterruptGroup::One => unsafe {
                (*gicd).igroupr[idx].modify(|v| (v & !mask) | (1 << 25));
            },
        }
    }
}

impl Display for GIC400 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use core::fmt::Write;

        let gicd = self.gicd.as_ptr();
        let gicc = self.gicc.as_ptr();

        let dctlr = unsafe { (*gicd).ctlr.read() };
        let isenabler0 = unsafe { (*gicd).isenabler[0].read() };
        let ispendr0 = unsafe { (*gicd).ispendr[0].read() };
        let isactiver0 = unsafe { (*gicd).isactiver[0].read() };
        let igroupr0 = unsafe { (*gicd).igroupr[0].read() };

        // ---- CPU Interface ----
        let cctlr = unsafe { (*gicc).ctlr.read() };
        let pmr = unsafe { (*gicc).pmr.read() };

        const INTID: u32 = PPI30_CNTPNSIRQ as u32;

        let enabled = ((isenabler0 >> INTID) & 1) != 0;
        let pending = ((ispendr0 >> INTID) & 1) != 0;
        let active = ((isactiver0 >> INTID) & 1) != 0;
        let group = ((igroupr0 >> INTID) & 1) as u8;

        let d_en_g0 = (dctlr & (1 << 0)) != 0;
        let d_en_g1 = (dctlr & (1 << 1)) != 0;

        let c_en_g0 = (cctlr & (1 << 0)) != 0;
        let c_en_g1 = (cctlr & (1 << 1)) != 0;
        let c_eoim_ns = (cctlr & (1 << 9)) != 0;

        writeln!(f, "================= GIC-400 DEBUG ================");

        writeln!(f, " Distributor:");
        writeln!(f, "   CTLR         : 0x{dctlr:08x}");
        writeln!(f, "     EnableGrp0 : {}", if d_en_g0 { 1 } else { 0 });
        writeln!(f, "     EnableGrp1 : {}", if d_en_g1 { 1 } else { 0 });
        writeln!(f, "   ISENABLER0   : 0x{isenabler0:08x}");
        writeln!(f, "   ISPENDR0     : 0x{ispendr0:08x}");
        writeln!(f, "   ISACTIVER0   : 0x{isactiver0:08x}");
        writeln!(f, "   IGROUPR0     : 0x{igroupr0:08x}");
        writeln!(f);

        writeln!(f, " CPU Interface:");
        writeln!(f, "   CTLR         : 0x{cctlr:08x}");
        writeln!(f, "     EnableGrp0 : {}", if c_en_g0 { 1 } else { 0 });
        writeln!(f, "     EnableGrp1 : {}", if c_en_g1 { 1 } else { 0 });
        writeln!(f, "     EOImodeNS  : {}", if c_eoim_ns { 1 } else { 0 });
        writeln!(f, "   PMR          : 0x{pmr:02x}");
        writeln!(f);

        writeln!(f, " INTID {INTID}:");
        writeln!(f, "   enabled      : {}", enabled);
        writeln!(
            f,
            "   group        : {} {}",
            group,
            if group == 0 { "(Group0)" } else { "(Group1)" }
        );
        writeln!(f, "   pending      : {}", pending);
        writeln!(f, "   active       : {}", active);
        writeln!(f, "===============================================");

        Ok(())
    }
}
