use crate::db::model::PageModel;

pub struct WebPageModel {
    pub data: Vec<PageModel>,
}

impl WebPageModel {
    pub fn new(data: Vec<PageModel>) -> Self {
        Self { data }
    }
}