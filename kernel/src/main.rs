#![no_std]
#![no_main]
#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
extern crate alloc;

mod actor;
mod boot;
mod bsp;
mod drivers;
mod hal;
mod isr;
mod platform;
mod services;
mod utils;
