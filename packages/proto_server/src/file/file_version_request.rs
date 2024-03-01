use qcdn_database::FileVersionWithTags;

use crate::GetFileVersionResponse;

impl From<FileVersionWithTags> for GetFileVersionResponse {
    fn from(value: FileVersionWithTags) -> Self {
        Self {
            id: value.id,
            file_id: value.file_id,
            name: value.name,
            size: value.size,
            tags: value.tags,
            is_deleted: value.deleted_at.is_some(),
        }
    }
}
