mod adapters;
mod api;
mod domain;
mod error;
mod inspect;
mod schema;
mod settings;
#[cfg(test)]
mod test_utils;

use did::user::UserInstallArgs;

#[ic_cdk::init]
fn init(args: UserInstallArgs) {
    api::init(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: UserInstallArgs) {
    api::post_upgrade(args);
}

#[ic_cdk::inspect_message]
fn inspect_message() {
    inspect::inspect();
}

ic_cdk::export_candid!();
