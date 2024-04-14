# splashtail

Splashtail is a monorepo containing all the code needed to run and setup Anti-Raid.

## Components

- **infra** => Core infrastructure for the bot
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
    - **Affidavits:** Licensed under the AGPLv3
- **arcadia-panel** => Internal staff website to manage Anti-Raid
    - **Forked from:** https://github.com/infinitybotlist/panelv2
    - **Commit:** 61c626e5bf383fc8b277e836a9fcb9f02250bcb6
    - **Permalink:** https://github.com/InfinityBotList/panelv2/commit/61c626e5bf383fc8b277e836a9fcb9f02250bcb6
    - **License:** AGPLv3
    - **Affidavits:** Licensed under the AGPLv3

### Integration

To increase our feature set and to popularize these bots, AntiRaid includes several integrations (forked/taken from other AGPL3-licensed projects that we either have permission to use or... owned/made the code for in the first place):

- **Git Logs** 
    - **Folders:** ``webserver/integration/gitlogs`` and ``botv2/src/modules/gitlogs``
    - Website: [https://gitlogs.xyz](https://gitlogs.xyz)
    - Github: [https://github.com/Git-Logs](https://github.com/Git-Logs)

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

### Prerequisites

- Latest version of go (1.22)
- Latest version of rust (``nightly`` recommended)
- ``pkg-config``
- ``openssl``
- ``libssl-dev``
- ``build-essential``
- ``sqlx-cli``
- ``clangd``
- ``clang``
- ``libclang-14-dev`` 
- ``lld`` 
- ``mold``

*lld dependency note:* Note that currently ``mold`` is used as the linker but this can change if required. ``mold`` is still new software and critical bugs may be present. You can switch to ``lld`` by editting ``botv2/.cargo/config.toml`` and changing ``link-arg=--ld-path=/usr/bin/mold`` to ``link-arg=--ld-path=/usr/bin/ld.lld``. ``lld`` is otherwise an optional (unused) dependency that may be required in the future and is hence listed here

On Ubuntu, you can use ``sudo apt install pkg-config openssl libssl-dev build-essential clangd clang libclang-14-dev lld mold`` to install most of the dependencies. ``sqlx-cli`` is a bit special as it is tied to the version of ``sqlx`` used on the project (??). Use ``cargo install --version 0.7.4 sqlx-cli`` to install ``sqlx-cli``.

For Go, use the following commands:

```bash
sudo add-apt-repository ppa:longsleep/golang-backports
sudo apt update
sudo apt install golang-go
```

For Rust, use the following command (using Rustup): ``curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh``

### Database Seeding

To load the database seed, follow the following steps:

0. Install and setup PostgreSQL 15.5 or 16 (production uses 15.5 currently)
1. Install ``iblcli`` using ``go install github.com/infinitybotlist/ibl@latest``. This will install ``iblcli`` which is used for database seeding to ``~/go/bin`` from which you can either copy it to ``/usr/bin`` or add it to your ``$PATH``.
2. Install postgres server development headers: ``apt install postgresql-server-dev-VERSION`` where version is >= 15.
3. Create the following databases and roles:
    - ``antiraid``
    - ``frostpaw``
    - ``YOUR_USER_ACCOUNT``
    - **TIP:** Use ``CREATE ROLE <role> WITH SUPERUSER`` to create a superuser role and ``CREATE DATABASE <db> WITH OWNER <role>`` to create a database with the role as the owner. Do this for all roles and databases.

4. Run ``ibl db load data/db_seed.iblcli-seed`` to try loading the seed into the database. 

**Note that due to a bug in ``ibl``, this may fail with an error related to copying the extension ``semver``. If this occurs, follow the below steps**

5. Enter the created ``semver`` folder and run ``make && sudo make install`` to build and install the extension with the correct permissions.
6. Exit the ``semver`` folder and delete it (``rm -r semver``)
7. Rerun Step 3

**End of note. All further steps can be followed by all**

8. If ``ibl`` fails with a ``seed_info`` error, rerun Step 3 to properly setup seeding
9. Install sqlx cli: 

``cargo install sqlx-cli``

### SurrealDB setup

AntiRaid internally uses SurrealDB for certain bot modules such as limits. To setup the external surreal server, use the following command:

``surreal start --bind 127.0.0.1;6318 --log info --auth --user splashtail --pass PASSWORD --deny-net --deny-guests file:antiraid-development.db``

Then, in config.yaml, set the following
```
surreal: 
      url: 127.0.0.1:6318
      username: splashtail
      password: PASSWORD
```


### Building Bot/API

- Run ``make buildbot`` to build the bot
- Run ``make`` to build just the go components
- Run ``make all`` to build everything
- Run ``make restartwebserver`` to restart the webserver [needs systemd working]

After first building the bot, you will need to run ``cp -v botv2/target/release/botv2 botv2`` to copy the binary to the correct location. This is later automated for you if you use the Makefile commands listed above to setup and update the bot

### Configuration

Run ``./splashtail webserver``. This will create ``config.yaml.sample``. Copy this to ``config.yaml`` and fill in the required fields as per the below:

- ``discord_auth``: use https://discord.dev to fill this out on a new application. Be sure to edit ``can_use_bot`` with your User ID 
- ``animus_magic_channel``: Set ``animus_magic_channel.staging`` to ``animus_magic-staging`` and ``animus_magic_channel.prod`` to ``animus_magic-prod``. These are the Redis PubSub channels used for communication between the bot, server and jobserver

### Running

- First run ``./splashtail webserver`` with the environment variable ``BOOTSTRAP_COMMANDS`` set to ``true``. This will start the bot and deploy the base commands.
- From the next time, run ``./splashtail webserver`` without the ``BOOTSTRAP_COMMANDS`` variable. This is the normal way to run the bot.
- Run the job server (typically before running the bot in production) using ``./splashtail jobs``

### Proxy

Anti-Raid has 2 gateway proxies that can optionally be used to reduce the number of ``IDENTIFY`` packets needed:

- **simplegwproxy** - Still buggy, not recommended for production
- **twiglight gateway-proxy** (fork with patches for serenity) - Also may be buggy but more production quality

# TODO List

For initial release:

- Stabilizing the module+command permission system [X]  
- Add a monitoring script to probe clusters [X]
- Add the ability to enable/disable modules [X] and commands [WIP, WU] and also override command permissions [WIP]
- Server Member Backup/Restore []
- Basic Moderation (warn/kick/ban+tempban/unban) [X]
- Punishment module (give punishment based on stings)
- Basic Anti-Raid (lockserver/unlockserver/lockchannel/unlockchannel) []
- Basic Anti-Spam (anti-invite, anti-everyone ping) [X]
- Basic Anti-Spam Punishment integration []
- Basic Anti-Spam audit log integration []
- Basic Anti-Spam server name/server icon protection []
- Audit Logs [X]
- Integrating audit logs into moderation [X]
- Basic utility functions (if needed) [NOT NEEDED YET]
- Stabilizing the `Limits` module [X]
- Stabilizing the ``Server Backups`` module [X]

*Key*

- X: Done
- WIP: Work in Progress
- WU: Website needs to be updated but added to bot
- BU: Bot needs to be updated but added to website

For future releases (or even initial if time permits):

- Advanced Active Anti-Spam (maybe AI-based image classification blocks) []

### Permissions

- A command is the base unit for access control. This means that all operations with differing access controls must have commands associated with them.
- This means that all operations (list backup, create/restore backup, delete backup) *MUST* have associated commands
- Sometimes, an operation (such as a web-only operation) may not have a module/command associated with it. In such cases, a 'virtual' module should be used. Virtual modules are modules with commands that are not registered via Discord's API. They are used to group commands together for access control purposes and to ensure that each operation is tied to a command

### Development

Run ``cargo sqlx prepare`` in the ``botv2`` folder before committing anything. This will regenerate the SQLX files used by the bot to interact with the database. Note that ``make buildbot`` will automatically run this now.
