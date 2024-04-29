## Getting Started

### 日志

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

### 开发环境准备

```bash
pnpm dev:prep
```

会依次执行
- `bash scripts/download-sidecar.sh` 下载 sidecars: qdrant, ffmpeg, ffprobe, whisper 等
- `cargo prisma generate` 生成 prisma 的代码 crates/prisma/src/prisma.rs
- `pnpm tauri build --debug` 仅用于复制 tauri 下的 sidecar 和 resources 到 target/debug，供单独运行 api_server 用

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

## File Handler

- video - 视频解码相关，包括抽帧、音频提取等，提供 `VideoHandler` 进行视频文件处理
  - decoder - 解码相关
    - transcode - 音频转码工具函数
    - utils - 相关的工具函数
- audio - 音频转文本（whisper.cpp）
- search_payload - 向量数据库对应的 payload 定义


### Update subtree quaint
```bash
git subtree pull --prefix=crates/quaint quaint main --squash
```
