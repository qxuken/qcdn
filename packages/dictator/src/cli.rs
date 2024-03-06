use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    /// Path to data dir
    #[arg(short, long, env = "QCDN_STORAGE_DIR", default_value = "data")]
    pub(crate) data: PathBuf,

    /// Bind ip address
    #[clap(short, long, env = "QCDN_BIND", default_value_t = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080)
    )]
    pub(crate) bind: SocketAddr,

    #[clap(flatten)]
    pub(crate) instrumentation: qcdn_tracing_setup::instrumentation::Instrumentation,
}
