use std::alloc::alloc;
use std::any::Any;
use std::ptr::null_mut;
use libc::size_t;
use unicorn::{RegisterARM, UnicornHandle};
use unicorn::ffi::uc_hook;
use unicorn::unicorn_const::Permission;
use crate::emulator::EmulatorFeature;

pub struct DynamicMemoryAllocations {
    memory_base: u64,
    hook: uc_hook,
}

impl DynamicMemoryAllocations {
    pub fn new(mem_sz: u64) -> DynamicMemoryAllocations {
        let mut align = mem_sz >> 22;
        if (align << 22) < mem_sz {
            align += 1;
        }
        let membase = align << 22;
        DynamicMemoryAllocations {
            memory_base: membase,
            hook: null_mut(),
        }
    }
}

fn align(value: u32, align: u32) -> u32 {
    let mut b = value >> align;
    if (b << align) < value {
        b += 1;
    }
    b << align
}

impl EmulatorFeature for DynamicMemoryAllocations {
    fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
        let mut vector = Box::new(Vec::<(u32, u32)>::new());
        let membase = self.memory_base;
        let hook = emulator.add_intr_hook(move |mut em, _syscall| {
            let syscall = em.reg_read_i32(RegisterARM::R7 as i32).unwrap();
            if syscall == 0x60 {
                let mut allocation_size = em.reg_read_i32(RegisterARM::R1 as i32).unwrap();
                allocation_size = align(allocation_size as u32, 12) as i32;

                let mut base =
                    if vector.len() > 0 {
                        let (b, sz) = vector.iter().max_by_key(|(base, _sz)| base).unwrap();
                        (b + sz) as u64
                    } else { membase };
                base = align(base as u32, 12) as u64;

                em.mem_map(base, allocation_size as size_t, Permission::ALL).unwrap();
                vector.push((base as u32, allocation_size as u32));
                em.reg_write(RegisterARM::R0 as i32, base).unwrap();
                // println!("{:#x} -> {:#x}", base, base+allocation_size as u64);
            }
        });
        self.hook = hook.unwrap();
        Ok(())
    }

    fn stop(&mut self, _emulator: &mut UnicornHandle) -> Result<(), String> {
        todo!()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> String {
        "DynamicMemory".to_string()
    }
}