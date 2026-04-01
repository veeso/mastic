use did::user::UserInstallArgs;

#[ic_cdk::init]
fn init(_args: UserInstallArgs) {}

ic_cdk::export_candid!();
