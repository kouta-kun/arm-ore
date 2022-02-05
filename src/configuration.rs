use crate::{console, dynmemory, EmulatorFeature, filesystem, gpu};
use clap::Parser;
use clap;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Arguments {
    #[clap(long)]
    pub iso: String,

    #[clap(short, long)]
    pub debug: bool,
}

pub fn get_features(args: &Arguments, mem_sz: u64) -> Vec<Box<dyn EmulatorFeature>> {
    let mut features = Vec::<Box<dyn EmulatorFeature>>::new();
    features.push(Box::new(console::ConsoleIO::new()));
    features.push(Box::new(filesystem::EmulatorDrive::new(String::from(&args.iso))));
    features.push(Box::new(dynmemory::DynamicMemoryAllocations::new(mem_sz)));
    #[cfg(feature = "gpu-feature")]
        features.push(gpu::create_feature(None));
    features
}
