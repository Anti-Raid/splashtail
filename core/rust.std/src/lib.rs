pub mod animusmagic;
pub mod jobserver;
pub mod objectstore;
pub mod permissions;
pub mod types;
pub mod utils;
pub mod value;

type Error = Box<dyn std::error::Error + Send + Sync>;
