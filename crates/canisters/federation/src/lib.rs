use did::federation::FederationInstallArgs;

mod api;
mod memory;
mod settings;

#[cfg(test)]
mod test_utils;

#[ic_cdk::init]
fn init(args: FederationInstallArgs) {
    api::init(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: FederationInstallArgs) {
    api::post_upgrade(args);
}

ic_cdk::export_candid!();
