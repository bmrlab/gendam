mod download;
pub use download::*;

#[tokio::test]
async fn test_download() {
    let download = FileDownload::new(FileDownloadConfig {
        resources_dir: std::path::PathBuf::from("Downloads"),
        ..Default::default()
    });

    let res = download
        .download_if_not_exists("CLIP-ViT-B-32-laion2B-s34B-b79K/textual")
        .await;

    assert!(res.is_ok());
}
