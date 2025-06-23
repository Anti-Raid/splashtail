# Services Developer Docs

This folder documents the main service implementations for AntiRaid. Each service is responsible for a core part of the system and communicates with others via internal APIs and message queues.

## Key Services
- **api/**: The HTTP API service, written in Go. Handles web, integration, and internal API requests.
- **bot/**: The main Discord bot, written in Rust. Handles events, commands, and moderation logic.
- **jobserver/**: The background job processor, written in Go. Handles backups, pruning, and other async jobs.
- **template-worker/**: The template execution engine, written in Rust. Runs user-defined logic in a sandboxed environment.

## Architecture
- Services communicate over HTTP, WebSocket, and internal RPC protocols.
- Each service is containerized for Docker deployments, but can also be run natively.
- Healthchecks and metrics endpoints are provided for all major services.

## Extension Points
- Add new services for additional features or integrations.
- Extend existing services by adding new endpoints, modules, or job types.

---

For more, see each service's README and the main project documentation.
