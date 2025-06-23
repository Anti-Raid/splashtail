# AntiRaid Infra Developer Docs

This folder documents the core infrastructure components of AntiRaid, including proxies, gateway managers, and supporting services.

## Overview
- **nirn-proxy:** Discord REST/gateway proxy. Handles all REST and gateway traffic between Discord and the internal services, providing rate limiting, endpoint rewriting, and security.
- **Sandwich-Daemon:** Gateway manager for scalable sharding. Manages Discord gateway connections, sharding, and presence, and provides a unified API for the bot and other services.
- **Nginx:** Used for exposing internal APIs and SeaweedFS to the outside world, with strict network segmentation.
- **SeaweedFS:** S3-compatible object storage for backups and file storage.
- **Filestash:** Web UI for managing SeaweedFS.

## Architecture
- All infra components are containerized for Docker deployments, but can also be run natively.
- Internal networks are segmented for security (see Docker Compose and systemd docs).
- Healthchecks and metrics endpoints are provided for all major infra services.

## Subfolders
- [nirn-proxy](./nirn-proxy/): Proxy architecture, config, and extension points
- [Sandwich-Daemon](./Sandwich-Daemon/): Gateway manager internals and config
- [templating-template](./templating-template/): Templating infra
- [templating-types](./templating-types/): Templating type system

## Security & Networking
- All external traffic is routed through proxies (nirn-proxy, Nginx)
- Internal-only networks prevent unauthorized access
- Rate limiting and endpoint rewriting are enforced at the proxy layer

---

See each subfolder for detailed docs on configuration, extension, and deployment.