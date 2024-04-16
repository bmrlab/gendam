# 日志和链路

## 日志

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

## 链路

使用 OpenTelemetry 通过一台中间服务器起来的 gRPC 服务器转发，上报到观测云

使用 span! 的方法一定要加上 #[tracing::instrument]，尤其是 async fn，不然 span 不会串联起来，包括子 span 还有 log 事件。
见 https://docs.rs/tracing/latest/tracing/struct.Span.html#in-asynchronous-code

然后如果用了 #[tracing::instrument]，一般 fn 里面不需要再自己写一个 span! 了，加了 tracing::instrument 的方法会先自动创建一个 span
