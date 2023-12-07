# Backups

**Note that this document describes the technical details of the backup system**

## Format

A backup is an [https://github.com/infinitybotlist/iblfile](iblfile) and has the following fields:

- ``backup_opts`` - JSON containing a ``types.BackupOpts`` object
- ``core`` - The core guild data (``discordgo.Guild``)
- ``messages/{channel_id}`` - The messages in a channel along with basic attachment metadata (``[]types.BackupMessage``).