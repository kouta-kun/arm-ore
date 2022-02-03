use crate::gpu::base::GPUBackend;

mod base;

#[cfg(feature = "euc-backend")]
pub mod euc;


pub mod feature;

#[cfg(any(feature="euc-backend"))]
pub(crate) fn create_feature(preference: Option<&str>) -> Box<feature::GPUFeature> {
    if preference == None {
        #[cfg(feature = "euc-backend")] {
            feature::GPUFeature::new(euc::EucGPUBackend::new)
        }
    } else if preference.unwrap().eq("euc") {
        #[cfg(feature = "euc-backend")] {
            feature::GPUFeature::new(euc::EucGPUBackend::new)
        }
    } else {
        panic!("Unknown preference! {}", preference.unwrap())
    }
}

