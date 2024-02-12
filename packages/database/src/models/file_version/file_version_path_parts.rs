use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileVersionPathParts {
    pub dir: String,
    pub file: String,
    pub version: String,
}
