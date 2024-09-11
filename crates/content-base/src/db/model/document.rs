use crate::db::model::PageModel;

pub struct DocumentModel {
    pub data: Vec<PageModel>,
}

impl DocumentModel {
    pub fn new(data: Vec<PageModel>) -> Self {
        Self { data }
    }
    
}