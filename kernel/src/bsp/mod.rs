#[cfg(all(feature = "qemu", feature = "hardware"))]
compile_error!("Features `qemu` and `hardware` are exclusive");

#[cfg(not(any(feature = "qemu", feature = "hardware")))]
compile_error!("One feature required [none selected]: `qemu` or `hardware`");

mod rpi4b;
mod rpi4b_qemu;

#[cfg(feature = "hardware")]
use rpi4b as imp;

#[cfg(feature = "qemu")]
use rpi4b_qemu as imp;

pub use imp::secondary_boot;

/// These constants are shared between QEMU and the Hardware.
/// Might be changed in the future to reflect f.e. memory differences (2GB vs 4GB etc.)
pub use rpi4b::constants;
