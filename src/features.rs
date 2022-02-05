use unicorn::UnicornHandle;
use std::any::Any;

pub trait EmulatorFeature {
    fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String>;
    fn stop(&mut self, emulator: &mut UnicornHandle) -> Result<(), String>;
    fn as_any(&mut self) -> &mut dyn Any;
    fn name(&self) -> String;
}
