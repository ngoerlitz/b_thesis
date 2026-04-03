use crate::drivers::timer::EL1PhysicalTimer;

static mut CPU_TIMER: [EL1PhysicalTimer; 4] = [EL1PhysicalTimer::new(); 4];

pub fn get_cpu_timer() -> &'static mut EL1PhysicalTimer {
    let cpuid = cpu::cpuid();
    unsafe { &mut CPU_TIMER[cpuid as usize] }
}

pub mod cpu;
pub mod irq;
pub mod registers;
