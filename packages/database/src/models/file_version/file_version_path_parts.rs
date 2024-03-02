use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileVersionPathParts {
    pub dir: String,
    pub file: String,
    pub version: String,
}

impl Display for FileVersionPathParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.dir, self.file, self.version)?;
        Ok(())
    }
}
