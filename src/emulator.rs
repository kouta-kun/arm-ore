use std::any::Any;
use unicorn::{RegisterARM, Unicorn, UnicornHandle};
use libc::size_t;
use unicorn::unicorn_const::{HookType, Mode, Permission};
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
use crate::filesystem::Drive;

pub trait EmulatorFeature {
    fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String>;
    fn stop(&mut self, emulator: &mut UnicornHandle) -> Result<(), String>;
    fn as_any(&mut self) -> &mut dyn Any;
    fn name(&self) -> String;
}

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
