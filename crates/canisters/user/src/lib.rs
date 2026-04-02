mod api;
mod error;
mod schema;
#[cfg_attr(not(test), expect(dead_code))]
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

ic_cdk::export_candid!();
