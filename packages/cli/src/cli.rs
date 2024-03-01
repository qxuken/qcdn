use clap::{Parser, Subcommand};

#[derive(Debug, Parser, Clone)]
pub struct Cli {
    /// Manager url
    #[arg(short, long, env = "QCDN_LEADER_URL")]
    pub url: String,

    #[clap(flatten)]
    pub instrumentation: qcdn_tracing_setup::instrumentation::Instrumentation,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand, Default, Clone)]
pub enum Command {
    /// Launch ui [default]
    #[default]
    Ui,

    /// Test connection to server
    Connect,

    /// List all directories
    Dirs,

    /// List all files in directory
    Files {
        /// Directory id
        dir_id: i64,
    },

    /// List all file versions
    Versions {
        /// File id
        file_id: i64,
    },

    /// simple file upload
    Upload {
        /// Version to upload
        #[arg(long, default_value = "1")]
        version: String,

        /// Whether delete or not immediately after upload
        #[arg(short, long, default_value = "false")]
        save_version: bool,
    },
}
