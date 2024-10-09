use crate::db::model::audio::{AudioFrameModel, AudioModel};
use crate::db::model::document::DocumentModel;
use crate::db::model::video::{ImageFrameModel, VideoModel};
use crate::db::model::web::WebPageModel;
use crate::db::model::{ImageModel, TextModel};
use crate::db::DB;
use crate::query::payload::image::ImageIndexMetadata;
use crate::query::payload::raw_text::RawTextIndexMetadata;
use crate::query::payload::video::VideoIndexMetadata;
use crate::query::payload::{ContentIndexMetadata, ContentIndexPayload};
use content_base_task::audio::trans_chunk::AudioTransChunkTask;
use content_base_task::image::desc_embed::ImageDescEmbedTask;
use content_base_task::image::ImageTaskType;
use content_base_task::raw_text::chunk_sum_embed::RawTextChunkSumEmbedTask;
use content_base_task::raw_text::RawTextTaskType;
use content_base_task::video::trans_chunk::VideoTransChunkTask;
use content_base_task::video::VideoTaskType;
use content_base_task::web_page::transform::WebPageTransformTask;
use content_base_task::web_page::WebPageTaskType;
use content_base_task::ContentTaskType;
use fake::faker::internet::en::Username;
use fake::faker::lorem::en::Sentence;
use fake::Fake;
use itertools::Itertools;
use rand::Rng;
use std::env;
use std::path::Path;

pub async fn setup(path: Option<&Path>) -> DB {
    dotenvy::dotenv().ok();
    DB::new(path.unwrap_or(env::current_exe().unwrap().parent().unwrap())).await
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

pub fn fake_audio_frame_model() -> AudioFrameModel {
    AudioFrameModel {
        id: None,
        data: vec![fake_text_model()],
        start_timestamp: (1..10).fake::<u32>() as f32,
        end_timestamp: (10..20).fake::<u32>() as f32,
    }
}

pub fn fake_audio_model() -> AudioModel {
    AudioModel {
        id: None,
        audio_frame: (1..10).map(|_| fake_audio_frame_model()).collect(),
    }
}

pub fn fake_image_frame_model() -> ImageFrameModel {
    ImageFrameModel {
        id: None,
        data: vec![fake_image_model()],
        start_timestamp: (1..10).fake::<u32>() as f32,
        end_timestamp: (10..20).fake::<u32>() as f32,
    }
}

pub fn fake_page_model() -> crate::db::model::PageModel {
    crate::db::model::PageModel {
        id: None,
        text: vec![fake_text_model()],
        image: vec![fake_image_model()],
        start_index: (1..10).fake(),
        end_index: (10..20).fake(),
    }
}

pub fn fake_web_page_model() -> WebPageModel {
    WebPageModel {
        id: None,
        page: (1..10).map(|_| fake_page_model()).collect(),
    }
}

pub fn fake_video_model() -> VideoModel {
    VideoModel {
        id: None,
        image_frame: (1..10).map(|_| fake_image_frame_model()).collect(),
        audio_frame: (1..10).map(|_| fake_audio_frame_model()).collect(),
    }
}

pub fn fake_document() -> DocumentModel {
    DocumentModel {
        id: None,
        page: (1..10).map(|_| fake_page_model()).collect(),
    }
}

pub fn fake_video_payload() -> ContentIndexPayload {
    ContentIndexPayload {
        file_identifier: (4..8).fake::<String>(),
        task_type: ContentTaskType::Video(VideoTaskType::TransChunk(VideoTransChunkTask {})),
        metadata: ContentIndexMetadata::Video(VideoIndexMetadata {
            start_timestamp: (1..20).fake(),
            end_timestamp: (30..100).fake(),
        }),
    }
}

pub fn fake_image_payload() -> ContentIndexPayload {
    ContentIndexPayload {
        file_identifier: (4..8).fake::<String>(),
        task_type: ContentTaskType::Image(ImageTaskType::DescEmbed(ImageDescEmbedTask {})),
        metadata: ContentIndexMetadata::Image(ImageIndexMetadata {}),
    }
}

pub fn fake_audio_payload() -> ContentIndexPayload {
    ContentIndexPayload {
        file_identifier: (4..8).fake::<String>(),
        task_type: ContentTaskType::Audio(crate::audio::AudioTaskType::TransChunk(
            AudioTransChunkTask {},
        )),
        metadata: ContentIndexMetadata::Audio(crate::query::payload::audio::AudioIndexMetadata {
            start_timestamp: (1..20).fake(),
            end_timestamp: (30..100).fake(),
        }),
    }
}

pub fn fake_web_page_payload() -> ContentIndexPayload {
    ContentIndexPayload {
        file_identifier: (4..8).fake::<String>(),
        task_type: ContentTaskType::WebPage(WebPageTaskType::Transform(WebPageTransformTask {})),
        metadata: ContentIndexMetadata::WebPage(
            crate::query::payload::web_page::WebPageIndexMetadata {
                start_index: (1..20).fake(),
                end_index: (30..100).fake(),
            },
        ),
    }
}

pub fn fake_document_payload() -> ContentIndexPayload {
    ContentIndexPayload {
        file_identifier: (4..8).fake::<String>(),
        task_type: ContentTaskType::RawText(RawTextTaskType::ChunkSumEmbed(
            RawTextChunkSumEmbedTask {},
        )),
        metadata: ContentIndexMetadata::RawText(RawTextIndexMetadata {
            start_index: (1..20).fake(),
            end_index: (30..100).fake(),
        }),
    }
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
