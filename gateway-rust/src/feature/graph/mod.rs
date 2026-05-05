use crate::AppState;
use crate::common::api_response::ApiResponse;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::error;

pub mod search;
pub use search::handle_search_graph;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub color: Option<String>,
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

pub async fn handle_get_graph(State(state): State<AppState>) -> ApiResponse<GraphData> {
    let rows = sqlx::query!(
        r#"
        SELECT metadata->'graph' as graph, metadata->>'type' as entry_type
        FROM knowledge_base
        WHERE (metadata->>'type' = 'summary' OR metadata->>'type' = 'memory')
        AND metadata->'graph' IS NOT NULL
        "#
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
                    if let Ok(graph) = serde_json::from_value::<GraphData>(graph_val) {
                        for mut node in graph.nodes {
                            // Ensure node IDs are cleaned up
                            node.id = node.id.trim().to_lowercase().replace(' ', "_");
                            
                            // Skip generic "summary" nodes that AI might have hallucinated
                            if node.id == "summary" || node.node_type.to_lowercase() == "summary" {
                                continue;
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
                            
                            // Deduplicate by ID, keep the first one or merge labels if needed
                            all_nodes.entry(node.id.clone()).or_insert(node);
                        }
                        for link in graph.links {
                            let mut link = link;
                            link.source = link.source.trim().to_lowercase().replace(' ', "_");
                            link.target = link.target.trim().to_lowercase().replace(' ', "_");
                            all_links.insert(link);
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
