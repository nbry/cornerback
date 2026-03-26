// Real-world example: Webhook processing pipeline
// Shows the new middleware chain pattern

use cornerback_core::channels::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct WebhookEvent {
    id: String,
    event_type: String,
    priority: String,
    payload: String,
}

impl Event for WebhookEvent {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═════════════════════════════════════════════");
    println!("  Event Channels - Middleware Pattern Demo");
    println!("═════════════════════════════════════════════\n");

    // ════════════════════════════════════════════════════════════════
    // Build pipeline from TERMINAL backwards
    // ════════════════════════════════════════════════════════════════

    // 1. TERMINAL: Final handler
    let logger = ProcessChannel::new("final_log", |event: WebhookEvent| {
        Box::pin(async move {
            println!(
                "  ✓ PROCESSED: {} (type: {}, priority: {})",
                event.id, event.event_type, event.priority
            );
        })
    });
    let logger: Arc<dyn Channel<WebhookEvent>> = Arc::new(logger);

    // 2. TRANSFORM: Enrich the event
    let enricher = TransformChannel::new(
        "enrich_metadata",
        |mut event: WebhookEvent| {
            Box::pin(async move {
                println!("  → Enriching: {}", event.id);
                event.payload = format!("{} [enriched at {}]", event.payload, chrono::Utc::now());
                Ok(event)
            })
        },
        logger,
    );
    let enricher: Arc<dyn Channel<WebhookEvent>> = Arc::new(enricher);

    // 3. FILTER: Only high priority
    let filter = FilterChannel::new(
        "priority_gate",
        |event: &WebhookEvent| {
            let accept = event.priority == "high" || event.priority == "critical";
            println!(
                "  ? Checking priority: {} → {}",
                event.priority,
                if accept { "PASS" } else { "REJECT" }
            );
            accept
        },
        enricher,
    );
    let filter: Arc<dyn Channel<WebhookEvent>> = Arc::new(filter);

    println!("\n📋 Pipeline Structure:");
    println!("  filter → enricher → logger");
    println!("\n  Event flows: Filter → Enrich → Log\n");

    // ════════════════════════════════════════════════════════════════
    // Send events through the pipeline
    // ════════════════════════════════════════════════════════════════

    // Event 1: High priority (accepted)
    println!("─────────────────────────────────────────────");
    println!("Event 1: High Priority Webhook\n");
    let event1 = WebhookEvent {
        id: "evt-001".into(),
        event_type: "user.created".into(),
        priority: "high".into(),
        payload: "User John Doe signed up".into(),
    };

    match filter.send(event1).await {
        Ok(()) => println!("\n✅ Success\n"),
        Err(e) => println!("\n❌ Error: {}\n", e),
    }

    // Event 2: Low priority (rejected)
    println!("─────────────────────────────────────────────");
    println!("Event 2: Low Priority Webhook\n");
    let event2 = WebhookEvent {
        id: "evt-002".into(),
        event_type: "analytics.ping".into(),
        priority: "low".into(),
        payload: "Heartbeat".into(),
    };

    match filter.send(event2).await {
        Ok(()) => println!("\n✅ Success\n"),
        Err(e) => println!("\n❌ Rejected: {}\n", e),
    }

    // Event 3: Critical priority (accepted)
    println!("─────────────────────────────────────────────");
    println!("Event 3: Critical Priority Webhook\n");
    let event3 = WebhookEvent {
        id: "evt-003".into(),
        event_type: "payment.failed".into(),
        priority: "critical".into(),
        payload: "Payment processing failed".into(),
    };

    match filter.send(event3).await {
        Ok(()) => println!("\n✅ Success\n"),
        Err(e) => println!("\n❌ Error: {}\n", e),
    }

    println!("════════════════════════════════════════════");
    println!("  Key Points:");
    println!("  1. Chain built right-to-left (from terminal)");
    println!("  2. Each channel calls next automatically");
    println!("  3. Errors stop propagation immediately");
    println!("  4. Type-safe at compile time");
    println!("  5. Observable with automatic tracing");
    println!("════════════════════════════════════════════\n");

    Ok(())
}
