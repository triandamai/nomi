use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use tracing::error;
use crate::common::app_state::AppState;

pub async fn handle_get_reminders(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
    axum::extract::Query(params): axum::extract::Query<
        crate::feature::conversation::model::MessageListParams,
    >,
) -> crate::common::api_response::ApiResponse<
    Vec<crate::feature::conversation::model::ReminderResponse>,
> {
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return crate::common::api_response::ApiResponse::failed("Invalid user ID in token");
        }
    };

    let limit = params.limit.unwrap_or(20);
    // Use a very far future date as default cursor for DESC sort
    let cursor = params
        .cursor
        .unwrap_or_else(|| Utc::now() + chrono::Duration::days(365 * 10));

    let result = sqlx::query!(
        r#"
        SELECT
            r.id,
            r.task_type as "task_type!",
            r.payload as "payload!",
            COALESCE(r.payload->>'message', r.content) as "content!",
            (r.due_at AT TIME ZONE 'Asia/Jakarta') as due_at,
            r.frequency,
            r.status,
            u.display_name as "user_display_name",
            c.title as "conversation_title",
            r.created_at
        FROM reminders r
        LEFT JOIN users u ON r.user_id = u.id
        LEFT JOIN conversations c ON r.conversation_id = c.id
        WHERE r.user_id = $1 AND r.due_at < $2
        ORDER BY r.due_at DESC
        LIMIT $3
        "#,
        user_id,
        cursor,
        limit
    )
        .fetch_all(&state.pool)
        .await;

    match result {
        Ok(rows) => {
            let tz: Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
            let reminders = rows
                .into_iter()
                .map(|r| crate::feature::conversation::model::ReminderResponse {
                    id: r.id,
                    task_type: r.task_type,
                    payload: r.payload,
                    content: r.content,
                    due_at: tz
                        .from_local_datetime(&r.due_at.unwrap())
                        .single()
                        .unwrap()
                        .with_timezone(&Utc),
                    frequency: r.frequency,
                    status: r.status.unwrap_or_default(),
                    user_display_name: r.user_display_name,
                    conversation_title: r.conversation_title,
                    created_at: r.created_at.unwrap_or_else(Utc::now),
                })
                .collect();
            crate::common::api_response::ApiResponse::ok(reminders, "Tasks retrieved")
        }
        Err(e) => {
            error!("Failed to fetch reminders: {}", e);
            crate::common::api_response::ApiResponse::failed("Failed to fetch reminders")
        }
    }
}

pub async fn handle_get_reminder_detail(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<crate::feature::conversation::auth::Claims>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> crate::common::api_response::ApiResponse<crate::feature::conversation::model::ReminderResponse> {
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return crate::common::api_response::ApiResponse::failed("Invalid user ID in token");
        }
    };

    let result = sqlx::query!(
        r#"
        SELECT
            r.id,
            r.task_type as "task_type!",
            r.payload as "payload!",
            COALESCE(r.payload->>'message', r.content) as "content!",
            (r.due_at AT TIME ZONE 'Asia/Jakarta') as due_at,
            r.frequency,
            r.status,
            u.display_name as "user_display_name",
            c.title as "conversation_title",
            r.created_at
        FROM reminders r
        LEFT JOIN users u ON r.user_id = u.id
        LEFT JOIN conversations c ON r.conversation_id = c.id
        WHERE r.id = $1 AND r.user_id = $2
        "#,
        id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await;

    match result {
        Ok(Some(r)) => {
            let tz: Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
            let reminder = crate::feature::conversation::model::ReminderResponse {
                id: r.id,
                task_type: r.task_type,
                payload: r.payload,
                content: r.content,
                due_at: tz
                    .from_local_datetime(&r.due_at.unwrap())
                    .single()
                    .unwrap()
                    .with_timezone(&Utc),
                frequency: r.frequency,
                status: r.status.unwrap_or_default(),
                user_display_name: r.user_display_name,
                conversation_title: r.conversation_title,
                created_at: r.created_at.unwrap_or_else(Utc::now),
            };
            crate::common::api_response::ApiResponse::ok(reminder, "Task retrieved")
        }
        Ok(None) => crate::common::api_response::ApiResponse::not_found("Reminder not found"),
        Err(e) => {
            error!("Failed to fetch reminder detail: {}", e);
            crate::common::api_response::ApiResponse::failed("Failed to fetch reminder detail")
        }
    }
}
