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

#[derive(Debug)]
pub struct NewEvent {
    pub webhook_id: String,
    pub headers: Value,
    pub body: Value,
}
