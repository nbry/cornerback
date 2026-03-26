use crate::channels::core::{Channel, Event};
use futures::future::BoxFuture;
use std::sync::Arc;

/// Routes events based on a predicate function
///
/// # Example
/// ```ignore
/// let process = ProcessChannel::new("logger", |event| {
///     Box::pin(async move {
///         println!("Processed: {:?}", event);
///     })
/// });
///
/// let filter = FilterChannel::new(
///     "priority_filter",
///     |event: &Event| event.priority == "high",
///     process
/// );
///
/// filter.send(event).await?;
/// ```
pub struct FilterChannel<T: Event, F: Fn(&T) -> bool + Send + Sync> {
    predicate: F,
    span_name: String,
    next: Arc<dyn Channel<T>>,
    rejection_handler: Option<Arc<dyn Fn(T) -> BoxFuture<'static, ()> + Send + Sync>>,
}

impl<T, F> FilterChannel<T, F>
where
    T: Event,
    F: Fn(&T) -> bool + Send + Sync + 'static,
{
    /// Create a new filter channel with the next channel
    pub fn new<S: Into<String>>(
        span_name: S,
        predicate: F,
        next: Arc<dyn Channel<T>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            predicate,
            span_name: span_name.into(),
            next,
            rejection_handler: None,
        })
    }

    /// Set a handler for rejected events
    pub fn with_rejection_handler<H>(mut self, handler: H) -> Self
    where
        H: Fn(T) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        self.rejection_handler = Some(Arc::new(handler));
        self
    }

    /// Internal: Process event with tracing
    async fn process_with_span(&self, event: T) -> Result<(), String> {
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!(&self.span_name);
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        #[cfg(feature = "tracing")]
        tracing::info!("Filtering event");

        if (self.predicate)(&event) {
            #[cfg(feature = "tracing")]
            tracing::info!("Event accepted, forwarding to next");

            self.next.send(event).await
        } else {
            #[cfg(feature = "tracing")]
            tracing::info!("Event rejected by filter");

            if let Some(handler) = &self.rejection_handler {
                handler(event).await;
            }

            Err("Event rejected by filter".into())
        }
    }
}

impl<T, F> Channel<T> for Arc<FilterChannel<T, F>>
where
    T: Event,
    F: Fn(&T) -> bool + Send + Sync + 'static,
{
    fn send(&self, event: T) -> BoxFuture<'static, Result<(), String>> {
        let this = self.clone();
        Box::pin(async move { this.process_with_span(event).await })
    }
}
