use std::any::Any;
use unicorn::{RegisterARM, Unicorn, UnicornHandle};
use libc::size_t;
use unicorn::unicorn_const::{HookType, Mode, Permission, uc_error};
use std::ptr::null_mut;
use unicorn::unicorn_const::Arch::ARM;
use std::fs;
use xmas_elf::{ElfFile, header, sections};
use std::cmp::max;
use xmas_elf::symbol_table::{Entry, Entry32};
use unicorn::ffi::uc_hook;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use xmas_elf::dynamic::Tag::Hash;
use capstone::arch::arm::ArchMode;
use capstone::arch::BuildsCapstone;
use crate::filesystem::Drive;

pub fn create_emulator() -> Unicorn {
    let cpu_mode = Mode::ARM946 | Mode::LITTLE_ENDIAN;
    let unicorn: Unicorn = Unicorn::new(ARM, cpu_mode)
        .expect("Couldn't create ARM emu");
    unicorn
}

pub fn load_executable(emu: &mut UnicornHandle, file_io: &Drive) -> Result<(u64, u64), ()> {
    let file_content: Vec<u8> = file_io.read_file("./main.elf").unwrap();
    let binary_blob: &[u8] = file_content.borrow();

    let elf_file = ElfFile::new(binary_blob).unwrap();

    header::sanity_check(&elf_file).unwrap();


    let mut mem_sz = 0;

    let mut prog_iter = elf_file.program_iter();
    prog_iter.next();
    for ph in prog_iter {
        mem_sz = max(mem_sz, ph.virtual_addr() + ph.mem_size());
    }

    let mut align = mem_sz >> 12;
    align = if (align << 12) < mem_sz { (align + 1) << 12 } else { align << 12 };

    emu.mem_map(0 as u64, align as libc::size_t, Permission::ALL).expect("couldn't map memory");

    let mut prog_iter = elf_file.program_iter();
    prog_iter.next();

    for ph in prog_iter {
        let header_data = &binary_blob[
            ph.offset() as usize..(ph.offset() + ph.file_size()) as usize];

        emu.mem_write(ph.virtual_addr(), header_data).unwrap();
    }

    let symbol_header = elf_file.find_section_by_name(".symtab").unwrap();
    let symbol_data = symbol_header.get_data(&elf_file).unwrap();

    let main_idx = if let sections::SectionData::SymbolTable32(data) = symbol_data {
        let main_entry: &Entry32 = data.iter().find(|e| e.get_name(&elf_file).unwrap().eq("_start")).unwrap();

        main_entry.value()
    } else {
        return Err(());
    };
    Ok((mem_sz, main_idx))
}

pub fn print_disassembly(unicorn_handle: &mut UnicornHandle, mem_sz: u64, main_idx: u64, e: Result<(), uc_error>) {
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
