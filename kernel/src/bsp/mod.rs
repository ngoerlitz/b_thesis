#[cfg(feature = "board-rpi4b")]
pub mod rpi4b;

#[cfg(feature = "board-rpi4b")]
pub use rpi4b as constants;
