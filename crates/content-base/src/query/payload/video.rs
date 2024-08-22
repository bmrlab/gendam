use super::SearchMetadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSearchMetadata {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}

impl VideoSearchMetadata {
    pub fn new(start_timestamp: i64, end_timestamp: i64) -> Self {
        Self {
            start_timestamp,
            end_timestamp,
        }
    }
}

impl TryFrom<SearchMetadata> for VideoSearchMetadata {
    type Error = anyhow::Error;

    fn try_from(metadata: SearchMetadata) -> Result<Self, Self::Error> {
        match metadata {
            SearchMetadata::Video(metadata) => Ok(metadata),
            _ => anyhow::bail!("metadata is not from video"),
        }
    }
}

impl From<VideoSearchMetadata> for SearchMetadata {
    fn from(metadata: VideoSearchMetadata) -> Self {
        SearchMetadata::Video(metadata)
    }
}

impl PartialEq for VideoSearchMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.start_timestamp == other.start_timestamp && self.end_timestamp == other.end_timestamp
    }
}

impl Eq for VideoSearchMetadata {}

impl PartialOrd for VideoSearchMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.start_timestamp.partial_cmp(&other.start_timestamp) {
            Some(std::cmp::Ordering::Equal) => self.end_timestamp.partial_cmp(&other.end_timestamp),
            other => other,
        }
    }
}

impl Ord for VideoSearchMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn merge_results_with_time_duration<T, TFn, TCompare>(
    items: &mut [(T, f32)],
    fn_metadata: TFn,
    fn_compare: TCompare,
) -> Vec<(T, f32)>
where
    T: PartialEq + Eq + PartialOrd + Ord + Clone,
    TFn: Fn(&[&T]) -> T,
    TCompare: Fn(&T, &T) -> bool,
{
    if items.len() <= 1 {
        return items.to_vec();
    }

    let mut results = vec![];

    items.sort_by(|a, b| a.0.cmp(&b.0));

    let mut handle_merge = |last_idx: usize, idx: usize| {
        let raw_items = items[last_idx..idx]
            .iter()
            .map(|v| &v.0)
            .collect::<Vec<_>>();
        let new_metadata = fn_metadata(&raw_items);
        let score = items[last_idx..idx]
            .iter()
            .map(|v| v.1)
            .max_by(|x, y| x.total_cmp(y))
            .expect("should have max");

        // 用匹配到的数量作为 bonus
        // 数量为1 时不加分，增加数量则按照 log 函数增加，超过5个的也不加分
        // TODO 效果有待验证，先去掉加分规则
        // let start_timestamp = items[last_idx..idx]
        //     .iter()
        //     .map(|v| v.0.start_timestamp())
        //     .min()
        //     .expect("should have min");
        // let end_timestamp = items[last_idx..idx]
        //     .iter()
        //     .map(|v| v.0.end_timestamp())
        //     .max()
        //     .expect("should have max");
        // let mut score = items[last_idx..idx]
        //     .iter()
        //     .map(|v| v.1)
        //     .max_by(|x, y| x.total_cmp(y))
        //     .expect("should have max");

        // score += ((idx - last_idx).min(5) as f32).log(5.0) * 0.15;

        // let new_metadata = fn_metadata(&items[last_idx].0, start_timestamp, end_timestamp);
        results.push((new_metadata, score));
    };

    let mut idx = 1;
    let mut last_idx = 0;

    while idx < items.len() {
        if fn_compare(&items[idx].0, &items[idx - 1].0) {
            handle_merge(last_idx, idx);

            last_idx = idx;
        }

        idx += 1;
    }

    // 处理最后一帧
    handle_merge(last_idx, items.len());

    results.sort_by(|a, b| b.1.total_cmp(&a.1));

    results
}
