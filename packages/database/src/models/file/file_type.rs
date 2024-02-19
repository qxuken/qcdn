use serde::{Deserialize, Serialize};

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[repr(i64)]
pub enum FileType {
    Other,
    Stylesheets,
    Javascript,
    Image,
    Font,
    Text,
    Unknown = -1,
}

impl From<i64> for FileType {
    fn from(value: i64) -> Self {
        match value {
            0 => FileType::Other,
            1 => FileType::Stylesheets,
            2 => FileType::Javascript,
            3 => FileType::Image,
            4 => FileType::Font,
            5 => FileType::Text,
            _ => FileType::Unknown,
        }
    }
}
