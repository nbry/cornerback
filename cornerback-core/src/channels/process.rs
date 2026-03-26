use crate::channels::{channel_span_name, core::{Event, EventChannel}};
use futures::future::BoxFuture;
use std::marker::PhantomData;
use std::sync::Arc;

/// Type alias for handler functions
pub type Handler<T> = Arc<dyn Fn(T) -> BoxFuture<'static, ()> + Send + Sync>;

/// Terminal channel that executes a handler function
///
/// This is the final channel in a chain and doesn't forward to another channel.
/// It's where the actual business logic execution happens.
///
/// # Example
/// ```ignore
/// let process = ProcessChannel::new("log_event", |event| {
///     Box::pin(async move {
///         println!("Event processed: {:?}", event);
///     })
/// });
///
/// process.send(event).await?;
/// ```
pub struct ProcessChannel<T: Event, H: Fn(T) -> BoxFuture<'static, ()> + Send + Sync> {
    handler: H,
    span_name: String,
    _phantom: PhantomData<T>,
}

impl<T, H> ProcessChannel<T, H>
where
    T: Event,
    H: Fn(T) -> BoxFuture<'static, ()> + Send + Sync + 'static,
{
    /// Create a new process channel (terminal handler)
    pub fn new<S: Into<String>>(span_name: S, handler: H) -> Arc<Self> {
        Arc::new(Self {
            handler,
            span_name: channel_span_name(span_name, "process_channel"),
            _phantom: PhantomData,
        })
    }

    /// Internal: Process with tracing
    async fn process_with_span(&self, event: T) -> Result<(), String> {
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!(&self.span_name);
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        #[cfg(feature = "tracing")]
        tracing::info!("Processing event");

        (self.handler)(event).await;

        #[cfg(feature = "tracing")]
        tracing::info!("Event processed successfully");

        Ok(())
    }
}

impl<T, H> EventChannel<T> for Arc<ProcessChannel<T, H>>
where
    T: Event,
    H: Fn(T) -> BoxFuture<'static, ()> + Send + Sync + 'static,
{
    fn send(&self, event: T) -> BoxFuture<'static, Result<(), String>> {
        let this = self.clone();
        Box::pin(async move { this.process_with_span(event).await })
    }
}
