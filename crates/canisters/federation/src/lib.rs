use did::federation::{
    FederationInstallArgs, RegisterUserArgs, RegisterUserResponse, SendActivityArgs,
    SendActivityResponse,
};

mod adapters;
mod api;
#[allow(dead_code, reason = "will be used by future activity processing")]
mod conversions;
mod directory;
mod domain;
mod inspect;
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

#[ic_cdk::inspect_message]
fn inspect_message() {
    inspect::inspect();
}

#[ic_cdk::update]
fn register_user(args: RegisterUserArgs) -> RegisterUserResponse {
    api::register_user(args)
}

#[ic_cdk::update]
async fn send_activity(args: SendActivityArgs) -> SendActivityResponse {
    api::send_activity(args).await
}

ic_cdk::export_candid!();
