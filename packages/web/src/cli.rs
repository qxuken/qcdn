use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};

#[derive(Debug, Default, ValueEnum, Clone, Copy)]
pub enum Mode {
    #[default]
    Production,
    Development,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Development => write!(f, "development"),
            Mode::Production => write!(f, "production"),
        }
    }
}

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    /// Path to data dir
    #[arg(short, long, env = "QCDN_MODE", default_value_t = Default::default())]
    pub(crate) mode: Mode,

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
