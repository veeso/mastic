mod adapters;
mod api;
mod domain;
mod error;

mod schema;
mod settings;
#[cfg(test)]
mod test_utils;

use did::directory::{DirectoryInstallArgs, RetrySignUpResponse, SignUpRequest, SignUpResponse};

#[ic_cdk::init]
fn init(args: DirectoryInstallArgs) {
    api::init(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: DirectoryInstallArgs) {
    api::post_upgrade(args);
}

#[ic_cdk::update]
fn sign_up(request: SignUpRequest) -> SignUpResponse {
    api::sign_up(request)
}

#[ic_cdk::update]
fn retry_sign_up() -> RetrySignUpResponse {
    api::retry_sign_up()
}

ic_cdk::export_candid!();
