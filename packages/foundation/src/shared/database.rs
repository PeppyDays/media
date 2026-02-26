use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(
    dsn: &str,
    max_connections: u32,
    connection_timeout: Duration,
    idle_timeout: Duration,
) -> PgPool {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(connection_timeout)
        .idle_timeout(idle_timeout)
        .connect(dsn)
        .await
        .unwrap_or_else(|e| panic!("failed to connect to database: {e}"))
}
