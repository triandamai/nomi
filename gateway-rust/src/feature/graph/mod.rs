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

pub async fn handle_get_workspace_graph(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl axum::response::IntoResponse {
    use sqlx::Row;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let user_uuid = Uuid::parse_str(&claims.sub).unwrap_or_default();
    let is_admin = claims.role == "admin";

    // 1. Fetch Users
    let users_res = if is_admin {
        sqlx::query("SELECT id, display_name, role FROM users LIMIT 10")
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

        // 2. Fetch Health Metrics directly linked to the User
        let health_res = sqlx::query(
            "SELECT log_date, metrics FROM user_health_metrics WHERE user_id = $1 ORDER BY log_date DESC LIMIT 1"
        )
        .bind(user.get::<uuid::Uuid, _>("id"))
        .fetch_optional(&state.pool)
        .await;

        if let Ok(Some(health)) = health_res {
            let health_id = format!("health-{}", user_id);
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
                source: user_id.clone(),
                target: health_id,
                relation: "tracks".to_string(),
            });
        }
    }

    // 3. Fetch Conversations
    let convs_res = if is_admin {
        sqlx::query("SELECT id, user_id, title, conversation_type FROM conversations LIMIT 20")
            .fetch_all(&state.pool)
            .await
    } else {
        sqlx::query(
            "SELECT id, user_id, title, conversation_type FROM conversations \
             WHERE user_id = $1 OR id IN (SELECT conversation_id FROM conversation_members WHERE user_id = $1) LIMIT 20"
        )
        .bind(user_uuid)
        .fetch_all(&state.pool)
        .await
    };

    if let Ok(convs) = convs_res {
        for conv in convs {
            let conv_id = conv.get::<uuid::Uuid, _>("id").to_string();
            let user_id = conv.get::<Option<uuid::Uuid>, _>("user_id").map(|uid| uid.to_string());
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

            // Connect creator to Conversation if user_id is set
            if let Some(ref uid) = user_id {
                edges.push(WorkspaceEdge {
                    source: uid.clone(),
                    target: conv_id.clone(),
                    relation: "member".to_string(),
                });
            }

            // Fetch and connect all conversation members from conversation_members table
            let members_res = sqlx::query("SELECT user_id FROM conversation_members WHERE conversation_id = $1")
                .bind(conv.get::<uuid::Uuid, _>("id"))
                .fetch_all(&state.pool)
                .await;

            if let Ok(members) = members_res {
                for member in members {
                    let member_uid = member.get::<uuid::Uuid, _>("user_id").to_string();
                    if Some(&member_uid) != user_id.as_ref() {
                        edges.push(WorkspaceEdge {
                            source: member_uid,
                            target: conv_id.clone(),
                            relation: "member".to_string(),
                        });
                    }
                }
            }

            // 4. Fetch Reminders and Scheduled Tasks for this conversation
            let reminders_res = sqlx::query(
                "SELECT id, task_type, content, status, due_at FROM reminders WHERE conversation_id = $1 LIMIT 15"
            )
            .bind(conv.get::<uuid::Uuid, _>("id"))
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

                    nodes.push(WorkspaceNode {
                        id: rem_id.clone(),
                        label,
                        node_type,
                        status: Some(status),
                        subtitle: Some(ttype),
                        info: due_at,
                    });

                    edges.push(WorkspaceEdge {
                        source: conv_id.clone(),
                        target: rem_id,
                        relation: "schedules".to_string(),
                    });
                }
            }

            // 5. Fetch Autonomous Tasks for this conversation
            let tasks_res = sqlx::query(
                "SELECT id, title, status, global_goal FROM autonomous_tasks WHERE conversation_id = $1 LIMIT 15"
            )
            .bind(conv.get::<uuid::Uuid, _>("id"))
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
                        source: conv_id.clone(),
                        target: task_id,
                        relation: "spawns".to_string(),
                    });
                }
            }

            // 6. Fetch Money Tracking Transactions for this conversation
            let money_res = sqlx::query(
                "SELECT id, merchant_name, category, total_amount::float8 as amount FROM money_tracking WHERE conversation_id = $1 LIMIT 15"
            )
            .bind(conv.get::<uuid::Uuid, _>("id"))
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
                        source: conv_id.clone(),
                        target: money_id,
                        relation: "logs".to_string(),
                    });
                }
            }

            // 7. Fetch Channels mapped to this conversation
            let channels_res = sqlx::query(
                "SELECT id, channel_type, external_id, external_chat_id, user_id FROM channels WHERE conversation_id = $1 LIMIT 10"
            )
            .bind(conv.get::<uuid::Uuid, _>("id"))
            .fetch_all(&state.pool)
            .await;

            if let Ok(channels) = channels_res {
                for chan in channels {
                    let chan_id = chan.get::<uuid::Uuid, _>("id").to_string();
                    let chan_type = chan.get::<String, _>("channel_type");
                    let ext_id = chan.get::<String, _>("external_id");
                    let ext_chat_id = chan.get::<String, _>("external_chat_id");
                    let chan_user_id = chan.get::<Option<uuid::Uuid>, _>("user_id").map(|uid| uid.to_string());

                    nodes.push(WorkspaceNode {
                        id: chan_id.clone(),
                        label: format!("Channel: {}", chan_type.to_uppercase()),
                        node_type: "CHANNEL".to_string(),
                        status: Some("connected".to_string()),
                        subtitle: Some(ext_id),
                        info: Some(ext_chat_id),
                    });

                    edges.push(WorkspaceEdge {
                        source: conv_id.clone(),
                        target: chan_id.clone(),
                        relation: "bridges".to_string(),
                    });

                    if let Some(uid) = chan_user_id {
                        edges.push(WorkspaceEdge {
                            source: uid,
                            target: chan_id,
                            relation: "owns".to_string(),
                        });
                    }
                }
            }
        }
    }

    // 8. Fetch SRP Proposals (Plugin suggestions) from staging
    let srp_res = sqlx::query(
        "SELECT slug, name, status, description FROM plugin_creation_suggestions ORDER BY created_at DESC LIMIT 15"
    )
    .fetch_all(&state.pool)
    .await;

    if let Ok(proposals) = srp_res {
        for prop in proposals {
            use sqlx::Row;
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

            // Connect current logged-in user to the proposal they/system suggested
            edges.push(WorkspaceEdge {
                source: user_uuid.to_string(),
                target: prop_node_id,
                relation: "proposes".to_string(),
            });
        }
    }

    axum::Json(ApiResponse::ok(
        WorkspaceGraphData { nodes, edges },
        "Workspace graph compiled successfully"
    ))
}
