use crate::{Ctx, R};
use anyhow::anyhow;
use docx_rs::{Docx, Paragraph, Run};
use rspc::Router;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;
use tracing::info;
use std::fmt::Write;

pub fn get_routes() -> Router<Ctx> {
    let router = R.router().procedure(
        "find_one",
        R.query(|ctx, hash: String| async move {
            let artifacts_dir = ctx.library.artifacts_dir.clone();
            let path = artifacts_dir.join(hash).join("transcript.txt");
            let reader = RadioReader::new(path);
            serde_json::to_value::<Vec<RadioData>>(reader.content()).unwrap_or_default()
        }),
    );
    router
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RadioData {
    start_timestamp: u32,
    end_timestamp: u32,
    text: String,
}

impl RadioData {
    pub fn format_duration(duration: u32) -> String {
        let seconds = duration % 60;
        let minutes = (duration / 60) % 60;
        let hours = duration / 60 / 60;
        let millis = duration % 1000;
        format!("{:02}:{:02}:{:02},{:03}", hours, minutes, seconds, millis)
    }

    pub fn format_time_for_vtt(time_in_milliseconds: u32) -> String {
        let hours = time_in_milliseconds / 3_600_000;
        let minutes = (time_in_milliseconds % 3_600_000) / 60_000;
        let seconds = (time_in_milliseconds % 60_000) / 1_000;
        let milliseconds = time_in_milliseconds % 1_000;
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            hours, minutes, seconds, milliseconds
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RadioReader {
    content: Vec<RadioData>,
}

impl RadioReader {
    pub fn new(path: PathBuf) -> Self {
        Self {
            content: RadioReader::parse(path).unwrap_or_default(),
        }
    }

    pub fn content(&self) -> Vec<RadioData> {
        self.content.clone()
    }

    /// 读取 transcript.txt 文件内容
    /// 文件格式为 JSON: [{"start_timestamp":0,"end_timestamp":1880,"text":"..."}]
    /// 返回 RadioData
    fn parse(path: PathBuf) -> anyhow::Result<Vec<RadioData>> {
        info!("path {}", path.display());
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn read_to_srt(&self) -> anyhow::Result<String> {
        let mut srt = String::new();
        for (i, item) in self.content.iter().enumerate() {
            srt.push_str(&(i + 1).to_string());
            srt.push_str("\n");
            srt.push_str(&RadioData::format_duration(item.start_timestamp));
            srt.push_str(" --> ");
            srt.push_str(&RadioData::format_duration(item.end_timestamp));
            srt.push_str("\n");
            srt.push_str(&item.text);
            srt.push_str("\n\n");
        }
        Ok(srt)
    }

    pub fn read_to_txt(&self) -> anyhow::Result<String> {
        let mut txt = String::new();
        for item in self.content.iter() {
            txt.push_str(&item.text);
            txt.push_str("\n");
        }
        Ok(txt)
    }

    pub fn read_to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self.content)?)
    }

    pub fn read_to_vtt(&self) -> anyhow::Result<String> {
        // Prepare the result string
        let mut result = "WEBVTT\n\n".to_string();

        // Loop over the content
        for (i, data) in self.content.iter().enumerate() {
            let cue = format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                RadioData::format_time_for_vtt(data.start_timestamp),
                RadioData::format_time_for_vtt(data.end_timestamp),
                data.text
            );
            result.push_str(&cue);
        }

        Ok(result)
    }

    pub fn read_to_csv(&self) -> anyhow::Result<String> {
        let mut csv = String::new();
        csv.push_str("start_timestamp,end_timestamp,text\n");
        for item in self.content.iter() {
            csv.push_str(&item.start_timestamp.to_string());
            csv.push_str(",");
            csv.push_str(&item.end_timestamp.to_string());
            csv.push_str(",");
            csv.push_str(&item.text);
            csv.push_str("\n");
        }
        Ok(csv)
    }

    // read to avid log exchange
    pub fn read_to_ale(&self) -> String {
        let mut ale_str = String::new();

        // 文件头
        writeln!(ale_str, "Heading").unwrap();
        writeln!(ale_str, "FIELD_DELIM	TABS").unwrap();
        writeln!(ale_str, "VIDEO_FORMAT	1080").unwrap();
        writeln!(ale_str, "AUDIO_FORMAT	48khz").unwrap();
        writeln!(ale_str, "FPS	25").unwrap();
        writeln!(ale_str, "\nColumn").unwrap();

        // 列定义
        writeln!(ale_str, "Start\tEnd\tName").unwrap();
        writeln!(ale_str, "\nData").unwrap();

        // 数据行
        for data in &self.content {
            let start_time = RadioData::format_time_for_vtt(data.start_timestamp);
            let end_time = RadioData::format_time_for_vtt(data.end_timestamp);
            writeln!(ale_str, "{}\t{}\t{}", start_time, end_time, data.text).unwrap();
        }

        ale_str
    }


    /// file_name: 文件名.docx
    pub fn read_to_docx(&self, file_name: &str) -> anyhow::Result<File> {
        let file =
            File::create(&file_name).map_err(|err| anyhow!("Failed to create file: {:?}", err))?;

        let mut doc = Docx::new();
        for item in &self.content {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&item.text)));
        }

        doc.build()
            .pack(&file)
            .map_err(|err| anyhow!("Failed to build docx: {}", err))?;

        Ok(file)
    }
}
