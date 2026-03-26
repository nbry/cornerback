// Example: Order Processing Pipeline with JoinChannel
//
// Real-world scenario: You receive order line items from webhooks.
// Instead of processing each item individually, you can:
// 1. Collect multiple items in a buffer
// 2. When predicate is met (e.g., 3 items collected), flush and process together
// 3. Reduced database load - batch write instead of individual writes
//
// Note: JoinChannel merges Vec<T> → T (same type in/out)
// For transforming types, use TransformChannel or custom processing

use cornerback_core::channels::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct OrderLineItem {
    order_id: String,
    item_id: String,
    product_name: String,
    quantity: i32,
    price: f64,
}

impl Event for OrderLineItem {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═════════════════════════════════════════════════════════");
    println!("  JoinChannel Use Case: Webhook Batching");
    println!("═════════════════════════════════════════════════════════\n");

    // ════════════════════════════════════════════════════════════════
    // JoinChannel: Buffer items, flush when predicate is met
    // ════════════════════════════════════════════════════════════════

    let joiner = JoinChannel::new(
        "batch_order_items",
        Arc::new(|items: &[OrderLineItem]| {
            // Predicate: flush when we have 3 items from same order
            if items.is_empty() {
                return false;
            }

            // All from same order
            let same_order = items.iter().all(|i| i.order_id == items[0].order_id);

            // We have 3 items
            let enough_items = items.len() >= 3;

            same_order && enough_items
        }),
        // Merger: "merge" items by creating summary and processing batch
        Arc::new(|items: Vec<OrderLineItem>| {
            Box::pin(async move {
                // In real world: batch write to DB, send notification, etc.
                let order_id = items[0].order_id.clone();
                let qty_total: i32 = items.iter().map(|i| i.quantity).sum();
                let value_total: f64 = items.iter().map(|i| i.price * i.quantity as f64).sum();

                println!(
                    "  ✓ BATCH READY: {} items | Order: {} | Qty: {} | Value: ${:.2}",
                    items.len(),
                    order_id,
                    qty_total,
                    value_total
                );

                // Return merged representation (aggregated as single item)
                Ok(OrderLineItem {
                    order_id,
                    item_id: "BATCH".into(),
                    product_name: format!("{} items (batch)", items.len()),
                    quantity: qty_total,
                    price: value_total,
                })
            })
        }),
    );

    println!("Pipeline demonstrates batching webhook events:\n");
    println!("Without batching (3 webhooks = 3 DB writes):");
    println!("  Event 1 → DB Write ❌");
    println!("  Event 2 → DB Write ❌");
    println!("  Event 3 → DB Write ❌\n");

    println!("With JoinChannel (3 webhooks = 1 batch write):");
    println!("  Event 1 → Buffer");
    println!("  Event 2 → Buffer");
    println!("  Event 3 → [FLUSH] → Single DB Write ✅\n");

    // ════════════════════════════════════════════════════════════════
    // Simulate receiving events
    // ════════════════════════════════════════════════════════════════

    println!("─────────────────────────────────────────────────────────");
    println!("Simulating webhooks for Order #ORD-001:\n");

    let items = vec![
        OrderLineItem {
            order_id: "ORD-001".into(),
            item_id: "SKU-001".into(),
            product_name: "Laptop".into(),
            quantity: 1,
            price: 999.99,
        },
        OrderLineItem {
            order_id: "ORD-001".into(),
            item_id: "SKU-002".into(),
            product_name: "Mouse".into(),
            quantity: 2,
            price: 29.99,
        },
        OrderLineItem {
            order_id: "ORD-001".into(),
            item_id: "SKU-003".into(),
            product_name: "Keyboard".into(),
            quantity: 1,
            price: 79.99,
        },
    ];

    // Feed items one by one
    for (i, item) in items.iter().enumerate() {
        println!(
            "  [{}] Webhook: {} - {} x{} @ ${:.2}",
            i + 1,
            item.order_id,
            item.product_name,
            item.quantity,
            item.price
        );

        let result = joiner.buffer(item.clone()).await?;

        // When batch is ready (predicate met)
        if let Some(batch) = result {
            println!("\n     [BATCH COMPLETE - {} items merged]\n", batch.item_id);
        }
    }

    // ════════════════════════════════════════════════════════════════
    // Use Cases
    // ════════════════════════════════════════════════════════════════

    println!("─────────────────────────────────────────────────────────");
    println!("\n💡 Real-world Use Cases:\n");

    println!("1️⃣  E-commerce Orders");
    println!("   Batch line items → Single inventory update transaction");
    println!("   Benefit: Atomicity, reduced DB load\n");

    println!("2️⃣  Analytics Events");
    println!("   Batch 100+ events → Single write to data warehouse");
    println!("   Benefit: 100x reduction in I/O, cost savings\n");

    println!("3️⃣  API Webhook Processing");
    println!("   Batch incoming webhooks from external service");
    println!("   Benefit: Fail-safe, easier to retry whole batch\n");

    println!("4️⃣  ETL Pipelines");
    println!("   Batch records before bulk insert/update");
    println!("   Benefit: 1000x+ performance improvement\n");

    println!("5️⃣  Notification Digests");
    println!("   Batch N user actions → Send digest email once");
    println!("   Benefit: Reduce spam, better UX\n");

    println!("6️⃣  Time-windowed Aggregation");
    println!("   Batch events from last N seconds for metrics");
    println!("   Benefit: Real-time aggregation with lower latency\n");

    println!("═════════════════════════════════════════════════════════\n");

    println!("📊 Performance Comparison:\n");
    println!("   Scenario: Process 1,000 webhook events\n");
    println!("   ❌ Without batching:");
    println!("      1,000 database round-trips");
    println!("      ~500ms latency (0.5ms per event)");
    println!("      High connection pool pressure\n");
    println!("   ✅ With JoinChannel (batch every 100 events):");
    println!("      10 database round-trips (-99%)");
    println!("      ~50ms latency (-90%)");
    println!("      Minimal connection pool pressure\n");

    println!("═════════════════════════════════════════════════════════\n");

    Ok(())
}
