use futures::future::BoxFuture;

/// Core trait for event types that can flow through channels
pub trait Event: Clone + Send + Sync + 'static {}

/// Implement Event for common types
impl Event for String {}
impl Event for serde_json::Value {}
impl<T: Clone + Send + Sync + 'static> Event for Option<T> where T: Event {}
impl<T: Clone + Send + Sync + 'static> Event for Vec<T> where T: Event {}

/// Core trait that all channels implement
///
/// Channels form a chain where each sends to the next channel
pub trait EventChannel<T: Event>: Send + Sync {
    /// Send an event through this channel to the next
    fn send(&self, event: T) -> BoxFuture<'static, Result<(), String>>;
}

/// Trait for channels that can buffer events
pub trait BufferingChannel<T: Event>: Send + Sync {
    /// Buffer an event, returns batch if ready for processing
    fn buffer(&self, event: T) -> BoxFuture<'static, Option<Vec<T>>>;
}
