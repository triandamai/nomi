use crate::AppState;
use crate::common::api_response::ApiResponse;
use axum::extract::{Query, State, Extension};
use crate::feature::conversation::auth::Claims;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceNode {
    pub id: String,
    pub label: String,
    pub node_type: String, // "USER" | "CONVERSATION" | "REMINDER" | "SCHEDULED_TASK" | "AUTONOMOUS_TASK" | "MONEY" | "HEALTH"
    pub status: Option<String>,
    pub subtitle: Option<String>,
    pub info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceGraphData {
    pub nodes: Vec<WorkspaceNode>,
    pub edges: Vec<WorkspaceEdge>,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceQuery {
    pub user_id: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn handle_get_workspace_graph(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<WorkspaceQuery>,
) -> impl axum::response::IntoResponse {
    use sqlx::Row;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let user_uuid = Uuid::parse_str(&claims.sub).unwrap_or_default();
    let is_admin = claims.role == "admin";

    let user_id = query.user_id.filter(|s| !s.is_empty());
    let category = query.category.filter(|s| !s.is_empty());

    match (user_id, category) {
        // --- LEVEL 1: Fetch Users Only ---
        (None, _) => {
            let users_res = if is_admin {
                let limit = query.limit.unwrap_or(10);
                let offset = query.offset.unwrap_or(0);
                sqlx::query("SELECT id, display_name, role FROM users LIMIT $1 OFFSET $2")
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&state.pool)
                    .await
            } else {
                sqlx::query("SELECT id, display_name, role FROM users WHERE id = $1")
                    .bind(user_uuid)
                    .fetch_all(&state.pool)
                    .await
            };

            let users = match users_res {
                Ok(u) => u,
                Err(e) => {
                    error!("Workspace Graph: Failed to fetch users: {}", e);
                    return axum::Json(ApiResponse::failed("Failed to fetch graph"));
                }
            };

            for user in &users {
                let user_id = user.get::<uuid::Uuid, _>("id").to_string();
                let display_name = user.get::<Option<String>, _>("display_name").unwrap_or_else(|| "User".to_string());
                let role = user.get::<Option<String>, _>("role").unwrap_or_else(|| "user".to_string());

                nodes.push(WorkspaceNode {
                    id: user_id.clone(),
                    label: display_name,
                    node_type: "USER".to_string(),
                    status: Some("active".to_string()),
                    subtitle: Some(format!("Role: {}", role)),
                    info: None,
                });
            }

            // Add static independent "Nomi System" node
            nodes.push(WorkspaceNode {
                id: "system-node".to_string(),
                label: "Nomi System".to_string(),
                node_type: "SYSTEM".to_string(),
                status: Some("active".to_string()),
                subtitle: Some("Core Platform".to_string()),
                info: Some("Autonomous intelligence orchestrator".to_string()),
            });
        }

        // --- LEVEL 2: Fetch Categories for specific User ---
        (Some(query_user_id), None) => {
            if query_user_id == "system-node" {
                // The Nomi System node contains independent SRP proposals!
                nodes.push(WorkspaceNode {
                    id: "category-system-node-SRP_PROPOSAL".to_string(),
                    label: "SRP Proposals".to_string(),
                    node_type: "CATEGORY_SRP_PROPOSAL".to_string(),
                    status: Some("expanded".to_string()),
                    subtitle: Some("Folder".to_string()),
                    info: None,
                });

                edges.push(WorkspaceEdge {
                    source: "system-node".to_string(),
                    target: "category-system-node-SRP_PROPOSAL".to_string(),
                    relation: "contains".to_string(),
                });
            } else {
                let user_uuid = Uuid::parse_str(&query_user_id).unwrap_or_default();
                let mut categories = Vec::new();

                // 1. HEALTH
                if sqlx::query("SELECT 1 FROM user_health_metrics WHERE user_id = $1 LIMIT 1")
                    .bind(user_uuid)
                    .fetch_optional(&state.pool)
                    .await
                    .map(|opt| opt.is_some())
                    .unwrap_or(false)
                {
                    categories.push("HEALTH");
                }

                // 2. CONVERSATION
                if sqlx::query("SELECT 1 FROM conversations WHERE user_id = $1 OR id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1) LIMIT 1")
                    .bind(user_uuid)
                    .fetch_optional(&state.pool)
                    .await
                    .map(|opt| opt.is_some())
                    .unwrap_or(false)
                {
                    categories.push("CONVERSATION");
                }

                // 3. CHANNEL
                if sqlx::query("SELECT 1 FROM channels WHERE conversation_id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1 UNION SELECT id FROM conversations WHERE user_id = $1) LIMIT 1")
                    .bind(user_uuid)
                    .fetch_optional(&state.pool)
                    .await
                    .map(|opt| opt.is_some())
                    .unwrap_or(false)
                {
                    categories.push("CHANNEL");
                }

                // 4. AUTONOMOUS_TASK
                let has_tasks = sqlx::query(
                    "SELECT 1 FROM autonomous_tasks WHERE conversation_id IN ( \
                     SELECT conversation_id FROM conversation_members WHERE user_id = $1 \
                     UNION \
                     SELECT id FROM conversations WHERE user_id = $1 \
                     ) LIMIT 1"
                )
                .bind(user_uuid)
                .fetch_optional(&state.pool)
                .await
                .map(|opt| opt.is_some())
                .unwrap_or(false);
                if has_tasks {
                    categories.push("AUTONOMOUS_TASK");
                }

                // 5. REMINDER / SCHEDULED_TASK
                let has_reminders = sqlx::query("SELECT 1 FROM reminders WHERE conversation_id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1 UNION SELECT id FROM conversations WHERE user_id = $1) LIMIT 1")
                    .bind(user_uuid)
                    .fetch_optional(&state.pool)
                    .await
                    .map(|opt| opt.is_some())
                    .unwrap_or(false);
                if has_reminders {
                    categories.push("REMINDER");
                    categories.push("SCHEDULED_TASK");
                }

                // 6. MONEY
                if sqlx::query("SELECT 1 FROM money_tracking WHERE conversation_id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1 UNION SELECT id FROM conversations WHERE user_id = $1) LIMIT 1")
                    .bind(user_uuid)
                    .fetch_optional(&state.pool)
                    .await
                    .map(|opt| opt.is_some())
                    .unwrap_or(false)
                {
                    categories.push("MONEY");
                }

                for cat in categories {
                    nodes.push(WorkspaceNode {
                        id: format!("category-{}-{}", query_user_id, cat),
                        label: format!("{}s", cat.replace('_', " ")),
                        node_type: format!("CATEGORY_{}", cat),
                        status: Some("expanded".to_string()),
                        subtitle: Some("Folder".to_string()),
                        info: None,
                    });

                    edges.push(WorkspaceEdge {
                        source: query_user_id.clone(),
                        target: format!("category-{}-{}", query_user_id, cat),
                        relation: "contains".to_string(),
                    });
                }
            }
        }

        // --- LEVEL 3: Fetch items belonging to Category ---
        (Some(query_user_id), Some(query_category)) => {
            let user_uuid = Uuid::parse_str(&query_user_id).unwrap_or_default();

            if query_category == "HEALTH" {
                let health_res = sqlx::query(
                    "SELECT log_date, metrics FROM user_health_metrics WHERE user_id = $1 ORDER BY log_date DESC LIMIT 1"
                )
                .bind(user_uuid)
                .fetch_optional(&state.pool)
                .await;

                if let Ok(Some(health)) = health_res {
                    let health_id = format!("health-{}", query_user_id);
                    let log_date = health.get::<chrono::NaiveDate, _>("log_date").to_string();
                    let metrics_val = health.get::<serde_json::Value, _>("metrics");
                    
                    let mut summary = "Active Biometrics".to_string();
                    if let Some(steps) = metrics_val.get("steps") {
                        summary = format!("Steps: {}", steps);
                    }

                    nodes.push(WorkspaceNode {
                        id: health_id.clone(),
                        label: "Health & Vitality".to_string(),
                        node_type: "HEALTH".to_string(),
                        status: Some("synced".to_string()),
                        subtitle: Some(summary),
                        info: Some(log_date),
                    });

                    edges.push(WorkspaceEdge {
                        source: format!("category-{}-HEALTH", query_user_id),
                        target: health_id,
                        relation: "item".to_string(),
                    });
                }
            } else if query_category == "CONVERSATION" {
                let convs_res = sqlx::query(
                    "SELECT id, title, conversation_type FROM conversations \
                     WHERE user_id = $1 OR id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1) LIMIT 20"
                )
                .bind(user_uuid)
                .fetch_all(&state.pool)
                .await;

                if let Ok(convs) = convs_res {
                    for conv in convs {
                        let conv_id = conv.get::<uuid::Uuid, _>("id").to_string();
                        let title = conv.get::<Option<String>, _>("title").unwrap_or_else(|| "General Chat".to_string());
                        let ctype = conv.get::<Option<String>, _>("conversation_type").unwrap_or_else(|| "private".to_string());

                        nodes.push(WorkspaceNode {
                            id: conv_id.clone(),
                            label: title,
                            node_type: "CONVERSATION".to_string(),
                            status: Some("open".to_string()),
                            subtitle: Some(ctype.to_uppercase()),
                            info: None,
                        });

                        edges.push(WorkspaceEdge {
                            source: format!("category-{}-CONVERSATION", query_user_id),
                            target: conv_id,
                            relation: "item".to_string(),
                        });
                    }
                }
            } else if query_category == "SRP_PROPOSAL" {
                let srp_res = sqlx::query(
                    "SELECT slug, name, status, description FROM plugin_creation_suggestions ORDER BY created_at DESC LIMIT 15"
                )
                .fetch_all(&state.pool)
                .await;

                if let Ok(proposals) = srp_res {
                    for prop in proposals {
                        let slug = prop.get::<String, _>("slug");
                        let name = prop.get::<String, _>("name");
                        let status = prop.get::<String, _>("status");
                        let desc = prop.get::<Option<String>, _>("description").unwrap_or_default();
                        let prop_node_id = format!("srp-{}", slug);

                        nodes.push(WorkspaceNode {
                            id: prop_node_id.clone(),
                            label: name,
                            node_type: "SRP_PROPOSAL".to_string(),
                            status: Some(status),
                            subtitle: Some(format!("SRP: {}", slug)),
                            info: Some(desc),
                        });

                        edges.push(WorkspaceEdge {
                            source: format!("category-{}-SRP_PROPOSAL", query_user_id),
                            target: prop_node_id,
                            relation: "item".to_string(),
                        });
                    }
                }
            } else if query_category == "AUTONOMOUS_TASK" {
                let conv_ids_res = sqlx::query(
                    "SELECT id FROM conversations WHERE user_id = $1 OR id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1)"
                )
                .bind(user_uuid)
                .fetch_all(&state.pool)
                .await;

                if let Ok(convs) = conv_ids_res {
                    for conv in convs {
                        let conv_id = conv.get::<uuid::Uuid, _>("id");
                        let tasks_res = sqlx::query(
                            "SELECT id, title, status, global_goal FROM autonomous_tasks WHERE conversation_id = $1 LIMIT 15"
                        )
                        .bind(conv_id)
                        .fetch_all(&state.pool)
                        .await;

                        if let Ok(tasks) = tasks_res {
                            for task in tasks {
                                let task_id = task.get::<uuid::Uuid, _>("id").to_string();
                                let title = task.get::<Option<String>, _>("title").unwrap_or_else(|| "Autonomous Task".to_string());
                                let status = task.get::<Option<String>, _>("status").unwrap_or_else(|| "running".to_string());
                                let goal = task.get::<Option<String>, _>("global_goal").unwrap_or_else(|| "".to_string());

                                nodes.push(WorkspaceNode {
                                    id: task_id.clone(),
                                    label: title,
                                    node_type: "AUTONOMOUS_TASK".to_string(),
                                    status: Some(status),
                                    subtitle: Some("HTO Loop".to_string()),
                                    info: Some(goal),
                                });

                                edges.push(WorkspaceEdge {
                                    source: format!("category-{}-AUTONOMOUS_TASK", query_user_id),
                                    target: task_id,
                                    relation: "item".to_string(),
                                });
                            }
                        }
                    }
                }
            } else if query_category == "REMINDER" || query_category == "SCHEDULED_TASK" {
                let conv_ids_res = sqlx::query(
                    "SELECT id FROM conversations WHERE user_id = $1 OR id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1)"
                )
                .bind(user_uuid)
                .fetch_all(&state.pool)
                .await;

                if let Ok(convs) = conv_ids_res {
                    for conv in convs {
                        let conv_id = conv.get::<uuid::Uuid, _>("id");
                        let reminders_res = sqlx::query(
                            "SELECT id, task_type, content, status, due_at FROM reminders WHERE conversation_id = $1 LIMIT 15"
                        )
                        .bind(conv_id)
                        .fetch_all(&state.pool)
                        .await;

                        if let Ok(reminders) = reminders_res {
                            for reminder in reminders {
                                let rem_id = reminder.get::<uuid::Uuid, _>("id").to_string();
                                let ttype = reminder.get::<Option<String>, _>("task_type").unwrap_or_else(|| "REMINDER".to_string());
                                let content = reminder.get::<Option<String>, _>("content").unwrap_or_else(|| "".to_string());
                                let status = reminder.get::<Option<String>, _>("status").unwrap_or_else(|| "pending".to_string());
                                let due_at = reminder.get::<Option<chrono::DateTime<chrono::Utc>>, _>("due_at")
                                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string());

                                let (node_type, label) = if ttype == "REMINDER" || ttype == "SEND_DM" {
                                    ("REMINDER".to_string(), format!("Reminder: {}", content))
                                } else {
                                    ("SCHEDULED_TASK".to_string(), format!("Schedule: {}", content))
                                };

                                if node_type == query_category {
                                    nodes.push(WorkspaceNode {
                                        id: rem_id.clone(),
                                        label,
                                        node_type,
                                        status: Some(status),
                                        subtitle: Some(ttype),
                                        info: due_at,
                                    });

                                    edges.push(WorkspaceEdge {
                                        source: format!("category-{}-{}", query_user_id, query_category),
                                        target: rem_id,
                                        relation: "item".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            } else if query_category == "MONEY" {
                let conv_ids_res = sqlx::query(
                    "SELECT id FROM conversations WHERE user_id = $1 OR id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1)"
                )
                .bind(user_uuid)
                .fetch_all(&state.pool)
                .await;

                if let Ok(convs) = conv_ids_res {
                    for conv in convs {
                        let conv_id = conv.get::<uuid::Uuid, _>("id");
                        let money_res = sqlx::query(
                            "SELECT id, merchant_name, category, total_amount::float8 as amount FROM money_tracking WHERE conversation_id = $1 LIMIT 15"
                        )
                        .bind(conv_id)
                        .fetch_all(&state.pool)
                        .await;

                        if let Ok(money_items) = money_res {
                            for money in money_items {
                                let money_id = money.get::<uuid::Uuid, _>("id").to_string();
                                let merchant = money.get::<Option<String>, _>("merchant_name").unwrap_or_else(|| "Transaction".to_string());
                                let category = money.get::<Option<String>, _>("category").unwrap_or_else(|| "Other".to_string());
                                
                                let amount = money.get::<Option<f64>, _>("amount").unwrap_or(0.0);
                                let amount_str = format!("${:.2}", amount);

                                nodes.push(WorkspaceNode {
                                    id: money_id.clone(),
                                    label: merchant,
                                    node_type: "MONEY".to_string(),
                                    status: Some("logged".to_string()),
                                    subtitle: Some(category),
                                    info: Some(amount_str),
                                });

                                edges.push(WorkspaceEdge {
                                    source: format!("category-{}-MONEY", query_user_id),
                                    target: money_id,
                                    relation: "item".to_string(),
                                });
                            }
                        }
                    }
                }
            } else if query_category == "CHANNEL" {
                let conv_ids_res = sqlx::query(
                    "SELECT id FROM conversations WHERE user_id = $1 OR id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1)"
                )
                .bind(user_uuid)
                .fetch_all(&state.pool)
                .await;

                if let Ok(convs) = conv_ids_res {
                    for conv in convs {
                        let conv_id = conv.get::<uuid::Uuid, _>("id");
                        let channels_res = sqlx::query(
                            "SELECT id, channel_type, external_id, external_chat_id FROM channels WHERE conversation_id = $1 LIMIT 10"
                        )
                        .bind(conv_id)
                        .fetch_all(&state.pool)
                        .await;

                        if let Ok(channels) = channels_res {
                            for chan in channels {
                                let chan_id = chan.get::<uuid::Uuid, _>("id").to_string();
                                let chan_type = chan.get::<String, _>("channel_type");
                                let ext_id = chan.get::<String, _>("external_id");
                                let ext_chat_id = chan.get::<String, _>("external_chat_id");

                                nodes.push(WorkspaceNode {
                                    id: chan_id.clone(),
                                    label: format!("Channel: {}", chan_type.to_uppercase()),
                                    node_type: "CHANNEL".to_string(),
                                    status: Some("connected".to_string()),
                                    subtitle: Some(ext_id),
                                    info: Some(ext_chat_id),
                                });

                                edges.push(WorkspaceEdge {
                                    source: format!("category-{}-CHANNEL", query_user_id),
                                    target: chan_id,
                                    relation: "item".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    axum::Json(ApiResponse::ok(
        WorkspaceGraphData { nodes, edges },
        "Workspace graph compiled successfully"
    ))
}
