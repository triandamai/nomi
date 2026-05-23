use crate::common::agent::agent_model::ExpenseData;
use chrono::{Datelike, Utc};
use uuid::Uuid;

pub(crate) async fn log_expense_transaction(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    conversation_id: Option<Uuid>,
    data: &ExpenseData,
) -> anyhow::Result<Uuid> {
    let mut tx = pool.begin().await?;

    // 1. Get or create category
    let category_id = sqlx::query!(
        "SELECT id FROM categories WHERE slug = $1 OR name = $1 LIMIT 1",
        data.category.to_lowercase()
    )
    .fetch_optional(&mut *tx)
    .await?
    .map(|r| r.id);

    // 2. Insert main record
    let record = sqlx::query!(
        r#"
        INSERT INTO money_tracking (
            user_id, conversation_id, category_id, category, merchant_name,
            total_amount, tax, service, discount
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        user_id,
        conversation_id,
        category_id,
        data.category,
        data.merchant,
        rust_decimal::Decimal::from_f64_retain(data.total).unwrap_or_default(),
        data.tax
            .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default()),
        data.service
            .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default()),
        data.discount
            .map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or_default())
    )
    .fetch_one(&mut *tx)
    .await?;

    // 3. Insert items
    for item in &data.items {
        sqlx::query!(
            r#"
            INSERT INTO money_tracking_items (money_tracking_id, name, quantity, total_amount)
            VALUES ($1, $2, $3, $4)
            "#,
            record.id,
            item.name,
            item.quantity,
            rust_decimal::Decimal::from_f64_retain(item.amount).unwrap_or_default()
        )
        .execute(&mut *tx)
        .await?;
    }

    // 4. Update Daily/Monthly Summary
    let _now = Utc::now();
    let period_start = Utc::now()
        .with_day(1)
        .unwrap()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO money_tracking_summary (user_id, period, total_expenses, total_income)
        VALUES ($1, $2, $3, 0)
        ON CONFLICT (user_id, period) DO UPDATE
        SET total_expenses = money_tracking_summary.total_expenses + EXCLUDED.total_expenses,
            updated_at = now()
        "#,
        user_id,
        period_start.date(),
        rust_decimal::Decimal::from_f64_retain(data.total).unwrap_or_default()
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(record.id)
}
