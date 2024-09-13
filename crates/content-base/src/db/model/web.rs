use crate::db::model::PageModel;

pub struct WebPageModel {
    pub page: Vec<PageModel>,
}

impl WebPageModel {
    pub fn new(page: Vec<PageModel>) -> Self {
        Self { page }
    }
}
