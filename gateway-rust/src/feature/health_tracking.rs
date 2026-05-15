use crate::AppState;
use crate::common::api_response::ApiResponse;
use crate::feature::conversation::auth::Claims;
use axum::{
    extract::{Query, State, Extension},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::chrono::{NaiveDate, DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;
use tracing::{error};

#[derive(Deserialize)]
pub struct HealthSyncRequest {
    pub user_id: Uuid,
    pub log_date: NaiveDate,
    pub metrics: Value,
}

#[derive(Deserialize)]
pub struct HealthHistoryQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Serialize)]
pub struct HealthMetricItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub log_date: NaiveDate,
    pub metrics: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn handle_health_sync(
    State(state): State<AppState>,
    Json(payload): Json<HealthSyncRequest>,
) -> ApiResponse<String> {
    let result = sqlx::query(
        r#"
        INSERT INTO user_health_metrics (user_id, log_date, metrics)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, log_date)
        DO UPDATE SET
            metrics = user_health_metrics.metrics || EXCLUDED.metrics,
            updated_at = NOW()
        "#,
    )
    .bind(payload.user_id)
    .bind(payload.log_date)
    .bind(payload.metrics)
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => ApiResponse::ok("Metrics synced successfully".to_string(), "Success"),
        Err(e) => {
            error!("Failed to sync health metrics: {}", e);
            ApiResponse::failed("Failed to sync metrics")
        }
    }
}

pub async fn handle_get_health_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<HealthHistoryQuery>,
) -> ApiResponse<Vec<HealthMetricItem>> {
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return ApiResponse::failed("Invalid user ID in token"),
    };

    let result = sqlx::query(
        r#"
        SELECT id, user_id, log_date, metrics, created_at, updated_at
        FROM user_health_metrics
        WHERE user_id = $1
          AND ($2::DATE IS NULL OR log_date >= $2)
          AND ($3::DATE IS NULL OR log_date <= $3)
        ORDER BY log_date ASC
        "#,
    )
    .bind(user_id)
    .bind(query.start_date)
    .bind(query.end_date)
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let mut items = Vec::new();
            for row in rows {
                items.push(HealthMetricItem {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    log_date: row.get("log_date"),
                    metrics: row.get("metrics"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                });
            }
            ApiResponse::ok(items, "History retrieved")
        }
        Err(e) => {
            error!("Failed to fetch health history: {}", e);
            ApiResponse::failed("Failed to fetch health history")
        }
    }
}
