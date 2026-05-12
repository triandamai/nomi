    async fn get_expense_summary(
        &self,
        params: tools_model::GetExpenseSummaryParameters,
        user_message: String,
    ) -> ToolResult {
        let user_id = match self.user_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "User not authenticated".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        use chrono::{Datelike, Timelike};
        let now_wib = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).unwrap());
        
        let (start_date, end_date) = match params.period.as_str() {
            "today" => {
                let start = now_wib.date_naive().and_hms_opt(0, 0, 0).unwrap();
                let end = now_wib.date_naive().and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            },
            "yesterday" => {
                let yesterday = now_wib.date_naive() - chrono::Duration::days(1);
                let start = yesterday.and_hms_opt(0, 0, 0).unwrap();
                let end = yesterday.and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            },
            "last_7_days" => {
                let start_date = now_wib.date_naive() - chrono::Duration::days(6);
                let start = start_date.and_hms_opt(0, 0, 0).unwrap();
                let end = now_wib.date_naive().and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            },
            "last_month" => {
                let month = if now_wib.month() == 1 { 12 } else { now_wib.month() - 1 };
                let year = if now_wib.month() == 1 { now_wib.year() - 1 } else { now_wib.year() };
                let start = chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let next_month = if month == 12 { 1 } else { month + 1 };
                let next_month_year = if month == 12 { year + 1 } else { year };
                let end = chrono::NaiveDate::from_ymd_opt(next_month_year, next_month, 1).unwrap().and_hms_opt(0, 0, 0).unwrap() - chrono::Duration::seconds(1);
                (start, end)
            },
            _ => { // "this_month" or fallback
                let start = chrono::NaiveDate::from_ymd_opt(now_wib.year(), now_wib.month(), 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                let end = now_wib.date_naive().and_hms_opt(23, 59, 59).unwrap();
                (start, end)
            }
        };

        let start_tz = start_date.and_local_timezone(chrono::FixedOffset::east_opt(7 * 3600).unwrap()).unwrap();
        let end_tz = end_date.and_local_timezone(chrono::FixedOffset::east_opt(7 * 3600).unwrap()).unwrap();

        let is_monthly = params.period == "this_month" || params.period == "last_month";
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
            .fetch_optional(&self.pool)
            .await
            {
                use rust_decimal::prelude::ToPrimitive;
                total_expenses = row.total_expenses.unwrap_or_default().to_f64().unwrap_or(0.0);
                total_income = row.total_income.unwrap_or_default().to_f64().unwrap_or(0.0);
                summary_found = total_expenses > 0.0 || total_income > 0.0;
            }
        }

        let mut top_category = None;
        let mut trend_percentage = None;
        
        if !summary_found {
            // Fallback calculation
            if let Ok(sum_row) = sqlx::query!(
                "SELECT SUM(total_amount) as total_expenses
                 FROM money_tracking 
                 WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3",
                user_id,
                start_tz,
                end_tz
            )
            .fetch_one(&self.pool)
            .await
            {
                use rust_decimal::prelude::ToPrimitive;
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
                    .fetch_optional(&self.pool)
                    .await
                    .unwrap_or(None);
                    
                    top_category = cat_row.and_then(|r| r.category);
                }
            }
        }

        // Calculate previous period for trend
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
        .fetch_one(&self.pool)
        .await
        {
            use rust_decimal::prelude::ToPrimitive;
            let prev_total = prev_sum_row.total_expenses.unwrap_or_default().to_f64().unwrap_or(0.0);
            if prev_total > 0.0 {
                trend_percentage = Some(((total_expenses - prev_total) / prev_total) * 100.0);
            }
        }

        if total_expenses == 0.0 {
            return ToolResult {
                error: "".to_string(),
                success: true,
                content: format!("Zero spending for {}! 💸✨", params.period),
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
            follow_up_prompt: build_follow_up_prompt(user_message, json_result.to_string(), "get_expense_summary".to_string()),
        }
    }