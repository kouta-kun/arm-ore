//! ARMChine_rs is a fantasy 5gen-ish console based on an ARM946 CPU. The emulator
//! (which is the reference implementation... for now 😊 ) is developed in a
//! [Modular manner](features::EmulatorFeature).
//!
//! There is an [Optical disk-like Filesystem](filesystem::EmulatorDrive),
//! a [3D Rasterizer](gpu::feature::GPUFeature) with multiple backends,
//! and of course [Dynamic memory](dynmemory::DynamicMemoryAllocations)!
//!
//! All of these are subject to change over the course of the initial development. Have fun!


#![allow(unused_imports)]

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use std::borrow::{Borrow, BorrowMut};
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::format;
use unicorn::{RegisterARM, Unicorn, UnicornHandle};
use unicorn::unicorn_const::Arch::ARM;
use unicorn::unicorn_const::{HookType, Mode, Permission, uc_error};
use xmas_elf::{ElfFile, header, sections};
use std::fs;
use std::io::Write;
use std::ops::DerefMut;
use std::ptr::{null, null_mut};
use capstone::arch::arm::ArchMode;
use capstone::arch::BuildsCapstone;
use capstone::Capstone;
use libc::{c_void, link, size_t};
use unicorn::ffi::uc_hook;
use xmas_elf::symbol_table::{Entry, Entry32};
use configuration::Arguments;
use features::EmulatorFeature;
use clap::Parser;
use crate::filesystem::Drive;
use crate::gpu::feature::GPUFeature;

mod emulator;
mod filesystem;
mod features;
mod console;
mod dynmemory;
mod gpu;
mod input;
mod configuration;

fn main() {

    let args: Arguments = Arguments::parse();

    let mut unicorn = emulator::create_emulator();
    let mut unicorn_handle: UnicornHandle = Unicorn::borrow(
        &mut unicorn);


    let (mem_sz, main_idx) = {
        let drive = Drive::new(args.iso.as_ref());

        emulator::load_executable(&mut unicorn_handle, &drive).unwrap()
    };
    let mut features = configuration::get_features(&args, mem_sz);

    emulator::initialize_all_features(&mut unicorn_handle, &mut features);


    unicorn_handle.add_intr_hook(|mut _emu, _syscall| _emu.emu_stop().unwrap()).unwrap();

    {
        unicorn_handle.reg_write(RegisterARM::SP as i32, 0x10000).unwrap();
        unicorn_handle.reg_write(RegisterARM::PC as i32, main_idx).unwrap();
        let mut must_loop = true;

        while must_loop {
            let pc = unicorn_handle.reg_read(RegisterARM::PC as i32).unwrap();

            let t1 = std::time::Instant::now();
            let e = unicorn_handle.emu_start(pc, mem_sz, 0, 0);
            let t2 = std::time::Instant::now();
            if args.debug {
                emulator::print_disassembly(&mut unicorn_handle, mem_sz, main_idx, e);
            }
            if e.is_err() {
                break;
            }

            let dt = t2.duration_since(t1).as_millis();
            print!("Execution time: {}; ", dt);

            #[cfg(feature = "gpu-feature")] {
                emulator::video_update(&mut features, &mut must_loop)
            }
            print!("\r");
            std::io::stdout().flush().unwrap();
        }
    }


    emulator::stop_all_features(&mut unicorn_handle, &mut features)
}
