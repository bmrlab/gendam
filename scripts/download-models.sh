# bash scripts/download-models.sh

resources_dir="apps/desktop/src-tauri/resources"

if [ ! -f "${resources_dir}/stella-base-zh-v3-1792d" ]; then
  curl -L "https://assets.bmr.art/gendam/models/stella-base-zh-v3-1792d.tar.gz" | tar xz -C "${resources_dir}/"
fi

if [ ! -f "${resources_dir}/whisper" ]; then
  curl -L "https://assets.bmr.art/gendam/models/whisper.tar.gz" | tar xz -C "${resources_dir}/"
fi

if [ ! -f "${resources_dir}/CLIP-ViT-B-32-multilingual-v1" ]; then
  curl -L "https://assets.bmr.art/gendam/models/CLIP-ViT-B-32-multilingual-v1.tar.gz" | tar xz -C "${resources_dir}/"
fi

if [ ! -f "${resources_dir}/blip-base" ]; then
  curl -L "https://assets.bmr.art/gendam/models/blip-base.tar.gz" | tar xz -C "${resources_dir}/"
fi
