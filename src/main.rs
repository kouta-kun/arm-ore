#![allow(unused_imports)]

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
use crate::emulator::{EmulatorFeature};
use clap::{Parser};
use clap;
use crate::filesystem::Drive;
use crate::gpu::feature::GPUFeature;

mod emulator;
mod filesystem;
mod features;
mod console;
mod dynmemory;
mod gpu;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Arguments {
    #[clap(long)]
    iso: String,

    #[clap(short, long)]
    debug: bool,
}

fn get_features(args: &Arguments, mem_sz: u64) -> Vec<Box<dyn EmulatorFeature>> {
    let mut features = Vec::<Box<dyn EmulatorFeature>>::new();
    features.push(Box::new(console::ConsoleIO::new()));
    features.push(Box::new(filesystem::EmulatorDrive::new(String::from(&args.iso))));
    features.push(Box::new(dynmemory::DynamicMemoryAllocations::new(mem_sz)));
    #[cfg(feature = "gpu-feature")]
        features.push(gpu::create_feature(None));
    features
}

fn main() {
    let args: Arguments = Arguments::parse();

    let mut unicorn = emulator::create_emulator();
    let mut unicorn_handle: UnicornHandle = Unicorn::borrow(
        &mut unicorn);


    let (mem_sz, main_idx) = {
        let drive = Drive::new(args.iso.as_ref());

        emulator::load_executable(&mut unicorn_handle, &drive).unwrap()
    };
    let mut features = get_features(&args, mem_sz);

    for feat in &mut features {
        feat.init(&mut unicorn_handle).unwrap();
    }




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
                print_disassembly(&mut unicorn_handle, mem_sz, main_idx, e);
            }
            if e.is_err() {
                break;
            }

            let dt = t2.duration_since(t1).as_millis();
            print!("Execution time: {}; ", dt);

            #[cfg(feature = "gpu-feature")] {
                for feat in &mut features {
                    if feat.name().eq("GPUFeature") {
                        let feat = feat.as_any().downcast_mut::<GPUFeature>().unwrap();
                        let t1 = std::time::Instant::now();
                        feat.update();
                        must_loop = feat.is_open();
                        let t2 = std::time::Instant::now();
                        let dt = t2.duration_since(t1).as_millis();

                        print!("Rendering and update time: {};", dt);
                    }
                }
            }
            print!("\r");
            std::io::stdout().flush().unwrap();
        }
    }


    for mut feat in features {
        feat.stop(&mut unicorn_handle).unwrap();
    }


}

fn print_disassembly(unicorn_handle: &mut UnicornHandle, mem_sz: u64, main_idx: u64, e: Result<(), uc_error>) {
    let pc = unicorn_handle.reg_read_i32(RegisterARM::PC as i32).unwrap();
    if let Err(error) = e {
        println!("failed because {:?}", error);
        println!("failed at {:#x}", pc);
    }
    let capstone = capstone::Capstone::new().arm().mode(ArchMode::Arm).build().unwrap();
    let instructions = capstone.disasm_all(unicorn_handle.mem_read_as_vec(main_idx, (mem_sz - main_idx) as usize).unwrap().as_slice(), pc as u64).unwrap();
    for i in instructions.iter() {
        println!("{:#x}: {} {}", i.address(), i.mnemonic().unwrap(), i.op_str().unwrap());
    }
}
