use sqlx::SqlitePool;
use std::sync::OnceLock;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub async fn init(db_path: &str) -> anyhow::Result<()> {
    let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path)).await?;
    
    // Run migrations
    create_tables(&pool).await?;
    
    DB_POOL.set(pool).map_err(|_| anyhow::anyhow!("Database already initialized"))?;
    
    Ok(())
}

pub fn get_pool() -> &'static SqlitePool {
    DB_POOL.get().expect("Database not initialized")
}

async fn create_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Energy consumption table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS energy_consumption (
            id TEXT PRIMARY KEY,
            device_id TEXT NOT NULL,
            kwh REAL NOT NULL,
            timestamp DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_device_timestamp (device_id, timestamp)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Energy predictions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS energy_predictions (
            id TEXT PRIMARY KEY,
            device_id TEXT NOT NULL,
            predicted_kwh REAL NOT NULL,
            confidence REAL NOT NULL,
            prediction_for DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_device_prediction (device_id, prediction_for)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Devices metadata table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            device_type TEXT NOT NULL,
            location TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_initialization() {
        let result = init(":memory:").await;
        assert!(result.is_ok());
    }
}
