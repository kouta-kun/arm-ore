use unicorn::UnicornHandle;
use std::any::Any;

/// Modularized (and possibly optional) features of the emulator
///
/// Structs that implement this trait hook into the emulator and provide
/// features to the emulated system through syscalls or mapped memory regions
///
/// ```
/// feature.init(&mut emulator);
/// feature.stop(&mut emulator);
/// ```
///
/// NOTE: Each syscall (of any feature) also suspends emulator and updates
/// the graphics backend if one is enabled. This may lead to waiting for vsync
///
/// | Feature | Reserved Memory Blocks | Reserved Syscalls |
/// | ------- | ---------------------- | ----------------- |
/// | [crate::filesystem::EmulatorDrive] | None | 0x0 → 0x10 |
/// | [crate::dynmemory::DynamicMemoryAllocations] | None | 0x60 → 0x80 |
/// | [crate::gpu::feature::GPUFeature] | None | 0x160 → 0x180 |
/// | [crate::console::ConsoleIO] | 0xFF000 → 0x100000 | None |
pub trait EmulatorFeature {
    fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String>;
    fn stop(&mut self, emulator: &mut UnicornHandle) -> Result<(), String>;
    fn as_any(&mut self) -> &mut dyn Any;
    fn name(&self) -> String;
}
