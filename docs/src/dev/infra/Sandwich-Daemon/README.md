# Sandwich-Daemon Developer Docs

Sandwich-Daemon is the gateway manager for scalable Discord sharding in AntiRaid. It manages all Discord gateway connections, sharding, presence, and provides a unified API for the bot and other services.

## Key Features
- **Sharding:** Manages multiple Discord gateway shards for scalability.
- **Presence Management:** Handles bot presence and status updates.
- **WebSocket API:** Exposes a WebSocket API for internal services to interact with Discord.
- **OAuth2 Integration:** Supports Discord OAuth2 for user authentication.
- **Healthchecks:** Exposes `/antiraid/api/v9/gateway/bot` for monitoring.

## Configuration
- Configured via YAML file (`sandwich.yaml` or `sandwich.docker.yaml`).
- Supports environment variables for secrets and tokens.

## Extension Points
- Add new event handlers or integrations by extending the Sandwich-Daemon source code.

## Deployment
- Used in both Docker and native deployments.
- Should be run on an internal network, with only required ports exposed.

---

For more, see the main infra README and deployment guides.
