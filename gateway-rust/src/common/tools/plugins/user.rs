use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::{build_follow_up_prompt, ToolDispatcher};
use crate::common::tools::tools_model::ToolResult;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use sqlx::{Postgres, Row};
use tracing::info;

pub struct UserPlugin;

impl NomiToolPlugin for UserPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "manage_user",
            "description": "Manage users: search for users or update user profile information (display name, name, or email).",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["search", "update"],
                        "description": "The type of user operation to perform."
                    },
                    "query": {
                        "type": "string",
                        "description": "General search query for search action."
                    },
                    "user_id": {
                        "type": "string",
                        "description": "Specific User ID to filter or update."
                    },
                    "name": {
                        "type": "string",
                        "description": "Username/Name to filter or update."
                    },
                    "display_name": {
                        "type": "string",
                        "description": "Display name to filter or update."
                    },
                    "email": {
                        "type": "string",
                        "description": "Email to filter or update."
                    },
                    "user_message": {
                        "type": "string",
                        "description": "The original user message to provide context."
                    }
                },
                "required": ["action", "user_message"]
            }
        })
    }

    fn matching_intents(&self) -> &[&str] {
        &["COMMUNICATION", "DASHBOARD", "GENERAL"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let action = args["action"].as_str().unwrap_or_default();
            let user_id = match dispatcher.user_id {
                Some(id) => id,
                None => return Ok("User not authenticated".to_string()),
            };

            let user_message = args["user_message"].as_str().unwrap_or_default();

            match action {
                "search" => {
                    let result = self.handle_search(dispatcher, &args, user_message).await;
                    Ok(serde_json::to_string(&result)?)
                }
                "update" => {
                    let result = self.handle_update(dispatcher, user_id, &args, user_message).await;
                    Ok(serde_json::to_string(&result)?)
                }
                _ => Ok(format!("Unknown action: {}", action)),
            }
        }
        .boxed()
    }
}

impl UserPlugin {
    async fn handle_search(
        &self,
        dispatcher: &ToolDispatcher,
        args: &Value,
        user_message: &str,
    ) -> ToolResult {
        info!("Searching users with plugin params: {:?}", args);

        let query_str = args["query"].as_str().unwrap_or_default();
        
        let results = if query_str.starts_with('@') {
            let id_part = &query_str[1..];
            let wa_lid = format!("{}@lid", id_part);
            let wa_jid = format!("{}@s.whatsapp.net", id_part);
            let tg_id = id_part.to_string();

            sqlx::query(
                "SELECT DISTINCT u.id, u.name as username, u.display_name, u.email \
                 FROM users u \
                 JOIN channels c ON u.id = c.user_id \
                 WHERE c.external_chat_id = $1 OR c.external_chat_id = $2 OR c.external_chat_id = $3 \
                 LIMIT 20"
            )
            .bind(wa_lid)
            .bind(wa_jid)
            .bind(tg_id)
            .fetch_all(&dispatcher.pool)
            .await
        } else {
            let mut query_builder = sqlx::QueryBuilder::<Postgres>::new(
                "SELECT id, name as username, display_name, email FROM users WHERE 1=1"
            );

            let mut has_filter = false;

            if !query_str.is_empty() {
                let pattern = format!("%{}%", query_str);
                query_builder.push(" AND (name ILIKE ");
                query_builder.push_bind(pattern.clone());
                query_builder.push(" OR display_name ILIKE ");
                query_builder.push_bind(pattern.clone());
                query_builder.push(" OR email ILIKE ");
                query_builder.push_bind(pattern.clone());
                query_builder.push(" OR id::text ILIKE ");
                query_builder.push_bind(pattern);
                query_builder.push(")");
                has_filter = true;
            }

            if let Some(uid) = args["user_id"].as_str() {
                query_builder.push(" AND id::text ILIKE ");
                query_builder.push_bind(format!("%{}%", uid));
                has_filter = true;
            }

            if let Some(name) = args["name"].as_str() {
                query_builder.push(" AND name ILIKE ");
                query_builder.push_bind(format!("%{}%", name));
                has_filter = true;
            }

            if let Some(dn) = args["display_name"].as_str() {
                query_builder.push(" AND display_name ILIKE ");
                query_builder.push_bind(format!("%{}%", dn));
                has_filter = true;
            }

            if let Some(email) = args["email"].as_str() {
                query_builder.push(" AND email ILIKE ");
                query_builder.push_bind(format!("%{}%", email));
                has_filter = true;
            }

            if !has_filter {
                return ToolResult {
                    error: "No search criteria provided".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }

            query_builder.push(" LIMIT 20");
            query_builder.build().fetch_all(&dispatcher.pool).await
        };

        match results {
            Ok(rows) => {
                if rows.is_empty() {
                    return ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: "No users found".to_string(),
                        follow_up_prompt: build_follow_up_prompt(
                            user_message.to_string(),
                            "No users found".to_string(),
                            "manage_user".to_string(),
                        ),
                    };
                }

                let mut summary = String::new();
                for row in rows {
                    let id: uuid::Uuid = row.get("id");
                    let username: Option<String> = row.get("username");
                    let display_name: Option<String> = row.get("display_name");
                    let email: Option<String> = row.get("email");

                    summary.push_str(&format!(
                        "- ID: {}, Username: {}, Display: {}, Email: {}\n",
                        id,
                        username.as_deref().unwrap_or("N/A"),
                        display_name.as_deref().unwrap_or("N/A"),
                        email.as_deref().unwrap_or("N/A")
                    ));
                }

                let content = format!("Found {} users:\n{}", summary.lines().count(), summary);

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message.to_string(),
                        content,
                        "manage_user".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Database error searching users: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn handle_update(
        &self,
        dispatcher: &ToolDispatcher,
        current_user_id: uuid::Uuid,
        args: &Value,
        user_message: &str,
    ) -> ToolResult {
        info!("Updating user profile with plugin params: {:?}", args);

        let target_id = if let Some(uid) = args["user_id"].as_str() {
            match uuid::Uuid::parse_str(uid) {
                Ok(id) => id,
                Err(_) => return ToolResult {
                    error: "Invalid User ID format".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                },
            }
        } else {
            current_user_id
        };

        // Security check: Only allow updating self unless admin
        if target_id != current_user_id {
             let is_admin: Result<bool, sqlx::Error> =
                sqlx::query_scalar("SELECT role = 'admin' FROM users WHERE id = $1")
                    .bind(current_user_id)
                    .fetch_one(&dispatcher.pool)
                    .await;
            
            if !matches!(is_admin, Ok(true)) {
                return ToolResult {
                    error: "Permission denied: Cannot update other user profiles".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        }

        let mut query_builder = sqlx::QueryBuilder::<Postgres>::new("UPDATE users SET ");
        let mut separated = query_builder.separated(", ");
        let mut has_update = false;

        if let Some(dn) = args["display_name"].as_str() {
            separated.push("display_name = ");
            separated.push_bind(dn);
            has_update = true;
        }

        if let Some(name) = args["name"].as_str() {
            separated.push("name = ");
            separated.push_bind(name);
            has_update = true;
        }

        if let Some(email) = args["email"].as_str() {
            separated.push("email = ");
            separated.push_bind(email);
            has_update = true;
        }

        if !has_update {
            return ToolResult {
                error: "No update fields provided".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            };
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(target_id);

        let result = query_builder.build().execute(&dispatcher.pool).await;

        match result {
            Ok(_) => {
                let content = format!("Successfully updated profile for user {}", target_id);
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message.to_string(),
                        content,
                        "manage_user".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Database error updating user: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }
}
