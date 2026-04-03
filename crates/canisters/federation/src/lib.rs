use did::federation::FederationInstallArgs;

mod memory;

#[ic_cdk::init]
fn init(_args: FederationInstallArgs) {
    ic_utils::log!("Federation canister initialized");
}

ic_cdk::export_candid!();
