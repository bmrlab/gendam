# bash scripts/download-models.sh

resources_dir="apps/desktop/src-tauri/resources"

if [ ! -d "${resources_dir}/puff-base-v1" ]; then
  curl -L "https://gendam.s3.us-west-1.amazonaws.com/models/puff-base-v1.tar.gz" | tar xz -C "${resources_dir}/"
fi

if [ ! -d "${resources_dir}/qwen2" ]; then
  curl -L "https://gendam.s3.us-west-1.amazonaws.com/models/qwen2.tar.gz" | tar xz -C "${resources_dir}/"
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
