# Data & Assets Developer Docs

This folder documents the data, assets, seeds, and generated files used by AntiRaid. These files are essential for deployment, configuration, and operation of the system.

## Key Areas
- **docker/**: Contains Dockerfiles, entrypoints, and config scripts for all services (Postgres, SeaweedFS, Nginx, etc.).
- **generated/**: Auto-generated JSON files for channel types, permissions, and other Discord-related data. Used by the bot and API for validation and logic.
- **misc/**: Miscellaneous data files, patches, and test assets.
- **seed.iblcli-seed**: The main database seed file, used for initializing and seeding the Postgres database with required tables and data.
- **systemd-example/**: Example systemd service and timer files for native hosting and automation (see hosting docs).

## Usage
- Docker configs are used for containerized deployments (see Docker hosting docs).
- Generated assets are loaded at runtime by services for validation and logic.
- The seed file is loaded by the database setup scripts or manually with `iblcli`.
- Systemd files are copied to `/etc/systemd/system/` for native hosting.

## Extension Points
- Add new generated assets as needed for new Discord features or modules.
- Update Dockerfiles and scripts for new services or dependencies.

---

For more, see the main project README and the hosting documentation.
