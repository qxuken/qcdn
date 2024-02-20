use qcdn_database::FileVersionTagUpsert;

use crate::TagVersionRequest;

impl From<TagVersionRequest> for FileVersionTagUpsert {
    fn from(value: TagVersionRequest) -> Self {
        Self {
            name: value.tag,
            file_version_id: value.file_version_id,
            created_at: None,
        }
    }
}
