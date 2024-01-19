# splashtail

Splashtail is a monorepo containing all the code needed to run and setup Anti-Raid.

## Components

- **botv2** => The core bot interface for AntiRaid
- **jobserver** => The jobserver is the component of AntiRaid responsible for handling tasks concurrently to ensure that Bot/API restarts/issues/outages does not affect ongoing backup creations/backup restores/member restores etc. The jobserver also allows code related to core functionality to be shared between the Bot (rust) and the API/website
- **webserver** (API) => The API interface for AntiRaid used for third-party integrations and the website
- **website** => The website for AntiRaid 
- **misc** => Miscellaneous code such as code used to test the WIP simpleproxy2
- **simpleproxy2** => Simple WIP gateway proxy to allow AntiRaid to be freely restarted/run multiple gateway sessions without needing to worry about IDENTITY's or compatibility with serenity/discordgo

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
