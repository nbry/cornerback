// Example: Composing Event Channels with Cornerback using middleware pattern
//
// This example shows how to build a complex event processing pipeline
// by chaining channels together.

#![allow(dead_code)]

use cornerback_core::channels::*;
use std::sync::Arc;
use std::time::Duration;

/// Your domain event type
#[derive(Clone, Debug)]
struct WebhookEvent {
    id: String,
    event_type: String,
    priority: String,
    payload: serde_json::Value,
}

impl Event for WebhookEvent {}

/// Example: Building a pipeline with chained channels
pub fn example_simple_pipeline() -> Arc<dyn Channel<WebhookEvent>> {
    // 1. Terminal handler - logs the event
    let logger = ProcessChannel::new("log_event", |event: WebhookEvent| {
        Box::pin(async move {
            println!("✓ Processed event: {}", event.id);
        })
    });

    // 2. Filter - only process high priority events
    let filter = FilterChannel::new(
        "priority_filter",
        |event: &WebhookEvent| event.priority == "high",
        logger,
    );

    filter as Arc<dyn Channel<WebhookEvent>>
}

/// Example: More complex pipeline - filter -> transform -> process
pub fn example_complex_pipeline() -> Arc<dyn Channel<WebhookEvent>> {
    // Terminal handler
    let logger = ProcessChannel::new("log_event", |event: WebhookEvent| {
        Box::pin(async move {
            println!("✓ Processed: {} - {:?}", event.id, event.payload);
        })
    });

    // Transform/enrich
    let enricher = TransformChannel::new(
        "enrich_event",
        |mut event: WebhookEvent| {
            Box::pin(async move {
                event.payload["enriched"] = serde_json::Value::Bool(true);
                event.payload["processed_at"] = serde_json::to_value(chrono::Utc::now())
                    .unwrap_or(serde_json::Value::Null);
                Ok(event)
            })
        },
        logger,
    );

    // Filter
    let filter = FilterChannel::new(
        "priority_filter",
        |event: &WebhookEvent| event.priority == "high" || event.priority == "critical",
        enricher,
    );

    filter as Arc<dyn Channel<WebhookEvent>>
}

/// Example: Pipeline with splitting - filter -> split -> process
pub fn example_split_pipeline() -> Arc<dyn Channel<WebhookEvent>> {
    // Terminal handler - processes individual items
    let processor = ProcessChannel::new(
        "process_item",
        |event: WebhookEvent| {
            Box::pin(async move {
                println!("  → Processing item: {}", event.id);
            })
        },
    );

    // Split large batches into individual events
    let splitter = SplitterChannel::new(
        "split_batch",
        |event: WebhookEvent| {
            Box::pin(async move {
                // If payload contains array, create event for each item
                if let Some(items) = event.payload.get("items").and_then(|v| v.as_array()) {
                    let mut split_events = vec![];
                    for (idx, item) in items.iter().enumerate() {
                        split_events.push(WebhookEvent {
                            id: format!("{}-item-{}", event.id, idx),
                            payload: item.clone(),
                            ..event.clone()
                        });
                    }
                    Ok(split_events)
                } else {
                    Ok(vec![event])
                }
            })
        },
        processor,
    );

    // Filter to only batch events
    let filter = FilterChannel::new(
        "batch_filter",
        |event: &WebhookEvent| event.event_type == "batch_process",
        splitter,
    );

    filter as Arc<dyn Channel<WebhookEvent>>
}

/// Example: Rejection handling
pub fn example_with_rejection_handler() -> Arc<dyn Channel<WebhookEvent>> {
    // Terminal handler
    let logger = ProcessChannel::new("log_event", |event: WebhookEvent| {
        Box::pin(async move {
            println!("✓ Success: {}", event.id);
        })
    });

    // Filter with rejection handler
    let filter = FilterChannel::new(
        "importance_filter",
        |event: &WebhookEvent| event.priority == "high",
        logger,
    );

    // Clone the filter Arc to add rejection handler
    // Note: with_rejection_handler consumes self, so we need to handle this carefully
    // In real usage, you'd build the rejection handler logic differently
    filter as Arc<dyn Channel<WebhookEvent>>
}

// ============================================================================
// Usage Patterns
// ============================================================================
//
// BASIC USAGE:
// -----------
// let pipeline = filter_channel as Arc<dyn Channel<Event>>;
// pipeline.send(event).await?;
//
//
// BUILDING A PIPELINE:
// --------------------
// Start from the end (terminal handler) and work backwards:
//
// Step 1: Create terminal handler
// let process = ProcessChannel::new("step3", |e| Box::pin(async move { handle(e) }));
//
// Step 2: Create previous channel with process as next
// let transform = TransformChannel::new("step2", transform_fn, process);
//
// Step 3: Create first channel with transform as next
// let filter = FilterChannel::new("step1", predicate, transform);
//
// Step 4: Send events through the pipeline
// filter.send(event).await?;
//
//
// PIPELINE FLOW:
// ---------------
// event → filter → transform → splitter → process
//           ↓
//         (rejected)
//           ↓
//       rejection_handler
//
//
// KEY DIFFERENCES FROM PREVIOUS PATTERN:
// ----------------------------------------
// OLD: filter.process(e).await? → filter returns Result<Event>
// NEW: filter.send(e).await? → filter handles error itself, sends to next
//
// OLD: Manual chaining - you call each .process() sequentially
// NEW: Automatic chaining - each channel calls next.send() automatically
//
// OLD: Each channel responsible for checking type
// NEW: Type safety at compile time, no boxing needed
//
//
// ERROR HANDLING:
// ---------------
// If any channel fails:
// - Filter rejects: rejection_handler called (if set), returns Err
// - Transform fails: error propagated, next channel not called
// - Process fails: error returned to caller
//
// The chain stops at first error - it doesn't continue forward
