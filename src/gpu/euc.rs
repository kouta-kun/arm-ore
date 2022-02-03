use euc::{Interpolate, Pipeline, rasterizer};
use euc::buffer::Buffer2d;
use minifb::{Window, WindowOptions};

use crate::gpu::base::{GPUBackend, Vert};

struct Triangle;

#[derive(Copy, Clone)]
struct PixelIntermediate {
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

pub struct EucGPUBackend {
    window: minifb::Window,
    triangles: Option<Vec<<Triangle as Pipeline>::Vertex>>,
    buffer: Buffer2d<u32>,
}

impl EucGPUBackend {
    pub fn new(window_label: &str, width: usize, height: usize) -> Self {
        let window = Window::new(window_label, width, height, WindowOptions::default()).unwrap();
        Self {
            buffer: Buffer2d::new([width, height], 0),
            window,
            triangles: None
        }
    }

    fn draw_vertex(&mut self) {
        if let Some(vx) = &self.triangles {
            self.buffer.as_mut().fill(0);
            Triangle.draw::<rasterizer::Triangles<(f32, )>, _>(vx,
                                                               &mut self.buffer,
                                                               None);
            self.window.update_with_buffer(self.buffer.as_ref(), 800, 600).unwrap();
            print!("Vertices: {}", vx.len());
            self.triangles = None;
        }
    }
}

#[cfg(feature="euc-backend")]
impl GPUBackend for EucGPUBackend {

    fn update(&mut self) {
        self.draw_vertex();
    }


    fn load_vertices(&mut self, vertices: Vec<Vert>) {
        self.triangles = Some(vertices);
    }

    fn is_open(&self) -> bool {
        self.window.is_open()
    }
}
