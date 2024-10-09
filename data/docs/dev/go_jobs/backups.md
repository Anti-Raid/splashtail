# Backups

**Note that this document describes the technical details of the backup system**

## Format

A backup is an [https://github.com/infinitybotlist/iblfile](iblfile) with the standard ``AutoEncryptedFile`` format and has the following fields:

- ``backup_opts`` - JSON containing a ``types.BackupCreateOpts`` object
- ``core/guild`` - The core guild data (``discordgo.Guild``)
- ``assets/{asset_name}`` - The guild icon data (``[]byte``)
- ``messages/{channel_id}`` - The messages in a channel along with basic attachment metadata (``[]types.BackupMessage``).
- ``dbg/*`` - Debug information. This may vary across backups and **MUST NOT** be used in restoring a backup.
- ``attachments/{attachment_id}`` - The attachments data itself