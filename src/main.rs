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

mod emulator;
mod filesystem;
mod features;
mod console;
mod dynmemory;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Arguments {
    #[clap(long)]
    iso: String,
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

    let mut console_io = console::ConsoleIO::new();
    console_io.init(&mut unicorn_handle).unwrap();

    let mut file_io = filesystem::EmulatorDrive::new(String::from(args.iso));
    file_io.init(&mut unicorn_handle).unwrap();

    let mut dynmem = dynmemory::DynamicMemoryAllocations::new(mem_sz);
    dynmem.init(&mut unicorn_handle).unwrap();

    unicorn_handle.add_intr_hook(|mut _emu, _syscall| _emu.emu_stop().unwrap()).unwrap();


    {
        unicorn_handle.reg_write(RegisterARM::SP as i32, 0x10000).unwrap();
        unicorn_handle.reg_write(RegisterARM::PC as i32, main_idx).unwrap();

        loop {
            let pc = unicorn_handle.reg_read(RegisterARM::PC as i32).unwrap();

            if let Err(_e) = unicorn_handle.emu_start(pc, mem_sz, 0, 0) {
                // print_disassembly(&mut unicorn_handle, mem_sz, main_idx, e)
                break;
            }
        }
    }
    console_io.stop(&mut unicorn_handle).unwrap();
    file_io.stop(&mut unicorn_handle).unwrap();
    dynmem.stop(&mut unicorn_handle).unwrap();
}

fn print_disassembly(unicorn_handle: &mut UnicornHandle, mem_sz: u64, main_idx: u64, e: uc_error) {
    let pc = unicorn_handle.reg_read_i32(RegisterARM::PC as i32).unwrap();
    println!("failed because {:?}", e);
    println!("failed at {:#x}", pc);
    let capstone = capstone::Capstone::new().arm().mode(ArchMode::Arm).build().unwrap();
    let instructions = capstone.disasm_all(unicorn_handle.mem_read_as_vec(main_idx, mem_sz as usize).unwrap().as_slice(), 0).unwrap();
    for i in instructions.iter() {
        println!("{:#x}: {} {}", i.address(), i.mnemonic().unwrap(), i.op_str().unwrap());
    }
}
