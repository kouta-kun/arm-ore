use std::any::Any;
use std::collections::VecDeque;
use unicorn::ffi::uc_hook;
use unicorn::{RegisterARM, UnicornHandle};
use crate::features::EmulatorFeature;

#[derive(Debug, Copy, Clone)]
pub enum InputButton {
    DirectionBase = 0x20,
    Up,
    Down,
    Left,
    Right,
    ButtonBase = 0xB0,
    A,
    B,
    C,
    D,
}

struct InputFeature {
    input_queue: VecDeque<InputButton>,
    hook: Option<uc_hook>
}
//
// impl EmulatorFeature for InputFeature {
//     fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
//         let feature: *mut InputFeature = self;
//         let syscall = move |mut em: UnicornHandle, _syscall: u32| {
//             let syscall = em.reg_read_i32(RegisterARM::R7 as i32).unwrap();
//
//         };
//     }
//
//     fn stop(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
//         todo!()
//     }
//
//     fn as_any(&mut self) -> &mut dyn Any {
//         self
//     }
//
//     fn name(&self) -> String {
//         String::from("InputFeature")
//     }
// }