# Backups

**Note that this document describes the technical details of the backup system**

## Format

A backup is an [https://github.com/infinitybotlist/iblfile](iblfile) and has the following fields:

- ``backup_opts`` - JSON containing a ``types.BackupOpts`` object
- ``core`` - The core guild data (``discordgo.Guild``)
- ``messages/{channel_id_hash}`` - The messages in a channel (``[]discordgo.Message``). A ``channel_id_hash`` is a random placeholder string for a channel that is used to avoid leaking channel info in backups
- ``channel_id_hash_table`` - A map of ``channel_id``s to ``channel_id_hash`` (``map[string]string``)