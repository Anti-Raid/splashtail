# Nirn Proxy Developer Docs

Nirn-proxy is the custom Discord REST/gateway proxy used by AntiRaid. It sits between Discord and all internal services, handling:
- REST and gateway traffic
- Rate limiting
- Endpoint rewriting
- Security and token mapping

## Key Features
- **Endpoint Rewriting:** Allows internal services to use custom endpoints for Discord API/gateway.
- **Token Mapping:** Maps internal tokens to real Discord tokens for security and multi-bot support.
- **Rate Limiting:** Enforces Discord's rate limits and protects against abuse.
- **Healthchecks:** Exposes `/nirn/healthz` for monitoring.

## Configuration
- Configured via command-line arguments and a secrets file (`secrets.docker.json`).
- See the Docker Compose file for example usage and environment variables.

## Extension Points
- Add new endpoint rewrites or custom logic by editing the proxy's configuration or source code.

## Deployment
- Used in both Docker and native deployments.
- Should be run on an internal network, with only required ports exposed.

---

For more, see the main infra README and deployment guides.
