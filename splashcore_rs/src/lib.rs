pub mod crypto;
pub mod animusmagic_protocol;
pub mod animusmagic_ext;

type Error = Box<dyn std::error::Error + Send + Sync>;