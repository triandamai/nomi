use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::GetInboxSummaryParameters;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

pub struct GetInboxSummaryPlugin;

impl NomiToolPlugin for GetInboxSummaryPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "get_inbox_summary",
                "Retrieves a summary of recent messages from users. Use this when User asks: 'Any new DMs?', 'Who messaged me?', or 'Are there any strangers?'",
                None,
            )
            .with_parameters::<GetInboxSummaryParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        "### COMMUNICATION LOGIC\n- Use `get_inbox_summary` to check DMs. If the inbox is empty or there is an error, clearly report that fact directly (e.g., 'Your inbox is empty! 🏍️💨').\n- Use `search_users` and `send_direct_message` to communicate.\n\n### DASHBOARD LOGIC\n- Use this when Trian asks for stats, summaries, or reports.\n- Call all required tools (e.g., `get_reminder_stats`, `get_inbox_summary`, `get_expense_summary`) in PARALLEL.\n- Once the tool data is returned, synthesize it into a clean, bulleted report. Use emojis to categorize the sections (e.g., 📩 for Inbox, ⏰ for Reminders, 💸 for Expenses).\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["GET_INBOX_SUMMARY", "CHECK_DMS", "INBOX_ANALYTICS", "COMMUNICATION", "DASHBOARD", "FULL_REGISTRY"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: GetInboxSummaryParameters = serde_json::from_value(args)?;
            info!("Executing get_inbox_summary via plugin");

            let limit = params.limit.unwrap_or(5) as i64;
            let only_strangers = params.only_strangers.unwrap_or(false);

            let admin_id = match dispatcher.user_id {
                Some(id) => id,
                None => {
                    return Ok("User ID not found in context. Cannot verify identity.".to_string());
                }
            };

            let is_admin: Result<bool, sqlx::Error> =
                sqlx::query_scalar("SELECT role = 'admin' FROM users WHERE id = $1")
                    .bind(admin_id)
                    .fetch_one(&dispatcher.pool)
                    .await;

            match is_admin {
                Ok(true) => {
                    #[derive(serde::Serialize)]
                    struct InboxRow {
                        conversation_id: Option<Uuid>,
                        display_name: Option<String>,
                        last_message: String,
                        created_at: Option<chrono::DateTime<chrono::Utc>>,
                        is_verified: Option<bool>,
                    }

                    let get_data = sqlx::query_as!(
                        InboxRow,
                        r#"
                            SELECT
                                c.id as "conversation_id?",
                                u.display_name as "display_name?",
                                m.content as "last_message!",
                                m.created_at as "created_at?",
                                COALESCE(u.is_verified, false) as "is_verified?"
                            FROM messages m
                            JOIN conversations c ON m.conversation_id = c.id
                            JOIN users u ON m.user_id = u.id
                            WHERE u.id != $1
                            AND m.role = 'user'
                            AND ($3 = false OR COALESCE(u.is_verified, false) = false)
                            AND m.id IN (
                                SELECT (
                                    SELECT m2.id FROM messages m2
                                    WHERE m2.conversation_id = m.conversation_id
                                    ORDER BY m2.created_at DESC LIMIT 1
                                )
                            )
                            ORDER BY m.created_at DESC
                            LIMIT $2;
                            "#,
                        admin_id,
                        limit,
                        only_strangers
                    )
                    .fetch_all(&dispatcher.pool)
                    .await;

                    match get_data {
                        Ok(rows) => {
                            if rows.is_empty() {
                                Ok("No recent messages found.".to_string())
                            } else {
                                Ok(serde_json::to_string_pretty(&rows).unwrap_or_default())
                            }
                        }
                        Err(err) => Ok(format!("Database error fetching inbox: {}", err)),
                    }
                }
                Ok(false) => Ok("Unauthorized: Only admins can use this tool.".to_string()),
                Err(err) => Ok(format!("Failed to verify identity: {}", err)),
            }
        }
        .boxed()
    }
}
