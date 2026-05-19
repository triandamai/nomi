use crate::common::agent::agent_model::{ExpenseData, ExpenseItem};
use crate::common::agent::classification::log_expense_transaction;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::{build_follow_up_prompt, ToolDispatcher};
use crate::common::tools::tools_model::ToolResult;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use chrono::{Datelike, FixedOffset, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;

pub struct FinancePlugin;

impl NomiToolPlugin for FinancePlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "manage_finance",
            "description": "Retrieve financial summaries, transaction details, or log a new expense. Use 'get_summary' for totals over a period (today, yesterday, last_7_days, this_month, last_month), 'get_details' for specific transaction lists on a date, and 'log_expense' to record a new spending.",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["get_summary", "get_details", "log_expense"],
                        "description": "The type of finance operation to perform."
                    },
                    "period": {
                        "type": "string",
                        "description": "The time period for the summary (used with get_summary). e.g., 'today', 'yesterday', 'last_7_days', 'this_month', 'last_month'."
                    },
                    "date": {
                        "type": "string",
                        "description": "Specific date in YYYY-MM-DD format, or 'today', 'yesterday' (used with get_details)."
                    },
                    "merchant": {
                        "type": "string",
                        "description": "Merchant name (used with log_expense)."
                    },
                    "total": {
                        "type": "number",
                        "description": "Total expense amount (used with log_expense)."
                    },
                    "category": {
                        "type": "string",
                        "description": "Expense category (used with log_expense)."
                    },
                    "tax": {
                        "type": "number",
                        "description": "Tax amount (optional, used with log_expense)."
                    },
                    "service": {
                        "type": "number",
                        "description": "Service charge (optional, used with log_expense)."
                    },
                    "discount": {
                        "type": "number",
                        "description": "Discount amount (optional, used with log_expense)."
                    },
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "quantity": { "type": "integer" },
                                "amount": { "type": "number" }
                            }
                        },
                        "description": "List of items in the expense (optional, used with log_expense)."
                    },
                    "user_message": {
                        "type": "string",
                        "description": "The original user message to provide context"
                    }
                },
                "required": ["action", "user_message"]
            }
        })
    }

    fn matching_intents(&self) -> &[&str] {
        &["FINANCE", "DASHBOARD","MONEY_TRACKING","EXPENSE"]
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
                "get_summary" => {
                    let period = args["period"].as_str().unwrap_or("this_month");
                    let result = self.handle_get_expense_summary(dispatcher, user_id, period, user_message).await;
                    Ok(serde_json::to_string(&result)?)
                }
                "get_details" => {
                    let date_str = args["date"].as_str().unwrap_or("today");
                    let result = self.handle_get_transaction_details(dispatcher, user_id, date_str, user_message).await;
                    Ok(serde_json::to_string(&result)?)
                }
                "log_expense" => {
                    let result = self.handle_log_expense(dispatcher, user_id, &args, user_message).await;
                    Ok(serde_json::to_string(&result)?)
                }
                _ => Ok(format!("Unknown action: {}", action)),
            }
        }
        .boxed()
    }
}

impl FinancePlugin {
    async fn handle_log_expense(
        &self,
        dispatcher: &ToolDispatcher,
        user_id: uuid::Uuid,
        args: &Value,
        user_message: &str,
    ) -> ToolResult {
        if dispatcher.conversation_id.is_none() {
            return ToolResult {
                error: "Conversation ID not found".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            };
        }

        let expense_data = ExpenseData {
            merchant: args["merchant"].as_str().unwrap_or_default().to_string(),
            total: args["total"].as_f64().unwrap_or_default(),
            tax: args["tax"].as_f64(),
            service: args["service"].as_f64(),
            discount: args["discount"].as_f64(),
            items: args["items"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|i| ExpenseItem {
                    name: i["name"].as_str().unwrap_or_default().to_string(),
                    quantity: i["quantity"].as_i64().unwrap_or(1) as i32,
                    amount: i["amount"].as_f64().unwrap_or_default(),
                })
                .collect(),
            category: args["category"].as_str().unwrap_or("General").to_string(),
        };

        match log_expense_transaction(
            &dispatcher.pool,
            user_id,
            dispatcher.conversation_id,
            &expense_data,
        )
        .await
        {
            Ok(_) => {
                if let Some(cid) = dispatcher.conversation_id {
                    let _ = crate::common::repository::message_repo::mark_last_media_processed(&dispatcher.pool, cid).await;
                }
                let content = format!(
                    "Expense of {} at {} logged successfully under {}. Attached image linked and cleared from pending queue.",
                    expense_data.total, expense_data.merchant, expense_data.category
                );
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message.to_string(),
                        content,
                        "manage_finance".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Failed to log expense: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn handle_get_expense_summary(
        &self,
        dispatcher: &ToolDispatcher,
        user_id: uuid::Uuid,
        period: &str,
        user_message: &str,
    ) -> ToolResult {
        let now_wib = Utc::now().with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap());

        let (start_date, end_date) = match period {
            "today" => {
                let start = now_wib.date_naive().and_hms_opt(0, 0, 0).unwrap();
                let end = now_wib.date_naive().and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            }
            "yesterday" => {
                let yesterday = now_wib.date_naive() - chrono::Duration::days(1);
                let start = yesterday.and_hms_opt(0, 0, 0).unwrap();
                let end = yesterday.and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            }
            "last_7_days" => {
                let start_date = now_wib.date_naive() - chrono::Duration::days(6);
                let start = start_date.and_hms_opt(0, 0, 0).unwrap();
                let end = now_wib.date_naive().and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            }
            "last_month" => {
                let month = if now_wib.month() == 1 { 12 } else { now_wib.month() - 1 };
                let year = if now_wib.month() == 1 { now_wib.year() - 1 } else { now_wib.year() };
                let start = NaiveDate::from_ymd_opt(year, month, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let next_month = if month == 12 { 1 } else { month + 1 };
                let next_month_year = if month == 12 { year + 1 } else { year };
                let end = NaiveDate::from_ymd_opt(next_month_year, next_month, 1).unwrap().and_hms_opt(0, 0, 0).unwrap() - chrono::Duration::seconds(1);
                (start, end)
            }
            _ => {
                let start = NaiveDate::from_ymd_opt(now_wib.year(), now_wib.month(), 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let end = now_wib.date_naive().and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            }
        };

        let start_tz = start_date.and_local_timezone(FixedOffset::east_opt(7 * 3600).unwrap()).unwrap();
        let end_tz = end_date.and_local_timezone(FixedOffset::east_opt(7 * 3600).unwrap()).unwrap();

        let is_monthly = period == "this_month" || period == "last_month";
        let mut total_expenses = 0.0;
        let mut total_income = 0.0;
        let mut summary_found = false;

        if is_monthly {
            let period_start_date = start_tz.date_naive();
            if let Ok(Some(row)) = sqlx::query!(
                "SELECT total_expenses, total_income FROM money_tracking_summary WHERE user_id = $1 AND period = $2",
                user_id,
                period_start_date
            )
            .fetch_optional(&dispatcher.pool)
            .await
            {
                total_expenses = row.total_expenses.unwrap_or_default().to_f64().unwrap_or(0.0);
                total_income = row.total_income.unwrap_or_default().to_f64().unwrap_or(0.0);
                summary_found = total_expenses > 0.0 || total_income > 0.0;
            }
        }

        let mut top_category = None;
        let mut trend_percentage = None;

        if !summary_found {
            if let Ok(sum_row) = sqlx::query!(
                "SELECT SUM(total_amount) as total_expenses
                 FROM money_tracking
                 WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3",
                user_id,
                start_tz,
                end_tz
            )
            .fetch_one(&dispatcher.pool)
            .await
            {
                total_expenses = sum_row.total_expenses.unwrap_or_default().to_f64().unwrap_or(0.0);

                if total_expenses > 0.0 {
                    let cat_row = sqlx::query!(
                        "SELECT category
                         FROM money_tracking
                         WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3
                         GROUP BY category
                         ORDER BY SUM(total_amount) DESC LIMIT 1",
                        user_id,
                        start_tz,
                        end_tz
                    )
                    .fetch_optional(&dispatcher.pool)
                    .await
                    .unwrap_or(None);

                    top_category = cat_row.and_then(|r| r.category);
                }
            }
        }

        let duration = end_tz.signed_duration_since(start_tz);
        let actual_duration = duration + chrono::Duration::seconds(1);
        let prev_end_tz = start_tz - chrono::Duration::seconds(1);
        let prev_start_tz = start_tz - actual_duration;

        if let Ok(prev_sum_row) = sqlx::query!(
            "SELECT SUM(total_amount) as total_expenses
             FROM money_tracking
             WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3",
            user_id,
            prev_start_tz,
            prev_end_tz
        )
        .fetch_one(&dispatcher.pool)
        .await
        {
            let prev_total = prev_sum_row.total_expenses.unwrap_or_default().to_f64().unwrap_or(0.0);
            if prev_total > 0.0 {
                trend_percentage = Some(((total_expenses - prev_total) / prev_total) * 100.0);
            }
        }

        if total_expenses == 0.0 {
            return ToolResult {
                error: "".to_string(),
                success: true,
                content: format!("Zero spending for {}! 💸✨", period),
                follow_up_prompt: "".to_string(),
            };
        }

        let json_result = json!({
            "total_expenses": total_expenses,
            "total_income": total_income,
            "top_category": top_category,
            "trend_percentage": trend_percentage
        });

        ToolResult {
            error: "".to_string(),
            success: true,
            content: json_result.to_string(),
            follow_up_prompt: build_follow_up_prompt(user_message.to_string(), json_result.to_string(), "manage_finance".to_string()),
        }
    }

    async fn handle_get_transaction_details(
        &self,
        dispatcher: &ToolDispatcher,
        user_id: uuid::Uuid,
        date_str: &str,
        user_message: &str,
    ) -> ToolResult {
        let now_wib = Utc::now().with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap());
        
        let target_date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                if date_str == "today" {
                    now_wib.date_naive()
                } else if date_str == "yesterday" {
                    now_wib.date_naive().pred_opt().unwrap_or(now_wib.date_naive())
                } else {
                    now_wib.date_naive()
                }
            }
        };

        let start_tz = target_date.and_hms_opt(0, 0, 0).unwrap()
            .and_local_timezone(FixedOffset::east_opt(7 * 3600).unwrap()).unwrap();
        let end_tz = target_date.and_hms_opt(23, 59, 59).unwrap()
            .and_local_timezone(FixedOffset::east_opt(7 * 3600).unwrap()).unwrap();

        let mut transactions = Vec::new();
        let mut total_day_amount = 0.0;

        let rows = sqlx::query!(
            r#"
            SELECT 
                mt.id, 
                mt.merchant_name, 
                mt.total_amount, 
                mt.category, 
                mt.description, 
                mt.created_at as "created_at!",
                COALESCE(
                    jsonb_agg(
                        jsonb_build_object(
                            'name', mti.name,
                            'quantity', mti.quantity,
                            'total_amount', mti.total_amount
                        )
                    ) FILTER (WHERE mti.id IS NOT NULL),
                    '[]'::jsonb
                ) as "items!"
            FROM money_tracking mt
            LEFT JOIN money_tracking_items mti ON mt.id = mti.money_tracking_id
            WHERE mt.user_id = $1 AND mt.created_at >= $2 AND mt.created_at <= $3
            GROUP BY mt.id
            ORDER BY mt.created_at DESC
            "#,
            user_id,
            start_tz,
            end_tz
        )
        .fetch_all(&dispatcher.pool)
        .await;

        if let Ok(rows) = rows {
            for row in rows {
                let amount = row.total_amount.to_f64().unwrap_or(0.0);
                let created_at = row.created_at.with_timezone(&FixedOffset::east_opt(7 * 3600).unwrap()).to_rfc3339();

                total_day_amount += amount;

                let items: Vec<crate::common::tools::tools_model::TransactionItem> = serde_json::from_value(row.items).unwrap_or_default();

                transactions.push(crate::common::tools::tools_model::TransactionDetail {
                    merchant_name: row.merchant_name,
                    total_amount: amount,
                    category: row.category,
                    description: row.description,
                    items,
                    created_at,
                });
            }
        }

        let result = crate::common::tools::tools_model::GetTransactionDetailsResponse {
            transactions,
            total_amount: total_day_amount,
        };

        let content_json = serde_json::to_string_pretty(&result).unwrap_or_default();

        ToolResult {
            error: "".to_string(),
            success: true,
            content: content_json.clone(),
            follow_up_prompt: build_follow_up_prompt(
                user_message.to_string(),
                content_json,
                "manage_finance".to_string(),
            ),
        }
    }
}
