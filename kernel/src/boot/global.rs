use crate::actor::env::root::environment::RootEnvironment;
use crate::actor::runtime::handler::RuntimeHandler;
use crate::bsp;
use crate::drivers::gic400::GIC400;
use crate::drivers::pl011::PL011;
use crate::services::irq_manager::IrqManagerService;
use alloc::alloc::Global;
use alloc::sync::Arc;
use core::cell::SyncUnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::AtomicU8;
use spin::mutex::Mutex;
use spin::rwlock::RwLock;

pub static UART0: Mutex<PL011> = Mutex::new(unsafe { PL011::new(bsp::constants::UART0_BASE) });
pub static IRQ_MANAGER: RwLock<IrqManagerService<GIC400, 216>> =
    RwLock::new(IrqManagerService::new(GIC400::new()));

pub static ACTOR_ROOT_ENVIRONMENT: SyncUnsafeCell<
    MaybeUninit<Arc<RootEnvironment<RuntimeHandler>>>,
> = SyncUnsafeCell::new(MaybeUninit::uninit());

pub static ROOT_ENVIRONMENT_READY: AtomicU8 = AtomicU8::new(0);