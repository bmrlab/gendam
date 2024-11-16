# content-base

A crate for managing and processing various types of content (video, audio, images, text) with advanced search capabilities.

## Testing

Example of running tests with the remote-db feature enabled:

```bash
cargo test -p content-base db::op::create --features remote-db -- --nocapture
```

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
5. Processed data is stored in SurrealDB.

## ContentTask process walkthrough

0. 创建新的 `TaskPool`，
   - 创建 `tx`, `rx` 来异步接收通过 `add_task` 进来的任务
   - 同时启动 `cpu_task_ctx`, `io_task_ctx` 两个任务队列，无限循环，每次取出并执行一个任务
1. 在 `ContentBase` 的 `upsert` 操作中：
   - 通过 `get_content_processing_tasks` 获取目标内容类型的任务列表
   - 将任务添加到 `TaskPool` 中
2. `TaskPool` 收到任务，预处理，放进 `TaskPoolContext` (`cpu_task_ctx` 或 `io_task_ctx`), 目前全都是 `cpu_task_ctx`
3. 在 `TaskPoolContext` 中调用 `pop_next_task`，然后把队列 permit 锁住，确保只有一个任务执行
4. 在 `TaskPoolContext` 中执行 `async_exec_task`，这会触发 `ContentTask` trait 中定义的 `run` 方法
   - 首先检查 `artifacts.json` 中的现有任务记录 - 如果发现相同参数的任务已完成，则直接标记为完成
   - 然后调用 `inner_run`，这个方法包含了各个具体任务类型的实现
5. 任务执行完毕，归还队列的 permit 权限，继续 pop

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
