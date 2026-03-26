use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub type EventId = Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: EventId,
    pub webhook_id: String,
    pub headers: Value,
    pub body: Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Event {
    pub fn new(webhook_id: String, headers: Value, body: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            webhook_id,
            headers,
            body,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str) -> Self {
        if let Value::Object(ref mut map) = self.headers {
            map.insert(key.to_string(), Value::String(value.to_string()));
        } else {
            self.headers = Value::Object(serde_json::Map::from_iter(vec![(
                key.to_string(),
                Value::String(value.to_string()),
            )]));
        }
        self.clone()
    }
}

#[derive(Debug)]
pub struct NewEvent {
    pub webhook_id: String,
    pub headers: Value,
    pub body: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation_and_headers() -> anyhow::Result<()> {
        let mut event = Event::new(
            "test-webhook".to_string(),
            serde_json::json!({ "Content-Type": "application/json" }),
            serde_json::json!({ "message": "Hello, World!" }),
        );

        assert_eq!(event.webhook_id, "test-webhook");
        assert_eq!(event.headers["Content-Type"], "application/json");
        assert_eq!(event.body["message"], "Hello, World!");

        event.add_header("X-Test-Header", "TestValue");
        assert_eq!(event.headers["X-Test-Header"], "TestValue");

        Ok(())
    }
}
