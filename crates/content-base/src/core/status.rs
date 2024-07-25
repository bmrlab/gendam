use std::collections::HashMap;

use crate::ContentBase;

pub struct StatusPayload {
    file_identifier: String,
}

impl ContentBase {
    async fn content_status(
        &self,
        payload: StatusPayload,
    ) -> anyhow::Result<HashMap<String, bool>> {
        // TODO the result hashmap should be:
        // { TASK_NAME: IS_FINISHED }
        todo!()
    }
}
