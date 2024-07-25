use ai::tokenizers::Tokenizer;

pub fn naive_chunk<T>(
    items: &[T],
    tokenizer: &Tokenizer,
    chunk_size: usize,
) -> anyhow::Result<Vec<Vec<T>>>
where
    T: ToString + Clone,
{
    let mut chunks: Vec<Vec<T>> = vec![];
    //                      👇 token count
    let mut buffer: Vec<(T, usize)> = vec![];
    let mut buffer_token_count = 0;

    let buffer_to_chunk =
        |buffer: &Vec<(T, usize)>| buffer.iter().map(|v| v.0.clone()).collect::<Vec<_>>();

    for item in items.iter() {
        let current_token_count = tokenizer
            .encode(item.to_string().as_str(), false)
            .map_err(|e| anyhow::anyhow!("failed to tokenize: {}", e))?
            .len();

        if current_token_count + buffer_token_count > chunk_size {
            // save current buffer to chunks
            let chunk = buffer_to_chunk(&buffer);
            chunks.push(chunk);

            // and reduce buffer to half of the chunk_size
            while buffer_token_count > chunk_size / 2 {
                let first_item = buffer.remove(0);
                buffer_token_count -= first_item.1;
            }
        }

        // push current item to buffer
        buffer.push((item.clone(), current_token_count));
        buffer_token_count += current_token_count;
    }

    // push remaining content to chunks
    if buffer.len() > 0 {
        let chunk = buffer_to_chunk(&buffer);
        chunks.push(chunk);
    }

    Ok(chunks)
}
