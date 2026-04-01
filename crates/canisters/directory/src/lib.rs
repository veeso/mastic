use did::directory::DirectoryInstallArgs;

#[ic_cdk::init]
fn init(_args: DirectoryInstallArgs) {}

ic_cdk::export_candid!();
