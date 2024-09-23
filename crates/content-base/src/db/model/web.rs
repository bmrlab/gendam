use crate::db::model::PageModel;

#[derive(Debug)]
pub struct WebPageModel {
    pub page: Vec<PageModel>,
}

impl WebPageModel {
    pub fn new(page: Vec<PageModel>) -> Self {
        Self { page }
    }
}
