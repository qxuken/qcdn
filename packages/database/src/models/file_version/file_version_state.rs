use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::Type, Serialize, Deserialize, Clone, Copy)]
#[repr(i64)]
pub enum FileVersionState {
    Created,
    Downloading,
    Ready,
    Error = -1,
}

impl From<i64> for FileVersionState {
    fn from(value: i64) -> Self {
        match value {
            0 => FileVersionState::Created,
            1 => FileVersionState::Downloading,
            2 => FileVersionState::Ready,
            _ => FileVersionState::Error,
        }
    }
}
