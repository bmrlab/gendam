This is a [Next.js](https://nextjs.org/) project bootstrapped with [`create-next-app`](https://github.com/vercel/next.js/tree/canary/packages/create-next-app).

## Getting Started

可以增加 `RUST_LOG` 环境变量，进行 `debug!` 日志的输出:
`RUST_LOG=debug cargo tauri dev`

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

```
generator client {
  provider = "cargo prisma"
  output = "src/prisma/mod.rs"
}
```

4. 执行 `cargo prisma generate` 生成 artifacts (e.g. Prisma Client), 根据上一步配置, 生成在 `src/prisma` 目录下

5. `cargo prisma migrate dev` 以后，在代码里直接引入 `PrismaClient` 开始使用

## File Handler

`file_handler` 当前作为一个简单的 lib 进行文件处理和检索，后续可以拆分为一个单独的 crate

- embedding - embedding 生成模块
  - clip - CLIP 模型
  - blip - BLIP 模型
  - preprocess - 相关的预处理器
  - utils - 相关的工具函数
- video - 视频解码相关，包括抽帧、音频提取等，提供 `VideoHandler` 进行视频文件处理
  - decoder - 解码相关
    - transcode - 音频转码工具函数
    - utils - 相关的工具函数
- audio - 音频转文本（whisper.cpp）
- search_payload - 向量数据库对应的 payload 定义

**可以按照下面的示例进行视频文件的处理👇视频文件处理完后结果会存入local_data_dir并写入向量数据库**

### Examples

```rust
// VideoHandler 初始化时会自动进行模型的下载
// 这里我们都用 tauri 提供的路径进行产品的存放
let video_handler = file_handler::video::VideoHandler::new(
    video_path,
    // 生产产物的存放路径
    // 最终会为单个视频文件在存放路径下生成一个sha256为名称的文件夹
    // 后续产物均存在这个文件夹中
    app_handle
        .path_resolver()
        .app_local_data_dir()
        .expect("failed to find local data dir"),
    // 模型等资源文件的存放路径
    app_handle
        .path_resolver()
        .resolve_resource("resources")
        .expect("failed to find resources dir"),
)
.await
.expect("failed to initialize video handler");

debug!("video handler initialized");

// 使用tokio spawn一个帧相关的异步任务
let vh = video_handler.clone();
let frame_handle = tokio::spawn(async move {
    // `get_frames` 对视频进行抽帧
    match vh.get_frames().await {
        // 抽帧成功后提取图像 embedding，并写入向量数据库
        Ok(_) => match vh.get_frame_content_embedding().await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("failed to get frame content embedding: {}", e);
                Err(e)
            }
        },
        Err(e) => {
            debug!("failed to get frames: {}", e);
            Err(e)
        }
    }
});

// 使用tokio spawn一个音频相关的异步任务
let vh = video_handler.clone();
let audio_handle = tokio::spawn(async move {
    // `get_audio` 提取音频
    match vh.get_audio().await {
        // `get_transcript` 使用whisper提取音频
        Ok(_) => match vh.get_transcript().await {
            Ok(_) => {
                // 根据提取的transcript提取文本特征
                let res = vh.get_transcript_embedding().await;

                if let Err(e) = res {
                    error!("failed to get transcript embedding: {}", e);
                    Err(e)
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                error!("failed to get audio embedding: {}", e);
                Err(e)
            }
        },
        Err(e) => {
            error!("failed to get audio: {}", e);
            Err(e)
        }
    }
});

// 注意因为使用了tokio::spawn
// 任务会同时进行
// 这里做一下等待
let frame_results = frame_handle.await;
let audio_results = audio_handle.await;

if let Err(frame_err) = frame_results.unwrap() {
    error!("failed to get frames: {}", frame_err);
    return Err(format!("failed to get frames: {}", frame_err));
}
if let Err(audio_err) = audio_results.unwrap() {
    error!("failed to get audio: {}", audio_err);
    return Err(format!("failed to get frames: {}", audio_err));
}
```

### 向量数据库（Qdrant）

视频处理结果均会存储在向量数据库中，启动项目后向量数据库会自动启动并创建相关 collection （默认的向量数据库名为`muse-v2`）

向量数据库相关文件也会被存储在 tauri 提供的 `app_handle.path_resolver().resolve_resource("resources")` 中

#### `file_handler` 提供的 `handle_search`

`file_handler` 提供了一个 `handle_search` 函数用于简单的结果查询，具体实现了以下逻辑：

- 输入文本通过 CLIP 编码
- 按照`record_type`和文本 embedding 查询结果并返回到前端

其中`record_type`用于进行结果类型的筛选，包括：仅搜索帧图像内容 (`Frame`)、仅搜索音频对应的文本内容 (`Transcript`)、全部搜索 (`None`)

使用示例

```rust
// resource_dir 为模型文件存放路径
file_handler::handle_search(file_handler::SearchRequest {
  text: "a man".to_string(),
  record_type: Some(file_handler::search_payload::SearchRecordType::Frame),
  skip: None,
  limit: None
}, resources_dir).await
```

#### 常用查询

查看 collection 状态（如当前有多少个数据点）

```bash
curl  -X GET 'http://localhost:6333/collections/muse-v2'
```

使用 filter 查询所有向量

```bash
curl  -X POST \
  'http://localhost:6333/collections/muse-v2/points/scroll' \
  --header 'Content-Type: application/json' \
  --data-raw '{
  "offset": 0,
  "limit": 10,
  "filter": {
    "should": [
      {
        "is_empty": {
          "key": "Frame"
        }
      }
    ]
  }
}'
```

### 相关 TODO

- [ ] 模型应该通过一个thread启动，然后通过channel丢入数据，再拿到返回结果（参照 spacedrive 的模式）
- [ ] 向量数据库的 payload 过滤代码还有待优化，包括枚举值的自动生成以及payload的格式（现在直接用SearchPayload转为json，多了一层没有用的数据）
- [ ] 整体处理速度、代码拆分逻辑还有待讨论和实现
