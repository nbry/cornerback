mod router;
mod shared;

pub mod handlers;

pub use router::router;

#[cfg(test)]
mod tests;
