pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

tonic::include_proto!("qcdn.general");
tonic::include_proto!("qcdn.files");
