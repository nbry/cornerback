use crate::channels::{channel_span_name, core::{EventChannel, Event}};
use futures::future::BoxFuture;
use std::sync::Arc;

/// Type alias for event splitter functions
pub type Splitter<T> = Arc<dyn Fn(T) -> BoxFuture<'static, Result<Vec<T>, String>> + Send + Sync>;

/// Splits one event into multiple events and forwards each to the next channel
///
/// # Example
/// ```ignore
/// let next = ProcessChannel::new("process", |e| {
///     Box::pin(async move { println!("Processing {:?}", e); })
/// });
///
/// let splitter = SplitterChannel::new(
///     "batch_split",
///     |event| Box::pin(async move {
///         Ok(vec![event.clone(), event])
///     }),
///     next
/// );
///
/// splitter.send(event).await?;
/// ```
pub struct SplitterChannel<T: Event> {
    splitter: Splitter<T>,
    span_name: String,
    next: Arc<dyn EventChannel<T>>,
}

impl<T: Event + 'static> SplitterChannel<T> {
    /// Create a new splitter channel
    pub fn new<S: Into<String>, F>(
        span_name: S,
        splitter: F,
        next: Arc<dyn EventChannel<T>>,
    ) -> Arc<Self>
    where
        F: Fn(T) -> BoxFuture<'static, Result<Vec<T>, String>> + Send + Sync + 'static,
    {
        Arc::new(Self {
            splitter: Arc::new(splitter),
            span_name: channel_span_name(span_name, "splitter_channel"),
            next,
        })
    }

    /// Internal: Process with tracing
    async fn process_with_span(&self, event: T) -> Result<(), String> {
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!(&self.span_name);
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        #[cfg(feature = "tracing")]
        tracing::info!("Splitting event");

        match (self.splitter)(event).await {
            Ok(split_events) => {
                #[cfg(feature = "tracing")]
                tracing::info!(count = split_events.len(), "Event split successfully");

                // Send each split event to next channel
                for event in split_events {
                    self.next.send(event).await?;
                }
                Ok(())
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(error = %e, "Event split failed");

                Err(e)
            }
        }
    }
}

impl<T: Event + 'static> EventChannel<T> for Arc<SplitterChannel<T>> {
    fn send(&self, event: T) -> BoxFuture<'static, Result<(), String>> {
        let this = self.clone();
        Box::pin(async move { this.process_with_span(event).await })
    }
}
