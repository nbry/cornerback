/// Event channel library - idiomatic Rust event processing with middleware pattern
///
/// This module provides composable event processing channels that form a chain.
/// Each channel processes an event and forwards to the next channel.
///
/// Channels:
/// - FilterChannel: Routes events based on predicates
/// - TransformChannel: Transforms/enriches event data
/// - SplitterChannel: Splits one event into many
/// - ProcessChannel: Terminal handler that executes business logic
/// - BufferChannel: Buffers events with batching
/// - JoinChannel: Combines multiple events into one
pub mod buffer;
pub mod core;
pub mod filter;
pub mod join;
pub mod process;
pub mod splitter;
pub mod transform;

pub(crate) fn channel_span_name<S: Into<String>>(span_name: S, channel_suffix: &str) -> String {
    format!("{}_{}", span_name.into(), channel_suffix)
}

pub use buffer::BufferChannel;
pub use core::{Event, EventChannel};
pub use filter::FilterChannel;
pub use join::JoinChannel;
pub use process::ProcessChannel;
pub use splitter::SplitterChannel;
pub use transform::TransformChannel;
