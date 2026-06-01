use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env::var;

#[tokio::test]
async fn test_seed_simulation() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Connecting to database for seeding simulation data...");
    
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let sql = std::fs::read_to_string("seed_simulation.sql")?;
    println!("Executing seed_simulation.sql statements against postgres...");
    
    for statement in sql.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() {
            sqlx::query(stmt).execute(&pool).await?;
        }
    }
    
    println!("Simulation database seeded successfully! 🏁");
    Ok(())
}

#[tokio::test]
async fn test_show_tasks() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    let tasks = sqlx::query!("SELECT id, title, status, current_step_index, checkpoints FROM autonomous_tasks WHERE title = 'stock market update' ORDER BY created_at DESC LIMIT 1")
        .fetch_all(&pool)
        .await?;

    for task in tasks {
        println!("TASK: id={}, title={}, status={}, current_step_index={}", task.id, task.title, task.status, task.current_step_index);
        println!("CHECKPOINTS: {}", serde_json::to_string_pretty(&task.checkpoints)?);

        let logs = sqlx::query!("SELECT step_index, event_type, log_content FROM autonomous_task_logs WHERE task_id = $1 ORDER BY created_at ASC", task.id)
            .fetch_all(&pool)
            .await?;

        println!("LOGS:");
        for log in logs {
            if log.log_content.len() > 100 {
                println!("  [Step {}] {} | {}...", log.step_index, log.event_type, &log.log_content[..100]);
            } else {
                println!("  [Step {}] {} | {}", log.step_index, log.event_type, log.log_content);
            }
        }
        println!("==================================================");
    }
    Ok(())
}
