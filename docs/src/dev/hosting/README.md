# AntiRaid Hosting Guide

Welcome to the AntiRaid hosting documentation! This section provides a comprehensive overview of all supported deployment methods for the AntiRaid (Splashtail) infrastructure, including both Docker-based and native (systemd) hosting.

---

## Overview
AntiRaid is a modular, production-grade Discord anti-raid and moderation system. It can be deployed using Docker Compose for ease and reproducibility, or natively on Linux servers using systemd for maximum control.

---

## Hosting Methods

### 1. Docker Compose Hosting
- Easiest and most reproducible way to deploy the full stack
- Orchestrates all services: bot, API, jobserver, template worker, database, object storage, proxies, and more
- See [`../docker-hosting.md`](../docker-hosting.md) for a detailed, project-specific guide

### 2. Native (Systemd) Hosting
- For advanced users who want to run each service as a native Linux process
- Provides fine-grained control, integration with systemd, and custom monitoring
- See [`../native-hosting.md`](../native-hosting.md) for a detailed, project-specific guide

---

## Quick Comparison
| Feature         | Docker Compose         | Native (Systemd)    |
|----------------|-----------------------|-----------------------|
| Ease of setup  | ⭐⭐⭐⭐⭐          | ⭐⭐⭐              |
| Customization  | ⭐⭐⭐               |  ⭐⭐⭐⭐⭐        |
| Monitoring     | Docker CLI/Compose    | systemd/journalctl    |
| Portability    | High                  | Medium                |
| Upgrades       | Easy (rebuild images) | Manual (rebuild/run)  |

---

## Where to Start
- **New users:** Start with Docker Compose for a fast, reliable deployment
- **Advanced users:** Use native hosting for maximum control and integration

---

## More Information
- [Docker Hosting Guide](../docker-hosting.md)
- [Native Hosting Guide](../native-hosting.md)
- [Main Project README](../../../README.md)

---

# AntiRaid Hosting Developer Docs

This folder provides an overview of all supported hosting and deployment methods for AntiRaid, with links to detailed guides for each method.

## Hosting Methods
- **Docker Compose:** Easiest, most reproducible way to deploy the full stack. See [`../docker-hosting.md`](../docker-hosting.md).
- **Native (Systemd):** For advanced users who want fine-grained control and integration with Linux. See [`../native-hosting.md`](../native-hosting.md).

## Structure
- `README.md`: This overview
- See sibling folders for Docker and native hosting details

## Where to Start
- New users: Start with Docker Compose
- Advanced users: Use native hosting for maximum control

---

For more, see the main project README and the full documentation in this directory.
