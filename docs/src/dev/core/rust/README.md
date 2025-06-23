# Rust Core Libraries

This folder contains the Rust core libraries used throughout the AntiRaid infrastructure. These libraries provide shared types, utilities, and abstractions for the bot, template worker, and other Rust-based services.

## Key Areas
- **Common Types and Traits:** Shared structs, enums, and traits for user, server, and moderation data.
- **Serialization/Deserialization Helpers:** Utilities for working with Serde, JSON, YAML, and other formats.
- **Error Handling Utilities:** Standardized error types and helpers for consistent error reporting.
- **Async and Concurrency Helpers:** Wrappers and utilities for async tasks, channels, and concurrency patterns.

## Example Modules
- `types.rs`: Defines core data types shared across services.
- `errors.rs`: Provides error types and helpers.
- `async_utils.rs`: Async helpers for tasks and channels.
- `serde_utils.rs`: Serialization helpers for custom formats.

## Usage
Import these libraries in your Rust service to access shared types and utilities. Extend or add new helpers as needed for new features.

---

For more, see the source code in this folder and the main project README.
