use crate::UartSink;
use crate::drivers::common::readonly::ReadOnly;
use crate::drivers::common::register::RegU32;
use crate::drivers::common::writeonly::WriteOnly;
use crate::hal::irq::{CpuTarget, InterruptController, InterruptGroup};
use bitflags::{Flags, bitflags};
use core::fmt::Write;
use core::ptr::{NonNull, addr_of, write_volatile};

const GICD_CTLR_ENABLE_GRP0: u32 = 1 << 0;
const GICD_CTLR_ENABLE_GRP1: u32 = 1 << 1;

const GICC_CTLR_ENABLE_GRP0: u32 = 1 << 0;
const GICC_CTLR_ENABLE_GRP1: u32 = 1 << 1;
const GICC_CTLR_ACKCTL: u32 = 1 << 2; // make NS CPU-IF self-contained
const GICC_CTLR_FIQEN: u32 = 1 << 3; // route Grp1 to FIQ if set (we keep 0)
const GICC_CTLR_EOIMODENS: u32 = 1 << 9; // 0: EOIR also deactivates; 1: DIR needed

const PPI30_CNTPNSIRQ: u32 = 30;

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
    _r3: [u32; 32],                // 0xC80 - 0xCFC (reserved gap)

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

impl GIC400 {
    pub const unsafe fn new(base: usize) -> Self {
        Self {
            gicd: NonNull::new_unchecked((base + 0x1000) as *mut GIC400GICDRegisters),
            gicc: NonNull::new_unchecked((base + 0x2000) as *mut GIC400GICCRegisters),
        }
    }

    fn calculate_word_idx_shift(intid: u32) -> (usize, u32) {
        ((intid / 32) as usize, 1u32 << (intid % 32))
    }

    fn calculate_idx_shift(intid: u32) -> (usize, u32) {
        ((intid / 4) as usize, ((intid % 4) * 8))
    }

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

    fn get_idx_shift(&self, irq_num: u32) -> (usize, u32) {
        ((irq_num / 32) as usize, irq_num % 32)
    }

    pub fn enable_irq(&mut self, irq_num: u32, cpu: CpuTarget) {
        let gicd = self.gicd.as_ptr();

        self.set_irq_target_cpu(irq_num, cpu);
        self.set_irq_target_group(irq_num, InterruptGroup::One);
        self.clear_irq_pending_active_state(irq_num);

        let (idx, shift) = self.get_idx_shift(irq_num);

        unsafe {
            // Enable SPI
            (*gicd).isenabler[idx].modify(|v| v | (1 << shift));
        }
    }

    pub fn disable_irq(&mut self, irq_num: u32) {
        let gicd = self.gicd.as_ptr();
        let (idx, shift) = self.get_idx_shift(irq_num);

        unsafe {
            (*gicd).icenabler[idx].modify(|v| v | (1 << shift));
        }

        self.set_irq_target_cpu(irq_num, CpuTarget::empty());
    }

    pub(crate) fn set_irq_target_cpu(&mut self, irq_num: u32, cpu: CpuTarget) {
        let gicd = self.gicd.as_ptr();

        let (idx, offset) = (((irq_num - 32) / 4) as usize, ((irq_num - 32) % 4) * 8);
        let cpu_bits: u32 = cpu.bits().into();
        let mask = 0xFF << offset;

        unsafe {
            (*gicd).itargetsr[idx].modify(|v| (v & !mask) | (cpu_bits << offset));
        }
    }

    pub(crate) fn set_irq_target_group(&mut self, irq_num: u32, group: InterruptGroup) {
        let gicd = self.gicd.as_ptr();
        let (idx, shift) = self.get_idx_shift(irq_num);
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

    fn clear_irq_pending_active_state(&mut self, irq_num: u32) {
        let gicd = self.gicd.as_ptr();
        let (idx, shift) = self.get_idx_shift(irq_num);

        unsafe {
            (*gicd).icpendr[idx].modify(|v| v | (1 << shift));
            (*gicd).icactiver[idx].modify(|v| v | (1 << shift));
        }
    }

    pub fn read_iar(&self) -> u32 {
        // TODO: maybe we want to enable EOImodeNS, and then use DIR after the interrupt has been handled.
        // TODO: That way, we might be able to fix the issue with the double UART-IRQ being read...

        let gicc = self.gicc.as_ptr();

        unsafe { (*gicc).iar.read() }
    }

    pub fn write_eoir(&self, val: u32) {
        // TODO: Maybe we want to remove these in favour of direct accesses. However, we'd have the
        // TODO: issue of missing the GIC's base address without hardcoding it. This way, we're able
        // TODO: to use the instance to directly access the relevant offsets, at the cost of an
        // TODO: atomic operation - or blocking, if the GIC is currently being mutated. This, however
        // TODO: should only be the case VERY rarely (once during configuration and that's about it).

        let gicc = self.gicc.as_ptr();

        unsafe {
            /// SAFETY:
            /// The CPU-interface registers (IAR/EOIR/DIR/...) are banked per CPU core.
            /// Therefore, writing to this value does NOT need to be synchronized between
            /// cores, hence why shared references can concurrently access IAR and EOIR.
            (*gicc).eoir.write(val);
        }
    }

    pub fn debug<W: Write>(&self, w: &mut W) {
        let d = self.gicd.as_ptr();
        let c = self.gicc.as_ptr();

        let _ = writeln!(w, "\n\n================= GIC DEBUG =================");

        unsafe {
            let dctlr = (*d).ctlr.read();
            let _ = writeln!(
                w,
                "GICD_CTLR: 0x{:08x} (EnableGrp0={}, EnableGrp1={})",
                dctlr,
                dctlr & 1,
                (dctlr >> 1) & 1
            );

            let isenabler0 = (*d).isenabler[0].read();
            let enabled = (isenabler0 >> PPI30_CNTPNSIRQ) & 1;
            let _ = writeln!(
                w,
                "INTID 30 enabled: {} (ISENABLER0=0x{:08x})",
                enabled != 0,
                isenabler0
            );

            let igroupr0 = (*d).igroupr[0].read();
            let group = (igroupr0 >> PPI30_CNTPNSIRQ) & 1;
            let _ = writeln!(w, "INTID 30 group: {} (IGROUPR0=0x{:08x})", group, igroupr0);

            let cctlr = (*c).ctlr.read();
            let _ = writeln!(
                w,
                "GICC_CTLR: 0x{:08x} (EnableGrp0={}, EnableGrp1={}, EOImodeNS={})",
                cctlr,
                cctlr & 1,
                (cctlr >> 1) & 1,
                (cctlr >> 9) & 1
            );

            let pmr = (*c).pmr.read();
            let _ = writeln!(w, "GICC_PMR: 0x{:02x}", pmr);

            let pending0 = (*d).ispendr[0].read();
            let timer_pending = (pending0 >> PPI30_CNTPNSIRQ) & 1;
            let _ = writeln!(
                w,
                "INTID 30 pending: {} (ISPENDR0=0x{:08x})",
                timer_pending != 0,
                pending0
            );
        }

        let _ = writeln!(w, "==============================================\n\n");
    }
}
