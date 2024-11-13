use crate::db::model::{
    audio::{AudioFrameModel, AudioModel},
    document::DocumentModel,
    image::ImageModel,
    page::PageModel,
    text::TextModel,
    video::{ImageFrameModel, VideoModel},
    web::WebPageModel,
};
use crate::db::DB;
use fake::faker::internet::en::Username;
use fake::faker::lorem::en::Sentence;
use fake::Fake;
use itertools::Itertools;
use rand::Rng;
use std::env;
use std::path::Path;

pub async fn setup(path: Option<&Path>) -> DB {
    dotenvy::dotenv().ok();

    #[cfg(feature = "embedded-db")]
    let db = DB::new(path.unwrap_or(env::current_exe().unwrap().parent().unwrap()))
        .await
        .unwrap();

    // #[cfg(not(feature = "embedded-db"))]
    #[cfg(feature = "remote-db")]
    let db = DB::new().await.unwrap();

    db
}

pub fn gen_vector(size: usize) -> Vec<f32> {
    (0..size)
        .map(|_| rand::thread_rng().gen_range(0.0..1.0))
        .collect()
}

pub fn gen_text_vector() -> Vec<f32> {
    gen_vector(1024)
}

pub fn gen_image_vector() -> Vec<f32> {
    gen_vector(512)
}

pub fn fake_text_model() -> TextModel {
    let data: String = Username().fake();
    let vector = gen_text_vector();
    TextModel {
        id: None,
        data: data.clone(),
        vector: vector.clone(),
        en_data: data,
        en_vector: vector,
    }
}

pub fn fake_image_model() -> ImageModel {
    ImageModel {
        id: None,
        prompt: Sentence(5..10).fake(),
        vector: gen_image_vector(),
        prompt_vector: gen_text_vector(),
    }
}

pub fn fake_audio_frame_model() -> (AudioFrameModel, Vec<TextModel>) {
    (
        AudioFrameModel {
            id: None,
            start_timestamp: (1..10).fake::<i64>(),
            end_timestamp: (10..20).fake::<i64>(),
        },
        vec![fake_text_model()],
    )
}

pub fn fake_audio_model() -> (AudioModel, Vec<(AudioFrameModel, Vec<TextModel>)>) {
    (
        AudioModel { id: None },
        (1..10).map(|_| fake_audio_frame_model()).collect(),
    )
}

pub fn fake_image_frame_model() -> (ImageFrameModel, Vec<ImageModel>) {
    (
        ImageFrameModel {
            id: None,
            start_timestamp: (1..10).fake::<i64>(),
            end_timestamp: (10..20).fake::<i64>(),
        },
        vec![fake_image_model()],
    )
}

pub fn fake_page_model() -> (PageModel, Vec<TextModel>, Vec<ImageModel>) {
    (
        PageModel {
            id: None,
            start_index: (1..10).fake(),
            end_index: (10..20).fake(),
        },
        vec![fake_text_model()],
        vec![fake_image_model()],
    )
}

pub fn fake_web_page_model() -> (
    WebPageModel,
    Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>,
) {
    (
        WebPageModel { id: None },
        (1..10).map(|_| fake_page_model()).collect(),
    )
}

pub fn fake_video_model() -> (
    VideoModel,
    Vec<(ImageFrameModel, Vec<ImageModel>)>,
    Vec<(AudioFrameModel, Vec<TextModel>)>,
) {
    (
        VideoModel { id: None },
        (1..10).map(|_| fake_image_frame_model()).collect(),
        (1..10).map(|_| fake_audio_frame_model()).collect(),
    )
}

pub fn fake_document() -> (
    DocumentModel,
    Vec<(PageModel, Vec<TextModel>, Vec<ImageModel>)>,
) {
    (
        DocumentModel { id: None },
        (1..10).map(|_| fake_page_model()).collect(),
    )
}

pub fn fake_file_identifier() -> String {
    (4..8).fake::<String>()
}

pub fn fake_upsert_text_clause() -> String {
    let fake_data = (4..8).fake::<String>();
    format!(
        "data = '{}', vector = [{}], en_data = '{}', en_vector = [{}]",
        fake_data,
        gen_text_vector()
            .into_iter()
            .map(|v| v.to_string())
            .join(","),
        fake_data,
        gen_text_vector()
            .into_iter()
            .map(|v| v.to_string())
            .join(",")
    )
}
