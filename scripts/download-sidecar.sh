# bash scripts/download-sidecar.sh

triple=$(rustc -Vv | grep host | cut -f2 -d' ')
if [ $# -eq 1 ]; then
  triple=$1
fi

sidecar_dir="apps/desktop/src-tauri/sidecar"
mkdir -p "${sidecar_dir}"

qdrant_version=v1.8.2
if [ ! -f "${sidecar_dir}/qdrant-${triple}" ]; then
  curl -L "https://github.com/qdrant/qdrant/releases/download/${qdrant_version}/qdrant-${triple}.tar.gz" | tar xz -C "${sidecar_dir}/"
  mv "${sidecar_dir}/qdrant" "${sidecar_dir}/qdrant-${triple}"
fi
chmod +x "${sidecar_dir}/qdrant-${triple}"

ffmpeg_version=6.1.1
if [ ! -f "${sidecar_dir}/ffmpeg-${triple}" ]; then
  curl -L "https://evermeet.cx/ffmpeg/ffmpeg-${ffmpeg_version}.zip" | tar xz -C "${sidecar_dir}/"
  mv "${sidecar_dir}/ffmpeg" "${sidecar_dir}/ffmpeg-${triple}"
fi
chmod +x "${sidecar_dir}/ffmpeg-${triple}"

ffprobe_version=6.1.1
if [ ! -f "${sidecar_dir}/ffprobe-${triple}" ]; then
  curl -L "https://evermeet.cx/ffmpeg/ffprobe-${ffprobe_version}.zip" | tar xz -C "${sidecar_dir}/"
  mv "${sidecar_dir}/ffprobe" "${sidecar_dir}/ffprobe-${triple}"
fi
chmod +x "${sidecar_dir}/ffprobe-${triple}"

if [ ! -f "${sidecar_dir}/whisper-${triple}" ]; then
  curl -L "https://tezign-ai-models.oss-cn-beijing.aliyuncs.com/whisper/main" -o "${sidecar_dir}/whisper-${triple}"
fi
chmod +x "${sidecar_dir}/whisper-${triple}"
