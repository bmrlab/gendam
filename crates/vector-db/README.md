# Muse Vector Database

**⚠️ 暂时不使用该 crate**

使用时，首先需要安装依赖 (此处仅针对 Apple Silicon 的 Mac)

```shell
brew install llvm libomp
```

并需要补充一些 cargo 配置在项目的 `.cargo/config.toml` 中

```toml
[env]
OpenMP_ROOT = "/opt/homebrew/opt/libomp"

[target.aarch64-apple-darwin]
rustflags = "-L/opt/homebrew/opt/llvm/lib"
```
