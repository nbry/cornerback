use crate::channels::core::Event;
use futures::future::BoxFuture;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Configuration for buffering behavior
#[derive(Clone, Debug)]
pub struct BufferConfig {
    /// Maximum batch size before flushing
    pub max_size: usize,
    /// Maximum time to wait before flushing
    pub max_age: Duration,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            max_age: Duration::from_secs(5),
        }
    }
}

/// Buffers events with time and size-based flushing and forwards batches to a handler
///
/// # Example
/// ```ignore
/// let buffer = BufferChannel::new(
///     "batch_buffer",
///     BufferConfig {
///         max_size: 50,
///         max_age: Duration::from_secs(10),
///     },
///     |batch| Box::pin(async move {
///         println!("Processing batch of {}", batch.len());
///     })
/// );
///
/// buffer.buffer(event).await?;
/// ```
pub struct BufferChannel<T: Event, H: Fn(Vec<T>) -> BoxFuture<'static, ()> + Send + Sync> {
    config: BufferConfig,
    buffer: Arc<RwLock<Vec<T>>>,
    last_flush: Arc<RwLock<Instant>>,
    span_name: String,
    handler: H,
}

impl<T, H> BufferChannel<T, H>
where
    T: Event,
    H: Fn(Vec<T>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
{
    /// Create a new buffer channel
    pub fn new<S: Into<String>>(span_name: S, config: BufferConfig, handler: H) -> Arc<Self> {
        Arc::new(Self {
            config,
            buffer: Arc::new(RwLock::new(Vec::new())),
            last_flush: Arc::new(RwLock::new(Instant::now())),
            span_name: span_name.into(),
            handler,
        })
    }

    /// Buffer an event - returns Ok if buffered, calls handler if flushed
    pub async fn buffer(&self, event: T) -> Result<(), String> {
        #[cfg(feature = "tracing")]
        let span = tracing::info_span!(&self.span_name);
        #[cfg(feature = "tracing")]
        let _guard = span.enter();

        let mut buffer = self.buffer.write().await;
        buffer.push(event);

        let should_flush = buffer.len() >= self.config.max_size
            || {
                let last_flush = *self.last_flush.read().await;
                last_flush.elapsed() >= self.config.max_age
            };

        if should_flush {
            #[cfg(feature = "tracing")]
            tracing::info!(buffer_size = buffer.len(), "Buffer flushing");

            let flushed = std::mem::take(&mut *buffer);
            drop(buffer);

            let mut last_flush = self.last_flush.write().await;
            *last_flush = Instant::now();

            (self.handler)(flushed).await;
        } else {
            #[cfg(feature = "tracing")]
            tracing::debug!(buffer_size = buffer.len(), "Event buffered");
        }

        Ok(())
    }

    /// Get current buffer size
    pub async fn size(&self) -> usize {
        self.buffer.read().await.len()
    }

    /// Force flush the buffer
    pub async fn flush(&self) -> Result<(), String> {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "tracing")]
        tracing::info!(buffer_size = buffer.len(), "Manual buffer flush");

        let flushed = std::mem::take(&mut *buffer);
        drop(buffer);

        let mut last_flush = self.last_flush.write().await;
        *last_flush = Instant::now();

        (self.handler)(flushed).await;
        Ok(())
    }
}
