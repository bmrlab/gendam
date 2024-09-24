use fake::faker::internet::en::Username;
use fake::faker::lorem::en::Sentence;
use fake::Fake;
use rand::Rng;

use crate::db::model::audio::{AudioFrameModel, AudioModel};
use crate::db::model::document::DocumentModel;
use crate::db::model::video::{ImageFrameModel, VideoModel};
use crate::db::model::web::WebPageModel;
use crate::db::model::{ImageModel, TextModel};
use crate::db::DB;

pub async fn setup() -> DB {
    dotenvy::dotenv().ok();
    DB::new().await
}

pub fn gen_vector(size: usize) -> Vec<f32> {
    (0..size)
        .map(|_| rand::thread_rng().gen_range(0.0..1.0))
        .collect()
}

pub fn fake_text_model() -> TextModel {
    let data: String = Username().fake();
    let vector = gen_vector(1024);
    TextModel {
        data: data.clone(),
        vector: vector.clone(),
        en_data: data,
        en_vector: vector,
    }
}

pub fn fake_image_model() -> ImageModel {
    ImageModel {
        prompt: Sentence(5..10).fake(),
        vector: gen_vector(512),
        prompt_vector: gen_vector(1024),
    }
}

pub fn fake_audio_frame_model() -> AudioFrameModel {
    AudioFrameModel {
        data: vec![fake_text_model()],
        start_timestamp: (1..10).fake::<u32>() as f32,
        end_timestamp: (10..20).fake::<u32>() as f32,
    }
}

pub fn fake_audio_model() -> AudioModel {
    AudioModel {
        audio_frame: (1..10).map(|_| fake_audio_frame_model()).collect(),
    }
}

pub fn fake_image_frame_model() -> ImageFrameModel {
    ImageFrameModel {
        data: vec![fake_image_model()],
        start_timestamp: (1..10).fake::<u32>() as f32,
        end_timestamp: (10..20).fake::<u32>() as f32,
    }
}

pub fn fake_page_model() -> crate::db::model::PageModel {
    crate::db::model::PageModel {
        text: vec![fake_text_model()],
        image: vec![fake_image_model()],
        start_index: (1..10).fake(),
        end_index: (10..20).fake(),
    }
}

pub fn fake_web_page_model() -> WebPageModel {
    WebPageModel {
        page: (1..10).map(|_| fake_page_model()).collect(),
    }
}

pub fn fake_video_model() -> VideoModel {
    VideoModel {
        image_frame: (1..10).map(|_| fake_image_frame_model()).collect(),
        audio_frame: (1..10).map(|_| fake_audio_frame_model()).collect(),
    }
}

pub fn fake_document() -> DocumentModel {
    DocumentModel {
        page: (1..10).map(|_| fake_page_model()).collect(),
    }
}
