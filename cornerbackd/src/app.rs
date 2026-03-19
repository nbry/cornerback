use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

use cornerback_core::store::EventStore;
use cornerback_postgres_eventstore::store::PostgresEventStore;

use crate::routes;

pub async fn build_app() -> anyhow::Result<Router> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/cornerback".into());

    let pool = PgPool::connect(&db_url).await?;

    let store: Arc<dyn EventStore> = Arc::new(PostgresEventStore::new(pool));

    Ok(routes::router(store))
}
