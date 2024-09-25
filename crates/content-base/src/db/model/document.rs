use crate::db::model::id::ID;
use crate::db::model::PageModel;

#[derive(Debug)]
pub struct DocumentModel {
    pub id: Option<ID>,
    pub page: Vec<PageModel>,
}

impl DocumentModel {
    pub fn new(page: Vec<PageModel>) -> Self {
        Self { id: None, page }
    }
}
