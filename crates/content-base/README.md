# content-base

A crate for managing and processing various types of content (video, audio, images, text) with advanced search capabilities.

## Structure

- `src/`
  - `core.rs`: Core functionality and initialization of ContentBase
  - `delete.rs`: Handling deletion of content and associated data
  - `query/`: Search and retrieval functionality
    - `mod.rs`: Main query implementation
    - `payload/`: Structs for different content types and search metadata
    - `search.rs`: Helper functions for search result processing
  - `task.rs`: Task management for content processing
  - `upsert.rs`: Handling content insertion and updating
  - `lib.rs`: Main library file exporting public interfaces

## Testing

Example of running tests with the remote-db feature enabled:

```bash
cargo test -p content-base db::op::create --features remote-db -- --nocapture
```

## Usage

1. Initialize a `ContentBase` instance:

```rust
let ctx = ContentBaseCtx::new("artifacts_dir", "");
let qdrant_client = Arc::new(Qdrant::new("http://localhost:6334"));
let content_base = ContentBase::new(&ctx, qdrant_client, "language_collection", "vision_collection")?;
```

2. Upsert content:

```rust
let payload = UpsertPayload::new("file_identifier", file_path, &metadata);
let notifications = content_base.upsert(payload).await?;
```

3. Query content:

```rust
let query_payload = QueryPayload::new("search query");
let results = content_base.query(query_payload).await?;
```

4. Delete content:

```rust
let delete_payload = DeletePayload::new("file_identifier");
content_base.delete(delete_payload).await?;
```

## How it works

1. Content Processing:

   - When content is upserted, it's broken down into tasks based on its type (video, audio, image, text).
   - Tasks are added to a task pool for processing.
   - Each task may generate embeddings, transcripts, or other derived data.

2. Data Storage:

   - Processed data and metadata are stored in Qdrant vector database.
   - Different collections are used for language-based and vision-based data.

3. Searching:
   - Queries are converted to embeddings.
   - Vector search is performed in Qdrant to find relevant content.
   - Results are post-processed and ranked before being returned.

## Content Processing Workflow

1. Content is submitted via `upsert()`.
2. Metadata is extracted and stored.
3. Tasks are generated based on content type.
4. Tasks are processed asynchronously.
   - For video: extract frames, audio, generate transcripts, create embeddings.
   - For audio: generate waveform, transcripts, embeddings.
   - For images: generate descriptions, embeddings.
   - For text: chunk content, generate summaries, embeddings.
   - **The specific tasks for each content type are defined in `ContentBase::get_content_processing_tasks` in the `content-base/src/core.rs` file**
5. Processed data is stored in Qdrant.
6. Search indexes are updated.

The modular design allows for easy extension to support new content types or processing tasks.
