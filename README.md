# splashtail

Splashtail is a monorepo containing all the code needed to run and setup Anti-Raid.

# Components

## Current

- **infra** => Core infrastructure for the bot such as our fork of [`nirn-proxy`](https://github.com/anti-raid/nirn-proxy) and our fork of [`Sandwich-Daemon`](https://github.com/anti-raid/Sandwich-Daemon).
- **core** => The core modules/code for AntiRaid, written in Serenity+Poise and Rust
- **services** => The actual runnable services of AntiRaid.
- **services/rust.assetgen** => This service provides automatic asset generation (used in builds) and initial startup testing (See ``make tests``)
- **services/rust.bot** => a production-ready configuration of the AntiRaid bot. ``rust.bot`` provides a production-ready implementation of AntiRaid.
- **services/go.jobserver** => The jobserver is the component of AntiRaid responsible for handling jobs concurrently to ensure that Bot/API restarts/issues/outages does not affect ongoing backup creations/backup restores/member restores etc. The jobserver also allows code related to core functionality to be shared between the Bot (rust) and the API/website while also being isolated and easily restartable with resumable jobs allowing for greater reliability and scalability.
- **services/go.api** (API) => The API interface for AntiRaid used for third-party integrations and the website
- **website** => The website for AntiRaid 
- **data** => Miscellaneous stuff such as code used to test the WIP simpleproxy2 as well as database seeds

## Former

- **simpleproxy2** => Simple WIP gateway proxy to allow AntiRaid to be freely restarted/run multiple gateway sessions without needing to worry about IDENTITY's or compatibility with serenity/discordgo like twilight-gateway-proxy forces us to be.
    - **Replaced by:** [`Sandwich-Daemon`](https://github.com/anti-raid/Sandwich-Daemon)
    - **Reason:** Sandwich-Daemon is a more stable and scalable long-term solution

# Integration

To increase our feature set and to ensure that we are synced with upstream, AntiRaid includes several integrations (forked/taken from other AGPL3-licensed projects that we either have permission to use or owned/made in the first place):

## Current

- **Git Logs** 
    - **Folders:** ``webserver/integration/gitlogs`` and ``botv2/src/modules/gitlogs``
    - Website: [https://gitlogs.xyz](https://gitlogs.xyz)
    - Github: [https://github.com/Git-Logs](https://github.com/Git-Logs)

# Communication

All communication between the webserver, the bot and the jobserver take place over an internal RPC API and standard HTTP. This allows for easy, yet high quality integration between services on Anti-Raid.

## Serializing/Deserializing for external usage

### Canonical Representations

In some cases, the internal representation of a type is not suitable for external usage. For example, the ``Module`` type used in ``botv2`` to store information about a module contains function pointers and internal fields that cannot be serialized/deserialized. In such cases, a canonical representation of the type can be made. For example, the ``Module`` type has a canonical representation of ``CanonicalModule`` which is sent by IServer etc. for use by external consumers such as the API and/or website.

Note that if a canonical representation is used, the structs should be in a seperate file and the ``From<T>`` trait should be implemented on the canonical type for the internal type. This allows for easy conversion between the internal and canonical representations.

## Self-Hosting and Deployment

### Prerequisites

- Latest version of go (1.23)
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

*lld dependency note:* Note that currently ``mold`` is used as the linker but this can change if required. ``mold`` is still new software and critical bugs may be present. You can switch to ``lld`` by editting ``.cargo/config.toml`` and changing ``link-arg=--ld-path=/usr/bin/mold`` to ``link-arg=--ld-path=/usr/bin/ld.lld``. ``lld`` is otherwise an optional (unused) dependency that may be required in the future and is hence listed here

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

0. Install and setup PostgreSQL 16.4.
1. Install ``iblcli`` using ``git clone https://github.com/InfinityBotList/ibldev && cd ibldev && make``. This will build ``ibldev`` which is used for database seeding which you can either copy it to ``/usr/bin`` or, through other means, add to your ``$PATH``.
2. Install postgres server development headers: ``apt install postgresql-server-dev-VERSION`` where version is >= 15.
3. Create the following databases and roles:
    - ``antiraid``
    - ``frostpaw``
    - ``YOUR_USER_ACCOUNT``
    - **TIP:** Use ``CREATE ROLE <role> WITH SUPERUSER`` to create a superuser role and ``CREATE DATABASE <db> WITH OWNER <role>`` to create a database with the role as the owner. Do this for all roles and databases.

4. Run ``ibl db load data/seed.iblcli-seed`` to try loading the seed into the database. 
5. If ``ibl`` fails with a ``seed_info`` error, rerun Steps 3 and 4 to properly setup seeding

### Building Anti-Raid

1. When building AntiRaid for the first time, use ``make infra`` to build all required infrastructure such as Sandwich. Then follow Step 2 as normal. For all future invocations, just go directly to Step 2 directly.
2. Use ``make build`` to build AntiRaid. Note that the first build may take a while however subsequent builds should be faster thanks to incremental compilation.

### Configuration

Enter the ``infra/Sandwich-Daemon`` folder. Copy ``example_sandwich.yaml`` to ``sandwich.yaml`` and adjust as desired (at the minimum, change the Token [indicated by ``TOKENHERE``], Client ID and Webhook)

Run ``./out/go.api``. This will create ``config.yaml.sample``. Copy this to ``config.yaml`` and fill in the required fields as per the below:

- ``discord_auth``: Use https://discord.dev to fill this out. Use a random string for ``dp_secret``.

Before proceeding, run ``make tests`` to ensure your configuration is setup correctly.

### Running

Systemd example services are provided in ``data/systemd-example``

1. Start nirn proxy

**Systemd:** ``ar-nirn-staging.service``
**Example Command:** ``infra/nirn-proxy/nirn-proxy cache-endpoints=false ws-proxy=ws://127.0.0.1:3600 port=3221 ratelimit-over-408 endpoint-rewrite=/api/gateway/bot@http://127.0.0.1:29334/antiraid,/api/v*/gateway/bot@http://127.0.0.1:29334/antiraid``


2. Run Sandwich

**Systemd:** ``ar-sandwich-staging.service``
**Example Command:** ``infra/Sandwich-Daemon/out/sandwich -configurationPath=sandwich.yaml -prometheusAddress :3931 -grpcHost localhost:10294 -grpcNetwork tcp -httpEnabled  --httpHost 0.0.0.0:29334 -level debug``

3. Run Job Server

**Systemd:** ``splashtail-staging-jobs.service``
**Example Commands:** ``out/go.jobserver``

4. Run Bot

**Systemd:** ``splashtail-staging-bot.service``
**Example Commands:** ``out/rust.bot.loader``

5. Run API

**Systemd:** ``splashtail-staging-webserver.service``
**Example Commands:** ``out/rust.api``

# TODO List

- Server lockdown on limits hit []
- Thorough testing of Anti-Raid []
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

For future releases (Not yet decided):

- Advanced Active Anti-Spam (maybe AI-based image classification blocks) []

-------------------------------------------------------------------------------------------

# Future idea

**Note: All of the below are future ideas for an automated antinuke system. Feedback and PRs to implement something like this would be appreciated**

## Anti-Nuke Methods

A nuke on a Discord server is defined as the following:

### Bans/Kicks

- A. Small Server (1 < m < 100): At least 10% of the server has been banned in a period of 10 minutes
- B. Medium Server (100 < m < 500): At least 5% of the server has been banned in a period of 15 minutes
- C. Large Server (500 < m < 1000): At least 2% of the server has been banned in a period of 17 minutes
- D. Very Large Server (m > 1000): At least 1% of the server has been banned in a period of 17 minutes

### Channel Mods

- A. Small Server (1 < m < 1000): At least 10% of the server's channels have been created/editted/deleted in a period of 10 minutes
- B. Large Server (m > 1000): At least 1% of the server's channels have been created/editted/deleted in a period of 17 minutes

### Role Mods

- A. Small Server (1 < m < 1000): At least 10% of the server's roles have been created/editted/deleted in a period of 10 minutes
- B. Large Server (m > 1000): At least 1% of the server's roles have been created/editted/deleted in a period of 17 minutes

**Note that the above set of constraints should be easy to change and should be stored in a database.**

Once a nuke has been detected, all users with the capability to perform the nuke should be temporarily neutered. Then, investigation should be performed prior to giving back permissions.

### Neutering

TODO

### Investigation

Multiple strategies should be launched in parallel to try and determine who were involved in the nuke.

A. Audit logs: This is the most reliable method to determine who was involved in the nuke. However, Discord's implementation tends to stall when a large number of actions are performed in a short period of time.
B. Deduction: This is a more manual method of determining who was involved in the nuke but may not be reliable:

- Check the roles of the banned users and see who could have performed it. Such users should be marked as suspect
- Check the channels that were created/editted/deleted and see who could have performed it. Such users should be marked as suspect
- Check the roles that were created/editted/deleted and see who could have performed it. Such users should be marked as suspect
- Moderator reports: Moderators should be allowed to volunteer information on who they suspect was involved in the nuke which can then be crowdsourced among all moderators.

### Reversal

Once the investigation has been completed, the nuke should be reversed. This involves:

- Unbanning all users who were banned
- Restoring all channels that were deleted if possible
- Restoring all roles that were deleted if possible
- Restoring all roles that were editted if possible
- Unneutering all moderators who were neutered
