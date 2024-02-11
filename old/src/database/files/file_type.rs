use serde::{Deserialize, Serialize};

use crate::grpc::FileType as GrpcFileType;

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[repr(i32)]
pub enum FileType {
    Other,
    Stylesheets,
    Javascript,
    Image,
    Font,
    Text,
}

impl From<GrpcFileType> for FileType {
    fn from(value: GrpcFileType) -> Self {
        match value {
            GrpcFileType::Other => Self::Other,
            GrpcFileType::Stylesheets => Self::Stylesheets,
            GrpcFileType::Javascript => Self::Javascript,
            GrpcFileType::Image => Self::Image,
            GrpcFileType::Font => Self::Font,
            GrpcFileType::Text => Self::Text,
        }
    }
}

impl From<FileType> for GrpcFileType {
    fn from(value: FileType) -> GrpcFileType {
        match value {
            FileType::Other => Self::Other,
            FileType::Stylesheets => Self::Stylesheets,
            FileType::Javascript => Self::Javascript,
            FileType::Image => Self::Image,
            FileType::Font => Self::Font,
            FileType::Text => Self::Text,
        }
    }
}
