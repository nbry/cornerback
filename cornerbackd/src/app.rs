use axum::Router;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::Arc;

use cornerback_core::store::EventStore;
use cornerback_core::store::InMemoryEventStore;
use cornerback_postgres_eventstore::store::PostgresEventStore;

use crate::config::{AppConfig, EventStoreConfig};
use crate::routes;

#[derive(Clone)]
pub struct RouteState {
    pub store: Arc<dyn EventStore>,
    pub config: Arc<AppConfig>,
    pub http_client: Client,
}

pub async fn build_app() -> anyhow::Result<Router> {
    let config_path =
        std::env::var("CORNERBACK_CONFIG").unwrap_or_else(|_| "cornerbackd.json".to_string());
    let config = Arc::new(AppConfig::from_file(std::path::Path::new(&config_path))?);

    let store: Arc<dyn EventStore> = match &config.event_store {
        EventStoreConfig::Postgres { database_url } => {
            let pool = PgPool::connect(database_url).await?;
            Arc::new(PostgresEventStore::new(pool))
        }
        EventStoreConfig::InMemory => Arc::new(InMemoryEventStore::new()),
        EventStoreConfig::Custom { name } => {
            return Err(anyhow::anyhow!(
                "custom event store '{}' is not wired in cornerbackd build_app; provide an app-specific builder",
                name
            ));
        }
    };

    let http_client = Client::new();

    let route_state = RouteState {
        store,
        config,
        http_client,
    };

    Ok(routes::router(route_state))
}
