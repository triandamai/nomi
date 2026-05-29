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
