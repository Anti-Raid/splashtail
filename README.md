# splashtail

Splashtail is a monorepo containing all the code needed to run and setup Anti-Raid.

# Components

## Current

- **infra** => Core infrastructure for the bot such as `wafflepaw` (monitoring service) our fork of [`nirn-proxy`](https://github.com/anti-raid/nirn-proxy) and our fork of [`Sandwich-Daemon`](https://github.com/anti-raid/Sandwich-Daemon).
- **botv2** => The core bot interface for AntiRaid, written in Serenity+Poise and Rust
- **jobserver** => The jobserver is the component of AntiRaid responsible for handling jobs concurrently to ensure that Bot/API restarts/issues/outages does not affect ongoing backup creations/backup restores/member restores etc. The jobserver also allows code related to core functionality to be shared between the Bot (rust) and the API/website while also being isolated and easily restartable with resumable jobs allowing for greater reliability and scalability.
- **webserver** (API) => The API interface for AntiRaid used for third-party integrations and the website
- **website** => The website for AntiRaid 
- **data** => Miscellaneous stuff such as code used to test the WIP simpleproxy2 as well as database seeds

## Former

- **simpleproxy2** => Simple WIP gateway proxy to allow AntiRaid to be freely restarted/run multiple gateway sessions without needing to worry about IDENTITY's or compatibility with serenity/discordgo like twilight-gateway-proxy forces us to be.
    - **Replaced by:** [`Sandwich-Daemon`](https://github.com/anti-raid/Sandwich-Daemon)
    - **Reason:** Sandwich-Daemon is a more stable and scalable long-term solution
- **arcadia** => Staff management bot for AntiRaid.
    - **Replaced by:** None
    - **Reason:** Removed for now as the bot's internals are still rapidly changing
    - **Forked from:** https://github.com/infinitybotlist/arcadia
    - **Commit:** 5554dadbd98ed4bd2a9594ac7af8d9ff06108322
    - **Permalink:** https://github.com/InfinityBotList/Arcadia/commit/5554dadbd98ed4bd2a9594ac7af8d9ff06108322
    - **License:** AGPLv3
    - **Affidavits:** Licensed under the AGPLv3
- **arcadia-panel** => Internal staff website to manage Anti-Raid.
    - **Replaced by:** None
    - **Reason:** Removed for now as the bot's internals are still rapidly changing
    - **Forked from:** https://github.com/infinitybotlist/panelv2
    - **Commit:** 61c626e5bf383fc8b277e836a9fcb9f02250bcb6
    - **Permalink:** https://github.com/InfinityBotList/panelv2/commit/61c626e5bf383fc8b277e836a9fcb9f02250bcb6
    - **License:** AGPLv3
    - **Affidavits:** Licensed under the AGPLv3

# Integration

To increase our feature set and to ensure that we are synced with upstream, AntiRaid includes several integrations (forked/taken from other AGPL3-licensed projects that we either have permission to use or owned/made in the first place):

## Current

- **Git Logs** 
    - **Folders:** ``webserver/integration/gitlogs`` and ``botv2/src/modules/gitlogs``
    - Website: [https://gitlogs.xyz](https://gitlogs.xyz)
    - Github: [https://github.com/Git-Logs](https://github.com/Git-Logs)

# Communication

Communication between the bot, jobserver, server and the ``mewld`` clusterer (used to run multiple clusters of the bot with each cluster responsible for a set of shards) happens in 3 primary ways.

## Mewld IPC

For ``mewld``-``bot`` communication, core state such as Cluster ID, Cluster Name, Shard List, Redis Channel etc. are given as command-line arguments and are parsed by the ``argparse`` module of botv2's IPC subsystem.

The ``mredis.LauncherCmd`` type from ``mewld`` (``import mredis "github.com/cheesycod/mewld/redis"``) is the primary type used for communication. ``Args`` should be used to send arguments for the IPC command and ``Output`` should be used to send arbitrary data to IPC. Diagnostic payloads (used to check uptimes and gather the guild/user count per cluster) are a special case and use ``mredis.LauncherCmd`` for the request and a ``diagPayload`` (renamed to ``MewldDiagResponse`` in the bot).

## RPC

All communication between the webserver, the bot and the jobserver take place over RPC and standard HTTP. This allows for easy, yet high quality integration between services on Anti-Raid.

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

### Building Anti-Raid

**When first compiling Anti-Raid, run ``make buildall`` to build all components including mewld_web.**. After the first run, you can use the below commands to build individual components.

- Run ``make buildservices`` to build the bot and all components
- Run ``make all`` to build everything
- Run ``make reloadwebserver`` to restart the webserver [needs systemd working]

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
- Add the ability to enable/disable modules [X] and commands [X] and also override command permissions [X]
- Basic Moderation (warn/kick/ban+tempban/unban) [X]
- Punishment module (give punishment based on stings) [X]
- Lockdown (qsl (quick server lockdown)/tsl (traditional server lockdown)/scl (single channel lockdown)) [X]
- Server lockdown on limits hit []
- Basic Anti-Spam (anti-invite, anti-everyone ping) [X]
- Basic Anti-Spam Punishment integration [X]
- Basic Anti-Spam audit log integration []
- Basic Anti-Spam server name/server icon protection [X]
- Audit Logs [X]
- Better serde-based audit log expansion [WIP]
- Integrating audit logs into moderation [X]
- Stabilizing the `Limits` module [X]
- Stabilizing the ``Server Backups`` module [X]
- Stabilizing the ``Inspector`` module [X]
- Integration of settings into bot+website [X]
- Thorough testing of Anti-Raid []


For later:
- Webhook creation and deletion monitoring []
- Creating and stabilizing the ``Captcha`` module []
- Server Member Backup/Restore []
- Dangerous role quarantine/removal []
- Tags/custom tags []

*Key*

- X: Done
- WIP: Work in Progress
- WU: Website needs to be updated but added to bot
- BU: Bot needs to be updated but added to website

For future releases (or even initial if time permits):

- Advanced Active Anti-Spam (maybe AI-based image classification blocks) []

# Permissions

- A command is the base unit for access control. This means that all operations with differing access controls must have commands associated with them.
- This means that all operations (list backup, create/restore backup, delete backup) *MUST* have associated commands
- Sometimes, an operation (such as a web-only operation) may not have a module/command associated with it. In such cases, a 'virtual' module should be used. Virtual modules are modules with commands that are not registered via Discord's API. They are used to group commands together for access control purposes and to ensure that each operation is tied to a command

# Development

Run ``cargo sqlx prepare`` in the ``botv2`` folder before committing anything. This will regenerate the SQLX files used by the bot to interact with the database. Note that ``make buildbot`` will automatically run this now.

When changing `PermissionChecks` validation rules, be sure to edit the following locations and keep them up to date:

- ``botv2 [silverpelt::validators::parse_permission_checks]`` (the consts)
- ``webserver/rpc/checks.go`` (the consts)

-------------------------------------------------------------------------------------------

# Anti-Nuke Methods

A nuke on a Discord server is defined as the following:

## Bans/Kicks

- A. Small Server (1 < m < 100): At least 10% of the server has been banned in a period of 10 minutes
- B. Medium Server (100 < m < 500): At least 5% of the server has been banned in a period of 15 minutes
- C. Large Server (500 < m < 1000): At least 2% of the server has been banned in a period of 17 minutes
- D. Very Large Server (m > 1000): At least 1% of the server has been banned in a period of 17 minutes

## Channel Mods

- A. Small Server (1 < m < 1000): At least 10% of the server's channels have been created/editted/deleted in a period of 10 minutes
- B. Large Server (m > 1000): At least 1% of the server's channels have been created/editted/deleted in a period of 17 minutes

## Role Mods

- A. Small Server (1 < m < 1000): At least 10% of the server's roles have been created/editted/deleted in a period of 10 minutes
- B. Large Server (m > 1000): At least 1% of the server's roles have been created/editted/deleted in a period of 17 minutes

**Note that the above set of constraints should be easy to change and should be stored in a database.**

Once a nuke has been detected, all users with the capability to perform the nuke should be temporarily neutered. Then, investigation should be performed prior to giving back permissions.

## Neutering

TODO

## Investigation

Multiple strategies should be launched in parallel to try and determine who were involved in the nuke.

A. Audit logs: This is the most reliable method to determine who was involved in the nuke. However, Discord's implementation tends to stall when a large number of actions are performed in a short period of time.
B. Deduction: This is a more manual method of determining who was involved in the nuke but may not be reliable:

- Check the roles of the banned users and see who could have performed it. Such users should be marked as suspect
- Check the channels that were created/editted/deleted and see who could have performed it. Such users should be marked as suspect
- Check the roles that were created/editted/deleted and see who could have performed it. Such users should be marked as suspect
- Moderator reports: Moderators should be allowed to volunteer information on who they suspect was involved in the nuke which can then be crowdsourced among all moderators.

## Reversal

Once the investigation has been completed, the nuke should be reversed. This involves:

- Unbanning all users who were banned
- Restoring all channels that were deleted if possible
- Restoring all roles that were deleted if possible
- Restoring all roles that were editted if possible
- Unneutering all moderators who were neutered

## Env files

- ``core/go.std/data/current-env`
- ``core/rust.config/current-env`