# splashtail

- Run ``make buildbot`` to build the bot
- Run ``make`` to build the stuff

## Extra/uneeded code

The following packages are currently unused and may be used in the future:

- mapofmu: Map of mutexes for concurrent access
- syncmap: Generic wrapper around sync.Map (potentially consider replacing with [https://github.com/puzpuzpuz/xsync](xsync))

## IPC Notes

- IPC uses the ``mredis.LauncherCmd`` type from ``mewld`` (``import mredis "github.com/cheesycod/mewld/redis"``)
- ``Args`` should be used to send arguments such as Task ID/Name etc.
- ``Output`` should be used to send arbitrary data to IPC