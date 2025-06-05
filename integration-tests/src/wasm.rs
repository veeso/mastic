use std::path::Path;

pub enum Canister {
    Mastic,
}

impl Canister {
    pub fn as_path(&self) -> &'static Path {
        match self {
            Canister::Mastic => Path::new("../.artifact/mastic.wasm.gz"),
        }
    }
}
