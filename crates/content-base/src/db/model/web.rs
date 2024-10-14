use crate::db::model::id::ID;
use crate::db::model::PageModel;

#[derive(Debug, Clone)]
pub struct WebPageModel {
    pub id: Option<ID>,
    pub page: Vec<PageModel>,
}

impl WebPageModel {
    pub fn new(page: Vec<PageModel>) -> Self {
        Self { id: None, page }
    }
}
