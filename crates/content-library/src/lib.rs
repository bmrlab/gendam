use prisma_lib::new_client_with_url;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Library {
    pub id: String,
    pub dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub index_dir: PathBuf,  // for faiss
    pub db_url: String,
}

pub async fn create_library(local_data_dir: PathBuf) -> Library {
    let library_id = "1234567";
    let library_dir = local_data_dir.join("libraries").join(library_id);
    let db_dir = library_dir.join("databases");
    let index_dir = library_dir.join("index");
    let artifacts_dir = library_dir.join("artifacts");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::create_dir_all(&index_dir).unwrap();
    std::fs::create_dir_all(&artifacts_dir).unwrap();
    let db_url = format!("file:{}", db_dir.join("muse-v2.db").to_str().unwrap());
    let client = new_client_with_url(db_url.as_str())
        .await
        .expect("failed to create prisma client");
    client._db_push().await.expect("failed to push db"); // apply migrations
    Library {
        id: library_id.to_string(),
        dir: library_dir,
        artifacts_dir,
        index_dir,
        db_url,
    }
}
