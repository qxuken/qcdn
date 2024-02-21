use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub(crate) struct Cli {
    /// Manager url
    #[arg(short, long, env = "QCDN_MANAGER_URL")]
    pub(crate) url: String,

    #[clap(flatten)]
    pub(crate) instrumentation: qcdn_tracing_setup::instrumentation::Instrumentation,

    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand, Default)]
pub enum Command {
    /// Launch ui
    #[default]
    Ui,

    /// simple file upload
    Upload {
        /// Version to upload
        #[arg(long, default_value = "1")]
        version: String,

        /// Whether delete or not immediately after upload
        #[arg(short, default_value = "false")]
        save_version: bool,
    },
}
