# Command-line Tools Developer Docs

This folder documents the command-line tools and helper scripts for AntiRaid. These tools are used for development, migration, maintenance, and automation tasks.

## Key Tools
- **check_guild_count.py:** Checks the number of guilds (servers) the bot is in. Usage: `python cmd/check_guild_count.py`
- **helper_scripts/:**
  - Migration scripts for database and config changes
  - Utility scripts for backups, seeding, and more
- **mkroute/:** Tools for generating and managing routing tables or configs
- **nursery/:** Experimental and prototype scripts (may be unstable)

## Usage
- Run Python scripts directly with `python <script>.py`
- Shell scripts can be run with `bash <script>.sh` or made executable
- See comments in each script for arguments and usage examples

## Extension Points
- Add new scripts for automation, migration, or maintenance tasks
- Organize scripts by function in subfolders as needed

---

For more, see the source code in this folder and the main project README.
