#![no_std]
#![no_main]

use crate::UartSink;
use core::arch::asm;
use core::fmt::Write;
/* ======================= 4 KiB granule, 3 levels ======================== */

const PAGE_SHIFT: u64 = 12; // 4 KiB
const PGTAB_ENTRIES: usize = 512; // entries per table
const TT_ALIGN_MASK: u64 = !((1 << PAGE_SHIFT) - 1);

const VA_4G: u64 = 0x1_0000_0000;
const L1_BLOCK_SIZE: u64 = 1u64 << 30; // 1 GiB covered by one L1 entry
const L2_BLOCK_SIZE: u64 = 1u64 << 21; // 2 MiB covered by one L2 entry
const L3_PAGE_SIZE: u64 = 1u64 << 12; // 4 KiB covered by one L3 entry

const L1_ENTRIES_4G: usize = (VA_4G / L1_BLOCK_SIZE) as usize; // 4 L1 slots used
const L3_TABLES_TOTAL: usize = L1_ENTRIES_4G * PGTAB_ENTRIES; // 4 * 512 = 2048

/* ===== Descriptor bit fields (S1, 4 KiB granule) ===== */
const DESC_VALID: u64 = 1 << 0;
const DESC_TABLE: u64 = 1 << 1; // L1/L2: 1 = table; L3: 1 = page
const AF: u64 = 1 << 10;
const SH_IS: u64 = 0b11 << 8; // inner-shareable
const AP_RW_EL1: u64 = 0b10 << 6; // EL1 RW, EL0 no access
const UXN: u64 = 1 << 54;
const PXN: u64 = 1 << 53;

const AP_EL1_RW_EL0_NO: u64 = 0b00 << 6; // correct for kernel RW, user none
const AP_EL1_RO_EL0_NO: u64 = 0b10 << 6; // use for .text / vectors (RO)
const AP_EL1_RW_EL0_RW: u64 = 0b01 << 6; // not used here
const AP_EL1_RO_EL0_RO: u64 = 0b11 << 6; // not used here

const ATTRIDX_NORMAL: u64 = 0; // MAIR[0]
const ATTRIDX_DEVICE: u64 = 1; // MAIR[1]

#[derive(Copy, Clone)]
#[repr(align(4096))]
struct PageTable([u64; PGTAB_ENTRIES]);

#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss.page_tables")]
static mut L1: PageTable = PageTable([0; PGTAB_ENTRIES]);

// 4 L2 tables (one per used L1 entry)
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss.page_tables")]
static mut L2: [PageTable; L1_ENTRIES_4G] = [PageTable([0; PGTAB_ENTRIES]); L1_ENTRIES_4G];

// 2048 L3 tables (one per L2 entry across the four L2 tables)
#[unsafe(no_mangle)]
#[unsafe(link_section = ".bss.page_tables")]
static mut L3: [PageTable; L3_TABLES_TOTAL] = [PageTable([0; PGTAB_ENTRIES]); L3_TABLES_TOTAL];

#[inline(always)]
fn align_pt(p: u64) -> u64 {
    p & !((1u64 << PAGE_SHIFT) - 1)
}

#[inline(always)]
fn table_desc(next_pt_pa: u64) -> u64 {
    align_pt(next_pt_pa) | DESC_VALID | DESC_TABLE
}

const MMIO_START: u64 = 0x0_FC00_0000;
const MMIO_END: u64 = 0x1_0000_0000; // exclusive

#[inline(always)]
fn is_mmio(pa: u64) -> bool {
    pa >= MMIO_START && pa < MMIO_END
}

#[inline(always)]
fn in_text(pa: u64) -> bool {
    false
}

#[inline(always)]
fn l3_page_desc(pa: u64, attridx: u64, share: u64, ap: u64, pxn: bool, uxn: bool) -> u64 {
    let xn = (if pxn { PXN } else { 0 }) | (if uxn { UXN } else { 0 });
    align_pt(pa)
        | DESC_VALID
        | DESC_TABLE         // at L3 this bit encodes a *page* descriptor
        | (attridx << 2)     // AttrIndx[4:2]
        | ap
        | share
        | AF
        | xn
}

unsafe fn parange_to_ips() -> u64 {
    let mut mmfr0: u64;
    asm!("mrs {x}, ID_AA64MMFR0_EL1", x = out(reg) mmfr0, options(nomem, preserves_flags));
    match mmfr0 & 0xF {
        0 => 0b000,
        1 => 0b001,
        2 => 0b010,
        3 => 0b011,
        4 => 0b100,
        5 => 0b101,
        6 => 0b110,
        _ => 0b001,
    }
}

const RAM_END: u64 = 0x8000_0000; // 1 GiB (match your `-m`)

#[inline(always)]
fn in_ram(pa: u64) -> bool {
    pa < RAM_END
}

fn dump_l3_entry(idx: usize, desc: u64) {
    let pa = desc & !0xFFF;
    let valid = (desc & 1) != 0;
    let is_pg = (desc & 2) != 0;
    let attr = (desc >> 2) & 0x7;
    let ap = (desc >> 6) & 0x3;
    let sh = (desc >> 8) & 0x3;
    let af = (desc >> 10) & 0x1;
    let ng = (desc >> 11) & 0x1;
    let pxn = (desc >> 53) & 1;
    let uxn = (desc >> 54) & 1;

    let _ = writeln!(
        UartSink,
        "[{idx}] desc={:#018X} pa={:#010X} valid={} page={} attr={} ap={} sh={} af={} ng={} pxn={} uxn={}",
        desc, pa, valid as u8, is_pg as u8, attr, ap, sh, af, ng, pxn, uxn
    );
}

// TODO: REMOVE
#[inline(always)]
pub const fn l3_indices_for_pa(pa: u64) -> Option<(usize, usize, usize, usize)> {
    // Only the first 4 GiB are mapped in this scheme.
    if pa >= VA_4G {
        return None;
    }

    // 4 KiB pages with 1 GiB / 2 MiB / 4 KiB strides.
    let l1i = ((pa >> 30) & 0x1ff) as usize; // bits [38:30]
    let l2i = ((pa >> 21) & 0x1ff) as usize; // bits [29:21]
    let l3i = ((pa >> 12) & 0x1ff) as usize; // bits [20:12]

    // Flattened L3-table index across all L2 entries of the 4 L1 slots.
    let l3_table_idx = l1i * PGTAB_ENTRIES + l2i; // matches your create_flat_mapping_4g_l3_pages()

    Some((l1i, l2i, l3i, l3_table_idx))
}

pub unsafe fn jump_to(addr: u64) -> ! {
    // CAUSES Instruction Fetch Fault, if the addr is marked as PXN!
    asm!(
    "br {target}",
    target = in(reg) addr,
    );

    unreachable!();
}

pub unsafe fn cause_data_translation_load(addr: u64) -> u64 {
    // Single LDR from addr. If addr is unmapped, this will raise a Data Abort.
    let mut val: u64;
    asm!(
    "ldr {out}, [{inreg}]",
    out = out(reg) val,
    inreg = in(reg) addr,
    options(nostack, preserves_flags)
    );
    val // not reached if the abort triggers
}

pub unsafe fn intentionally_break() {
    let va = 0x4000_0000;

    let (_l1, _l2, l3i, l3_table_idx) = l3_indices_for_pa(va).unwrap(); // pick page-aligned PA
    let pte = &mut L3[l3_table_idx].0[l3i];

    // change only AP bits
    *pte |= PXN;

    dump_pte_for_va(va);

    asm!(
    "dsb ishst",
    "tlbi vae1is, {va}",
    "dsb ish",
    "isb",
    va = in(reg) va,
    options(nostack, preserves_flags)
    );
}

// TODO: END

unsafe fn create_flat_mapping_4g_l3_pages() {
    (*&raw mut L1).0.fill(0);
    for mut t in *&raw mut L2 {
        t.0.fill(0);
    }
    for mut t in *&raw mut L3 {
        t.0.fill(0);
    }

    for l1i in 0..L1_ENTRIES_4G {
        let l2_pa = (&L2[l1i] as *const _ as u64) & TT_ALIGN_MASK;
        L1.0[l1i] = table_desc(l2_pa);
        for l2i in 0..PGTAB_ENTRIES {
            let l3_idx = l1i * PGTAB_ENTRIES + l2i;
            let l3_pa = (&raw const L3[l3_idx] as *const _ as u64) & TT_ALIGN_MASK;
            L2[l1i].0[l2i] = table_desc(l3_pa);

            let base = ((l1i as u64) << 30) | ((l2i as u64) << 21);
            for l3j in 0..PGTAB_ENTRIES {
                let pa = base | ((l3j as u64) << 12);

                if in_ram(pa) {
                    // Normal WBWA; if you want “everything exec” in RAM, keep pxn/uxn=false.
                    L3[l3_idx].0[l3j] =
                        l3_page_desc(pa, ATTRIDX_NORMAL, SH_IS, AP_EL1_RW_EL0_NO, false, false);
                } else if is_mmio(pa) {
                    // Device, XN
                    L3[l3_idx].0[l3j] =
                        l3_page_desc(pa, ATTRIDX_DEVICE, SH_IS, AP_EL1_RW_EL0_RW, true, true);
                } else {
                    // Unmapped: leave zero to avoid external aborts from speculation.
                }
            }
        }
    }

    let (low, high) = get_user_text();
    let stack = unsafe { &__stack_bottom as *const _ as u64 };

    for i in 0..10 {
        let _ = writeln!(UartSink, "Mapping: {:X}", low + (i * 4096));
        manually_map(low + (i * 4096));

        manually_map(stack + (i * 4096));
    }

    manually_map(0x7ffefff0);
    manually_map(0x351a0);
    manually_map(0x36d98);
    manually_map(0x39b30);
    manually_map(0x3fbe0);
    manually_map(0x3e537);

    // let mut i = 0;
    // for pt in *&raw const L3 {
    //     dump_l3_entry(i, pt.0[0]);
    //
    //     i += 1;
    // }
}

unsafe extern "C" {
    static __user_text_start: u8;
    static __user_text_end: u8;
    static __stack_bottom: u8;
}

fn get_user_text() -> (u64, u64) {
    unsafe {
        let s = &__user_text_start as *const _ as u64;
        let e = &__user_text_end as *const _ as u64;

        // Make it robust if linker symbols appear swapped
        let (lo_raw, hi_raw) = if s <= e { (s, e) } else { (e, s) };

        // Page-align: [lo, hi)
        let lo = lo_raw & !0xfff; // align down
        let hi = (hi_raw + 0xfff) & !0xfff; // align up

        let _ = writeln!(UartSink, "{:X}-{:X}", lo, hi);

        (lo, hi)
    }
}

unsafe fn manually_map(va: u64) {
    let l1i = ((va >> 30) & 0x1ff) as usize; // 1
    let l2i = ((va >> 21) & 0x1ff) as usize; // 511
    let l3i = ((va >> 12) & 0x1ff) as usize; // 495

    let l3_idx = l1i * PGTAB_ENTRIES + l2i; // 1*512 + 511 = 1023

    // Identity map: PA == VA & !0xfff (otherwise, use the real PA)
    let pa = va & !0xfff;

    // Make the EL0 stack page RW at EL0 and XN
    L3[l3_idx].0[l3i] = l3_page_desc(
        pa,
        ATTRIDX_NORMAL,
        SH_IS,
        /*AP=*/ (0b01 << 6),
        /*pxn=*/ false,
        /*uxn=*/ false,
    );
}

const PA_MASK_4K_L3: u64 = 0x0000_FFFF_FFFF_F000; // [47:12]

unsafe fn dump_pte_for_va(va: u64) {
    let l1i = ((va >> 30) & 0x1ff) as usize;
    let l2i = ((va >> 21) & 0x1ff) as usize;
    let l3i = ((va >> 12) & 0x1ff) as usize;

    let l1d = L1.0[l1i];
    let l2_pa = l1d & PA_MASK_4K_L3;
    let l2 = &*(l2_pa as *const PageTable);
    let l2d = l2.0[l2i];
    let l3_pa = l2d & PA_MASK_4K_L3;
    let l3 = &*(l3_pa as *const PageTable);
    let l3d = l3.0[l3i];

    let attr = (l3d >> 2) & 0x7;
    let ap = (l3d >> 6) & 0x3;
    let sh = (l3d >> 8) & 0x3;
    let af = (l3d >> 10) & 0x1;
    let ng = (l3d >> 11) & 0x1;
    let pxn = (l3d >> 53) & 1;
    let uxn = (l3d >> 54) & 1;

    let _ = writeln!(
        UartSink,
        "VA={:#x}  L1i/L2i/L3i={}/{}/{}  L1={:#018x}  L2={:#018x}  L3={:#018x}  L3.pa={:#x} attr={} ap={} sh={} af={} ng={} pxn={} uxn={}",
        va,
        l1i,
        l2i,
        l3i,
        l1d,
        l2d,
        l3d,
        (l3d & PA_MASK_4K_L3),
        attr,
        ap,
        sh,
        af,
        ng,
        pxn,
        uxn
    );
}

pub unsafe fn init_enable_mmu_4k_l3_identity() {
    create_flat_mapping_4g_l3_pages();

    // MAIR: [0] = Normal WB/WA/RA (0xFF), [1] = Device-nGnRE (0x04)
    let mair = (0xFFu64 << (8 * ATTRIDX_NORMAL)) | (0x04u64 << (8 * ATTRIDX_DEVICE));

    // TCR: TTBR0 using 4 KiB TG0, 4 GiB VA (T0SZ=32), WB/RA/WA, inner-shareable.
    let t0sz = 32u64;
    let tg0 = 0b00u64 << 14; // 4 KiB
    let sh0 = 0b11u64 << 12; // IS
    let orgn0 = 0b01u64 << 10; // WB RA WA
    let irgn0 = 0b01u64 << 8; // WB RA WA
    let ips = parange_to_ips() << 32;
    let epd1 = 1u64 << 23; // disable TTBR1 walks
    let tcr = t0sz | tg0 | sh0 | orgn0 | irgn0 | ips | epd1;

    // TTBR0 to L1 base
    let ttbr0 = (&raw const L1 as *const _ as u64) & TT_ALIGN_MASK;

    dump_pte_for_va(0x79d8);

    asm!(
    "msr    MAIR_EL1, {mair}",
    "msr    TCR_EL1,  {tcr}",
    "msr    TTBR0_EL1,{ttbr0}",
    "isb",

    // Invalidate stage-1 TLBs for EL1&0
    "dsb    ish",
    "tlbi   vmalle1",
    "dsb    ish",
    "isb",

    // Enable I/D cache and MMU
    "mrs    x0, SCTLR_EL1",
    "bic    x0, x0, #(1 << 19)", // WXN = 0
    "orr    x0, x0, #(1 << 0)",   // M
    "orr    x0, x0, #(1 << 2)",   // C
    "orr    x0, x0, #(1 << 12)",  // I
    "msr    SCTLR_EL1, x0",
    "isb",
    mair = in(reg) mair,
    tcr = in(reg) tcr,
    ttbr0 = in(reg) ttbr0,
    out("x0") _,
    options(nostack, preserves_flags),
    );
}
