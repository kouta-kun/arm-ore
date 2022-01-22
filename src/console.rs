use std::ptr::null_mut;
use unicorn::UnicornHandle;
use libc::size_t;
use unicorn::unicorn_const::{HookType, Permission};
use std::any::Any;
use unicorn::ffi::uc_hook;
use crate::EmulatorFeature;

pub struct ConsoleIO {
    hook: uc_hook,
}

impl ConsoleIO {
    pub(crate) fn new() -> ConsoleIO {
        ConsoleIO { hook: null_mut() }
    }
}

impl EmulatorFeature for ConsoleIO {
    fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
        emulator.mem_map(0xFF000, 4096 as size_t, Permission::ALL).unwrap();
        match emulator.add_mem_hook(HookType::MEM_ALL, 0xFF000, 0xFF001, |_emu, _memtype, _idx, _size, value| {
            print!("{}", (value as u8) as char);
        }) {
            Ok(r) => {
                self.hook = r;
                Ok(())
            }
            Err(err) => { Err(format!("{:?}", err)) }
        }
    }

    fn stop(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
        emulator.mem_unmap(0xFF000, 4096).unwrap();
        let r = emulator.remove_hook(self.hook).map_err(|e| format!("{:?}", e));
        self.hook = null_mut();
        r
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> String {
        "ConsoleIO".parse().unwrap()
    }
}
