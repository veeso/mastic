use std::path::Path;

pub enum Canister {
    Directory,
    Federation,
}

impl Canister {
    pub fn as_path(&self) -> &'static Path {
        match self {
            Canister::Directory => Path::new("../.artifact/directory.wasm.gz"),
            Canister::Federation => Path::new("../.artifact/federation.wasm.gz"),
        }
    }
}
