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
    async fn update_event(&self, event: Event) -> anyhow::Result<()>;
    async fn delete_event(&self, id: EventId) -> anyhow::Result<()>;
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

// Simple in-memory implementation of EventStore, primiarily for testing purposes.
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

    async fn update_event(&self, event: Event) -> anyhow::Result<()> {
        let mut events = self.events.lock().await;
        if let Some(existing_event) = events.iter_mut().find(|e| e.id == event.id) {
            *existing_event = event;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Event not found"))
        }
    }

    async fn delete_event(&self, id: EventId) -> anyhow::Result<()> {
        let mut events = self.events.lock().await;
        if let Some(pos) = events.iter().position(|e| e.id == id) {
            events.remove(pos);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Event not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[tokio::test]
    async fn test_in_memory_event_store() -> anyhow::Result<()> {
        let store = InMemoryEventStore {
            events: tokio::sync::Mutex::new(vec![]),
        };

        let incoming_event_1 = NewEvent {
            webhook_id: "test-webhook".to_string(),
            headers: serde_json::json!({ "Content-Type": "application/json" }),
            body: serde_json::json!({ "message": "Hello, World!" }),
        };

        let incoming_event_2 = NewEvent {
            webhook_id: "test-webhook".to_string(),
            headers: serde_json::json!({ "Content-Type": "application/json" }),
            body: serde_json::json!({ "message": "Hello again!" }),
        };

        // Insert events and verify they are stored correctly.
        let event_1 = store.insert_event(incoming_event_1).await?;
        let event_2 = store.insert_event(incoming_event_2).await?;

        assert_eq!(event_1.webhook_id, "test-webhook");
        assert_eq!(event_1.headers["Content-Type"], "application/json");
        assert_eq!(event_1.body["message"], "Hello, World!");

        // Get events by ID and verify they match the inserted events.
        let fetched_event_1 = store
            .get_event(event_1.id)
            .await?
            .context("expected event_1 to exist in store")?;
        assert_eq!(fetched_event_1.id, event_1.id);

        let fetched_event_2 = store
            .get_event(event_2.id)
            .await?
            .context("expected event_2 to exist in store")?;
        assert_eq!(fetched_event_2.id, event_2.id);

        // List events for the webhook and verify both events are returned.
        let events = store
            .list_events(
                "test-webhook",
                Paging {
                    limit: None,
                    offset: None,
                },
            )
            .await?;
        assert_eq!(events.len(), 2);

        // limit
        let events_limited = store
            .list_events(
                "test-webhook",
                Paging {
                    limit: Some(1),
                    offset: None,
                },
            )
            .await?;
        assert_eq!(events_limited.len(), 1);
        assert_eq!(events_limited[0].id, event_1.id);

        // offset
        let events_offset = store
            .list_events(
                "test-webhook",
                Paging {
                    limit: None,
                    offset: Some(1),
                },
            )
            .await?;
        assert_eq!(events_offset.len(), 1);
        assert_eq!(events_offset[0].id, event_2.id);

        Ok(())
    }
}
