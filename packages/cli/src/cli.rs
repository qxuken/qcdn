use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    /// Manager url
    #[arg(short, long, env = "QCDN_MANAGER_URL")]
    pub url: String,

    /// Version to upload
    #[arg(long, default_value = "1")]
    pub version: String,

    /// Whether delete or not immediately after upload
    #[arg(short, default_value = "false")]
    pub save_version: bool,

    #[clap(flatten)]
    pub(crate) instrumentation: qcdn_tracing_setup::instrumentation::Instrumentation,
}
