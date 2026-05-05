pub mod rag_model;

use crate::rag::rag_model::EmbeddingResponse;
use reqwest::Client as ReqwestClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, Pool, Postgres};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub content: String,
    pub similarity: Option<f64>,
}

pub async fn get_embedding(api_key: &str, text: &str) -> anyhow::Result<Vec<f32>, String> {
    let client = ReqwestClient::new();
    // Ensure this is NOT set to 768, or explicitly set to 3072
    let res = client
        .post(format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-embedding-2:embedContent?key={}", api_key))
        .json(&json!({
            "content": {
                "parts": [{ "text": &text }]
            },
            "output_dimensionality":3072}))
        .send().await;

    if let Err(err) = res {
        return Err(format!("{:?}", err));
    }
    let response = res.unwrap();
    let status = response.status();

    if !status.is_success() {
        return Err(format!("{:?}", status));
    }
    let data = response.json::<EmbeddingResponse>().await;
    if let Err(err) = data {
        return Err(format!("{:?}", err));
    }

    Ok(data.unwrap().embedding.values)
}

pub async fn search_similar(
    pool: &Pool<Postgres>,
    embedding: Vec<f32>,
    limit: i32,
) -> anyhow::Result<Vec<SearchResult>> {
    let results = sqlx::query_as!(
        SearchResult,
        r#"
        SELECT content, (1 - (embedding <=> $1::vector)) as similarity
        FROM knowledge_base
        ORDER BY embedding <=> $1::vector
        LIMIT $2
        "#,
        embedding as Vec<f32>,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn search_similar_with_summaries(
    pool: &Pool<Postgres>,
    embedding: Vec<f32>,
    limit: i32,
) -> anyhow::Result<Vec<SearchResult>> {
    // Prioritize summaries by using a union or a conditional ordering
    // Here we fetch both summaries and normal entries, ordered by similarity,
    // but giving a slight boost or priority to summaries if they are relevant enough.
    // For simplicity and directness to requirements, we use a query that handles the metadata filtering.
    let results = sqlx::query_as!(
        SearchResult,
        r#"
        SELECT content, (1 - (embedding <=> $1::vector)) as similarity
        FROM knowledge_base
        ORDER BY 
            (metadata->>'type' = 'summary') DESC,
            embedding <=> $1::vector
        LIMIT $2
        "#,
        embedding as Vec<f32>,
        limit as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

pub async fn save_to_knowledge_base(
    pool: &Pool<Postgres>,
    content: &str,
    embedding: Vec<f32>,
    metadata: Option<serde_json::Value>,
) -> Result<PgQueryResult, Error> {
    sqlx::query!(
        r#"
        INSERT INTO knowledge_base (content, embedding, metadata)
        VALUES ($1, $2, $3)
        "#,
        content,
        embedding as Vec<f32>,
        metadata.unwrap_or(json!({}))
    )
    .execute(pool)
    .await
}
