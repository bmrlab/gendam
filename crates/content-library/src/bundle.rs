use crate::Library;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
use zip::write::SimpleFileOptions;

impl Library {
    pub fn generate_bundle(
        &self,
        hashes: &[String],
        output_path: impl AsRef<Path>,
    ) -> anyhow::Result<()> {
        let bundle_file = std::fs::File::create(output_path.as_ref())?;
        let mut zip = zip::ZipWriter::new(bundle_file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let mut buffer = Vec::new();

        for hash in hashes {
            let file_path = self.absolute_file_path(hash);
            let file_name = file_path.strip_prefix(&self.dir)?;
            zip.start_file(file_name.to_string_lossy().to_string(), options)?;
            let mut file = std::fs::File::open(file_path)?;
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();

            // add artifacts
            let results = walkdir::WalkDir::new(self._absolute_artifacts_dir(hash))
                .into_iter()
                .filter_map(|v| v.ok());

            for result in results {
                let path = result.path();
                if path.is_file() {
                    let name = path.strip_prefix(&self.dir)?;
                    zip.start_file(name.to_string_lossy().to_string(), options)?;
                    let mut file = std::fs::File::open(path)?;
                    file.read_to_end(&mut buffer)?;
                    zip.write_all(&buffer)?;
                    buffer.clear();
                }
            }
        }

        zip.finish()?;

        Ok(())
    }

    pub fn unpack_bundle(&self, bundle_file_path: impl AsRef<Path>) -> anyhow::Result<Vec<String>> {
        let file = fs::File::open(bundle_file_path.as_ref())?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut file_hash_list = vec![];

        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                let output_path = match file.enclosed_name() {
                    Some(output_path) => {
                        tracing::debug!("output_path: {output_path:?}");

                        if output_path.starts_with("files") {
                            if let Some(file_name) = output_path.file_name() {
                                file_hash_list.push(file_name.to_string_lossy().to_string());
                            }
                        }
                        self.dir.join(output_path)
                    }
                    _ => continue,
                };

                if file.name().ends_with("/") {
                    fs::create_dir_all(output_path)?;
                } else {
                    if let Some(p) = output_path.parent() {
                        if !p.exists() {
                            fs::create_dir_all(p)?;
                        }
                    }
                    let mut output_file = fs::File::create(&output_path)?;
                    std::io::copy(&mut file, &mut output_file)?;
                }
            }
        }

        Ok(file_hash_list)
    }
}
