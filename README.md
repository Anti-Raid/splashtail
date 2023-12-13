# splashtail

## Building

- Run ``make buildbot`` to build the bot
- Run ``make`` to build the stuff

## Running

- First run ``./splashtail webserver`` with the environment variable ``BOOTSTRAP_COMMANDS`` set to ``true``. This will start the bot and deploy the base commands.
- From the next time, run ``./splashtail webserver`` without the ``BOOTSTRAP_COMMANDS`` variable. This is the normal way to run the bot.
- Run the job server (typically before running the bot in production) using ``./splashtail jobs``

## Extra/uneeded code

The following packages are currently unused and may be used in the future:

- mapofmu: Map of mutexes for concurrent access
- syncmap: Generic wrapper around sync.Map (potentially consider replacing with [https://github.com/puzpuzpuz/xsync](xsync))

## IPC Notes

- IPC uses the ``mredis.LauncherCmd`` type from ``mewld`` (``import mredis "github.com/cheesycod/mewld/redis"``)
- ``Args`` should be used to send arguments such as Task ID/Name etc.
- ``Output`` should be used to send arbitrary data to IPC