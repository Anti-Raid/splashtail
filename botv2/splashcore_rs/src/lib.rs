pub mod animusmagic_protocol;
pub mod animusmagic_ext;
pub mod objectstore;
pub mod utils;

type Error = Box<dyn std::error::Error + Send + Sync>;