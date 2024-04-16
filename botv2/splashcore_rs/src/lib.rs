pub mod crypto;
pub mod animusmagic_protocol;
pub mod animusmagic_ext;
pub mod objectstore;

type Error = Box<dyn std::error::Error + Send + Sync>;