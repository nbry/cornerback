use crate::models::*;
use async_trait::async_trait;

pub struct Paging {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn insert_event(&self, event: NewEvent) -> anyhow::Result<Event>;
    async fn get_event(&self, id: EventId) -> anyhow::Result<Option<Event>>;
    async fn list_events(&self, webhook_id: &str, paging: Paging) -> anyhow::Result<Vec<Event>>;
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: Event) -> anyhow::Result<()>;
}
