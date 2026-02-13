#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(arbitrary_self_types)]
#![feature(box_as_ptr)]
#![no_std]
#![no_main]

extern crate alloc;

pub mod actor;
pub mod boot;
mod bsp;
mod drivers;
mod hal;
pub mod isr;
mod platform;
mod services;
pub mod user;
pub mod utils;
mod memory;
pub mod test;