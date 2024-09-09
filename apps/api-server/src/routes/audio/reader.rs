use ai::AudioTranscriptOutput;
use csv::WriterBuilder;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Write;
use std::path::PathBuf;
use storage_macro::Storage;
use tracing::debug;

// 检查 content 是否为空，如何为空直接返回空字符串
macro_rules! check_empty {
    ($self:expr, $body:block) => {
        if $self.content.is_empty() {
            Ok(String::new())
        } else {
            $body
        }
    };
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AudioData {
    start_timestamp: u32,
    end_timestamp: u32,
    text: String,
}

// 自定义序列化逻辑的包装器
struct AudioDataCsvSer<'a>(&'a AudioData);

impl<'a> Serialize for AudioDataCsvSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("AudioData", 3)?;
        s.serialize_field(
            "start_timestamp",
            &AudioData::format_timestamp(self.0.start_timestamp, b','),
        )?;
        s.serialize_field(
            "end_timestamp",
            &AudioData::format_timestamp(self.0.end_timestamp, b','),
        )?;
        s.serialize_field("text", &self.0.text.trim())?;
        s.end()
    }
}

impl AudioData {
    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn format_timestamp(time_in_milliseconds: u32, millis_delimiter: u8) -> String {
        let hours = time_in_milliseconds / 3600000;
        let minutes = (time_in_milliseconds % 3600000) / 60000;
        let seconds = (time_in_milliseconds % 60000) / 1000;
        let millis = time_in_milliseconds % 1000;
        let delimiter_char = millis_delimiter as char;
        format!(
            "{:02}:{:02}:{:02}{}{:03}",
            hours, minutes, seconds, delimiter_char, millis
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Storage)]
pub struct AudioReader {
    content: Vec<AudioData>,
}

impl AudioReader {
    pub fn new(path: PathBuf) -> Self {
        Self {
            content: AudioReader::parse(path).unwrap_or_default(),
        }
    }

    pub fn content(&self) -> Vec<AudioData> {
        self.content.clone()
    }

    /// 读取 transcript.txt 文件内容
    /// 文件格式为 JSON: [{"start_timestamp":0,"end_timestamp":1880,"text":"..."}]
    /// 返回 AudioData
    fn parse(path: PathBuf) -> anyhow::Result<Vec<AudioData>> {
        debug!("audio parse path {}", path.display());

        let storage = get_current_fs_storage!().map_err(|e| {
            rspc::Error::new(
                rspc::ErrorCode::InternalServerError,
                format!("failed to get current storage: {}", e),
            )
        })?;

        let content = storage.read_to_string(path)?;
        let raw_content = serde_json::from_str::<AudioTranscriptOutput>(&content)?;

        Ok(raw_content
            .transcriptions
            .iter()
            .map(|v| AudioData {
                start_timestamp: v.start_timestamp as u32,
                end_timestamp: v.end_timestamp as u32,
                text: v.text.clone(),
            })
            .collect())
    }

    pub fn read_to_srt(&self) -> anyhow::Result<String> {
        let mut srt = String::new();
        for (i, item) in self.content.iter().enumerate() {
            srt.push_str(&(i + 1).to_string());
            srt.push_str("\n");
            srt.push_str(&AudioData::format_timestamp(item.start_timestamp, b','));
            srt.push_str(" --> ");
            srt.push_str(&AudioData::format_timestamp(item.end_timestamp, b','));
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
        check_empty!(self, { Ok(serde_json::to_string(&self.content)?) })
    }

    pub fn read_to_vtt(&self) -> anyhow::Result<String> {
        check_empty!(self, {
            // Prepare the result string
            let mut result = "WEBVTT\n\n".to_string();

            // Loop over the content
            for (i, data) in self.content.iter().enumerate() {
                let cue = format!(
                    "{}\n{} --> {}\n{}\n\n",
                    i + 1,
                    AudioData::format_timestamp(data.start_timestamp, b'.'),
                    AudioData::format_timestamp(data.end_timestamp, b'.'),
                    data.text
                );
                result.push_str(&cue);
            }

            Ok(result)
        })
    }

    pub fn read_to_csv(&self) -> anyhow::Result<String> {
        check_empty!(self, {
            let mut wtr = WriterBuilder::new().delimiter(b';').from_writer(vec![]);

            // 序列化并写入每条音频数据
            for record in self.content.clone() {
                wtr.serialize(AudioDataCsvSer(&record))?;
            }
            wtr.flush()?;
            // 从内存中获取生成的 CSV 字符串
            let csv_content = String::from_utf8(wtr.into_inner()?)?;
            Ok(csv_content)
        })
    }

    // read to avid log exchange
    pub fn read_to_ale(&self) -> anyhow::Result<String> {
        check_empty!(self, {
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
                let start_time = AudioData::format_timestamp(data.start_timestamp, b':');
                let end_time = AudioData::format_timestamp(data.end_timestamp, b':');
                writeln!(ale_str, "{}\t{}\t{}", start_time, end_time, data.text).unwrap();
            }

            Ok(ale_str)
        })
    }

    /// file_name: 文件名.docx
    pub fn read_to_docx(&self) -> anyhow::Result<String> {
        self.read_to_txt()

        // let file =
        //     File::create(&file_name).map_err(|err| anyhow!("Failed to create file: {:?}", err))?;
        //
        // let mut doc = Docx::new();
        // for item in &self.content {
        //     doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&item.text)));
        // }
        //
        // doc.build()
        //     .pack(&file)
        //     .map_err(|err| anyhow!("Failed to build docx: {}", err))?;
        //
        // Ok(file)
    }
}

#[cfg(test)]
mod audio_tests {
    use super::*;
    use csv::ReaderBuilder;
    use serde_json::from_str;
    use std::env;

    fn setup() -> AudioReader {
        let path = env::current_dir()
            .unwrap()
            .join("src/tests/mock/transcript.txt");
        let reader = AudioReader::new(path);
        reader
    }

    #[test]
    fn test_audio_reader() {
        let reader = setup();
        let content = reader.content();
        assert_eq!(content.len(), 40);
        assert_eq!(content[0].text.trim(), "How much does each person have?");
        assert_eq!(content.last().map(|c| c.text.clone()).is_some(), true);
        assert_eq!(
            content.last().map(|c| c.text.clone()).unwrap().trim(),
            "You can watch the complete teaching."
        );
    }

    #[test]
    fn test_audio_reader_to_srt() {
        let reader = setup();
        let srt = reader.read_to_srt();
        assert!(srt.is_ok());

        let mut is_srt = true;
        let mut expect_header = true;
        let mut line_count = 0;

        for line in srt.unwrap().lines() {
            line_count += 1;

            if expect_header {
                // 验证序号
                if line_count % 4 == 1 {
                    if line.parse::<u32>().is_err() {
                        is_srt = false;
                        break;
                    }
                }
                // 验证时间码
                else if line_count % 4 == 2 {
                    if !line.contains("-->") {
                        is_srt = false;
                        break;
                    }
                }
                // 验证空行
                else if line_count % 4 == 0 {
                    if !line.is_empty() {
                        is_srt = false;
                        break;
                    }
                    expect_header = true; // 准备下一个序号
                } else {
                    expect_header = false;
                }
            }
        }

        assert!(is_srt);
        assert!(line_count > 0);
    }

    #[test]
    fn test_audio_reader_to_txt() {
        let reader = setup();
        let txt = reader.read_to_txt();
        assert!(txt.is_ok());
        assert_eq!(txt.unwrap().lines().count(), 40);
    }

    #[test]
    fn test_audio_reader_to_json() {
        let reader = setup();
        let json_str = reader.read_to_json();
        assert!(json_str.is_ok());
        assert!(from_str::<Vec<AudioData>>(json_str.unwrap().as_str()).is_ok());
    }

    #[test]
    fn test_audio_reader_to_vtt() {
        let reader = setup();
        let vtt = reader.read_to_vtt();

        assert!(vtt.is_ok());

        let mut is_vtt = false;

        for line in vtt.unwrap().lines() {
            // 忽略空行
            if line.trim().is_empty() {
                continue;
            }
            is_vtt = if line.starts_with("WEBVTT") {
                true
            } else {
                false
            };
            break;
        }

        assert!(is_vtt);
    }

    #[test]
    fn test_audio_reader_to_csv() {
        let reader = setup();
        let csv = reader.read_to_csv();
        assert!(csv.is_ok());
        if let Ok(csv) = csv {
            // 包含标题行
            assert_eq!(csv.lines().count(), 41);
            let mut rdr = ReaderBuilder::new()
                .has_headers(true)
                .delimiter(b';')
                .from_reader(csv.as_bytes());
            // 尝试读取每一条记录
            let result = rdr.records().all(|record| record.is_ok());
            assert!(result)
        }
    }

    #[test]
    fn test_audio_reader_to_ale() {
        let reader = setup();
        let ale = reader.read_to_ale();
        assert!(ale.is_ok());
        assert!(ale.unwrap().len() > 0);
    }

    #[test]
    fn test_audio_reader_to_docx() {
        let reader = setup();
        let txt = reader.read_to_docx();
        assert!(txt.is_ok());
        assert_eq!(txt.unwrap().lines().count(), 40);
    }
}
