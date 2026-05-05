use crate::AppState;
use crate::common::api_response::ApiResponse;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResultNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
}

pub async fn handle_search_graph(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> ApiResponse<Vec<SearchResultNode>> {
    let search_pattern = format!("%{}%", params.q);

    // Search for nodes in metadata->'graph'->'nodes' and in the content itself
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT 
            node->>'id' as id, 
            node->>'label' as label, 
            node->>'node_type' as node_type
        FROM knowledge_base,
             jsonb_array_elements(metadata->'graph'->'nodes') AS node
        WHERE (node->>'label' ILIKE $1 OR node->>'id' ILIKE $1 OR content ILIKE $1)
        LIMIT 10
        "#,
        search_pattern
    )
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let mut results = Vec::new();
            let mut seen_ids = HashSet::new();

            for row in rows {
                if let (Some(id), Some(label), Some(node_type)) = (row.id, row.label, row.node_type)
                {
                    if seen_ids.insert(id.clone()) {
                        results.push(SearchResultNode {
                            id,
                            label,
                            node_type,
                        });
                    }
                }
            }

            ApiResponse::ok(results, "Search results retrieved")
        }
        Err(e) => {
            error!("Failed to search graph: {}", e);
            ApiResponse::failed("Failed to search graph")
        }
    }
}
