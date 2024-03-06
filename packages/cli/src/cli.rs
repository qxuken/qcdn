use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::utils::std_table::Format;

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
    /// Run tui [default]
    #[default]
    Ui,

    /// Test connection to server
    Ping,

    /// List all directories
    Dirs {
        /// Format of data output
        #[arg(long, default_value_t = Default::default(), global = true)]
        format: Format,
    },

    /// List all files in directory
    Files {
        /// Directory id
        dir_id: i64,

        /// Format of data output
        #[arg(long, default_value_t = Default::default(), global = true)]
        format: Format,
    },

    /// List all file versions
    Versions {
        /// File id
        file_id: i64,

        /// Format of data output
        #[arg(long, default_value_t = Default::default(), global = true)]
        format: Format,
    },

    /// Download file version
    Download {
        /// File version id
        file_version_id: i64,

        /// Destination file.
        /// Otherwise data will be converted to utf-8 and printed to stdin
        path: Option<PathBuf>,
    },

    /// File upload
    Upload {
        /// Manually tag file type
        #[arg(long)]
        media_type: Option<String>,

        /// Target server dir
        dir: String,

        /// Target server file name
        name: String,

        /// Version to upload
        version: String,

        /// Path to upload file
        src: PathBuf,
    },

    /// Tag specific file version
    Tag {
        /// File version id
        file_version_id: i64,

        /// Tag
        tag: String,
    },

    /// Delete specific file version
    Delete {
        /// File version id
        file_version_id: i64,
    },
}
