use crate::channels::core::{EventChannel, Event};
use futures::future::BoxFuture;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for collection strategies
pub type CollectionStrategy<T> = Arc<dyn Fn(Vec<T>) -> BoxFuture<'static, Result<T, String>> + Send + Sync>;

/// Collects multiple events into a single event using a strategy
///
/// Collector reduces `n -> 1` outputs, typically used downstream of SplitterChannel
/// in RequestPipeline contexts to produce a single response.
///
/// # Example
/// ```ignore
/// let splitter = SplitterChannel::new(
///     "split_document",
///     |doc| Box::pin(async move {
///         Ok(vec![page1, page2, page3])
///     }),
///     next_channel
/// );
///
/// let collector = CollectorChannel::new(
///     "collect_pages",
///     |pages| Box::pin(async move {
///         // Combine pages back into single document
///         Ok(Document::merge(pages))
///     }),
///     next_channel
/// );
/// ```
pub struct CollectorChannel<T: Event> {
    buffer: Arc<RwLock<Vec<T>>>,
    strategy: CollectionStrategy<T>,
    span_name: String,
    next: Arc<dyn EventChannel<T>>,
}

impl<T: Event + 'static> CollectorChannel<T> {
    /// Create a new collector channel with a custom strategy
    pub fn new<S: Into<String>, F>(
        span_name: S,
        strategy: F,
        next: Arc<dyn EventChannel<T>>,
    ) -> Arc<Self>
    where
        F: Fn(Vec<T>) -> BoxFuture<'static, Result<T, String>> + Send + Sync + 'static,
    {
        Arc::new(Self {
            buffer: Arc::new(RwLock::new(Vec::new())),
            strategy: Arc::new(strategy),
            span_name: crate::channels::channel_span_name(span_name, "collector_channel"),
            next,
        })
    }

    /// Collect events into buffer
    pub async fn collect(&self, event: T) -> Result<(), String> {
        let mut buffer = self.buffer.write().await;
        buffer.push(event);
        Ok(())
    }

    /// Flush buffer, apply strategy, forward result
    pub async fn flush(&self) -> Result<(), String> {
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!(&self.span_name);
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            #[cfg(feature = "tracing")]
            tracing::debug!("Buffer empty, nothing to collect");
            return Ok(());
        }

        let events_to_collect = std::mem::take(&mut *buffer);
        let _buffer_size = events_to_collect.len();
        drop(buffer);

        #[cfg(feature = "tracing")]
        tracing::info!(buffer_size, "Collecting events");

        match (self.strategy)(events_to_collect).await {
            Ok(collected) => {
                #[cfg(feature = "tracing")]
                tracing::info!("Events collected successfully");
                self.next.send(collected).await
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(error = %e, "Event collection failed");
                Err(e)
            }
        }
    }

    /// Get current buffer size
    pub async fn buffer_size(&self) -> usize {
        self.buffer.read().await.len()
    }
}

impl<T: Event + 'static> EventChannel<T> for Arc<CollectorChannel<T>> {
    fn send(&self, event: T) -> BoxFuture<'static, Result<(), String>> {
        let this = self.clone();
        Box::pin(async move { this.collect(event).await })
    }
}

/// Collect all events and merge via provided function
///
/// Accumulates events in buffer, caller must trigger flush.
/// Returns merged result of applying merge function to all buffered events.
pub fn collect_all<T, F>(merge_fn: F) -> CollectionStrategy<T>
where
    T: Event + 'static,
    F: Fn(Vec<T>) -> T + Send + Sync + 'static,
{
    Arc::new(move |events| {
        let merged = merge_fn(events);
        Box::pin(async move { Ok(merged) })
    })
}

/// Collect and return first event only
pub fn collect_first<T: Event + 'static>() -> CollectionStrategy<T> {
    Arc::new(|mut events| {
        Box::pin(async move {
            if events.is_empty() {
                Err("No events to collect".into())
            } else {
                Ok(events.remove(0))
            }
        })
    })
}
