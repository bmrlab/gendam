# for dev use only !
# tauri 会根据环境来判断 externalBin 的路径，release 环境是在 target 目录下，dev 环境保持原来的 src-tauri/sidecar 目录
# 为了 api-server 和 tauri 都可以使用 externalBin，所以需要将他们复制到拷贝到 target 目录下，都使用 ./ 相对路径来访问
# 用 `pnpm tauri build --debug` 可以完成同样的目的，但是太慢了

triple=$(rustc -Vv | grep host | cut -f2 -d' ')
if [ $# -eq 1 ]; then
  triple=$1
fi

sidecar_dir="apps/desktop/src-tauri/sidecar"

mkdir -p "target/debug"

for file in qdrant ffmpeg ffprobe whisper; do
  cp "${sidecar_dir}/${file}-${triple}" "target/debug/${file}"
  # chmod +x "target/debug/${file}"
done
