use prisma_lib::new_client_with_url;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Library {
    pub id: String,
    pub dir: PathBuf,
    pub files_dir: PathBuf,  // for content files
    pub artifacts_dir: PathBuf,
    pub index_dir: PathBuf,  // for faiss
    pub db_url: String,
}

pub fn load_library(local_data_root: &PathBuf, library_id: &str) -> Library {
    let library_dir = local_data_root.join("libraries").join(library_id);
    let db_dir = library_dir.join("databases");
    let index_dir = library_dir.join("index");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    let db_url = format!("file:{}", db_dir.join("muse-v2.db").to_str().unwrap());
    Library {
        id: library_id.to_string(),
        dir: library_dir,
        files_dir,
        artifacts_dir,
        index_dir,
        db_url,
    }
}

pub async fn create_library_with_title(local_data_root: &PathBuf, title: &str) -> Library {
    let _ = title;
    let library_id = sha256::digest(format!("{}", chrono::Utc::now()));
    let library_dir = local_data_root.join("libraries").join(&library_id);
    let db_dir = library_dir.join("databases");
    let index_dir = library_dir.join("index");
    let artifacts_dir = library_dir.join("artifacts");
    let files_dir = library_dir.join("files");
    std::fs::create_dir_all(&db_dir).unwrap();
    std::fs::create_dir_all(&index_dir).unwrap();
    std::fs::create_dir_all(&artifacts_dir).unwrap();
    std::fs::create_dir_all(&files_dir).unwrap();
    let db_url = format!("file:{}", db_dir.join("muse-v2.db").to_str().unwrap());
    let client = new_client_with_url(db_url.as_str())
        .await
        .expect("failed to create prisma client");
    client._db_push().await.expect("failed to push db"); // apply migrations
    Library {
        id: library_id,
        dir: library_dir,
        files_dir,
        artifacts_dir,
        index_dir,
        db_url,
    }
}
