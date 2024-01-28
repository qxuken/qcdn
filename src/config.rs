use std::path::PathBuf;

use clap::{arg, command, value_parser, Parser};
use tracing_subscriber::filter;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliConfig {
    #[arg(
        short,
        long,
        help = "Path to sqlite db",
        env = "FS_DB_PATH",
        default_value = "data/qcdn.db"
    )]
    pub db_path: PathBuf,

    #[arg(
        short,
        long,
        help = "Path to storage dir",
        env = "FS_STORAGE_DIR",
        default_value = "data/storage"
    )]
    pub storage_dir: PathBuf,

    #[arg(
        short,
        long,
        help = "Url node will serve on",
        env = "FS_BASE_URL",
        default_value = "http://localhost:8080"
    )]
    pub base_url: String,

    #[arg(
        long,
        help = "Local interface address",
        env = "FS_HOST",
        default_value = "127.0.0.1"
    )]
    pub host: String,

    #[arg(
        short,
        long,
        help = "TCP port",
        env = "FS_PORT",
        default_value = "8080",
        value_parser = value_parser!(u16).range(1..)
    )]
    pub port: u16,

    #[arg(long, help = "Log level", env = "FS_LOG_LEVEL", default_value = "info")]
    pub log_level: filter::LevelFilter,

    #[arg(short, long, help = "Url to main server", env = "FS_MAIN_SERVER_URL")]
    pub main_server_url: Option<String>,
}

impl CliConfig {
    pub fn init() -> CliConfig {
        CliConfig::parse()
    }
}
