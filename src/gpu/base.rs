use std::any::Any;
use std::ptr::{null, null_mut};
use std::time::Duration;

use unicorn::ffi::uc_hook;
use unicorn::{RegisterARM, UnicornHandle};
use crate::features::EmulatorFeature;

pub type Vert = [f32; 8];

pub trait GPUBackend {
    fn update(&mut self);
    fn load_vertices(&mut self, vertices: Vec<Vert>, indexes: Vec<u16>);
    fn is_open(&self) -> bool;
}
