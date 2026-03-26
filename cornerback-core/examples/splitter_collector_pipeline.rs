use cornerback_core::channels::{Event, collect_all, collect_first};

/// Simple event type for demonstration
#[derive(Clone, Debug)]
struct Page {
    id: String,
    num: usize,
    content: String,
}

impl Event for Page {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RequestPipeline Pattern: Collector Use Cases ===\n");

    // Example 1: Collect all pages, merge into one
    println!("1. CollectAll Strategy: Fold multiple events into one");
    println!("   ─────────────────────────────────────────────────");
    {
        // Simulate pages coming from a splitter
        let pages = vec![
            Page {
                id: "doc-1".into(),
                num: 1,
                content: "First page content".into(),
            },
            Page {
                id: "doc-1".into(),
                num: 2,
                content: "Second page content".into(),
            },
            Page {
                id: "doc-1".into(),
                num: 3,
                content: "Third page content".into(),
            },
        ];

        // Strategy: merge all pages into a single page (combine content)
        let merge_strategy = collect_all(|pages: Vec<Page>| {
            let doc_id = pages.first().map(|p| p.id.clone()).unwrap_or_default();
            let combined_content = pages
                .iter()
                .map(|p| format!("[Page {}] {}", p.num, p.content))
                .collect::<Vec<_>>()
                .join("\n");

            Page {
                id: doc_id,
                num: pages.len(),
                content: combined_content,
            }
        });

        // Simulating collector behavior
        println!("   Input: 3 Page events");
        println!("   Merge strategy applies: combine pages into single Page");
        let collected = (merge_strategy)(pages.clone()).await.expect("merge failed");
        println!(
            "   Output: Single Page with {} source pages merged\n",
            collected.num
        );
    }

    // Example 2: Collect first result only
    println!("2. CollectFirst Strategy: Take first event, discard rest");
    println!("   ─────────────────────────────────────────────────");
    {
        let pages = vec![
            Page {
                id: "doc-2".into(),
                num: 1,
                content: "Important first page".into(),
            },
            Page {
                id: "doc-2".into(),
                num: 2,
                content: "Additional pages (ignored)".into(),
            },
        ];

        let first_only = collect_first::<Page>();
        let result = (first_only)(pages).await.expect("collect failed");
        println!("   Input: 2 Page events");
        println!("   Strategy: take first only");
        println!("   Output: Page #{}\n", result.num);
    }

    // Example 3: Demonstrate why this matters for RequestPipeline
    println!("3. Why Collector is Critical for RequestPipeline");
    println!("   ─────────────────────────────────────────────────");
    println!("   Without Collector:");
    println!("     Document →[Splitter]→ Page₁, Page₂, Page₃ →  ???");
    println!("     Problem: Multiple outputs, no single response!\n");
    println!("   With Collector:");
    println!("     Document →[Splitter]→ Page₁,₂,₃ →[Collector]→ Page");
    println!("     Solution: Single merged output, request/response-safe!\n");

    // Example 4: Pipeline structure
    println!("4. Complete RequestPipeline Structure");
    println!("   ─────────────────────────────────────────────────");
    println!("   TransformChannel (1→1)");
    println!("     ↓");
    println!("   FilterChannel (1→0|1)");
    println!("     ↓");
    println!("   SplitterChannel (1→n) ← START: one event");
    println!("     ↓");
    println!("   CollectorChannel (n→1) ← REQUIRED: back to one");
    println!("     ↓");
    println!("   ProcessChannel/Response  ← END: single response\n");

    println!("=== Key Takeaway ===");
    println!("Collector enforces cardinality contract:");
    println!("  • Splitter breaks 1→1 invariant: OK in EventPipeline only");
    println!("  • Collector restores 1→1 invariant: makes Splitter safe for HTTP");
    println!("  • Type-level guarantee: pipeline always produces exactly one response");

    Ok(())
}
