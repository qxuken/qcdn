use qcdn_database::File;

use crate::GetFileResponse;

impl From<File> for GetFileResponse {
    fn from(value: File) -> Self {
        Self {
            id: value.id,
            dir_id: value.dir_id,
            name: value.name,
            media_type: value.media_type,
        }
    }
}
