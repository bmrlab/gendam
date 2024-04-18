use serde::Serialize;
use specta::Type;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::warn;

struct SimpleReporterPrivate {
    last_update: std::time::Instant,
    max_progress: Option<u64>,
    message: String,
}
pub(crate) struct SimpleReporter {
    reporter: DownloadReporter,
    file_name: String,
    target_dir: PathBuf,
    url: String,
    private: std::sync::Mutex<Option<SimpleReporterPrivate>>,
}

impl SimpleReporter {
    pub(crate) fn create(
        file_name: String,
        target_dir: PathBuf,
        url: String,
        reporter: DownloadReporter,
    ) -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            reporter,
            file_name,
            target_dir,
            url,
            private: std::sync::Mutex::new(None),
        })
    }
}

impl downloader::progress::Reporter for SimpleReporter {
    fn setup(&self, max_progress: Option<u64>, message: &str) {
        let private = SimpleReporterPrivate {
            last_update: std::time::Instant::now(),
            max_progress,
            message: message.to_string(),
        };

        let mut guard = self.private.lock().unwrap();
        *guard = Some(private);

        self.reporter.setup(
            self.file_name.clone(),
            max_progress,
            self.url.clone(),
            self.target_dir.clone(),
            message.to_string(),
        )
    }

    fn progress(&self, current: u64) {
        if let Some(p) = self.private.lock().unwrap().as_mut() {
            let max_bytes = match p.max_progress {
                Some(bytes) => format!("{:?}", bytes),
                None => "{unknown}".to_owned(),
            };
            if p.last_update.elapsed().as_millis() >= 1000 {
                println!(
                    "test file: {} of {} bytes. [{}]",
                    current, max_bytes, p.message
                );
                p.last_update = std::time::Instant::now();

                self.reporter.progress(self.file_name.clone(), current);
            }
        }
    }

    fn set_message(&self, message: &str) {
        println!("test file: Message changed to: {}", message);
    }

    fn done(&self) {
        let mut guard = self.private.lock().unwrap();
        *guard = None;

        self.reporter.end(self.file_name.clone(), 0);

        println!("test file: [DONE]");
    }
}

pub(crate) fn file_name_from_url(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    let Ok(url) = reqwest::Url::parse(url) else {
        return String::new();
    };

    url.path_segments()
        .map_or_else(String::new, |f| f.last().unwrap_or("").to_string())
}

enum DownloadReportPayload {
    Setup((String, Option<u64>, String, PathBuf, String)),
    Progress((String, u64)),
    End((String, usize)),
}

#[derive(Clone, Debug, Type, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStatus {
    /// should be unique
    pub file_name: String,
    pub target_dir: std::path::PathBuf,
    pub url: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub message: Option<String>,
    pub started_at: String,
    pub exit_code: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct DownloadReporter {
    tx: std::sync::mpsc::Sender<DownloadReportPayload>,
}

#[derive(Clone, Debug)]
pub struct DownloadHub {
    file_list: Arc<Mutex<Vec<DownloadStatus>>>,
    inner: DownloadReporter,
}

impl DownloadHub {
    pub fn new() -> Self {
        let file_list: Vec<DownloadStatus> = vec![];
        let file_list = Arc::new(Mutex::new(file_list));
        let file_list_clone = file_list.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(payload) = rx.recv() {
                match payload {
                    DownloadReportPayload::Setup((file_id, bytes, url, target_dir, message)) => {
                        let mut file_list = file_list.lock().unwrap();
                        let file = file_list.iter_mut().find(|f| f.file_name == file_id);
                        if file.is_some() {
                            warn!("file exist, ignore");
                        } else {
                            let start = SystemTime::now();
                            let since_the_epoch = start
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards");
                            file_list.push(DownloadStatus {
                                file_name: file_id,
                                target_dir,
                                url,
                                total_bytes: bytes,
                                downloaded_bytes: 0,
                                message: Some(message),
                                started_at: since_the_epoch.as_secs().to_string(),
                                exit_code: None,
                            })
                        }
                    }
                    DownloadReportPayload::Progress((file_id, bytes)) => {
                        let mut file_list = file_list.lock().unwrap();
                        let file = file_list.iter_mut().find(|f| f.file_name == file_id);
                        if let Some(file) = file {
                            file.downloaded_bytes = bytes;
                        }
                    }
                    DownloadReportPayload::End((file_id, exit_code)) => {
                        let mut file_list = file_list.lock().unwrap();
                        let file = file_list.iter_mut().find(|f| f.file_name == file_id);
                        if let Some(file) = file {
                            file.exit_code = Some(exit_code);
                        }
                    }
                }
            }
        });

        Self {
            file_list: file_list_clone,
            inner: DownloadReporter { tx },
        }
    }

    pub fn get_reporter(&self) -> DownloadReporter {
        self.inner.clone()
    }

    pub fn get_file_list(&self) -> Vec<DownloadStatus> {
        self.file_list.lock().unwrap().clone()
    }
}

impl DownloadReporter {
    fn setup(
        &self,
        file_name: String,
        bytes: Option<u64>,
        url: String,
        target_dir: PathBuf,
        message: String,
    ) {
        let _ = self.tx.send(DownloadReportPayload::Setup((
            file_name, bytes, url, target_dir, message,
        )));
    }

    fn progress(&self, file_name: String, bytes: u64) {
        let _ = self
            .tx
            .send(DownloadReportPayload::Progress((file_name, bytes)));
    }

    fn end(&self, file_name: String, exit_code: usize) {
        let _ = self
            .tx
            .send(DownloadReportPayload::End((file_name, exit_code)));
    }
}
