use crate::AppState;
use crate::common::api_response::ApiResponse;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::error;
use uuid::Uuid;

pub mod search;
pub use search::handle_search_graph;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub color: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphEdge>,
}


#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    pub conversation_id: Option<Uuid>,
    pub month: Option<u32>,
    pub year: Option<i32>,
}

pub async fn handle_get_graph(
    State(state): State<AppState>,
    Query(query): Query<GraphQuery>,
) -> ApiResponse<GraphData> {
    let conv_id_str = query.conversation_id.map(|id| id.to_string());
    
    // Determine Temporal Window (0 means all-time)
    let month = query.month.unwrap_or(0) as i32;
    let year = query.year.unwrap_or(0);

    // 🧠 TEMPORAL PRUNING: Fetch knowledge nodes with optional monthly filter
    // Increased LIMIT to 300 to ensure we find enough unique nodes across history
    let rows = sqlx::query!(
        r#"
        SELECT metadata->'graph' as graph, metadata->>'type' as entry_type, metadata->>'conversation_id' as conversation_id
        FROM knowledge_base
        WHERE (metadata->>'type' = 'summary' OR metadata->>'type' = 'memory')
        AND metadata->'graph' IS NOT NULL
        AND (
            $1::text IS NULL 
            OR metadata->>'conversation_id' = $1 
            OR metadata->>'conversation_id' IS NULL 
            OR metadata->>'conversation_id' = 'global'
        )
        AND ($2 = 0 OR CAST(EXTRACT(MONTH FROM created_at) AS INTEGER) = $2)
        AND ($3 = 0 OR CAST(EXTRACT(YEAR FROM created_at) AS INTEGER) = $3)
        ORDER BY created_at DESC
        LIMIT 300
        "#,
        conv_id_str,
        month,
        year
    )
    .fetch_all(&state.pool)
    .await;

    let colors = vec![
        "#3b82f6", // blue-500
        "#10b981", // emerald-500
        "#f59e0b", // amber-500
        "#ef4444", // red-500
        "#8b5cf6", // violet-500
        "#ec4899", // pink-500
        "#06b6d4", // cyan-500
        "#f97316", // orange-500
    ];

    match rows {
        Ok(rows) => {
            let mut all_nodes = std::collections::HashMap::new();
            let mut all_links = HashSet::new();

            for row in rows {
                if let Some(graph_val) = row.graph {
                    let entry_type = row.entry_type.unwrap_or_else(|| "summary".to_string());
                    let row_conv_id = row.conversation_id;

                    if let Ok(graph) = serde_json::from_value::<GraphData>(graph_val) {
                        for mut node in graph.nodes {
                            // Ensure node IDs are cleaned up
                            node.id = node.id.trim().to_lowercase().replace(' ', "_");
                            
                            // Skip generic "summary" nodes that AI might have hallucinated
                            if node.id == "summary" || node.node_type.to_lowercase() == "summary" {
                                continue;
                            }

                            // Set conversation_id from metadata if not already set in node
                            if node.conversation_id.is_none() {
                                node.conversation_id = row_conv_id.clone();
                            }

                            // Deterministic color based on ID
                            if node.color.is_none() {
                                if entry_type == "memory" {
                                    node.color = Some("#fbbf24".to_string()); // Golden for memories
                                } else {
                                    let idx = node.id.chars().map(|c| c as usize).sum::<usize>()
                                        % colors.len();
                                    node.color = Some(colors[idx].to_string());
                                }
                            }
                            
                            // Deduplicate by ID
                            if all_nodes.len() < 250 { // Safety Cap: Max 250 unique nodes
                                all_nodes.entry(node.id.clone()).or_insert(node);
                            }
                        }
                        for link in graph.links {
                            let mut link = link;
                            link.source = link.source.trim().to_lowercase().replace(' ', "_");
                            link.target = link.target.trim().to_lowercase().replace(' ', "_");
                            if all_links.len() < 500 { // Safety Cap: Max 500 links
                                all_links.insert(link);
                            }
                        }
                    }
                }
            }

            // Filter links to ensure both source and target exist
            let nodes_vec: Vec<GraphNode> = all_nodes.into_values().collect();
            let node_ids: HashSet<String> = nodes_vec.iter().map(|n| n.id.clone()).collect();
            
            let links_vec: Vec<GraphEdge> = all_links
                .into_iter()
                .filter(|l| node_ids.contains(&l.source) && node_ids.contains(&l.target))
                .collect();

            ApiResponse::ok(
                GraphData {
                    nodes: nodes_vec,
                    links: links_vec,
                },
                "Graph data retrieved",
            )
        }
        Err(e) => {
            error!("Failed to fetch graph data: {}", e);
            ApiResponse::failed("Failed to fetch graph data")
        }
    }
}
