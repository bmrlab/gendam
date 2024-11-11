#[cfg(test)]
mod tests {
    use crate::db::model::id::ID;
    use crate::db::model::video::VideoModel;
    use crate::db::shared::test::{
        fake_file_identifier, fake_upsert_text_clause, fake_video_model, setup,
    };
    // use itertools::Itertools;
    // use std::process::id;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_select_text() {
        let db = setup(None).await;
        let text_res = db
            .select_text(vec!["text:7dd12x11yvt5fgamdjb0"])
            .await
            .unwrap();
        println!("text_res: {:?}", text_res);
    }

    #[test(tokio::test)]
    async fn test_select_image() {
        let db = setup(None).await;
        let image_res = db
            .select_image(vec!["image:flzkn6ncniglqttxnrsm"])
            .await
            .unwrap();
        println!("image_res: {:?}", image_res);
    }

    #[test(tokio::test)]
    async fn test_select_audio() {
        let db = setup(None).await;
        let audio_res = db
            .select_audio(vec!["audio:gkzq6db9jwr34l3j0gmz"])
            .await
            .unwrap();
        println!("audio_res: {:?}", audio_res);
    }

    #[test(tokio::test)]
    async fn test_select_video() {
        let db = setup(None).await;
        let video_res = db
            .select_video(vec!["video:u456grwuvl6w74zgqemc"])
            .await
            .unwrap();
        println!("video_res: {:?}", video_res);
    }

    #[test(tokio::test)]
    async fn test_select_web_page() {
        let db = setup(None).await;
        let web_page_res = db
            .select_web_page(vec!["web:nobc02c8ffyol3kqbsln"])
            .await
            .unwrap();
        println!("web_page_res: {:?}", web_page_res);
    }

    #[test(tokio::test)]
    async fn test_select_document() {
        let db = setup(None).await;
        let document_res = db
            .select_document(vec!["document:6dr6glzpf7ixefh7vjks"])
            .await
            .unwrap();
        println!("document_res: {:?}", document_res);
    }

    #[test(tokio::test)]
    async fn test_backtrace_by_ids() {
        let db = setup(None).await;
        let single_text_id = ID::from("text:11232131");
        db.upsert(&single_text_id, fake_upsert_text_clause().as_str())
            .await
            .unwrap();

        let video_id = db
            .insert_video(fake_video_model(), fake_file_identifier())
            .await
            .unwrap();

        let mut video: VideoModel = db
            .select_video(vec![video_id.id_with_table()])
            .await
            .unwrap()
            .pop()
            .unwrap()
            .into();

        println!("video: {video:?}");

        if video.audio_frame.is_empty() {
            println!("audio_frame is empty skip");
            return;
        }

        let mut audio_frame = video.audio_frame.pop().unwrap();
        println!("audio_frame: {:?}", audio_frame.id);

        let text = audio_frame.data.pop().unwrap();
        println!("text: {:?}", text);

        let res = db
            .backtrace_by_ids(vec![text.id.unwrap(), single_text_id.clone()])
            .await
            .unwrap();
        println!("res: {:?}", res[0]);
        println!("single_res: {:?}", res[1]);
        assert_eq!(res.len(), 2);
        assert!(res[0].hit_id.len() > 0);
        assert_eq!(res[1].hit_id.len(), 1);
        assert_eq!(res[1].hit_id[0], single_text_id);
    }

    #[test(tokio::test)]
    async fn test_full_text_search_with_highlight() {
        let db = setup(None).await;
        let res = db
            .full_text_search_with_highlight(vec!["LVL小河板".to_string()])
            .await
            .unwrap();
        println!("res: {res:#?}");
    }
}
