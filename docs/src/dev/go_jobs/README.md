# Go Jobserver

The Go Jobserver handles server backups, message prunes and other long running tasks on a server

# Go Jobserver Developer Docs

This folder documents the Go Jobserver component of AntiRaid. The jobserver is responsible for handling long-running and background tasks such as server backups, message pruning, and other asynchronous jobs.

## Files
- `backups.md`: Details on the backup job system and implementation.
- `README.md`: Overview of the jobserver's responsibilities and architecture.

## Key Concepts
- Jobs are queued and executed independently of the main bot/API processes.
- Designed for reliability and resumability: jobs can survive bot/API restarts.
- Written in Go for concurrency and performance.

---

For more details, see each file in this folder.