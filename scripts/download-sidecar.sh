# bash scripts/download-sidecar.sh

triple=$(rustc -Vv | grep host | cut -f2 -d' ')
qdrant_version=v1.8.2
sidecar_dir="apps/desktop/src-tauri/sidecar"

curl -L "https://github.com/qdrant/qdrant/releases/download/${qdrant_version}/qdrant-${triple}.tar.gz" | tar xz -C "${sidecar_dir}/"

mv "${sidecar_dir}/qdrant" "${sidecar_dir}/qdrant-${triple}"
