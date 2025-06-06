use std::path::Path;

pub enum Canister {
    CyclesMinting,
    Directory,
    Federation,
    IcpIndex,
    IcpLedger,
    OrbitStation,
    OrbitUpgrader,
    User,
}

impl Canister {
    pub fn as_path(&self) -> &'static Path {
        match self {
            Canister::CyclesMinting => Path::new("../.artifact/cycles-minting-canister.wasm.gz"),
            Canister::Directory => Path::new("../.artifact/directory.wasm.gz"),
            Canister::IcpIndex => Path::new("../.artifact/icp-index.wasm.gz"),
            Canister::IcpLedger => Path::new("../.artifact/icp-ledger.wasm.gz"),
            Canister::Federation => Path::new("../.artifact/federation.wasm.gz"),
            Canister::OrbitStation => Path::new("../.artifact/orbit-station.wasm.gz"),
            Canister::OrbitUpgrader => Path::new("../.artifact/orbit-upgrader.wasm.gz"),
            Canister::User => Path::new("../.artifact/user.wasm.gz"),
        }
    }
}
