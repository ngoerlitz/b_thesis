#![allow(warnings)] // Temporary until rewrite

use crate::drivers::pl011::PL011;
use crate::hal::driver::Driver;
use crate::{kprintln, linker_symbols};
use core::arch::asm;
use core::fmt::Write;

/// 4 KiB translation granule.
pub const PAGE_SIZE: u64 = 4096;

/// Level sizes for 4 KiB granule (AArch64).
pub const L1_BLOCK_SIZE: u64 = 1 << 30; // 1 GiB
pub const L2_BLOCK_SIZE: u64 = 1 << 21; // 2 MiB

/// We want to cover a 4 GiB VA space.
pub const VA_SPACE_SIZE: u64 = 0x1_0000_0000; // 4 GiB

/// Number of 2 MiB blocks needed to cover 4 GiB.
pub const NUM_L2_BLOCKS: usize = (VA_SPACE_SIZE / L2_BLOCK_SIZE) as usize; // 2048

/// Unmapped range: [940 MiB, 1 GiB)
pub const UNMAPPED_START: u64 = 940 * 1024 * 1024; // 0x3AC0_0000
pub const UNMAPPED_END: u64 = 1024 * 1024 * 1024; // 0x4000_0000

/// Device MMIO range: [0xFC00_0000, 4 GiB)
pub const MMIO_START: u64 = 0xFC00_0000;
pub const MMIO_END: u64 = 0x2_0000_0000;

/// Memory Attribute Indirection Register (MAIR_EL1):
///
/// Attr0 = Device-nGnRE
/// Attr1 = Normal, Inner/Outer WB WA cacheable
///
/// MAIR layout: each AttrN is one byte.
const MAIR_ATTR_DEVICE: u64 = 0x04; // Device-nGnRE
const MAIR_ATTR_NORMAL: u64 = 0xFF; // Normal WB WA
const MAIR_EL1_VALUE: u64 = (MAIR_ATTR_NORMAL << 8) | (MAIR_ATTR_DEVICE << 0);

/// AttrIndx values used in descriptors.
const ATTR_INDEX_DEVICE: u64 = 0;
const ATTR_INDEX_NORMAL: u64 = 1;

/// Shareability
const SH_NON_SHAREABLE: u64 = 0b00;
const SH_OUTER_SHAREABLE: u64 = 0b10;
const SH_INNER_SHAREABLE: u64 = 0b11;

/// Access Permissions AP[2:1] (stage 1):
const AP_EL1_RW_EL0_NO: u64 = 0b00;
const AP_EL1_RW_EL0_RW: u64 = 0b01;

/// Descriptor bit positions for stage-1 block/table descriptors (4 KiB granule).
const DESC_VALID: u64 = 1 << 0;
const DESC_TABLE: u64 = 1 << 1; // when set with VALID → table descriptor
// For block descriptors, bit[1] = 0, bit[0] = 1

const DESC_AF: u64 = 1 << 10; // Access flag
const DESC_SH_SHIFT: u64 = 8; // SH[1:0]
const DESC_AP_SHIFT: u64 = 6; // AP[2:1]
const DESC_ATTRIDX_SHIFT: u64 = 2; // AttrIndx[2:0]
const DESC_PXN: u64 = 1 << 53; // Privileged Execute Never
const DESC_UXN: u64 = 1 << 54; // Unprivileged Execute Never

/// A single page table (L1 or L2): 512 entries of 8 bytes = 4 KiB.
#[repr(align(4096))]
pub struct PageTable([u64; 512]);

static mut L1_TABLE: PageTable = PageTable([0; 512]);
static mut L2_TABLE0: PageTable = PageTable([0; 512]); // VA [0 .. 1 GiB)
static mut L2_TABLE1: PageTable = PageTable([0; 512]); // VA [1 .. 2 GiB)
static mut L2_TABLE2: PageTable = PageTable([0; 512]); // VA [2 .. 3 GiB)
static mut L2_TABLE3: PageTable = PageTable([0; 512]); // VA [3 .. 4 GiB)

/// Create a table descriptor that points to the next-level table.
fn make_table_desc(next_level_table_pa: u64) -> u64 {
    (next_level_table_pa & !0xFFF) | DESC_VALID | DESC_TABLE
}

/// Create a level-2 **block** descriptor for a 2 MiB region.
///
/// `pa` must be aligned to 2 MiB.
/// - AttrIndx from MAIR.
/// - SH: shareability.
/// - AP: access permissions.
/// - PXN/UXN: execute-never.
fn make_l2_block_desc(pa: u64, attr_index: u64, sh: u64, ap: u64, pxn: bool, uxn: bool) -> u64 {
    let mut desc = 0u64;

    // Output address bits [47:21] for 2 MiB block; low 21 bits must be 0.
    desc |= pa & !((1 << 21) - 1);

    // Mark as valid block.
    desc |= DESC_VALID; // bit 0 = 1
    // bit 1 = 0 for block descriptor (already 0)

    // Access flag
    desc |= DESC_AF;

    // Shareability
    desc |= (sh & 0b11) << DESC_SH_SHIFT;

    // AP[2:1]
    desc |= (ap & 0b11) << DESC_AP_SHIFT;

    // AttrIndx
    desc |= (attr_index & 0b111) << DESC_ATTRIDX_SHIFT;

    // Execute-never bits
    if pxn {
        desc |= DESC_PXN;
    }
    if uxn {
        desc |= DESC_UXN;
    }

    desc
}

/// Return a mutable reference to the L2 table corresponding to an L1 index.
unsafe fn l2_table_for_l1_index(l1_index: usize) -> *mut PageTable {
    match l1_index {
        0 => &raw mut L2_TABLE0,
        1 => &raw mut L2_TABLE1,
        2 => &raw mut L2_TABLE2,
        3 => &raw mut L2_TABLE3,
        _ => core::hint::unreachable_unchecked(),
    }
}

/// Initialize the stage-1 translation tables for a 4 GB VA space.
///
/// Must be called before enabling the MMU.
pub unsafe fn init_page_tables() {
    // 1 GiB chunks → 4 L1 entries.
    for l1_index in 0..4usize {
        let l2 = l2_table_for_l1_index(l1_index) as *mut PageTable as u64;
        L1_TABLE.0[l1_index] = make_table_desc(l2);
    }

    // Fill L2 blocks for the whole 4 GiB range.
    for block_idx in 0..NUM_L2_BLOCKS {
        let va = (block_idx as u64) * L2_BLOCK_SIZE;

        // Skip unmapped [940 MiB .. 1 GiB)
        if va >= UNMAPPED_START && va < UNMAPPED_END {
            continue;
        }

        let is_mmio = va >= MMIO_START && va < MMIO_END;

        // Identity mapping VA → PA.
        let pa = va;

        let (attr_index, sh, pxn, uxn) = if is_mmio {
            // Device-nGnRE, outer shareable, non-exec.
            (ATTR_INDEX_DEVICE, SH_OUTER_SHAREABLE, true, true)
        } else {
            // Normal WB WA, inner shareable, executable (tune to taste).
            (ATTR_INDEX_NORMAL, SH_INNER_SHAREABLE, false, false)
        };

        let l1_index = block_idx / 512; // 512 × 2 MiB = 1 GiB per L2 table
        let l2_index = block_idx % 512;

        let l2_table = l2_table_for_l1_index(l1_index);
        (*l2_table).0[l2_index] =
            make_l2_block_desc(pa, attr_index, sh, AP_EL1_RW_EL0_NO, pxn, uxn);
    }
}

linker_symbols! {
    USER_START = __user_start;
    USER_END   = __user_end;
    USER_STACK = __stack_el0_top;
}

pub unsafe fn init_user_page_tables() {
    let user_start_pa = USER_START() as u64;

    let mut writer = PL011::default();
    writer.enable();

    writeln!(writer, "Mapping user code to: {:#X}", user_start_pa);
    writeln!(writer, "USER_STACK_EL0_TOP: {:#X}", USER_STACK());

    debug_assert_eq!(
        user_start_pa & (L2_BLOCK_SIZE - 1),
        0,
        "__user_start not 2MiB-aligned"
    );

    let user_end_pa = USER_END() as u64;
    debug_assert!(user_end_pa >= user_start_pa, "__user_end < __user_start");
    debug_assert!(
        (user_end_pa - user_start_pa) <= L2_BLOCK_SIZE,
        "user image > 2MiB"
    );

    let user_va = user_start_pa;

    let block_idx = (user_va / L2_BLOCK_SIZE) as usize;
    let l1_index = block_idx / 512;
    let l2_index = block_idx % 512;

    let l2_table = l2_table_for_l1_index(l1_index);

    (*l2_table).0[l2_index] = make_l2_block_desc(
        user_start_pa,      // PA base (2MiB aligned)
        ATTR_INDEX_NORMAL,  // normal cacheable
        SH_INNER_SHAREABLE, // shareable like normal RAM
        AP_EL1_RW_EL0_RW,   // allow EL0 access (RW)
        true,               // PXN: EL1 execute-never
        false,              // UXN: allow EL0 execution
    );
}

/// Configure MAIR_EL1, TCR_EL1, TTBR0_EL1 and enable the MMU.
///
/// Assumes:
/// - Page tables have been initialized with `init_page_tables()`.
/// - Identity mapping VA == PA for the code that executes after enabling MMU.
pub unsafe fn enable_mmu_el1() {
    // Set MAIR_EL1: Attr0 = Device-nGnRE, Attr1 = Normal WB WA.
    asm!(
    "msr mair_el1, {0}",
    "isb",
    in(reg) MAIR_EL1_VALUE,
    options(nostack, preserves_flags),
    );

    // TCR_EL1 configuration:
    //
    // - T0SZ = 32  → 4 GiB VA space (2^(64 - 32)).
    // - TG0  = 0b00 → 4 KiB granule.
    // - SH0  = 0b11 → Inner Shareable.
    // - ORGN0 = 0b01, IRGN0 = 0b01 → Normal WB WA for walks.
    // - IPS  = 0b010 → 36-bit IPA (enough for BCM2711).
    // - EPD1 = 1 → disable TTBR1 walks.
    const TCR_T0SZ_4G: u64 = 32;

    let tcr_el1_value: u64 = (0b010 << 32) |      // IPS[34:32] = 36-bit PA
            (1 << 23)        |   // EPD1 = 1 (disable TTBR1 walks)
            (0b00 << 14)     |   // TG0 = 4 KiB
            (SH_INNER_SHAREABLE << 12) |
            (0b01 << 10)     |   // ORGN0 = WB WA
            (0b01 << 8)      |   // IRGN0 = WB WA
            (0 << 7)         |   // EPD0 = 0 (enable TTBR0 walks)
            (TCR_T0SZ_4G); // T0SZ = 32

    asm!(
    "msr tcr_el1, {0}",
    "isb",
    in(reg) tcr_el1_value,
    options(nostack, preserves_flags),
    );

    // TTBR0_EL1: base of L1 table.
    let l1_pa = &raw const L1_TABLE as *const PageTable as u64;
    asm!(
    "msr ttbr0_el1, {0}",
    "isb",
    in(reg) l1_pa,
    options(nostack, preserves_flags),
    );

    // Finally enable MMU (SCTLR_EL1.M) and caches (C/I).
    let mut sctlr: u64;
    asm!(
    "mrs {0}, sctlr_el1",
    out(reg) sctlr,
    options(nostack, preserves_flags),
    );

    // M  = bit 0  → enable MMU
    // C  = bit 2  → data cache
    // I  = bit 12 → instruction cache
    sctlr |= 1 << 0;
    sctlr |= 1 << 2;
    sctlr |= 1 << 12;

    asm!(
    "msr sctlr_el1, {0}",
    "isb",
    in(reg) sctlr,
    options(nostack, preserves_flags),
    );
}
