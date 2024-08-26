pub mod core;
pub mod field;

type Error = Box<dyn std::error::Error + Send + Sync>; // This is constant and should be copy pasted
