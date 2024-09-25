use htmd::HtmlToMarkdown;
use std::{io::Read, path::Path};

#[cfg(feature = "webpage")]
use chromiumoxide::{page::ScreenshotParams, Browser, BrowserConfig};
#[cfg(feature = "webpage")]
use futures::StreamExt;

#[cfg(feature = "webpage")]
pub async fn fetch_url(url: &str) -> anyhow::Result<(Option<String>, String, Vec<u8>)> {
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .window_size(1920, 1080)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to launch browser: {}", e))?,
    )
    .await?;

    // spawn a new task that continuously polls the handler
    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                tracing::debug!("error: {:?}", h);
                break;
            }
        }
    });

    let page = browser.new_page(url).await?;
    let title = page.get_title().await?;
    let html_content = page.content().await?;

    let mut page_params = ScreenshotParams::default();
    page_params.full_page = Some(true);
    let screenshot = page.screenshot(page_params).await?;

    let _ = browser.close().await;
    let _ = handle.await;

    Ok((title, html_content, screenshot))
}

pub fn convert_to_markdown(file_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(file_path)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    let converter = HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style"])
        .build();

    let markdown_string = converter.convert(&buf)?;

    Ok(markdown_string)
}
