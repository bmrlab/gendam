use crate::routes::audio::reader::AudioReader;
use anyhow::{anyhow, Result};
use docx_rs::{Docx, Paragraph, Run};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadHelper {
    reader: AudioReader,
    dir: PathBuf,
}

impl DownloadHelper {
    pub fn new(reader: AudioReader, dir: PathBuf) -> Self {
        Self { reader, dir }
    }

    fn save_to_path(&self, content: String, file_name: String) -> Result<()> {
        let mut file = File::create(self.dir.join(file_name))?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn download_to_txt(&self, file_name: String) -> Result<()> {
        self.save_to_path(self.reader.read_to_txt()?, file_name)
    }

    pub fn download_to_srt(&self, file_name: String) -> Result<()> {
        self.save_to_path(self.reader.read_to_srt()?, file_name)
    }

    pub fn download_to_json(&self, file_name: String) -> Result<()> {
        self.save_to_path(self.reader.read_to_json()?, file_name)
    }

    pub fn download_to_vtt(&self, file_name: String) -> Result<()> {
        self.save_to_path(self.reader.read_to_vtt()?, file_name)
    }

    pub fn download_to_csv(&self, file_name: String) -> Result<()> {
        self.save_to_path(self.reader.read_to_csv()?, file_name)
    }

    pub fn download_to_ale(&self, file_name: String) -> Result<()> {
        self.save_to_path(self.reader.read_to_ale()?, file_name)
    }

    pub fn download_to_docx(&self, file_name: String) -> Result<()> {
        let full_path = self.dir.join(&file_name);
        let file =
            File::create(&full_path).map_err(|err| anyhow!("Failed to create file: {:?}", err))?;

        let mut doc = Docx::new();
        for item in &self.reader.content() {
            doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(&item.text())));
        }

        doc.build()
            .pack(&file)
            .map_err(|err| anyhow!("Failed to build docx: {}", err))?;

        Ok(())
    }
}
