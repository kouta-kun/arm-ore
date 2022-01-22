use std::any::Any;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::ops::Index;
use std::path::Path;
use std::ptr::null_mut;
use std::rc::Rc;
use capstone::arch::tms320c64x::Tms320c64xMemDisplayType::Register;
use unicorn::{RegisterARM, UnicornHandle};
use unicorn::ffi::uc_hook;
use crate::emulator::{EmulatorFeature};
use iso9660::{ISO9660, DirectoryEntry, ISODirectory};

pub struct Drive {
    drive_archive: ISO9660<File>,
    file_listing: Vec<String>,
}

impl Drive {
    pub fn new(path: &Path) -> Drive {
        let drive_file = File::open(path).expect("Could not open tar file.");
        let archive = ISO9660::new(drive_file).unwrap();
        let listing = Self::file_listing(&archive.root).unwrap();
        let drive = Drive {
            drive_archive: archive,
            file_listing: listing,
        };
        drive
    }

    pub fn get_listing(&self) -> &Vec<String> {
        &self.file_listing
    }

    fn file_listing(archive: &ISODirectory<File>) -> Result<Vec<String>, &str> {
        let mut vec = Vec::new();
        let directory = archive;
        Self::traverse_directory(&mut vec, directory, directory.identifier.to_string());
        Ok(vec)
    }

    fn traverse_directory(vec: &mut Vec<String>, directory: &ISODirectory<File>, abs_dir: String) {
        for x in directory.contents() {
            match x.unwrap() {
                DirectoryEntry::Directory(dir) => {
                    if dir.identifier != "." && dir.identifier != ".." {
                        Self::traverse_directory(vec, &dir, format!("{}/{}", abs_dir, dir.identifier));
                    }
                }
                DirectoryEntry::File(file) => {
                    vec.push(format!("{}/{}", abs_dir, file.identifier));
                }
            }
        };
    }

    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, String> {
        if let Ok(Some(DirectoryEntry::File(file))) = self.drive_archive.open(path) {
            let bytes = file.read().bytes().into_iter();
            let x: Result<Vec<u8>, _> = bytes.collect();
            x.map_err(|e| format!("{}", e))
        } else {
            Err(format!("path was not a file: {}", path))
        }
    }

    pub fn read_file_region(&self, path: &str, index: u32, count: u32) -> Result<Vec<u8>, String> {
        if let Ok(Some(DirectoryEntry::File(file))) = self.drive_archive.open(path) {
            let bytes = file.read().bytes().skip(index as usize).take(count as usize).into_iter();
            let x: Result<Vec<u8>, _> = bytes.collect();
            x.map_err(|e| format!("{}", e))
        } else {
            Err(format!("path was not a file: {}", path))
        }
    }

    pub fn file_size(&self, path: &str) -> Result<u32, String> {
        if let Ok(Some(DirectoryEntry::File(file))) = self.drive_archive.open(path) {
            let bytes = file.size();
            Ok(bytes)
        } else {
            Err(format!("path was not a file: {}", path))
        }
    }
}

pub struct EmulatorDrive {
    path: String,
    hook: uc_hook,
}

impl EmulatorDrive {
    pub fn new(path: String) -> EmulatorDrive {
        EmulatorDrive {
            path,
            hook: null_mut(),
        }
    }

    fn file_count(drive: &mut Drive, em: &mut UnicornHandle) {
        let length = drive.get_listing().len() as u32;
        em.reg_write(RegisterARM::R0 as i32, length as u64).unwrap();
    }

    fn filename_len(drive: &mut Drive, em: &mut UnicornHandle) {
        let index = em.reg_read_i32(RegisterARM::R1 as i32).unwrap() as usize;

        let listing = drive.get_listing();
        let files = listing.iter().as_slice();
        let filelen = files[index].len();

        em.reg_write(RegisterARM::R0 as i32, filelen as u64).unwrap();
    }

    fn filename_index(drive: &mut Drive, em: &mut UnicornHandle) {
        let index = em.reg_read_i32(RegisterARM::R1 as i32).unwrap() as usize;

        let listing = drive.get_listing();
        let files = listing.iter().as_slice();
        let file = &files[index];

        let strindex = em.reg_read_i32(RegisterARM::R2 as i32).unwrap() as usize;
        let filename = file.to_string();
        let character = filename.as_bytes()[strindex];
        em.reg_write(RegisterARM::R0 as i32, character.into()).unwrap();
    }

    fn file_size(drive: &mut Drive, em: &mut UnicornHandle) {
        let filepath = Self::read_string_from_r1(em);
        let filesize = drive.file_size(filepath.as_str()).unwrap();
        em.reg_write(RegisterARM::R0 as i32, filesize as u64).unwrap();
    }

    fn read_string_from_r1(em: &mut UnicornHandle) -> String {
        let mut string_address = em.reg_read_i32(RegisterARM::R1 as i32).unwrap();
        let mut string = Vec::new();
        while *(string.last().or(Some(&1u8)).unwrap()) != 0u8 {
            string.append(&mut em.mem_read_as_vec(
                string_address as u64, 1).unwrap());
            string_address += 1;
        }
        string.pop();
        let string = String::from_utf8(string).unwrap();
        string
    }

    fn read_file(drive: &mut Drive, mut em: &mut UnicornHandle) {
        let filepath = Self::read_string_from_r1(&mut em);

        let file_offset = em.reg_read_i32(RegisterARM::R2 as i32).unwrap();
        let file_size = em.reg_read_i32(RegisterARM::R3 as i32).unwrap();

        let file_bytes = drive.read_file_region(
            filepath.as_str(), file_offset as u32, file_size as u32).unwrap();

        let output_addr = em.reg_read_i32(RegisterARM::R4 as i32).unwrap();

        em.mem_write(output_addr as u64, file_bytes.as_slice()).unwrap();
    }
}

impl EmulatorFeature for EmulatorDrive {
    fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
        let mut drive = Drive::new(self.path.as_ref());
        let syscall = move |mut em: UnicornHandle, _syscall: u32| {
            let syscall = em.reg_read_i32(RegisterARM::R7 as i32).unwrap();
            match syscall {
                0 => {
                    Self::file_count(&mut drive, &mut em);
                }
                1 => {
                    Self::filename_len(&mut drive, &mut em);
                }
                2 => {
                    Self::filename_index(&mut drive, &mut em);
                }
                3 => {
                    Self::file_size(&mut drive, &mut em);
                }
                4 => {
                    Self::read_file(&mut drive, &mut em);
                }
                _ => {}
            }
        };
        match emulator.add_intr_hook(syscall) {
            Ok(hook) => {
                self.hook = hook;
                Ok(())
            }
            Err(err) => {
                Err(format!("{:?}", err))
            }
        }
    }

    fn stop(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
        emulator.remove_hook(self.hook).map_err(|e| format!("{:?}", e))
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> String {
        "EmulatorDrive".parse().unwrap()
    }
}