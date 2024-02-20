use qcdn_database::File;

use crate::{FileType, GetFileResponse};

impl From<File> for GetFileResponse {
    fn from(value: File) -> Self {
        let ft: FileType = value.file_type.into();
        Self {
            id: value.id,
            dir_id: value.dir_id,
            name: value.name,
            file_type: ft.into(),
        }
    }
}
