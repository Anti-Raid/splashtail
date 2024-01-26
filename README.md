# splashtail

Splashtail is a monorepo containing all the code needed to run and setup Anti-Raid.

## Components

- **botv2** => The core bot interface for AntiRaid
- **jobserver** => The jobserver is the component of AntiRaid responsible for handling tasks concurrently to ensure that Bot/API restarts/issues/outages does not affect ongoing backup creations/backup restores/member restores etc. The jobserver also allows code related to core functionality to be shared between the Bot (rust) and the API/website
- **webserver** (API) => The API interface for AntiRaid used for third-party integrations and the website
- **website** => The website for AntiRaid 
- **misc** => Miscellaneous code such as code used to test the WIP simpleproxy2
- **simpleproxy2** => Simple WIP gateway proxy to allow AntiRaid to be freely restarted/run multiple gateway sessions without needing to worry about IDENTITY's or compatibility with serenity/discordgo
- **arcadia** => Staff management bot for AntiRaid
    - **Forked from:** https://github.com/infinitybotlist/arcadia
    - **Commit:** 5554dadbd98ed4bd2a9594ac7af8d9ff06108322
    - **Permalink:** https://github.com/InfinityBotList/Arcadia/commit/5554dadbd98ed4bd2a9594ac7af8d9ff06108322
    - **License:** AGPLv3
    - **Affidavits:** The developers of Anti-Raid are copyright owners of Arcadia and so may use it under the AGPLv3 or any license of their choice
- **arcadia-panel** => Internal staff website to manage Anti-Raid
    - **Forked from:** https://github.com/infinitybotlist/panelv2
    - **Commit:** 61c626e5bf383fc8b277e836a9fcb9f02250bcb6
    - **Permalink:** https://github.com/InfinityBotList/panelv2/commit/61c626e5bf383fc8b277e836a9fcb9f02250bcb6
    - **License:** AGPLv3
    - **Affidavits:** The developers of Anti-Raid are copyright owners of Arcadia and so may use it under the AGPLv3 or any license of their choice

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

### Animus Magic

Due to the need for bidirectional server-bot communication, AntiRaid uses an unstable (constantly changing not crashing-type unstable) Redis PubSub API for communication

## Serializing/Deserializing for external usage

### Canonical Representations

In some cases, the internal representation of a type is not suitable for external usage. For example, the ``Module`` type used in ``botv2`` to store information about a module contains function pointers and internal fields that cannot be serialized/deserialized. In such cases, a canonical representation of the type can be made. For example, the ``Module`` type has a canonical representation of ``CanonicalModule`` which is sent by IServer etc. for use by external consumers such as the API and/or website.

**Botv2 notes:** If a canonical representation is used, the structs should be in a seperate file and the ``From<T>`` trait should be implemented on the canonical type for the internal type. This allows for easy conversion between the internal and canonical representations.

## Self-Hosting and Deployment

### Building Bot/API

- Run ``make buildbot`` to build the bot
- Run ``make`` to build just the go components
- Run ``make all`` to build everything

### Running

- First run ``./splashtail webserver`` with the environment variable ``BOOTSTRAP_COMMANDS`` set to ``true``. This will start the bot and deploy the base commands.
- From the next time, run ``./splashtail webserver`` without the ``BOOTSTRAP_COMMANDS`` variable. This is the normal way to run the bot.
- Run the job server (typically before running the bot in production) using ``./splashtail jobs``
