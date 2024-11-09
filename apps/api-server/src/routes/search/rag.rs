use super::search::retrieve_assets_for_search;
use crate::{ai::AIHandler, routes::assets::types::FilePathWithAssetObjectData};
use ai::llm::{LLMInferenceParams, LLMMessage};
use content_base::{
    query::{payload::ContentIndexMetadata, ContentQueryPayload},
    ContentBase,
};
use content_library::Library;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RAGRequestPayload {
    pub query: String,
}

#[derive(Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalResultData {
    pub file_path: FilePathWithAssetObjectData,
    pub metadata: ContentIndexMetadata,
    pub score: f32,
    pub reference_content: String, // 检索到的相关内容片段
}

#[derive(Serialize, Type)]
#[serde(tag = "resultType", content = "data")]
pub enum RAGResult {
    Reference(RetrievalResultData),
    Response(String),
    Error(String),
    Done,
}

pub async fn rag(
    library: &Library,
    content_base: &ContentBase,
    ai_handler: &AIHandler,
    input: RAGRequestPayload,
    tx: Sender<RAGResult>,
) -> anyhow::Result<()> {
    let query_payload = ContentQueryPayload {
        query: input.query.clone(),
        with_highlight: false,
        with_reference_content: true,
        ..Default::default()
    };
    let retrieval_results = content_base.query(query_payload).await?;
    let results = retrieve_assets_for_search(library, &retrieval_results, |item, file_path| {
        RetrievalResultData {
            file_path: file_path.clone().into(),
            metadata: item.metadata.clone(),
            score: item.score,
            reference_content: item.reference_content.clone().unwrap_or_default(),
        }
    })
    .await?;

    for ref_item in results.into_iter() {
        tx.send(RAGResult::Reference(ref_item)).await?;
    }

    let references_content = retrieval_results
        .iter()
        .map(|item| item.reference_content.clone())
        .collect::<Vec<_>>();

    let mut reference_content = String::new();
    references_content
        .iter()
        .enumerate()
        // 过滤掉没有内容的文档
        .filter_map(|(idx, v)| v.as_ref().map(|content| (idx, content)))
        .for_each(|(idx, v)| {
            reference_content.push_str(&format!("Document {}:\n{}\n\n", idx + 1, v));
        });

    let system_prompt = r#"You are an assistant good at answer questions according to some pieces from different document.
You should try to answer user question according to the provided document pieces.
Keep your answer ground in the facts of the DOCUMENT.
Try to response in markdown, with proper title, subtitles and bullet points.

If the DOCUMENT doesn't contain the facts to answer the QUESTION, you have 2 options:
- If you know the answer, just response without these information.
- Else, return "I don't know" in the question's language.

You should answer in the language of the QUESTION.
"#;
    let user_prompt = format!(
        r#"DOCUMENTS:
{}

QUESTION:
{}
"#,
        reference_content, &input.query
    );

    let llm = ai_handler.llm.clone();
    let mut response = llm
        .0
        .process_single((
            vec![
                LLMMessage::new_system(system_prompt),
                LLMMessage::new_user(user_prompt.as_str()),
            ],
            LLMInferenceParams::default(),
        ))
        .await?;

    while let Some(content) = response.next().await {
        match content {
            Ok(Some(data)) => {
                tx.send(RAGResult::Response(data)).await?;
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                tx.send(RAGResult::Error(e.to_string())).await?;
            }
        }
    }

    tx.send(RAGResult::Done).await?;

    Ok(())
}
