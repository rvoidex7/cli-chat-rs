pub mod demo;
pub mod traits;

pub use demo::DemoAdapter;
pub use traits::*;

/// Result type for adapter operations
pub type AdapterResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
