# splashtail

Splashtail is a monorepo containing all the code needed to run and setup Anti-Raid.

## Components

- **botv2** => The core bot interface for AntiRaid
- **jobserver** => The jobserver is the component of AntiRaid responsible for handling tasks concurrently to ensure that Bot/API restarts/issues/outages does not affect ongoing backup creations/backup restores/member restores etc. The jobserver also allows code related to core functionality to be shared between the Bot (rust) and the API/website
- **webserver** (API) => The API interface for AntiRaid used for third-party integrations and the website
- **website** => The website for AntiRaid 
- **misc** => Miscellaneous code such as code used to test the WIP simpleproxy2
- **simpleproxy2** => Simple WIP gateway proxy to allow AntiRaid to be freely restarted/run multiple gateway sessions without needing to worry about IDENTITY's or compatibility with serenity/discordgo

## Communication

Communication between the bot, jobserver, server and the ``mewld`` clusterer (used to run multiple clusters of the bot with each cluster responsible for a set of shards) happens in 3 primary ways.

### Mewld IPC

For ``mewld``-``bot`` communication, core state such as Cluster ID, Cluster Name, Shard List, Redis Channel etc. are given as command-line arguments and are parsed by the ``argparse`` module of botv2's IPC subsystem.

The ``mredis.LauncherCmd`` type from ``mewld`` (``import mredis "github.com/cheesycod/mewld/redis"``) is the primary type used for communication. ``Args`` should be used to send arguments for the IPC command and ``Output`` should be used to send arbitrary data to IPC. Diagnostic payloads (used to check uptimes and gather the guild/user count per cluster) are a special case and use ``mredis.LauncherCmd`` for the request and a ``diagPayload`` (renamed to ``MewldDiagResponse`` in the bot).

### Jobserver HTTP

Jobserver provides an HTTP API for all communication with the jobserver. Individual clients must be provided a Client Name and Client Secret to both identify them and to protect against unauthorized access to the jobserver. These secrets must be placed under ``jobserver_secrets`` in the ``config.yaml`` file (``meta`` section) and *must be cryptographically secure and unique per client*. E.g:

```yaml
jobserver_secrets:
staging:
    api: SECRET_1
    bot: SECRET_2
prod:
    api: SECRET_3
    bot: SECRET_$
```

### Bot IServer

Due to the need for unidirectional server-bot communication, redis simply did not scale that well (does not provide functionality such as timeouts, structured parsing). The bot therefore provides an unstable (constantly changing not crashing-type unstable) HTTP API (``IServer``) to allow for server-bot communication.

The base port for the IServer is defined by ``bot_iserver_base_port`` in the ``meta`` section of ``config.yaml``. The IServer for a given cluster is then available at ``http://localhost:<bot_iserver_base_port + cluster id>``.

## Building Bot/API

- Run ``make buildbot`` to build the bot
- Run ``make`` to build just the go components
- Run ``make all`` to build everything

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
- ``Args`` should be used to send arguments for the IPC command
- ``Output`` should be used to send arbitrary data to IPC

Note that the jobserver has a custom HTTP-based API for managing tasks
