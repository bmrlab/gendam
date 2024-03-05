use crate::video::split::split_video;
use ai::whisper::WhisperItem;
use llm::LLMMessage;
use prisma_lib::{
    video_clip,
    video_frame::{self, OrderByParam},
    PrismaClient,
};
use std::{fs::File, io::BufReader, sync::Arc};
use tokio::{io::AsyncReadExt, sync::RwLock};
use tracing::{debug, error, info, warn};

const BATCH_FRAME_COUNT: i64 = 1000;

pub async fn get_video_clips(
    file_identifier: String,
    transcript_path: Option<impl AsRef<std::path::Path>>,
    frames_dir: Option<impl AsRef<std::path::Path>>,
    client: Arc<RwLock<PrismaClient>>,
) -> anyhow::Result<()> {
    // if transcript exists, use its timestamp to split video into clip
    if let Some(transcript_path) = transcript_path {
        if transcript_path.as_ref().exists() {
            let file = File::open(transcript_path.as_ref())?;
            let reader = BufReader::new(file);
            let whisper_results: Vec<WhisperItem> = serde_json::from_reader(reader)?;

            return create_video_clips(
                file_identifier.clone(),
                client.clone(),
                whisper_results
                    .iter()
                    .map(|v| (v.start_timestamp as i32, v.end_timestamp as i32))
                    .collect(),
            )
            .await;
        }
    }

    let frames_dir = frames_dir.ok_or(anyhow::anyhow!("no frames dir provided"))?;

    // otherwise, use KTS to split video
    // NOTICE when video is too long, we need to split it into multiple batch task
    // TODO we need to test the ideal maximum number of features we send to KTS, current the value is 1000

    let mut timestamps = vec![];

    let n_total_frames = client
        .read()
        .await
        .video_frame()
        .count(vec![video_frame::WhereParam::FileIdentifier(
            prisma_lib::read_filters::StringFilter::Contains(file_identifier.clone()),
        )])
        .exec()
        .await
        .expect("failed to count video frames");

    let mut current_frame_count = 0;
    let mut next_batch_size = BATCH_FRAME_COUNT;
    loop {
        // how much frames to be processed at once
        let current_batch_count = (n_total_frames - current_frame_count).min(next_batch_size);

        let frames = client
            .read()
            .await
            .video_frame()
            .find_many(vec![video_frame::WhereParam::FileIdentifier(
                prisma_lib::read_filters::StringFilter::Contains(file_identifier.clone()),
            )])
            .order_by(OrderByParam::Timestamp(prisma_client_rust::Direction::Asc))
            .skip(current_frame_count)
            .take(current_batch_count)
            .exec()
            .await
            .expect("failed to get frames");

        let mut video_features = vec![];
        // record frame idx to timestamp mapping
        let mut idx_to_timestamp = vec![];

        let n_frames = frames.len();

        debug!("frame count: {}", n_frames);

        // get embeddings for these frames from local file
        for frame in frames {
            let mut file = tokio::fs::File::open(
                frames_dir
                    .as_ref()
                    .join(format!("{}.embedding", frame.timestamp)),
            )
            .await?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).await?;
            let embedding: Vec<f64> = serde_json::from_str(&buffer)?;

            video_features.extend(embedding);
            idx_to_timestamp.push(frame.timestamp);
        }

        let video_features = ndarray::Array2::from_shape_vec(
            (n_frames, video_features.len() / n_frames),
            video_features,
        )?;

        // TODO `split_video` will cause OOM, if next_batch_size has been increased a lot of times
        let best_split_points = split_video(video_features, None)?;

        if best_split_points.len() == 0 {
            warn!("no best split points found");
        }

        let is_last_batch = current_batch_count < next_batch_size;

        // split timestamp using best split points
        let mut last_split_point = 0;
        for idx in 0..best_split_points.len() {
            // every split point will create a new video clip
            timestamps.push((
                idx_to_timestamp[last_split_point],
                idx_to_timestamp[best_split_points[idx]],
            ));
            last_split_point = best_split_points[idx];
        }

        if !is_last_batch {
            if let Some(&last_point) = best_split_points.last() {
                // if current is not last batch and best split points IS NOT empty
                let consuming_frame_count = last_point as i64;
                current_frame_count += consuming_frame_count;
                next_batch_size = BATCH_FRAME_COUNT;
            } else {
                // if current is not last and best split points IS empty
                // this means no frame is consumed, we need to increase next batch size
                // FIXME should not increase without any limitation
                next_batch_size += BATCH_FRAME_COUNT;
            }
        } else {
            timestamps.push((
                idx_to_timestamp[last_split_point],
                *idx_to_timestamp.last().unwrap(),
            ));
            break;
        }
    }

    info!("timestamps: {:?}", timestamps);

    create_video_clips(file_identifier.clone(), client.clone(), timestamps).await
}

pub async fn get_video_clips_summarization(
    file_identifier: String,
    resources_dir: impl AsRef<std::path::Path>,
    client: Arc<RwLock<PrismaClient>>,
) -> anyhow::Result<()> {
    let llm = llm::LLM::new(resources_dir.as_ref(), llm::model::Model::Gemma2B).await?;
    let llm = Arc::new(RwLock::new(llm));

    let video_frame_args = video_frame::ManyArgs::new(vec![]).with(video_frame::caption::fetch());
    let clips = client
        .read()
        .await
        .video_clip()
        .find_many(vec![video_clip::WhereParam::FileIdentifier(
            prisma_lib::read_filters::StringFilter::Contains(file_identifier.clone()),
        )])
        .with(video_clip::WithParam::Frames(video_frame_args))
        .exec()
        .await?;

    for clip in clips {
        let frames = clip.frames()?;
        let mut captions: Vec<(String, i32)> = frames
            .iter()
            .filter_map(|v| match v.caption().expect("failed to fetch caption") {
                Some(caption) => Some((caption.caption.clone(), v.timestamp)),
                _ => None,
            })
            .collect();
        captions.sort_by(|a, b| a.1.cmp(&b.1));

        let mut prompt = String::from(
            r#"You are an AI assistant designed for summarizing a video.
Following document records caption of frames in a video.
Please summarize the video content in one sentence based on the document.

You should not consider captions as separate scenes, they have a temporal relationship.
For example, if there are two captions with same content, this means video do not change during these two captions.
So you can consider them as one scene when summarization.

The sentence should not exceed 30 words.
If you cannot summarize, just response with empty message.
Please start with "The video contains".
Do not repeat the information in document.
Do not response any other information.

Here is the document:"#,
        );

        captions.iter().for_each(|v| {
            prompt = format!("{}\n{}", prompt, v.0);
        });

        let response = llm
            .read()
            .await
            .call(vec![LLMMessage::User(prompt)], None)
            .await?;
        let response = response.trim().to_string();

        debug!("summarization response: {:?}", response);

        // remove the prefix hard coded in prompt
        let response = response.replace("The video contains ", "");
        // uppercase the first letter
        let response = {
            let mut c = response.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect(),
                None => String::new(),
            }
        };

        client
            .write()
            .await
            .video_clip()
            .update(
                video_clip::UniqueWhereParam::IdEquals(clip.id),
                vec![video_clip::SetParam::SetCaption(Some(response))],
            )
            .exec()
            .await?;
    }

    Ok(())
}

async fn create_video_clips(
    file_identifier: String,
    client: Arc<RwLock<PrismaClient>>,
    timestamps: Vec<(i32, i32)>,
) -> anyhow::Result<()> {
    for (index, item) in timestamps.iter().enumerate() {
        // find frames between [start, end)
        let frames = client
            .read()
            .await
            .video_frame()
            .find_many(vec![
                video_frame::WhereParam::Timestamp(prisma_lib::read_filters::IntFilter::Gte(
                    item.0,
                )),
                // for last clip, include the right bound
                if index == timestamps.len() - 1 {
                    video_frame::WhereParam::Timestamp(prisma_lib::read_filters::IntFilter::Lte(
                        item.1,
                    ))
                } else {
                    video_frame::WhereParam::Timestamp(prisma_lib::read_filters::IntFilter::Lt(
                        item.1,
                    ))
                },
            ])
            .exec()
            .await
            .unwrap_or(vec![]);

        let frames: Vec<video_frame::UniqueWhereParam> = frames
            .iter()
            .map(|v| video_frame::UniqueWhereParam::IdEquals(v.id))
            .collect();

        if let Err(e) = client
            .write()
            .await
            .video_clip()
            .create(
                file_identifier.clone(),
                item.0,
                item.1,
                vec![video_clip::SetParam::ConnectFrames(frames)],
            )
            .exec()
            .await
        {
            error!("Failed to create video clip: {e}")
        }
    }

    Ok(())
}

#[test_log::test(tokio::test)]
async fn test_video_clip() {
    let local_data_dir =
        std::path::Path::new("/Users/zhuo/Library/Application Support/cc.musedam.local")
            .to_path_buf();
    let library = content_library::load_library(
        &local_data_dir,
        "98f19afbd2dee7fa6415d5f523d36e8322521e73fd7ac21332756330e836c797",
    );
    let client = prisma_lib::new_client_with_url(&library.db_url)
        .await
        .expect("");
    let client = Arc::new(RwLock::new(client));

    let file_identifier =
        String::from("1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b");
    let frames_dir = "/Users/zhuo/Library/Application Support/cc.musedam.local/libraries/98f19afbd2dee7fa6415d5f523d36e8322521e73fd7ac21332756330e836c797/artifacts/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/frames";

    let result = get_video_clips(
        file_identifier,
        None::<std::path::PathBuf>,
        Some(frames_dir),
        client,
    )
    .await;

    assert!(result.is_ok())
}
