use crate::channels::core::{Channel, Event};
use futures::future::BoxFuture;
use std::sync::Arc;

/// Type alias for transformation functions
pub type Transformer<T> = Arc<dyn Fn(T) -> BoxFuture<'static, Result<T, String>> + Send + Sync>;

/// Transforms/enriches event data and forwards to the next channel
///
/// # Example
/// ```ignore
/// let next = ProcessChannel::new("logger", |e| {
///     Box::pin(async move { println!("{:?}", e); })
/// });
///
/// let transform = TransformChannel::new(
///     "enrich",
///     |mut e| Box::pin(async move { e.enriched = true; Ok(e) }),
///     next
/// );
///
/// transform.send(event).await?;
/// ```
pub struct TransformChannel<T: Event> {
    transformer: Transformer<T>,
    span_name: String,
    next: Arc<dyn Channel<T>>,
}

impl<T: Event + 'static> TransformChannel<T> {
    /// Create a new transform channel
    pub fn new<S: Into<String>, F>(
        span_name: S,
        transformer: F,
        next: Arc<dyn Channel<T>>,
    ) -> Arc<Self>
    where
        F: Fn(T) -> BoxFuture<'static, Result<T, String>> + Send + Sync + 'static,
    {
        Arc::new(Self {
            transformer: Arc::new(transformer),
            span_name: span_name.into(),
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
        tracing::info!("Transforming event");

        match (self.transformer)(event).await {
            Ok(transformed) => {
                #[cfg(feature = "tracing")]
                tracing::info!("Event transformed successfully");

                self.next.send(transformed).await
            }
            Err(e) => {
                #[cfg(feature = "tracing")]
                tracing::warn!(error = %e, "Event transformation failed");

                Err(e)
            }
        }
    }
}

impl<T: Event + 'static> Channel<T> for Arc<TransformChannel<T>> {
    fn send(&self, event: T) -> BoxFuture<'static, Result<(), String>> {
        let this = self.clone();
        Box::pin(async move { this.process_with_span(event).await })
    }
}
