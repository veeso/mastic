mod api;
mod error;
#[cfg_attr(not(test), expect(dead_code))]
mod moderators;
mod schema;
#[cfg_attr(not(test), expect(dead_code))]
mod settings;
#[cfg(test)]
mod test_utils;

use did::directory::DirectoryInstallArgs;

#[ic_cdk::init]
fn init(args: DirectoryInstallArgs) {
    api::init(args);
}

#[ic_cdk::post_upgrade]
fn post_upgrade(args: DirectoryInstallArgs) {
    api::post_upgrade(args);
}

ic_cdk::export_candid!();
