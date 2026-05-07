use sqlx::{Pool, Postgres};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HybridSearchResult {
    pub content: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub vector_score: f64,
    pub keyword_score: f64,
    pub final_score: f64,
}

pub async fn hybrid_retrieve(
    pool: &Pool<Postgres>,
    query_text: &str,
    embedding: Vec<f32>,
    conversation_id: Option<uuid::Uuid>,
) -> anyhow::Result<Vec<String>> {
    // Task 1: Hybrid Vector + Keyword Search
    // Fetch candidates using both vector similarity and keyword search
    // Using a FULL OUTER JOIN to combine results and scores
    let rows = sqlx::query!(
        r#"
        WITH vector_candidates AS (
            SELECT id, content, metadata, created_at, 
                   (1.0 - (embedding <=> $1::vector))::float8 as score
            FROM knowledge_base
            WHERE ($3::uuid IS NULL OR conversation_id IS NULL OR conversation_id = $3)
            ORDER BY embedding <=> $1::vector
            LIMIT 15
        ),
        keyword_candidates AS (
            SELECT id, content, metadata, created_at, 
                   ts_rank_cd(to_tsvector('english', content), plainto_tsquery('english', $2))::float8 as score
            FROM knowledge_base
            WHERE to_tsvector('english', content) @@ plainto_tsquery('english', $2)
              AND ($3::uuid IS NULL OR conversation_id IS NULL OR conversation_id = $3)
            LIMIT 15
        )
        SELECT 
            COALESCE(v.id, k.id) as "id!",
            COALESCE(v.content, k.content) as "content!",
            COALESCE(v.metadata, k.metadata) as "metadata!",
            COALESCE(v.created_at, k.created_at) as "created_at!",
            COALESCE(v.score, 0.0)::float8 as "vector_score!",
            COALESCE(k.score, 0.0)::float8 as "keyword_score!"
        FROM vector_candidates v
        FULL OUTER JOIN keyword_candidates k ON v.id = k.id
        "#,
        embedding as Vec<f32>,
        query_text,
        conversation_id
    )
    .fetch_all(pool)
    .await?;

    let now = Utc::now();
    let seven_days_ago = now - Duration::days(7);

    let mut results: Vec<HybridSearchResult> = rows.into_iter().map(|row| {
        // Basic weighted score
        let mut final_score = (row.vector_score * 0.7) + (row.keyword_score * 0.3);
        
        // Task 3: Memory Recency Bias
        // Score Boost for newer entries (within 7 days)
        if row.created_at > seven_days_ago {
            final_score *= 1.2;
        }
        
        // Score Boost for 'Manual Memories' (type='memory')
        if row.metadata.get("type").and_then(|t| t.as_str()) == Some("memory") {
            final_score *= 1.1;
        }

        HybridSearchResult {
            content: row.content,
            metadata: row.metadata,
            created_at: row.created_at,
            vector_score: row.vector_score,
            keyword_score: row.keyword_score,
            final_score,
        }
    }).collect();

    // Task 1 (cont): Re-ranking
    // Sort by the final calculated score
    results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap_or(std::cmp::Ordering::Equal));
    
    // Task 4: Deduplication
    // Ensure the same information isn't sent multiple times
    let mut unique_results = Vec::new();
    let mut seen_content = HashSet::new();
    
    for res in results {
        let normalized = res.content.trim().to_lowercase();
        // Simple deduplication based on content prefix/hash could be better, 
        // but for now, exact match after trim and lowercase.
        if !seen_content.contains(&normalized) {
            seen_content.insert(normalized);
            unique_results.push(res);
        }
        if unique_results.len() >= 5 {
            break;
        }
    }

    // Task 2: Graph-Hop (Context Expansion)
    let mut final_context = Vec::new();
    let mut related_facts = Vec::new();
    let mut root_node_ids = HashSet::new();

    for res in &unique_results {
        final_context.push(res.content.clone());
        
        // Extract nodes from metadata if they exist
        if let Some(graph) = res.metadata.get("graph") {
            if let Some(nodes) = graph.get("nodes").and_then(|n| n.as_array()) {
                for node in nodes {
                    if let Some(id) = node.get("id").and_then(|i| i.as_str()) {
                        root_node_ids.insert(id.to_string());
                    }
                }
            }
        }
    }

    // Pull 1-hop neighbors (Relationship Facts)
    if !root_node_ids.is_empty() {
        let node_ids: Vec<String> = root_node_ids.into_iter().collect();
        let neighbors = sqlx::query!(
            r#"
            SELECT DISTINCT
                link->>'source' as "source!",
                link->>'target' as "target!",
                link->>'relationship' as "relationship!"
            FROM knowledge_base,
                 jsonb_array_elements(metadata->'graph'->'links') AS link
            WHERE link->>'source' = ANY($1) OR link->>'target' = ANY($1)
            LIMIT 15
            "#,
            &node_ids
        )
        .fetch_all(pool)
        .await?;

        for n in neighbors {
            related_facts.push(format!("Fact: {} --({})--> {}", n.source, n.relationship, n.target));
        }
    }

    if !related_facts.is_empty() {
        // Deduplicate related facts
        let mut unique_facts: Vec<String> = related_facts.into_iter().collect();
        unique_facts.sort();
        unique_facts.dedup();
        
        final_context.push(format!("
### Relationship Facts (Graph Context)
{}", unique_facts.join("
")));
    }

    // Tracing logs
    for (i, res) in unique_results.iter().enumerate() {
        info!(
            "RAG Fragment {}: FinalScore={:.4} (Vec={:.4}, Key={:.4}), Type={:?}, Date={}",
            i + 1,
            res.final_score,
            res.vector_score,
            res.keyword_score,
            res.metadata.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"),
            res.created_at.to_rfc3339()
        );
    }

    Ok(final_context)
}
