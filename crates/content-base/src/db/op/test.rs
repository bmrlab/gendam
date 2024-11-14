#[cfg(test)]
mod tests {
    use crate::check_db_error_from_resp;
    use crate::db::model::id::{ID, TB};
    use crate::db::model::text::TextModel;
    use crate::db::shared::test::{
        fake_audio_model, fake_document, fake_file_identifier, fake_image_model, fake_page_model,
        fake_video_model, fake_web_page_model, gen_vector, setup,
    };
    use itertools::Itertools;
    use test_log::test;

    // 让 test 串行执行
    static TEST_LOCK: std::sync::OnceLock<tokio::sync::Mutex<()>> = std::sync::OnceLock::new();
    async fn get_test_lock() -> &'static tokio::sync::Mutex<()> {
        TEST_LOCK.get_or_init(|| tokio::sync::Mutex::new(()))
    }

    #[test(tokio::test)]
    async fn test_insert_text() {
        let _guard = get_test_lock().await.lock().await;
        let id = setup(None)
            .await
            .insert_text(
                None,
                TextModel {
                    id: None,
                    content: "测试".to_string(),
                    embedding: gen_vector(1024),
                    // en_content: "test".to_string(),
                    // en_embedding: gen_vector(1024),
                },
            )
            .await
            .unwrap();
        println!("{:?}", id);
        assert_eq!(id.tb(), &TB::Text);
    }

    #[test(tokio::test)]
    async fn test_insert_image() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let _ = db
            .insert_image(Some(fake_file_identifier()), fake_image_model())
            .await;
    }

    #[test(tokio::test)]
    async fn test_insert_audio() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_audio(fake_file_identifier(), fake_audio_model())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Audio);
    }

    #[test(tokio::test)]
    async fn test_insert_video() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_video(fake_file_identifier(), fake_video_model())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Video);
    }

    #[test(tokio::test)]
    async fn test_insert_page() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db.insert_page(fake_page_model()).await.unwrap();
        assert_eq!(id.tb(), &TB::Page);
    }

    #[test(tokio::test)]
    async fn test_insert_web_page() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_web_page(fake_file_identifier(), fake_web_page_model())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Web);
    }

    #[test(tokio::test)]
    async fn test_insert_document() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let id = db
            .insert_document(fake_file_identifier(), fake_document())
            .await
            .unwrap();
        assert_eq!(id.tb(), &TB::Document);
    }

    #[tokio::test]
    async fn test_delete_image() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let file_identifier = fake_file_identifier();
        let _image = db
            .insert_image(Some(file_identifier.clone()), fake_image_model())
            .await
            .unwrap();
        db.delete_by_file_identifier(&file_identifier)
            .await
            .expect("delete image");
        // let select_image_res = db.select_image(vec![&image.id_with_table()]).await.unwrap();
        // assert!(select_image_res.is_empty());
    }

    #[tokio::test]
    async fn test_delete_audio() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let file_identifier = fake_file_identifier();
        let _ = db
            .insert_audio(file_identifier.clone(), fake_audio_model())
            .await
            .unwrap();
        db.delete_by_file_identifier(&file_identifier)
            .await
            .expect("delete audio");
        // let select_audio_res = db.select_audio(vec![&audio.id_with_table()]).await.unwrap();
        // assert!(select_audio_res.is_empty());
    }

    #[tokio::test]
    async fn test_delete_video() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        let file_identifier = fake_file_identifier();
        // let file_identifier = "test_delete_video".to_string();
        let _video_id = db
            .insert_video(file_identifier.clone(), fake_video_model())
            .await
            .unwrap();
        db.delete_by_file_identifier(&file_identifier)
            .await
            .expect("delete video");
        // let video_id_string = video_id.id_with_table();
        // let video_res = db.select_video(vec![video_id_string]).await.unwrap();
        // assert!(video_res.is_empty());
    }

    #[test(tokio::test)]
    async fn test_upsert() {
        let _guard = get_test_lock().await.lock().await;
        let db = setup(None).await;
        db.upsert(
            &ID::from("text:11232131"),
            format!(
                "content = 't-1', embedding = [{}]", // , en_content = 't-1', en_embedding = [{}]",
                gen_vector(1024)
                    .into_iter()
                    .map(|v| v.to_string())
                    .join(","),
            )
            .as_str(),
        )
        .await
        .unwrap();
        let mut resp = db
            .client
            .query(format!("(SELECT * FROM {}).id", "text:11232131"))
            .await
            .unwrap();
        check_db_error_from_resp!(resp)
            .map_err(|errors_map| anyhow::anyhow!("select text error: {:?}", errors_map))
            .unwrap();
        let result = resp.take::<Vec<surrealdb::sql::Thing>>(0).unwrap();
        assert_eq!(result.len(), 1);
    }
}
