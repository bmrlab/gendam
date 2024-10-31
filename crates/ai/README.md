# AI Crate

This crate provides a collection of AI models and utilities for various tasks such as image captioning, text embedding, audio transcription, and more.

## Features

- Image captioning
- Text embedding
- Audio transcription
- Multi-modal embedding
- Large Language Models (LLM) integration
- YOLO object detection

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ai = { path = "../path/to/ai" }
```

### Image Captioning Example

Here's how you can use different versions of image caption models:

```rust
use ai::{
    blip::{BLIP, BLIPModel},
    clip::{CLIP, CLIPModel},
    llm::{openai::OpenAI, LLM},
    AIModel, ImageCaptionModel,
};
use std::time::Duration;

async fn image_caption_example() -> anyhow::Result<()> {
    // 1. Using BLIP model for direct image captioning
    let blip_model = AIModel::new(
        || async {
            BLIP::new(
                "path/to/blip/model",
                "path/to/blip/tokenizer",
                BLIPModel::Base,
            )
            .await
        },
        Some(Duration::from_secs(30)),
    )?;

    // 2. Using CLIP model for image embedding
    let clip_model = AIModel::new(
        || async {
            CLIP::new(
                "path/to/clip/image_model",
                "path/to/clip/text_model",
                "path/to/clip/tokenizer",
                CLIPModel::MViTB32,
            )
            .await
        },
        Some(Duration::from_secs(30)),
    )?;

    // 3. Using LLM for caption generation
    let llm_model = AIModel::new(
        || async {
            Ok(LLM::OpenAI(
                OpenAI::new("http://api.openai.com", "your-api-key", "gpt-3.5-turbo").expect("OpenAI client"),
            ))
        },
        Some(Duration::from_secs(30)),
    )?;

    let clip_caption_model = llm_model.create_image_caption_ref();

    // Using image captioning
    let image_path = "path/to/your/image.jpg";

    let blip_caption = blip_model.process_single(image_path.into()).await?;
    println!("BLIP Caption: {}", blip_caption);

    let clip_caption = clip_caption_model.process_single(image_path.into()).await?;
    println!("CLIP+LLM Caption: {}", clip_caption);

    Ok(())
}
```

This example demonstrates three different approaches to image captioning:

1. Using BLIP model for direct image captioning.
2. Using CLIP model for generating image embeddings.
3. Using an LLM (in this case, OpenAI's GPT) to generate captions based on CLIP embeddings.

The BLIP model directly generates captions, while the CLIP+LLM approach uses CLIP to create image embeddings and then uses an LLM to generate captions based on those embeddings.

## Models

- BLIP: Vision-language model for image captioning
- CLIP: Contrastive Language-Image Pre-training model
- Whisper: Audio transcription model
- YOLO: Object detection model
- Various LLM integrations (OpenAI, Qwen2, etc.)

## Configuration

Most models require paths to model weights and tokenizers. Ensure you have the necessary model files downloaded and provide the correct paths when initializing the models.
