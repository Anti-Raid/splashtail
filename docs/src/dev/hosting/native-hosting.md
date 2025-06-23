# Native (Systemd) Hosting for AntiRaid Infrastructure

This guide explains how to set up and run the AntiRaid project using native (systemd) hosting, covering all core services, dependencies, and operational tips.

---

## Prerequisites
- Linux server with `systemd`
- Required language runtimes: Go 1.23+, Rust (nightly recommended), Python 3
- PostgreSQL 16.4+ and Redis
- All build dependencies (see README)
- All binaries built and available in your deployment directory (see `make infra` and `make build`)

---

## Building and Preparing
1. **Clone the repository recursively:**
   ```sh
   git clone https://github.com/Anti-Raid/antiraid --recursive
   ```
2. **Install dependencies:**
   - See the main `README.md` for required packages (Go, Rust, build-essential, etc.)
3. **Build all binaries:**
   ```sh
   make infra
   make build
   ```
4. **Configure services:**
   - Copy and edit config files as needed (see `infra/Sandwich-Daemon/example_sandwich.yaml`, `config.yaml.sample`)
   - Fill in Discord tokens, client IDs, secrets, and webhook URLs

---

## Database Seeding
- Install and set up PostgreSQL 16.4+
- Create required roles and databases (`antiraid`, `frostpaw`, etc.)
- Use `iblcli` to seed the database:
  ```sh
  ibl db load data/seed.iblcli-seed
  ```
- See `data/docker/postgres/setup_seed.sh` for the automated seeding logic

---

## Systemd Service Files

Example service files are provided in `data/systemd-example/`:
- `ar-nirn-staging.service` (nirn-proxy)
- `ar-sandwich-staging.service` (Sandwich-Daemon)
- `splashtail-staging-bot.service` (Bot)
- `splashtail-staging-jobs.service` (Jobserver)
- `splashtail-staging-templateworker.service` (Template Worker)
- `splashtail-staging-webserver.service` (API)
- `ar-postgres-backup.service` and `ar-postgres-backup.timer` (DB backup)

### Example: Installing a Service
```sh
sudo cp data/systemd-example/ar-sandwich-staging.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ar-sandwich-staging.service
sudo systemctl start ar-sandwich-staging.service
```

### Service Details
- **ExecStart**: Points to the built binary and its arguments
- **User/Group**: Should be a dedicated user (e.g., `antiraid`)
- **WorkingDirectory**: Set to the directory containing the binary/config
- **Restart**: Most services are set to always restart
- **Dependencies**: Use `After=` and `PartOf=` to control startup order

---

## Service Startup Order
1. **nirn-proxy** (ar-nirn-staging.service)
2. **Sandwich-Daemon** (ar-sandwich-staging.service)
3. **Jobserver** (splashtail-staging-jobs.service)
4. **Bot** (splashtail-staging-bot.service)
5. **Template Worker** (splashtail-staging-templateworker.service)
6. **API** (splashtail-staging-webserver.service)

---

## Backups
- Use `ar-postgres-backup.service` and `ar-postgres-backup.timer` for automated DB backups
- Backup script uses `mkarbackup` and stores backups securely

---

## Monitoring & Logs
- Use `systemctl status <service>` and `journalctl -u <service> -f` to monitor services
- All services have health endpoints (see Docker guide for details)

---

## Troubleshooting
- Ensure all dependencies and environment variables are set
- Check logs for errors (`journalctl`)
- For database or seeding issues, see helper scripts in `cmd/helper_scripts/` and `data/docker/postgres/`
- For config issues, verify all tokens, secrets, and paths

---

For more details, see the main [README.md](../../README.md) and the Docker hosting guide for comparison.
