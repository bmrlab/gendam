use ndarray::Array2;

mod kts;

/// Split video using frame features (e.g., CLIP feature) with Kernel Temporal Segmentation.
///
/// # Arguments
/// * `features` - frame features, should be normalized by row
/// * `alpha` - Determine the maximum segment number for KTS algorithm, the larger the value, the fewer segments, default is 10
///
/// # Returns
/// * `Vec<usize>` - best split points for input features
pub fn split_video(features: Array2<f64>, alpha: Option<usize>) -> anyhow::Result<Vec<usize>> {
    let alpha = alpha.unwrap_or(10);

    if features.shape()[0] <= alpha {
        // not enough frames, just return empty vector
        return Ok(vec![]);
    }

    let features = features.dot(&features.t());

    let clip_num = features.shape()[0];
    let max_seg_num = clip_num / alpha;

    kts::cpd_auto(features, max_seg_num - 1, 1.0, None)
}

#[test_log::test(tokio::test)]
async fn test_split_video() {
    let frames_dir = "/Users/zhuo/Library/Application Support/cc.musedam.local/libraries/98f19afbd2dee7fa6415d5f523d36e8322521e73fd7ac21332756330e836c797/artifacts/1aaa451c0bee906e2d1f9cac21ebb2ef5f2f82b2f87ec928fc04b58cbceda60b/frames";
    let resources_dir = "/Users/zhuo/Library/Application Support/cc.musedam.local/resources";

    let clip_model = ai::clip::CLIP::new(resources_dir, ai::clip::CLIPModel::MViTB32)
        .await
        .expect("failed to load CLIP");

    let mut frame_paths = std::fs::read_dir(frames_dir)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()
        .unwrap();

    let mut features = vec![];

    frame_paths.sort_by_key(|v| {
        v.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split(".")
            .into_iter()
            .next()
            .unwrap()
            .parse::<usize>()
            .unwrap()
    });

    let mut n = 0;

    for path in frame_paths {
        if path.extension() == Some(std::ffi::OsStr::new("png")) {
            let feat = clip_model
                .get_image_embedding_from_file(&path)
                .await
                .unwrap();
            let feat: Vec<f64> = feat.iter().map(|&x| x as f64).collect();

            features.extend(feat);

            n += 1;
        }
    }

    let features = ndarray::Array2::from_shape_vec((n, clip_model.dim()), features)
        .unwrap()
        .into();

    let cps = split_video(features, None);

    tracing::info!("{:?}", cps);
}
