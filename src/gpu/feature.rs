use std::ptr::null_mut;
use unicorn::{RegisterARM, UnicornHandle};
use std::any::Any;
use std::mem::size_of;
use unicorn::ffi::uc_hook;
use crate::features::EmulatorFeature;
use crate::gpu::base::{GPUBackend, Vert};

/// Video output
///
/// This feature provides syscalls to draw 3D graphics on a 800x600 screen
pub struct GPUFeature {
    hook: uc_hook,
    backend: Box<dyn GPUBackend>,
}


impl GPUFeature {
    pub fn new<Backend: 'static + GPUBackend>(constructor: fn(label: &str, width: usize, height: usize) -> Backend) -> Box<GPUFeature> {
        let backend = Box::new(constructor("ARMchine", 800, 600));
        Box::new(GPUFeature {
            hook: null_mut(),
            backend,
        })
    }

    pub fn is_open(&self) -> bool {
        self.backend.is_open()
    }

    pub fn update(&mut self) {
        self.backend.update();
    }

    unsafe fn copy_vertex_from_memory(gpuptr: *mut GPUFeature, emu: UnicornHandle) {
        let addr = emu.reg_read_i32(RegisterARM::R1 as i32).unwrap();
        let vert_count = emu.reg_read_i32(RegisterARM::R2 as i32).unwrap();

        let vertex = emu.mem_read_as_vec(addr as u64, (vert_count as usize * size_of::<Vert>()) as usize).unwrap();
        let mut vx = Vec::<Vert>::new();

        for i in 0..vert_count {
            let mut v: [f32; 8] = [0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32, 0f32];
            for f in 0..8 {
                let v_idx = (i * 8 * 4) as usize;
                let f_idx = (f * 4) as usize;
                let float: [u8; 4] = vertex.as_slice()[v_idx + f_idx..v_idx + f_idx + 4].try_into().expect("slice of incorrect length");
                v[f] = f32::from_le_bytes(float);
            }
            vx.push(v);
        }

        // println!("Vertex count: {}", vx.len());

        let index_addr = emu.reg_read_i32(RegisterARM::R3 as i32).unwrap();
        let index_count = emu.reg_read_i32(RegisterARM::R4 as i32).unwrap() as usize;

        let index = emu.mem_read_as_vec(index_addr as u64, index_count as usize * size_of::<u16>()).unwrap();

        let index: Vec<u16> = (0..index_count).map(|i| u16::from_le_bytes([index[i*2], index[i*2+1]])).collect();

        (*gpuptr).backend.load_vertices(vx, index);
    }
}

/// | Syscall | Parameters | Description |
/// | ------- | ---------- | ----------- |
/// | 0x160 | Vertex*: address of vertex list to copy, size_t: vertex count, uint16_t*: address of indexes to copy, size_t: index count | Copies vertices from array into the drawing backend |
/// | 0x161 | None | Currently does nothing. It's supposed to use as a sort of vsync without having to copy new vertices |
impl EmulatorFeature for GPUFeature {
    fn init(&mut self, emulator: &mut UnicornHandle) -> Result<(), String> {
        let gpuptr: *mut GPUFeature = self;

        let hook = emulator.add_intr_hook(move |emu, _syscall| unsafe {
            let syscall = emu.reg_read_i32(RegisterARM::R7 as i32).unwrap();

            match syscall {
                0x160 => {
                    Self::copy_vertex_from_memory(gpuptr, emu);
                }
                0x161 => {}
                _ => {}
            }
        });

        match hook {
            Ok(hook) => {
                self.hook = hook;
                Ok(())
            }
            Err(err) => {
                Err(format!("{:?}", err))
            }
        }
    }

    fn stop(&mut self, _emulator: &mut UnicornHandle) -> Result<(), String> {
        _emulator.remove_hook(self.hook).unwrap();
        self.hook = null_mut();
        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn name(&self) -> String {
        String::from("GPUFeature")
    }
}
