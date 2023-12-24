use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::filter;

#[derive(Debug, Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[arg(
        long,
        help = "path to sqlite db",
        env = "FS_DB_PATH",
        default_value = "data/filestore.db"
    )]
    pub db_path: PathBuf,

    #[arg(
        long,
        help = "path to storage dir",
        env = "FS_STORAGE_DIR",
        default_value = "data/storage"
    )]
    pub storage_dir: PathBuf,

    #[arg(
        long,
        help = "base url",
        env = "FS_BASE_URL",
        default_value = "http://localhost:8080"
    )]
    pub base_url: String,

    #[arg(
        long,
        help = "local interface address",
        env = "FS_HOST",
        default_value = "127.0.0.1"
    )]
    pub host: String,

    #[arg(
        short,
        long,
        help = "http port",
        env = "FS_HTTP_PORT",
        default_value = "8080"
    )]
    pub port: u16,

    #[arg(long, help = "log level", env = "FS_LOG_LEVEL", default_value = "info")]
    pub log_level: filter::LevelFilter,
}
