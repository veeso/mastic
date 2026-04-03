use did::federation::{FederationInstallArgs, SendActivityArgs, SendActivityResponse};

mod api;
#[allow(dead_code, reason = "will be used by WI-0.10 routing logic")]
mod conversions;
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

#[ic_cdk::update]
fn send_activity(_args: SendActivityArgs) -> SendActivityResponse {
    // TODO: no-op for now. Implement in WI-0.10
    SendActivityResponse::Ok
}

ic_cdk::export_candid!();
