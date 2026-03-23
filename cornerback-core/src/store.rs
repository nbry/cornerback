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

pub struct InMemoryEventStore {
    events: tokio::sync::Mutex<Vec<Event>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn insert_event(&self, new_event: NewEvent) -> anyhow::Result<Event> {
        let mut events = self.events.lock().await;
        let event = Event {
            id: uuid::Uuid::new_v4(),
            webhook_id: new_event.webhook_id,
            headers: new_event.headers,
            body: new_event.body,
            created_at: chrono::Utc::now(),
        };
        events.push(event.clone());
        Ok(event)
    }

    async fn get_event(&self, id: EventId) -> anyhow::Result<Option<Event>> {
        let events = self.events.lock().await;
        Ok(events.iter().cloned().find(|e| e.id == id))
    }

    async fn list_events(
        &self,
        webhook_id: &str,
        Paging { limit, offset }: Paging,
    ) -> anyhow::Result<Vec<Event>> {
        let events = self.events.lock().await;
        let filtered_events: Vec<Event> = events
            .iter()
            .cloned()
            .filter(|e| e.webhook_id == webhook_id)
            .skip(offset.unwrap_or(0) as usize)
            .take(limit.unwrap_or(100) as usize)
            .collect();
        Ok(filtered_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_event_store() {
        let store = InMemoryEventStore {
            events: tokio::sync::Mutex::new(vec![]),
        };

        let new_event = NewEvent {
            webhook_id: "test-webhook".to_string(),
            headers: serde_json::json!({ "Content-Type": "application/json" }),
            body: serde_json::json!({ "message": "Hello, World!" }),
        };

        let event = store.insert_event(new_event).await.unwrap();
        assert_eq!(event.webhook_id, "test-webhook");
        assert_eq!(event.headers["Content-Type"], "application/json");
        assert_eq!(event.body["message"], "Hello, World!");

        let fetched_event = store.get_event(event.id).await.unwrap().unwrap();
        assert_eq!(fetched_event.id, event.id);

        let events = store
            .list_events(
                "test-webhook",
                Paging {
                    limit: None,
                    offset: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
    }
}
