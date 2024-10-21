pub const LLAVA_PHI3_CONFIG: super::llava::LLaVAPhi3Config = super::llava::LLaVAPhi3Config {
    eos_token_id: 32007, // 模型实际输出的是 32007 <|end|>, 而不是 gguf 里配置的 32000 <|endoftext|>
    bos_token_id: 1,
    image_token_id: 32038, // see tokenizer.json, <image>
    // from metadata of llava-phi-3-mini-mmproj-f16.gguf
    clip_vision_config: super::clip::ClipVisionConfig {
        activation: super::clip::Activation::QuickGelu,
        hidden_size: 1024,       // clip.vision.embedding_length
        intermediate_size: 4096, // clip.vision.feed_forward_length
        num_hidden_layers: 23,   // clip.vision.block_count
        num_attention_heads: 16, // clip.vision.attention.head_count
        projection_dim: 768,     // clip.vision.projection_dim
        num_channels: 3,
        image_size: 336,
        patch_size: 14,
    },
    // from metadata of llava-phi-3-mini-int4.gguf
    llama_text_config: super::quantized_llama::LlamaTextConfig {
        vocab_size: 32064,                     // llama.vocab_size
        max_length: 4096,                      // llama.context_length
        hidden_size: 3072,                     // llama.embedding_length
        intermediate_size: 8192,               // llama.feed_forward_length
        num_hidden_layers: 32,                 // llama.block_count
        num_attention_heads: 32,               // llama.attention.head_count
        num_key_value_heads: 32,               // llama.attention.head_count_kv
        rms_norm_eps: 0.000009999999747378752, // llama.attention.layer_norm_rms_epsilon
    },
};
