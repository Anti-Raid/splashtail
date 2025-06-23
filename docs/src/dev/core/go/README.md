# Go Core Libraries

This folder contains the Go core libraries used throughout the AntiRaid infrastructure. These libraries provide shared types, utilities, and abstractions for services such as the API and jobserver.

## Key Areas
- **Common Types and Interfaces:** Shared structs and interfaces for user, server, and moderation data.
- **Serialization/Deserialization Helpers:** Utilities for marshaling/unmarshaling JSON, YAML, and other formats.
- **Error Handling Utilities:** Standardized error types and helpers for consistent error reporting.
- **RPC and HTTP Abstractions:** Wrappers and helpers for making internal RPC and HTTP calls between services.

## Example Modules
- `types.go`: Defines core data types shared across services.
- `errors.go`: Provides error types and helpers.
- `rpc.go`: Implements RPC client/server logic for internal communication.
- `utils.go`: Miscellaneous helpers used throughout the codebase.

## Usage
Import these libraries in your Go service to access shared types and utilities. Extend or add new helpers as needed for new features.

---

For more, see the source code in this folder and the main project README.
