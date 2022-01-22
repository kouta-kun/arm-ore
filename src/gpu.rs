use std::any::Any;
use std::ptr::{null, null_mut};
use std::time::Duration;
use euc::{DepthStrategy, Interpolate, Pipeline, Rasterizer, rasterizer, Target};
use euc::buffer::Buffer2d;
use minifb::{Window, WindowOptions};
use unicorn::ffi::uc_hook;
use unicorn::{RegisterARM, UnicornHandle};
use crate::EmulatorFeature;

pub struct Triangle;

#[derive(Copy, Clone)]
pub struct PixelIntermediate {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl PixelIntermediate {
    pub fn as_pixel(&self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        let a = (self.a * 255.0) as u32;
        // println!("Pixel: {} {} {} {}", r, g, b, a);
        let ret = (r << 16)
            | (g << 8)
            | (b << 0)
            | (a << 24);
        return ret;
    }
}


impl Interpolate for PixelIntermediate {
    fn lerp2(a: Self, b: Self, x: f32, y: f32) -> Self {
        PixelIntermediate {
            r: f32::lerp2(a.r,
                          b.r,
                          x,
                          y),
            b: f32::lerp2(a.g,
                          b.g,
                          x,
                          y),
            g: f32::lerp2(a.b,
                          b.b,
                          x,
                          y),
            a: f32::lerp2(a.a,
                          b.a,
                          x,
                          y),
        }
    }

    fn lerp3(a: Self, b: Self, c: Self, x: f32, y: f32, z: f32) -> Self {
        PixelIntermediate {
            r: f32::lerp3(a.r,
                          b.r,
                          c.r,
                          x,
                          y
                          , z),
            b: f32::lerp3(a.g,
                          b.g,
                          c.g,
                          x,
                          y,
                          z),
            g: f32::lerp3(a.b,
                          b.b,
                          c.b,
                          x,
                          y,
                          z, ),
            a: f32::lerp3(a.a,
                          b.a,
                          c.a,
                          x,
                          y,
                          z),
        }
    }
}

impl Pipeline for Triangle {
    type Vertex = [f32; 8];
    type VsOut = PixelIntermediate;
    type Pixel = u32;

    fn vert(&self, vertex: &Self::Vertex) -> ([f32; 4], Self::VsOut) {
        let (vertex_out, frag_out) = vertex.split_at(4);
        // println!("Vertex: {:?}, Fragment: {:?}", vertex_out, frag_out);
        (vertex_out.try_into().expect("slice not 4 long"), PixelIntermediate {
            r: frag_out[0],
            g: frag_out[1],
            b: frag_out[2],
            a: frag_out[3],
        })
    }

    fn frag(&self, vs_out: &Self::VsOut) -> Self::Pixel {
        let i = vs_out.as_pixel();
        // println!("Pixel: {:#x}", i);
        i
    }
}

pub struct GPUFeature {
    pub window: minifb::Window,
    pub triangles: Option<Vec<<Triangle as Pipeline>::Vertex>>,
    pub buffer: Buffer2d<u32>,
    hook: uc_hook,
}

impl GPUFeature {
    pub fn new() -> GPUFeature {
        let mut window = Window::new("ARMChine", 800, 600, WindowOptions::default()).unwrap();
        // window.limit_update_rate(Some(Duration::from_millis((1000.0 / 60.0) as u64)));
        GPUFeature {
            window,
            triangles: None,
            hook: null_mut(),
            buffer: Buffer2d::new([800, 600], 0),
        }
    }

    fn draw_vertex(&mut self) {
        self.buffer.as_mut().fill(0);
        if let Some(vx) = &self.triangles {
            Triangle.draw::<rasterizer::Triangles<(f32, )>, _>(vx,
                                                               &mut self.buffer,
                                                               None);
            self.window.update_with_buffer(self.buffer.as_ref(), 800, 600).unwrap();
            self.triangles = None;
        }
    }

    pub fn update(&mut self) {
        self.draw_vertex();
    }

    unsafe fn copy_vertex_from_memory(gpuptr: *mut GPUFeature, emu: UnicornHandle) {
        let addr = emu.reg_read_i32(RegisterARM::R1 as i32).unwrap();
        let vert_count = emu.reg_read_i32(RegisterARM::R2 as i32).unwrap();

        let vertex = emu.mem_read_as_vec(addr as u64, (vert_count * 8 * 4) as usize).unwrap();
        let mut vx = Vec::<<Triangle as Pipeline>::Vertex>::new();

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

        (*gpuptr).triangles = Some(vx);
    }
}

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
        todo!()
    }

    fn name(&self) -> String {
        todo!()
    }
}