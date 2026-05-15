use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use axum::response::IntoResponse;
use sqlx::QueryBuilder;
use uuid::Uuid;
use crate::common::api_response::ApiResponse;
use crate::common::app_state::AppState;
use crate::feature::admin::{MoneyHistoryItem, MoneyHistoryQuery, MoneyHistoryResponse, UpdateMoneyRequest};
use crate::feature::conversation::auth::Claims;

pub async fn handle_get_money_history(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(q): Query<MoneyHistoryQuery>,
) -> impl IntoResponse {
    let page = q.page.unwrap_or(1).max(1);
    let limit = 20;
    let offset = (page - 1) * limit;

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Json(ApiResponse::create(401, None::<()>, "Invalid user ID in token")).into_response(),
    };

    let mut qb = QueryBuilder::new(
        r#"
        SELECT 
            mt.id, mt.merchant_name, mt.category, mt.description, mt.total_amount, mt.created_at,
            u.display_name as user_display_name,
            c.title as conversation_title,
            COALESCE(
                jsonb_agg(
                    jsonb_build_object(
                        'name', mti.name,
                        'quantity', mti.quantity,
                        'total_amount', mti.total_amount
                    )
                ) FILTER (WHERE mti.id IS NOT NULL),
                '[]'::jsonb
            ) as items
        FROM money_tracking mt
        LEFT JOIN money_tracking_items mti ON mt.id = mti.money_tracking_id
        LEFT JOIN users u ON mt.user_id = u.id
        LEFT JOIN conversations c ON mt.conversation_id = c.id
        WHERE mt.user_id = 
    "#,
    );
    qb.push_bind(user_id);

    let mut count_qb = QueryBuilder::new("SELECT COUNT(*) FROM money_tracking mt WHERE mt.user_id = ");
    count_qb.push_bind(user_id);

    if let Some(ref search) = q.query {
        if !search.trim().is_empty() {
            let search_term = format!("%{}%", search);
            qb.push(" AND (mt.merchant_name ILIKE ");
            qb.push_bind(search_term.clone());
            qb.push(" OR mt.description ILIKE ");
            qb.push_bind(search_term.clone());
            qb.push(" OR u.display_name ILIKE ");
            qb.push_bind(search_term.clone());
            qb.push(" OR c.title ILIKE ");
            qb.push_bind(search_term.clone());
            qb.push(") ");

            count_qb.push(" AND (mt.merchant_name ILIKE ");
            count_qb.push_bind(search_term.clone());
            count_qb.push(" OR mt.description ILIKE ");
            count_qb.push_bind(search_term.clone());
            count_qb.push(" OR u.display_name ILIKE ");
            count_qb.push_bind(search_term.clone());
            count_qb.push(" OR c.title ILIKE ");
            count_qb.push_bind(search_term.clone());
            count_qb.push(") ");
        }
    }

    if let Some(ref cat) = q.category {
        if !cat.trim().is_empty() {
            qb.push(" AND mt.category = ");
            qb.push_bind(cat.clone());
            count_qb.push(" AND mt.category = ");
            count_qb.push_bind(cat.clone());
        }
    }

    qb.push(" GROUP BY mt.id, u.display_name, c.title ORDER BY mt.created_at DESC LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let count: i64 = match count_qb.build_query_scalar().fetch_one(&state.pool).await {
        Ok(c) => c,
        Err(e) => {
            return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response();
        }
    };

    let items: Vec<MoneyHistoryItem> = match qb.build_query_as().fetch_all(&state.pool).await {
        Ok(items) => items,
        Err(e) => {
            return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response();
        }
    };

    Json(ApiResponse::ok(
        MoneyHistoryResponse {
            items,
            total_count: count,
        },
        "Success",
    ))
        .into_response()
}

pub async fn handle_update_money_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateMoneyRequest>,
) -> impl IntoResponse {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response();
        }
    };

    let result = sqlx::query!(
        "UPDATE money_tracking SET total_amount = COALESCE($1, total_amount), merchant_name = COALESCE($2, merchant_name), category = COALESCE($3, category) WHERE id = $4",
        req.amount,
        req.merchant_name,
        req.category,
        id
    )
        .execute(&mut *tx)
        .await;

    if let Err(e) = result {
        let _ = tx.rollback().await;
        return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response();
    }

    if let Err(e) = tx.commit().await {
        return Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response();
    }

    Json(ApiResponse::ok((), "Transaction updated successfully")).into_response()
}

pub async fn handle_delete_money_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let result = sqlx::query!("DELETE FROM money_tracking WHERE id = $1", id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => Json(ApiResponse::ok((), "Transaction deleted successfully")).into_response(),
        Err(e) => Json(ApiResponse::create(500, None::<()>, &e.to_string())).into_response(),
    }
}
