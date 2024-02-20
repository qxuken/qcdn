use qcdn_database::Dir;

use crate::GetDirResponse;

impl From<Dir> for GetDirResponse {
    fn from(value: Dir) -> Self {
        GetDirResponse {
            id: value.id,
            name: value.name,
        }
    }
}
