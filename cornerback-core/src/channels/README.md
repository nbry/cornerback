# Event Channels Library - Middleware Pattern

A composable, idiomatic Rust event processing library using a **middleware chain pattern** with automatic OpenTelemetry tracing.

## Key Insight: Channels Form a Chain

Unlike many event systems, channels here work like **Express/Axum middleware** - each channel processes an event and calls the next channel. This creates natural, composable pipelines.

```
Event → FilterChannel → TransformChannel → ProcessChannel
                ↓
              (rejected)
```

## Core Concept: Channel Trait

All channels implement the `Channel<T>` trait:

```rust
pub trait Channel<T: Event>: Send + Sync {
    fn send(&self, event: T) -> BoxFuture<'static, Result<(), String>>;
}
```

Each channel:

1. Receives an event
2. Processes it
3. Calls `next.send(event)` to forward
4. Returns error if processing fails

## Channels

### ProcessChannel (Terminal)

The final handler in a chain. Executes business logic without forwarding.

```rust
let logger = ProcessChannel::new("log_event", |event: MyEvent| {
    Box::pin(async move {
        println!("✓ Event processed: {:?}", event);
    })
});

logger.send(event).await?;
```

### FilterChannel

Routes events based on a predicate. Optional rejection handler.

```rust
let filter = FilterChannel::new(
    "priority_filter",
    |event: &Event| event.priority == "high",
    logger  // next channel
);

filter.send(event).await?;
```

### TransformChannel

Enriches/transforms events asynchronously.

```rust
let enricher = TransformChannel::new(
    "add_timestamp",
    |mut event: Event| {
        Box::pin(async move {
            event.processed_at = now();
            Ok(event)
        })
    },
    logger
);

enricher.send(event).await?;
```

### SplitterChannel

Splits one event into multiple and forwards each to the next channel.

```rust
let splitter = SplitterChannel::new(
    "batch_split",
    |event: Event| {
        Box::pin(async move {
            let split = vec![event.clone(), event];
            Ok(split)
        })
    },
    logger
);

splitter.send(event).await?;
```

### JoinChannel

Buffers multiple events and merges them when a predicate is satisfied.

```rust
let joiner = JoinChannel::new(
    "batch_merge",
    Arc::new(|events| events.len() >= 100),  // merge when 100 buffered
    Arc::new(|events| {
        Box::pin(async move {
            Ok(CombinedEvent { items: events })
        })
    })
);

if let Some(merged) = joiner.buffer(event).await? {
    process_batch(merged).await?;
}
```

## Building Pipelines

**Key Pattern**: Build from the end backward!

```rust
// Step 1: Create terminal handler (the end of the chain)
let logger = ProcessChannel::new("step3_log", |event| {
    Box::pin(async move {
        println!("✓ Done: {:?}", event);
    })
});

// Step 2: Create enrichment (points to logger)
let enricher = TransformChannel::new(
    "step2_enrich",
    |mut event| {
        Box::pin(async move {
            event.metadata = Some("enriched".into());
            Ok(event)
        })
    },
    logger
);

// Step 3: Create filter (points to enricher)
let filter = FilterChannel::new(
    "step1_filter",
    |event: &Event| event.priority == "high",
    enricher
);

// Now send events through the pipeline
// filter → enricher → logger
filter.send(event).await?;
```

## Example: Complete Pipeline

```rust
use cornerback_core::channels::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct WebhookEvent {
    id: String,
    priority: String,
    data: String,
}

impl Event for WebhookEvent {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal: log the event
    let logger = ProcessChannel::new("logger", |event: WebhookEvent| {
        Box::pin(async move {
            println!("✓ Processed: {} ({})", event.id, event.priority);
        })
    });

    // Enrich with timestamp
    let enricher = TransformChannel::new(
        "add_metadata",
        |mut event: WebhookEvent| {
            Box::pin(async move {
                event.data = format!("{} [enriched]", event.data);
                Ok(event)
            })
        },
        logger
    );

    // Split if marked
    let splitter = SplitterChannel::new(
        "split_if_needed",
        |event: WebhookEvent| {
            Box::pin(async move {
                if event.data.contains("batch") {
                    Ok(vec![event.clone(), event])  // duplicate
                } else {
                    Ok(vec![event])
                }
            })
        },
        enricher
    );

    // Filter high-priority only
    let filter = FilterChannel::new(
        "high_priority_only",
        |event: &WebhookEvent| event.priority == "high",
        splitter
    );

    // Send events
    let event = WebhookEvent {
        id: "evt-123".into(),
        priority: "high".into(),
        data: "batch processing".into(),
    };

    filter.send(event).await?;
    Ok(())
}
```

Output:

```
✓ Processed: evt-123 (high)
✓ Processed: evt-123 (high)  // sent twice due to split
```

## Error Handling

Errors stop the chain at that point:

```rust
// If filter rejects: rejection_handler called, returns Err
// If transform fails: error returned, next channel NOT called
// If process errors: error bubbles to caller

filter.send(event).await
    .map_err(|e| println!("Pipeline failed: {}", e))?;
```

## Design Benefits

✅ **Natural Composition** - Chains feel like Express/Tower middleware
✅ **Automatic Tracing** - Spans emitted at each stage
✅ **Type Safe** - Compile-time guarantee of correct channel types
✅ **No Manual Sequencing** - Each channel calls next automatically
✅ **Error Isolation** - Errors stop propagation immediately
✅ **Thread-Safe** - All Send + Sync, shareable via Arc

## Comparison: Old vs New

| Pattern        | Old                       | New                                 |
| -------------- | ------------------------- | ----------------------------------- |
| Composition    | Manual `.process()` calls | Automatic chain calls               |
| Direction      | Build left-to-right       | Build right-to-left (from terminal) |
| Error Handling | Returns Result            | Stops chain at error                |
| Type Safety    | Runtime value passing     | Compile-time generic types          |
| Tracing        | Manual in handlers        | Automatic in channels               |

## When to Use

✅ **Great For:**

- Webhook pipelines
- Event processing streams
- Request enrichment
- Complex validation chains
- Batch processing workflows

❌ **Not For:**

- Simple one-off logic (overhead not justified)
- Standalone operations (use plain async functions)

---

**Philosophy**: Middleware pattern + Rust generics = composable, observable event processing with zero boilerplate.
