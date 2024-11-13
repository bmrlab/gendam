# GenDAM

## Getting Started

### 开发环境准备

```bash
pnpm dev:prep
```

会依次执行

- `bash scripts/download-sidecar.sh` 下载 sidecars: qdrant, ffmpeg, ffprobe, whisper 等
- `bash scripts/copy-sidecar-to-target.sh` 复制 tauri 下的 sidecar 和 resources 到 target/debug，供单独运行 api_server 用
- `cargo prisma generate` 生成 prisma 的代码 crates/prisma/src/prisma.rs

模型下载需要单独执行，这个取决于 library 的模型配置，模型下载目录在 `apps/desktop/src-tauri/resources`

```bash
pnpm dev:model
```

### 运行 tauri

```bash
pnpm tauri dev
```

### 单独运行 web

在开发环境，运行 tauri 的时候会同时启动 web 服务，但是也可以单独运行 web 服务

```bash
pnpm dev:web
```

### 单独运行 rspc 服务

```bash
pnpm dev:api-server

# or
cargo run -p api-server
cargo run -p api-server --no-default-features --features remote-search # 使用 remote surrealdb 进行搜索，方便调试
```

**单独运行 api_server 需要设置环境变量指定本地目录和资源目录, 比如**

```yaml
# 根目录 .env 文件中配置
# 本地数据目录，存储 Library 数据，包括素材文件、索引、数据库
LOCAL_DATA_DIR="/Users/xddotcom/Library/Application Support/ai.gendam.desktop"
# 本地资源目录，存储模型等，一般用当前项目目录下的 /apps/desktop/src-tauri/resources
LOCAL_RESOURCES_DIR="/Users/xddotcom/workspace/muse/gendam/apps/desktop/src-tauri/resources"
```

## Prisma Rust Client

1. 添加 `prisma-client-rust` 和 `prisma-client-rust-cli` 两个 crate
2. 添加 `bin/prisma.rs` 并在 `main` 中执行 `prisma_client_rust_cli::run();`, 搞定 prisma cli

```bash
cd src-tauri
cargo run --bin prisma
# or
cargo run --bin prisma -- <command>
```

为了方便使用，可以在 `.cargo/config.toml` 中添加一个 alias

```toml
[alias]
prisma = "run --bin prisma --"
```

3. 执行 `cargo prisma init` 初始化 prisma 配置, 这时候会在 `src-tauri` 下生成一个 `prisma` 目录, 接着需要把 schema.prisma 里面的 client 配置修改成如下

```prisma
generator client {
  provider = "cargo prisma"
  output = "src/prisma/mod.rs"
}
```

4. 执行 `cargo prisma generate` 生成 artifacts (e.g. Prisma Client), 根据上一步配置, 生成在 `src/prisma` 目录下

5. `cargo prisma migrate dev` 以后，在代码里直接引入 `PrismaClient` 开始使用

## 测试

```bash
cargo test -p api-server test_check_materialized_path_exists -- --nocapture
```

使用 cargo test 进行测试，最好通过 `-p` 参数指定测试的 crate，如果要指定测试函数，可以加上函数名，比如 `test_check_materialized_path_exists`，最后使用 `-- --nocapture` 参数可以显示 `println!` 的输出

## 内容处理的流程

1. 初始导入：

当一个视频文件被导入时，首先会调用 `ContentBase::upsert` 方法（在 `src/upsert.rs` 中）。这个方法接收一个 `UpsertPayload`，包含文件标识符、文件路径和元数据。

2. 任务生成：

在 `upsert` 方法中，系统会：

- 创建或更新 `TaskRecord`（在 `src/record.rs` 中定义）
- 调用 `ContentBase::tasks` 方法（在 `src/core.rs` 中）来生成适合视频的任务列表

对于视频，通常会生成以下任务：

- `VideoFrameTask`：提取视频帧
- `VideoTransChunkTask`：生成视频转录并分块
- `VideoTransChunkSumTask`：对转录块进行摘要
- `VideoTransChunkSumEmbedTask`：为摘要生成嵌入向量

3. 任务执行：

这些任务被添加到 `TaskPool`（在 `content-base-pool` crate 中定义）进行异步执行。每个任务的具体实现在 `content-base-task` crate 中的相应模块里：

- `src/video/frame.rs`：处理帧提取
- `src/video/trans_chunk.rs`：处理转录和分块
- `src/video/trans_chunk_sum.rs`：处理摘要生成
- `src/video/trans_chunk_sum_embed.rs`：处理嵌入向量生成

4. AI 模型调用：

在执行这些任务时，系统会调用各种 AI 模型：

- 使用音频转录模型（如 Whisper）生成转录
- 使用语言模型（如 GPT）生成摘要
- 使用文本嵌入模型生成向量

这些模型的调用通过 `content-base-context` crate 中定义的 `ContentBaseCtx` 进行。

5. 存储处理结果：

处理结果会被存储在文件系统中，路径由 `artifacts_dir` 方法（在 `content-base-context` crate 中）决定。

6. 更新数据库：

处理完成后，系统会更新 SQLite 数据库（通过 Prisma 客户端，在 `prisma` crate 中定义）中的任务状态和元数据。

7. 向量索引：

对于生成的嵌入向量，系统会将其添加到 Qdrant 向量数据库中。这个过程在 `task_post_process` 函数（`src/upsert.rs`）中处理。

8. 通知：

处理过程中，系统会通过 `TaskNotification`（在 `content-base-pool` crate 中定义）发送进度更新。

9. 完成：

所有任务完成后，`upsert` 方法返回，视频导入过程结束。

## 日志

~~可以增加 `RUST_LOG` 环境变量，进行 `debug!` 日志的输出: `RUST_LOG=debug cargo tauri dev`~~

项目使用 `tracing_subscriber` 来配置日志,，api_server 和 tauri 的 main 入口启动的时候通过 `init_tracing` 方法来进行日志格式的初始化，并支持每个 crate 单独配置日志 level，格式是：

```yaml
# 根目录 .env 文件中配置
# 配置单个
RUST_LOG="api_server=debug"
# 配置多个
RUST_LOG="api_server=debug,ai=debug,file_downloader=debug,file_handler=debug,muse_desktop=debug,content_library=debug"
```

打包后的 app 会同时打印日志到 oslog 和 ` ~/Library/Logs/ai.gendam.desktop` 下，oslog 的查看方式是：

```bash
log stream --debug --predicate 'subsystem=="ai.gendam.desktop" and category=="default"'
log stream --type log --level debug | grep "\[ai.gendam.desktop"
log stream --type log --level debug | grep ai.gendam.desktop
```

## File Handler (deprecated)

- video - 视频解码相关，包括抽帧、音频提取等，提供 `VideoHandler` 进行视频文件处理
  - decoder - 解码相关
    - transcode - 音频转码工具函数
    - utils - 相关的工具函数
- audio - 音频转文本（whisper.cpp）
- search_payload - 向量数据库对应的 payload 定义
