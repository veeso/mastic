mod adapters;
mod api;
mod domain;
mod error;
mod inspect;
mod repository;
mod schema;
mod settings;
#[cfg(test)]
mod test_utils;

use candid::Principal;
use did::directory::{
    DeleteProfileResponse, DirectoryInstallArgs, GetUserArgs, GetUserResponse,
    RetryDeleteProfileResponse, RetrySignUpResponse, SearchProfilesArgs, SearchProfilesResponse,
    SignUpRequest, SignUpResponse, UserCanisterResponse, WhoAmIResponse,
};

#[ic_cdk::init]
fn init(args: DirectoryInstallArgs) {
    api::init(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: DirectoryInstallArgs) {
    api::post_upgrade(args);
}

#[ic_cdk::inspect_message]
fn inspect_message() {
    inspect::inspect();
}

#[ic_cdk::update]
fn delete_profile() -> DeleteProfileResponse {
    api::delete_profile()
}

#[ic_cdk::query]
fn get_user(args: GetUserArgs) -> GetUserResponse {
    api::get_user(args)
}

#[ic_cdk::update]
fn retry_sign_up() -> RetrySignUpResponse {
    api::retry_sign_up()
}

#[ic_cdk::update]
fn retry_delete_profile() -> RetryDeleteProfileResponse {
    api::retry_delete_profile()
}

#[ic_cdk::query]
fn search_profiles(query: SearchProfilesArgs) -> SearchProfilesResponse {
    api::search_profiles(query)
}

#[ic_cdk::update]
fn sign_up(request: SignUpRequest) -> SignUpResponse {
    api::sign_up(request)
}

#[ic_cdk::query]
fn user_canister(principal: Option<Principal>) -> UserCanisterResponse {
    api::user_canister(principal)
}

#[ic_cdk::query]
fn whoami() -> WhoAmIResponse {
    api::whoami()
}

ic_cdk::export_candid!();
