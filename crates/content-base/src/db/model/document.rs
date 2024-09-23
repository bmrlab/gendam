use crate::db::model::PageModel;

#[derive(Debug)]
pub struct DocumentModel {
    pub page: Vec<PageModel>,
}

impl DocumentModel {
    pub fn new(page: Vec<PageModel>) -> Self {
        Self { page }
    }
}
