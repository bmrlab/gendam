# bash scripts/download-models.sh

resources_dir="apps/desktop/src-tauri/resources"

if [ ! -d "${resources_dir}/stella-base-zh-v3-1792d" ]; then
  curl -L "https://gendam.s3.us-west-1.amazonaws.com/models/stella-base-zh-v3-1792d.tar.gz" | tar xz -C "${resources_dir}/"
fi

if [ ! -d "${resources_dir}/whisper" ]; then
  curl -L "https://gendam.s3.us-west-1.amazonaws.com/models/whisper.tar.gz" | tar xz -C "${resources_dir}/"
fi

if [ ! -d "${resources_dir}/CLIP-ViT-B-32-multilingual-v1" ]; then
  curl -L "https://gendam.s3.us-west-1.amazonaws.com/models/CLIP-ViT-B-32-multilingual-v1.tar.gz" | tar xz -C "${resources_dir}/"
fi

if [ ! -d "${resources_dir}/blip-base" ]; then
  curl -L "https://gendam.s3.us-west-1.amazonaws.com/models/blip-base.tar.gz" | tar xz -C "${resources_dir}/"
fi
