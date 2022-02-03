use crate::gpu::base::GPUBackend;

mod base;

#[cfg(feature = "euc-backend")]
pub mod euc;

#[cfg(feature = "wgpu-backend")]
pub mod wgpu;

pub mod feature;

#[cfg(feature = "gpu-feature")]
pub(crate) fn create_feature(preference: Option<&str>) -> Box<feature::GPUFeature> {
    if preference == None {
        #[cfg(feature = "euc-backend")] {
            return feature::GPUFeature::new(euc::EucGPUBackend::new);
        }
        #[cfg(feature = "wgpu-backend")] {
            return feature::GPUFeature::new(wgpu::WgpuBackend::new);
        }
        panic!("No backends available!");
    } else if preference.unwrap().eq("euc") {
        #[cfg(feature = "euc-backend")] {
            return feature::GPUFeature::new(euc::EucGPUBackend::new);
        }
        panic!("Requested euc which is unavailable!");
    } else if preference.unwrap().eq("wgpu") {
        #[cfg(feature = "wgpu-backend")] {
            return feature::GPUFeature::new(wgpu::WgpuBackend::new);
        }
        panic!("Requested wgpu which is unavailable!");
    }
    else {
        panic!("Unknown or unsupported preference! {}", preference.unwrap())
    }
}

