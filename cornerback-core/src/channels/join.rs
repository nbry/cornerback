use crate::channels::{channel_span_name, core::Event};
use futures::future::BoxFuture;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for join predicates that determine when to flush the buffer
pub type JoinPredicate<T> = Arc<dyn Fn(&[T]) -> bool + Send + Sync>;

/// Type alias for join merger functions
pub type Merger<T> = Arc<dyn Fn(Vec<T>) -> BoxFuture<'static, Result<T, String>> + Send + Sync>;

/// Joins multiple events into a single event based on a predicate
///
/// # Example
/// ```ignore
/// let join = JoinChannel::new(
///     "batch_collect",
///     Arc::new(|events| events.len() >= 10),  // Merge when we have 10 events
///     Arc::new(|events| Box::pin(async move {
///         Ok(Event { combined: events })
///     }))
/// );
///
/// if let Some(merged) = join.buffer(event).await? {
///     process_merged(merged).await?;
/// }
/// ```
pub struct JoinChannel<T: Event> {
    buffer: Arc<RwLock<Vec<T>>>,
    join_predicate: JoinPredicate<T>,
    merger: Merger<T>,
    span_name: String,
}

impl<T: Event + 'static> JoinChannel<T> {
    /// Create a new join channel
    pub fn new<S: Into<String>>(
        span_name: S,
        join_predicate: JoinPredicate<T>,
        merger: Merger<T>,
    ) -> Arc<Self> {
        Arc::new(Self {
            buffer: Arc::new(RwLock::new(Vec::new())),
            join_predicate,
            merger,
            span_name: channel_span_name(span_name, "join_channel"),
        })
    }

    /// Buffer an event, returns merged result if predicate is satisfied
    pub async fn buffer(&self, event: T) -> Result<Option<T>, String> {
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!(&self.span_name);
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        let mut buffer = self.buffer.write().await;
        buffer.push(event);

        #[cfg(feature = "tracing")]
        tracing::debug!(buffer_size = buffer.len(), "Event buffered");

        if (self.join_predicate)(&buffer) {
            #[cfg(feature = "tracing")]
            tracing::info!(buffer_size = buffer.len(), "Join predicate satisfied");

            let events_to_merge = std::mem::take(&mut *buffer);
            let result = (self.merger)(events_to_merge).await;

            #[cfg(feature = "tracing")]
            if result.is_ok() {
                tracing::info!("Events merged successfully");
            } else {
                tracing::warn!("Event merge failed");
            }

            Ok(result.ok())
        } else {
            Ok(None)
        }
    }

    /// Get current buffer size without consuming
    pub async fn buffer_size(&self) -> usize {
        self.buffer.read().await.len()
    }

    /// Flush remaining events in buffer
    pub async fn flush(&self) -> Result<Option<T>, String> {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return Ok(None);
        }

        let events_to_merge = std::mem::take(&mut *buffer);
        (self.merger)(events_to_merge).await.map(Some)
    }
}
