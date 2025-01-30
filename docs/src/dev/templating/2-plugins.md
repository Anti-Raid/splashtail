# @antiraid/datetime

This plugin allows for the managing timezones.

## Types

<div id="type.Timezone" />

### Timezone

A timezone object.

#### Methods

##### Timezone:utcToTz

```lua
function Timezone:utcToTz(year: number, month: number, day: number, hours: number, minutes: number, secs: number, all: boolean?)
```

Translates a timestamp in UTC time to a datetime in the said specific timezone.

###### Parameters

- `year` ([number](#type.number)): The year to translate.
- `month` ([number](#type.number)): The month to translate.
- `day` ([number](#type.number)): The day to translate.
- `hours` ([number](#type.number)): The hours to translate.
- `minutes` ([number](#type.number)): The minutes to translate.
- `secs` ([number](#type.number)): The seconds to translate.
- `all` ([boolean?](#type.boolean)): Whether to return both offsets if the time is ambiguous.


###### Returns

- `date` ([DateTime](#type.DateTime)): The translated datetime.- `date2` ([DateTime?](#type.DateTime)): The second translated datetime if the time is ambiguous.
##### Timezone:tzToUtc

```lua
function Timezone:tzToUtc(year: number, month: number, day: number, hours: number, minutes: number, secs: number, all: boolean?)
```

Translates a timestamp in the specified timezone to a datetime in UTC.

###### Parameters

- `year` ([number](#type.number)): The year to translate.
- `month` ([number](#type.number)): The month to translate.
- `day` ([number](#type.number)): The day to translate.
- `hours` ([number](#type.number)): The hours to translate.
- `minutes` ([number](#type.number)): The minutes to translate.
- `secs` ([number](#type.number)): The seconds to translate.
- `all` ([boolean?](#type.boolean)): Whether to return both offsets if the time is ambiguous.


###### Returns

- `date` ([DateTime](#type.DateTime)): The translated datetime.- `date2` ([DateTime?](#type.DateTime)): The second translated datetime if the time is ambiguous.
##### Timezone:timeUtcToTz

```lua
function Timezone:timeUtcToTz(hours: number, minutes: number, secs: number): DateTime
```

Translates a time of the current day in UTC time to a datetime in the said specific timezone.

###### Parameters

- `hours` ([number](#type.number)): The hours to translate.
- `minutes` ([number](#type.number)): The minutes to translate.
- `secs` ([number](#type.number)): The seconds to translate.


###### Returns

- `date` ([DateTime](#type.DateTime)): The translated datetime.
##### Timezone:timeTzToUtc

```lua
function Timezone:timeTzToUtc(hours: number, minutes: number, secs: number): DateTime
```

Translates a time of the current day in the said specific timezone to a datetime in UTC.

###### Parameters

- `hours` ([number](#type.number)): The hours to translate.
- `minutes` ([number](#type.number)): The minutes to translate.
- `secs` ([number](#type.number)): The seconds to translate.


###### Returns

- `date` ([DateTime](#type.DateTime)): The translated datetime.
##### Timezone:now

```lua
function Timezone:now(): DateTime
```

Translates the current timestamp to a datetime in the said specific timezone.

###### Returns

- `date` ([DateTime](#type.DateTime)): The translated datetime.


##### Timezone:fromTime

Given a unix time, returns a DateTime object with this timezone

```lua
function Timezone:fromTime(time: number): DateTime
```

###### Parameters

- `time` ([number](#type.number)): The unix time (in seconds) to convert.

###### Returns

- `dt` ([DateTime](#type.DateTime)): The DateTime object with the specified timezone converted from the time.

##### Timezone:fromTimeMillis

Given a unix time in milliseconds, returns a DateTime object with this timezone

```lua
function Timezone:fromTimeMillis(time: i64): DateTime
```

###### Parameters
- `time` ([i64](#type.i64)): The unix time (in milliseconds) to convert.

###### Returns
- `dt` ([DateTime](#type.DateTime)): The DateTime object with the specified timezone converted from the time.

##### Timezone:fromTimeMicros
Given a unix time in microseconds, returns a DateTime object with this timezone

```lua
function Timezone:fromTimeMicros(time: i64): DateTime
```

###### Parameters
- `time` ([i64](#type.i64)): The unix time (in microseconds) to convert.

###### Returns
- `dt` ([DateTime](#type.DateTime)): The DateTime object with the specified timezone converted from the time.

##### Timezone:fromTimeNanos
Given a unix time in nanoseconds, returns a DateTime object with this timezone

```lua
function Timezone:fromTimeNanos(time: i64): DateTime
```

###### Parameters
- `time` ([i64](#type.i64)): The unix time (in nanoseconds) to convert.

###### Returns
- `dt` ([DateTime](#type.DateTime)): The DateTime object with the specified timezone converted from the time.

<div id="type.TimeDelta" />

### TimeDelta

A time delta object. Supports addition/subtraction with another TimeDelta object as well as comparisons with them.



#### Fields

- `nanos` ([number](#type.number)): The number of nanoseconds in the time delta.
- `micros` ([number](#type.number)): The number of microseconds in the time delta.
- `millis` ([number](#type.number)): The number of milliseconds in the time delta.
- `seconds` ([number](#type.number)): The number of seconds in the time delta.
- `minutes` ([number](#type.number)): The number of minutes in the time delta.
- `hours` ([number](#type.number)): The number of hours in the time delta.
- `days` ([number](#type.number)): The number of days in the time delta.
- `weeks` ([number](#type.number)): The number of weeks in the time delta.


#### Methods

##### TimeDelta:offset_string

```lua
function TimeDelta:offset_string(): string
```

Returns the offset as a string.

###### Returns

- `offset` ([string](#type.string)): The offset as a string.


<div id="type.DateTime" />

### DateTime

A datetime object. Supports addition/subtraction with TimeDelta objects as well as comparisons with other DateTime objects.



#### Fields

- `year` ([number](#type.number)): The year of the datetime.
- `month` ([number](#type.number)): The month of the datetime.
- `day` ([number](#type.number)): The day of the datetime.
- `hour` ([number](#type.number)): The hour of the datetime.
- `minute` ([number](#type.number)): The minute of the datetime.
- `second` ([number](#type.number)): The second of the datetime.
- `timestamp_seconds` ([number](#type.number)): The timestamp in seconds of the datetime from the Unix epoch.
- `timestamp_millis` ([number](#type.number)): The timestamp in milliseconds of the datetime from the Unix epoch.
- `timestamp_micros` ([number](#type.number)): The timestamp in microseconds of the datetime from the Unix epoch.
- `timestamp_nanos` ([number](#type.number)): The timestamp in nanoseconds of the datetime from the Unix epoch.
- `tz` ([Timezone](#type.Timezone)): The timezone of the datetime.
- `offset` ([TimeDelta](#type.TimeDelta)): The offset of the datetime.


#### Methods

##### DateTime:with_timezone

```lua
function DateTime:with_timezone(tz: Timezone): DateTime
```

Converts the datetime to the specified timezone.

###### Parameters

- `tz` ([Timezone](#type.Timezone)): The timezone to convert to.


###### Returns

- `dt` ([DateTime](#type.DateTime)): The converted datetime.
##### DateTime:format

```lua
function DateTime:format(format: string): string
```

Formats the datetime using the specified format string.

###### Parameters

- `format` ([string](#type.string)): The format string to use.


###### Returns

- `formatted` ([string](#type.string)): The formatted datetime.
##### DateTime:duration_since

```lua
function DateTime:duration_since(other: DateTime): TimeDelta
```

Calculates the duration between the current datetime and another datetime.

###### Parameters

- `other` ([DateTime](#type.DateTime)): The other datetime to calculate the duration to.


###### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The duration between the two datetimes.


## Methods

### new

```lua
function new(timezone: string): Timezone
```

Returns a new Timezone object if the timezone is recognized/supported.

#### Parameters

- `timezone` ([string](#type.string)): The timezone to get the offset for.


#### Returns

- `tzobj` ([Timezone](#type.Timezone)): The timezone userdata object.

### timedelta_weeks

```lua
function timedelta_weeks(weeks: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of weeks.

#### Parameters

- `weeks` ([number](#type.number)): The number of weeks.


#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

### timedelta_days

```lua
function timedelta_days(days: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of days.

#### Parameters

- `days` ([number](#type.number)): The number of days.


#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

### timedelta_hours

```lua
function timedelta_hours(hours: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of hours.

#### Parameters

- `hours` ([number](#type.number)): The number of hours.


#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

### timedelta_minutes

```lua
function timedelta_minutes(minutes: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of minutes.

#### Parameters

- `minutes` ([number](#type.number)): The number of minutes.


#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

### timedelta_seconds

```lua
function timedelta_seconds(seconds: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of seconds.

#### Parameters

- `seconds` ([number](#type.number)): The number of seconds.


#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

### timedelta_millis

```lua
function timedelta_millis(millis: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of milliseconds.

#### Parameters

- `millis` ([number](#type.number)): The number of milliseconds.


#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

### timedelta_micros

```lua
function timedelta_micros(micros: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of microseconds.

#### Parameters

- `micros` ([number](#type.number)): The number of microseconds.


#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

### timedelta_nanos

```lua
function timedelta_nanos(nanos: number): TimeDelta
```

Creates a new TimeDelta object with the specified number of nanoseconds.

#### Parameters

- `nanos` ([number](#type.number)): The number of nanoseconds.

#### Returns

- `td` ([TimeDelta](#type.TimeDelta)): The TimeDelta object.

---
# @antiraid/discord

This plugin allows for templates to interact with the Discord API. Types are as defined by Discord if not explicitly documented

## Types

<div id="type.GetAuditLogOptions" />

### GetAuditLogOptions

Options for getting audit logs in Discord

```json
{
  "action_type": 1,
  "user_id": "0",
  "before": "0",
  "limit": 0
}
```

#### Fields

- `action_type` ([Serenity.AuditLogs.Action?](#type.Serenity.AuditLogs.Action)): The action type to filter by
- `user_id` ([string?](#type.string)): The user ID to filter by
- `before` ([string?](#type.string)): The entry ID to filter by
- `limit` ([number?](#type.number)): The limit of entries to return


<div id="type.GetChannelOptions" />

### GetChannelOptions

Options for getting a channel in Discord

```json
{
  "channel_id": "0"
}
```

#### Fields

- `channel_id` ([string](#type.string)): The channel ID to get


<div id="type.EditChannel" />

### EditChannel

The data for editing a channel in Discord

```json
{
  "name": "my-channel",
  "type": 0,
  "position": 7,
  "topic": "My channel topic",
  "nsfw": true,
  "rate_limit_per_user": 5,
  "user_limit": 10,
  "parent_id": "0",
  "rtc_region": "us-west",
  "video_quality_mode": 1,
  "default_auto_archive_duration": 1440,
  "flags": 18,
  "default_reaction_emoji": {
    "emoji_id": "0",
    "emoji_name": null
  },
  "status": "online",
  "archived": false,
  "auto_archive_duration": 1440,
  "locked": false,
  "invitable": true
}
```

#### Fields

- `type` ([number?](#type.number)): The type of the channel
- `position` ([number?](#type.number)): The position of the channel
- `topic` ([string?](#type.string)): The topic of the channel
- `nsfw` ([bool?](#type.bool)): Whether the channel is NSFW
- `rate_limit_per_user` ([number?](#type.number)): The rate limit per user/Slow mode of the channel
- `bitrate` ([number?](#type.number)): The bitrate of the channel
- `permission_overwrites` ([{Serenity.PermissionOverwrite}?](#type.Serenity.PermissionOverwrite)): The permission overwrites of the channel
- `parent_id` ([string??](#type.string)): The parent ID of the channel
- `rtc_region` ([string??](#type.string)): The RTC region of the channel
- `video_quality_mode` ([number?](#type.number)): The video quality mode of the channel
- `default_auto_archive_duration` ([number?](#type.number)): The default auto archive duration of the channel
- `flags` ([string?](#type.string)): The flags of the channel
- `available_tags` ([{Serenity.ForumTag}?](#type.Serenity.ForumTag)): The available tags of the channel
- `default_reaction_emoji` ([Serenity.ForumEmoji??](#type.Serenity.ForumEmoji)): The default reaction emoji of the channel
- `default_thread_rate_limit_per_user` ([number?](#type.number)): The default thread rate limit per user
- `default_sort_order` ([number?](#type.number)): The default sort order of the channel
- `default_forum_layout` ([number?](#type.number)): The default forum layout of the channel
- `archived` ([bool?](#type.bool)): Whether the thread is archived (thread only)
- `auto_archive_duration` ([number?](#type.number)): The auto archive duration of the thread (thread only)
- `locked` ([bool?](#type.bool)): Whether the thread is locked (thread only)
- `invitable` ([bool?](#type.bool)): Whether the thread is invitable (thread only)
- `applied_tags` ([{Serenity.ForumTag}?](#type.Serenity.ForumTag)): The applied tags of the thread (thread only)


<div id="type.EditChannelOptions" />

### EditChannelOptions

Options for editing a channel in Discord

```json
{
  "channel_id": "0",
  "reason": "",
  "data": {
    "name": "my-channel",
    "type": 0,
    "position": 7,
    "topic": "My channel topic",
    "nsfw": true,
    "rate_limit_per_user": 5,
    "user_limit": 10,
    "parent_id": "0",
    "rtc_region": "us-west",
    "video_quality_mode": 1,
    "default_auto_archive_duration": 1440,
    "flags": 18,
    "default_reaction_emoji": {
      "emoji_id": "0",
      "emoji_name": null
    },
    "status": "online",
    "archived": false,
    "auto_archive_duration": 1440,
    "locked": false,
    "invitable": true
  }
}
```

#### Fields

- `channel_id` ([string](#type.string)): The channel ID to edit
- `reason` ([string](#type.string)): The reason for editing the channel
- `data` ([EditChannel](#type.EditChannel)): The new channels' data


<div id="type.DeleteChannelOptions" />

### DeleteChannelOptions

Options for deleting a channel in Discord

```json
{
  "channel_id": "0",
  "reason": ""
}
```

#### Fields

- `channel_id` ([string](#type.string)): The channel ID to delete
- `reason` ([string](#type.string)): The reason for deleting the channel


<div id="type.CreateMessageAttachment" />

### CreateMessageAttachment

An attachment in a message

```json
[
  {
    "id": 0,
    "filename": "test.txt",
    "description": "Test file"
  }
]
```

#### Fields

- `filename` ([string](#type.string)): The filename of the attachment
- `description` ([string?](#type.string)): The description (if any) of the attachment
- `content` ([{byte}](#type.byte)): The content of the attachment


<div id="type.CreateMessageOptions" />

### CreateMessageOptions

Options for sending a message in a channel in Discord

```json
{
  "channel_id": "0",
  "data": {
    "tts": false,
    "embeds": [],
    "sticker_ids": [],
    "enforce_nonce": false
  }
}
```

#### Fields

- `channel_id` ([string](#type.string)): The channel ID to send the message in
- `data` ([Serenity.CreateMessage](#type.Serenity.CreateMessage)): The data of the message to send


<div id="type.CreateInteractionResponse" />

### CreateInteractionResponse

Options for creating an interaction response in Discord



#### Fields

- `interaction_id` ([string](#type.string)): The interaction ID to respond to
- `interaction_token` ([string](#type.string)): The interaction token to respond to
- `data` ([Serenity.InteractionResponse](#type.Serenity.InteractionResponse)): The interaction response body
- `files` ([{Serenity.CreateMessageAttachment}?](#type.Serenity.CreateMessageAttachment)): The files to send with the response


<div id="type.DiscordExecutor" />

### DiscordExecutor

DiscordExecutor allows templates to access/use the Discord API in a sandboxed form.



#### Methods

##### DiscordExecutor:get_audit_logs

```lua
function DiscordExecutor:get_audit_logs(data: GetAuditLogOptions): 
```

Gets the audit logs

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([GetAuditLogOptions](#type.GetAuditLogOptions)): Options for getting audit logs.


###### Returns

- `Lazy<Serenity.AuditLogs>` ([](#type.)): The audit log entry
##### DiscordExecutor:get_channel

```lua
function DiscordExecutor:get_channel(data: GetChannelOptions): 
```

Gets a channel

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([GetChannelOptions](#type.GetChannelOptions)): Options for getting a channel.


###### Returns

- `Lazy<Serenity.GuildChannel>` ([](#type.)): The guild channel
##### DiscordExecutor:edit_channel

```lua
function DiscordExecutor:edit_channel(data: EditChannelOptions): 
```

Edits a channel

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([EditChannelOptions](#type.EditChannelOptions)): Options for editing a channel.


###### Returns

- `Lazy<Serenity.GuildChannel>` ([](#type.)): The guild channel
##### DiscordExecutor:delete_channel

```lua
function DiscordExecutor:delete_channel(data: DeleteChannelOptions): 
```

Deletes a channel

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([DeleteChannelOptions](#type.DeleteChannelOptions)): Options for deleting a channel.


###### Returns

- `Lazy<Serenity.GuildChannel>` ([](#type.)): The guild channel
##### DiscordExecutor:create_message

```lua
function DiscordExecutor:create_message(data: SendMessageChannelAction): 
```

Creates a message

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([SendMessageChannelAction](#type.SendMessageChannelAction)): Options for creating a message.


###### Returns

- `Lazy<Message>` ([](#type.)): The message
##### DiscordExecutor:create_interaction_response

```lua
function DiscordExecutor:create_interaction_response(data: CreateInteractionResponse): 
```

Creates an interaction response

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([CreateInteractionResponse](#type.CreateInteractionResponse)): Options for creating a message.


###### Returns

- `Lazy<Message>` ([](#type.)): The message


## Methods

### new

```lua
function new(token: TemplateContext): DiscordExecutor
```

#### Parameters

- `token` ([TemplateContext](#type.TemplateContext)): The token of the template to use.


#### Returns

- `executor` ([DiscordExecutor](#type.DiscordExecutor)): A discord executor.
---
# @antiraid/img_captcha

This plugin allows for the creation of text/image CAPTCHA's with customizable filters which can be useful in protecting against bots.

## Types

<div id="type.CaptchaConfig" />

### CaptchaConfig

Captcha configuration. See examples for the arguments

```json
{
  "char_count": 5,
  "filters": [
    {
      "filter": "Noise",
      "prob": 0.1
    },
    {
      "filter": "Wave",
      "f": 4.0,
      "amp": 2.0,
      "d": "horizontal"
    },
    {
      "filter": "Line",
      "p1": [
        1.0,
        0.0
      ],
      "p2": [
        20.0,
        20.0
      ],
      "thickness": 2.0,
      "color": {
        "r": 0,
        "g": 30,
        "b": 100
      }
    },
    {
      "filter": "RandomLine"
    },
    {
      "filter": "Grid",
      "y_gap": 30,
      "x_gap": 10
    },
    {
      "filter": "ColorInvert"
    }
  ],
  "viewbox_size": [
    512,
    512
  ],
  "set_viewbox_at_idx": null
}
```

#### Fields

- `char_count` ([u8](#type.u8)): The number of characters the CAPTCHA should have.
- `filters` ([{any}](#type.any)): See example for the parameters to pass for the filter as well as https://github.com/Anti-Raid/captcha
- `viewbox_size` ([(u32, u32)](#type.(u32, u32))): The size of the viewbox to render the CAPTCHA in.
- `set_viewbox_at_idx` ([Option<usize>](#type.Option<usize>)): At what index of CAPTCHA generation should a viewbox be created at.


## Methods

### new

```lua
function new(config: CaptchaConfig): {u8}
```

Creates a new CAPTCHA with the given configuration.

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



#### Parameters

- `config` ([CaptchaConfig](#type.CaptchaConfig)): The configuration to use for the CAPTCHA.


#### Returns

- `captcha` ([{u8}](#type.u8)): The created CAPTCHA object.
---
# @antiraid/interop

This plugin allows interoperability with the Luau controller.

## Types

<div id="type.null" />

### null

`null` is a special value that represents nothing. It is often used in AntiRaid instead of `nil` due to issues regarding existence etc. `null` is not equal to `nil` but is also an opaque type.



<div id="type.array_metatable" />

### array_metatable

`array_metatable` is a special metatable that is used to represent arrays across the Lua-AntiRaid templating subsystem boundary. This metatable must be set on all arrays over this boundary and is required to ensure AntiRaid knows the value you're sending it is actually an array and not an arbitrary Luau table.



## Methods

### memusage

```lua
function memusage(): f64
```

Returns the current memory usage of the Lua VM.

#### Returns

- `memory_usage` ([f64](#type.f64)): The current memory usage, in bytes, of the Lua VM.

---
# @antiraid/kv

Utilities for key-value operations.

## Types

<div id="type.KvRecord" />

### KvRecord

KvRecord represents a key-value record with metadata.

```json
{
  "key": "",
  "value": null,
  "exists": false,
  "created_at": null,
  "last_updated_at": null
}
```

#### Fields

- `key` ([string](#type.string)): The key of the record.
- `value` ([any](#type.any)): The value of the record.
- `exists` ([bool](#type.bool)): Whether the record exists.
- `created_at` ([datetime](#type.datetime)): The time the record was created.
- `last_updated_at` ([datetime](#type.datetime)): The time the record was last updated.


<div id="type.KvExecutor" />

### KvExecutor

KvExecutor allows templates to get, store and find persistent data within a scope.

#### Methods

##### KvExecutor:find

Finds records in a scoped key-value database. ``%`` can be used as wildcards before/after the query. E.g. ``%{KEY}%`` will search for ``{KEY}`` anywhere in the string, ``%{KEY}`` will search for keys which end with ``{KEY}`` and ``_{KEY}`` will search for a single character before ``{KEY}``.

```lua
function KvExecutor:find(key: string): {KvRecord}
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**

###### Parameters

- `key` ([string](#type.string)): The key to search for. % matches zero or more characters; _ matches a single character. To search anywhere in a string, surround {KEY} with %, e.g. %{KEY}%

###### Returns

- `records` ([{KvRecord}](#type.KvRecord)): The records found.

##### KvExecutor:exists

Determines if a key exists in the scoped key-value database.

```lua
function KvExecutor:exists(key: string): bool
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**

###### Parameters

- `key` ([string](#type.string)): The key to check for existence.

###### Returns

- `exists` ([bool](#type.bool)): Whether the key exists.

##### KvExecutor:get

Returns the value of a key in the scoped key-value database.

```lua
function KvExecutor:get(key: string)
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `key` ([string](#type.string)): The key to get.


###### Returns

- `value` ([any](#type.any)): The value of the key.- `exists` ([bool](#type.bool)): Whether the key exists.
##### KvExecutor:getrecord

```lua
function KvExecutor:getrecord(key: string): KvRecord
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `key` ([string](#type.string)): The key to get.


###### Returns

- `record` ([KvRecord](#type.KvRecord)): The record of the key.
##### KvExecutor:set

```lua
function KvExecutor:set(key: string, value: any)
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `key` ([string](#type.string)): The key to set.
- `value` ([any](#type.any)): The value to set.

##### KvExecutor:delete

```lua
function KvExecutor:delete(key: string)
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `key` ([string](#type.string)): The key to delete.



## Methods

### new

```lua
function new(token: TemplateContext, scope: string?): KvExecutor
```

#### Parameters

- `token` ([TemplateContext](#type.TemplateContext)): The token of the template to use.
- `scope` ([string?](#type.string)): The scope of the executor. `this_guild` to use the originating guilds data, `owner_guild` to use the KV of the guild that owns the template on the shop. Defaults to `this_guild` if not specified.


#### Returns

- `executor` ([KvExecutor](#type.KvExecutor)): A key-value executor.


TO MOVE TO PRIMITIVES DOCS

- `guild_id` ([string](#type.string)): The guild ID the executor will perform key-value operations on.
- `origin_guild_id` ([string](#type.string)): The originating guild ID (the guild ID of the template itself).
- `allowed_caps` ([{string}](#type.{string})): The allowed capabilities in the current context.
- `has_cap` ([function](#type.function)): A function that returns `true` if the current context has the capability specified.
- `scope` ([string](#type.string)): The scope of the executor. Either ``this_guild`` for the originating guild, or ``owner_guild`` for the guild that owns the template (the template that owns the template on the shop if a shop template or the guild that owns the template otherwise).
---
# @antiraid/lazy

This plugin allows for templates to interact with and create 'lazy' data as well as providing documentation for the type. Note that events are *not* 'lazy' data's and have their own semantics.

## Types

<div id="type.Lazy<T>" />

### Lazy<T>

A lazy data type that is only serialized to Lua upon first access. This can be much more efficient than serializing the data every time it is accessed. Note that events are *not* 'lazy' data's and have their own semantics.

#### Fields

- `data` ([T](#type.T)): The inner data. This is cached upon first access
- `lazy` ([boolean](#type.boolean)): Always returns true. Allows the user to check if the data is a lazy or not


## Methods

### new

```lua
function new(data: TemplateContext): Lazy<any>
```

Creates a new Lazy type from data. This can be useful as a deep-copy implementation [``lazy.new(value).data`` is guaranteed to do a deepcopy of data as long as ``value`` is serializable]

#### Parameters

- `data` ([TemplateContext](#type.TemplateContext)): The data to wrap in a lazy

#### Returns

- `lazy` ([Lazy<any>](#type.Lazy)): A lazy value
---
# @antiraid/lockdowns

This plugin allows for templates to interact with AntiRaid lockdowns

## Types

<div id="type.Lockdown" />

### Lockdown

A created lockdown

```json
{
  "id": "805c0dd1-a625-4875-81e4-8edc6a14f659",
  "reason": "Testing",
  "type": "qsl",
  "data": {},
  "created_at": "2025-01-13T04:57:43.488340240Z"
}
```

#### Fields

- `id` ([string](#type.string)): The id of the lockdown
- `reason` ([string](#type.string)): The reason for the lockdown
- `type` ([string](#type.string)): The type of lockdown in string form
- `data` ([any](#type.any)): The data associated with the lockdown
- `created_at` ([string](#type.string)): The time the lockdown was created


<div id="type.LockdownExecutor" />

### LockdownExecutor

An executor for listing, creating and removing lockdowns



#### Methods

##### LockdownExecutor:list

```lua
function LockdownExecutor:list(): {Lockdown}
```

Lists all active lockdowns

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Returns

- `lockdowns` ([{Lockdown}](#type.Lockdown)): A list of all currently active lockdowns
##### LockdownExecutor:qsl

```lua
function LockdownExecutor:qsl(reason: string)
```

Starts a quick server lockdown

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `reason` ([string](#type.string)): The reason for the lockdown

##### LockdownExecutor:tsl

```lua
function LockdownExecutor:tsl(reason: string)
```

Starts a traditional server lockdown

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `reason` ([string](#type.string)): The reason for the lockdown

##### LockdownExecutor:scl

```lua
function LockdownExecutor:scl(channel: string, reason: string)
```

Starts a lockdown on a single channel

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `channel` ([string](#type.string)): The channel to lock down
- `reason` ([string](#type.string)): The reason for the lockdown

##### LockdownExecutor:role

```lua
function LockdownExecutor:role(role: string, reason: string)
```

Starts a lockdown on a role

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `role` ([string](#type.string)): The role to lock down
- `reason` ([string](#type.string)): The reason for the lockdown

##### LockdownExecutor:remove

```lua
function LockdownExecutor:remove(id: string)
```

Removes a lockdown

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `id` ([string](#type.string)): The id of the lockdown to remove



## Methods

### new

```lua
function new(token: TemplateContext): LockdownExecutor
```

#### Parameters

- `token` ([TemplateContext](#type.TemplateContext)): The token of the template to use


#### Returns

- `executor` ([LockdownExecutor](#type.LockdownExecutor)): A lockdown executor
---
# @antiraid/luau

Make and evaluate custom luau code chunks.

## Types

<div id="type.Chunk" />

### Chunk

A luau code chunk.

#### Fields

- `environment` (table): The environment the chunk can access. Requires ``luau:eval.set_environment`` to modify
- `optimization_level` ([number](#type.number)): The Luau compiler optimization level to use. Can be either ``0``, ``1`` or ``2``. Requires ``luau:eval.set_optimization_level`` to modify
- `code` ([string](#type.string)): The luau code to execute. Can be set at chunk creation time with just ``luau:eval`` and modified after creation with ``luau:eval.modify_set_code``
- ``chunk_name`` ([string](#type.string)): The name of the chunk. Requires  ``luau:eval.set_chunk_name`` to modify

#### Methods

##### call

```lua
function call(...): ...
```

Calls the chunk synchronously with the provided arguments. Requires ``luau:eval.call`` to use.

###### Parameters

- `...` ([...](#type.any)): The arguments to pass to the chunk.


###### Returns

- `result` ([...](#type.any)): The result of evaluating the chunk.

##### call_async

```lua
function call_async(...): Promise<...>
```

Calls the chunk asynchronously with the provided arguments. Requires ``luau:eval.call_async`` to use.

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**

###### Parameters

- `...` ([...](#type.any)): The arguments to pass to the chunk.


###### Returns

- `result` ([...](#type.any)): The result of evaluating the chunk.

---

## Methods

### load

```lua
function load(token: TemplateContext, code: string): Chunk
```

Loads a luau code chunk. Note that setting other properties on the chunk must be done after loading and requires their respective capabilities. Loading a chunk does *not* parse or validate the code, it is only stored for later use.

#### Parameters

- `code` ([string](#type.string)): The luau code to load.

#### Returns

- `chunk` ([Chunk](#type.Chunk)): The loaded chunk.
---
# @antiraid/pages

Make and modify AntiRaid template pages

## Enums

<div id="type.Setting.Column.InnerColumnType" />

### Setting.Column.InnerColumnType

The inner column type of the value. See examples for full typings until we have a full spec.

<div id="type.Setting.Column.ColumnType" />

### Setting.Column.ColumnType

The type of a setting column

#### Variants

##### Setting.Column.ColumnType::Scalar
A scalar column type.

**Fields**
- `inner` ([Setting.Column.InnerColumnType](#type.Setting.Column.InnerColumnType)): The inner type of the column.

##### Setting.Column.ColumnType::Array
An array column type.

**Fields**
- `inner` ([Setting.Column.InnerColumnType](#type.Setting.Column.InnerColumnType)): The array type of the column.

## Types

<div id="type.Setting.Column" />

### Setting.Column

A setting column

```json
{
  "id": "created_at",
  "name": "Created At",
  "description": "The time the record was created.",
  "column_type": {
    "type": "Scalar",
    "inner": "String",
    "min_length": 120,
    "max_length": 120,
    "allowed_values": [
      "allowed_value"
    ],
    "kind": "timestamp"
  },
  "nullable": false,
  "suggestions": {
    "type": "None"
  },
  "secret": false,
  "ignored_for": [
    "Create",
    "Update"
  ]
}
```

#### Fields
- `id` ([string](#type.string)): The ID of the column.
- `name` ([string](#type.string)): The name of the column.
- `description` ([string](#type.string)): The description of the column.
- `column_type` ([Setting.Column.ColumnType](#type.Setting.Column.ColumnType)): The type of the column.
- `nullable` ([bool](#type.bool)): Whether the column can be null.
- `suggestions` ([Setting.Column.ColumnSuggestion](#type.Setting.Column.ColumnSuggestion)): The suggestions for the column.
- `secret` ([bool](#type.bool)): Whether the column is secret.
- `ignored_for` ([{OperationType}](#type.OperationType)): The operations that the column is ignored for [read-only]. It is *not guaranteed* that ignored field are sent to the template.

<div id="type.Setting" />

### Setting

A setting

```json
{
  "id": "setting_id",
  "name": "Setting Name",
  "description": "Setting Description",
  "primary_key": "id",
  "title_template": "{col1} - {col2}",
  "columns": [
    {
      "id": "col1",
      "name": "Column 1",
      "description": "Column 1 desc",
      "column_type": {
        "type": "Scalar",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "Normal"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Create"
      ]
    },
    {
      "id": "col2",
      "name": "Column 2",
      "description": "Column 2 desc",
      "column_type": {
        "type": "Array",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "something cool?" // Is ignored...
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "View"
      ]
    },
    {
      "id": "col3",
      "name": "Column 3",
      "description": "Column 3 desc",
      "column_type": {
        "type": "Array",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "textarea",
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Create"
      ]
    },
    {
      "id": "col4",
      "name": "Column 4",
      "description": "Column 4 desc",
      "column_type": {
        "type": "Array",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "templateref"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Create"
      ]
    },
    {
      "id": "col5",
      "name": "Column 5",
      "description": "Column 5 desc",
      "column_type": {
        "type": "Array",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "kittycat-permission"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Create"
      ]
    },
    {
      "id": "col6",
      "name": "Column 6",
      "description": "Column 6 desc",
      "column_type": {
        "type": "Array",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "user"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Create"
      ]
    },
    {
      "id": "col7",
      "name": "Column 7",
      "description": "Column 7 desc",
      "column_type": {
        "type": "Array",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "role"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Create"
      ]
    },
    {
      "id": "col8",
      "name": "Column 8",
      "description": "Column 8 desc",
      "column_type": {
        "type": "Array",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "channel"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Update"
      ]
    },
    {
      "id": "col9",
      "name": "Column 9",
      "description": "Column 9 desc",
      "column_type": {
        "type": "Scalar",
        "inner": "Integer"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Update"
      ]
    },
    {
      "id": "col10",
      "name": "Column 10",
      "description": "Column 10 desc",
      "column_type": {
        "type": "Scalar",
        "inner": "Boolean"
      },
      "nullable": false,
      "suggestions": {
        "type": "Static",
        "suggestions": [
          "suggestion"
        ]
      },
      "secret": false,
      "ignored_for": [
        "Update"
      ]
    },
    {
      "id": "created_at",
      "name": "Created At",
      "description": "The time the record was created.",
      "column_type": {
        "type": "Scalar",
        "inner": "String",
        "min_length": 120,
        "max_length": 120,
        "allowed_values": [
          "allowed_value"
        ],
        "kind": "timestamp"
      },
      "nullable": false,
      "suggestions": {
        "type": "None"
      },
      "secret": false,
      "ignored_for": [
        "Create",
        "Update"
      ]
    }
  ],
  "operations": {
    "view": true,
    "create": true,
    "update": false,
    "delete": true
  },
}

```
#### Fields
- `id` ([string](#type.string)): The ID of the setting.
- `name` ([string](#type.string)): The name of the setting.
- `description` ([string](#type.string)): The description of the setting.
- `operations` ([{OperationType}](#type.OperationType)): The operations that can be performed on the setting. 
- `primary_key` ([string](#type.string)): The primary key of the setting that UNIQUELY identifies the row. When ``Delete`` is called, the value of this is what will be sent in the event. On ``Update``, this key MUST also exist (otherwise, the template MUST error out)
- `title_template` ([string](#type.string)): The template for the title of each row for the setting. This is a string that can contain placeholders for columns. The placeholders are in the form of ``{column_id}``. For example, if you have a column with ID ``col1`` and another with ID ``col2``, you can have a title template of ``{col1} - {col2}`` etc..
- `columns` ([{Setting.Column}](#type.Setting.Column)): The columns of the setting.

<div id="type.Page" />

### Page

An AntiRaid template page

#### Fields

- `title` ([string](#type.string)): The title of the page.
- `description` ([string](#type.string)): The description of the page.
- `settings` ([{Setting}](#type.Setting)): The settings of the page.

### PageExecutor

A page executor. This is used to manipulate the page.

#### Methods

##### PageExecutor:get

```luau
function get(): LuaPromise<Page?>
```

Returns the page associated with the template, if any. Using the token from another template to create a PageExecutor and then calling get can be used to get that templates' page.

##### PageExecutor:set

```luau
function set(page: Page): LuaPromise<void>
```

Sets a page to be the templates page. This will overwrite any existing page if one exists.

##### PageExecutor:delete

```luau
function delete(): LuaPromise<void>
```

Deletes the templates page. This will not delete the page itself, but will remove it from the server's list of custom pages.

---

## Methods

### new

```luau
function new(token: TemplateContext): LuaPromise<PageExecutor>
```

Returns a page executor associated with the template which can then be used to manipulate the templates page.
---
# @antiraid/permissions

Utilities for handling permission checks.

## Types

<div id="type.Permission" />

### Permission

Permission is the primitive permission type used by AntiRaid. See https://github.com/InfinityBotList/kittycat for more information

```json
{
  "namespace": "moderation",
  "perm": "ban",
  "negator": false
}
```

#### Fields

- `namespace` ([string](#type.string)): The namespace of the permission.
- `perm` ([string](#type.string)): The permission bit on the namespace.
- `negator` ([bool](#type.bool)): Whether the permission is a negator permission or not


<div id="type.StaffPermissions" />

### StaffPermissions

StaffPermissions as per kittycat terminology.

```json
{
  "user_positions": [
    {
      "id": "1234567890",
      "index": 1,
      "perms": [
        {
          "namespace": "moderation",
          "perm": "ban",
          "negator": false
        },
        {
          "namespace": "moderation",
          "perm": "kick",
          "negator": false
        }
      ]
    },
    {
      "id": "0987654321",
      "index": 2,
      "perms": [
        {
          "namespace": "moderation",
          "perm": "ban",
          "negator": false
        },
        {
          "namespace": "moderation",
          "perm": "kick",
          "negator": false
        }
      ]
    }
  ],
  "perm_overrides": [
    {
      "namespace": "moderation",
      "perm": "ban",
      "negator": true
    },
    {
      "namespace": "moderation",
      "perm": "kick",
      "negator": true
    }
  ]
}
```

#### Fields

- `perm_overrides` ([{Permission}](#type.Permission)): Permission overrides on the member.
- `user_positions` ([{PartialStaffPosition}](#type.PartialStaffPosition)): The staff positions of the user.


<div id="type.PartialStaffPosition" />

### PartialStaffPosition

PartialStaffPosition as per kittycat terminology.

```json
{
  "id": "1234567890",
  "index": 1,
  "perms": [
    {
      "namespace": "moderation",
      "perm": "ban",
      "negator": false
    },
    {
      "namespace": "moderation",
      "perm": "kick",
      "negator": false
    }
  ]
}
```

#### Fields

- `id` ([string](#type.string)): The ID of the staff member.
- `index` ([number](#type.number)): The index of the staff member.
- `perms` ([{Permission}](#type.Permission)): The permissions of the staff member.


## Methods

### permission_from_string

```lua
function permission_from_string(perm_string: string): Permission
```

Returns a Permission object from a string.

#### Parameters

- `perm_string` ([string](#type.string)): The string to parse into a Permission object.


#### Returns

- `permission` ([Permission](#type.Permission)): The parsed Permission object.

### permission_to_string

```lua
function permission_to_string(permission: Permission): string
```

Returns a string from a Permission object.

#### Parameters

- `permission` ([Permission](#type.Permission)): The Permission object to parse into a string.


#### Returns

- `perm_string` ([string](#type.string)): The parsed string.

### has_perm

```lua
function has_perm(permissions: {Permission}, permission: Permission): bool
```

Checks if a list of permissions in Permission object form contains a specific permission.

#### Parameters

- `permissions` ([{Permission}](#type.Permission)): The list of permissions
- `permission` ([Permission](#type.Permission)): The permission to check for.


#### Returns

- `has_perm` ([bool](#type.bool)): Whether the permission is present in the list of permissions as per kittycat rules.

### has_perm_str

```lua
function has_perm_str(permissions: {string}, permission: string): bool
```

Checks if a list of permissions in canonical string form contains a specific permission.

#### Parameters

- `permissions` ([{string}](#type.string)): The list of permissions
- `permission` ([string](#type.string)): The permission to check for.


#### Returns

- `has_perm` ([bool](#type.bool)): Whether the permission is present in the list of permissions as per kittycat rules.

### staff_permissions_resolve

```lua
function staff_permissions_resolve(sp: StaffPermissions): {Permission}
```

Resolves a StaffPermissions object into a list of Permission objects. See https://github.com/InfinityBotList/kittycat for more details

#### Parameters

- `sp` ([StaffPermissions](#type.StaffPermissions)): The StaffPermissions object to resolve.


#### Returns

- `permissions` ([{Permission}](#type.Permission)): The resolved list of Permission objects.

### check_patch_changes

```lua
function check_patch_changes(manager_perms: {Permission}, current_perms: {Permission}, new_perms: {Permission})
```

Checks if a list of permissions can be patched to another list of permissions.

#### Parameters

- `manager_perms` ([{Permission}](#type.Permission)): The permissions of the manager.
- `current_perms` ([{Permission}](#type.Permission)): The current permissions of the user.
- `new_perms` ([{Permission}](#type.Permission)): The new permissions of the user.

#### Returns

- `can_patch` ([bool](#type.bool)): Whether the permissions can be patched.- `error` ([any](#type.any)): The error if the permissions cannot be patched. Will contain ``type`` field with the error type and additional fields depending on the error type.
---
# @antiraid/promise

Lua Promises, yield for a promise to execute the async action returning its result.

## Types

<div id="type.LuaPromise" />

### LuaPromise<T>

LuaPromise<T> provides a promise that must be yielded to actually execute and get the result of the async action.

## Methods

### yield

```lua
function yield(promise: LuaPromise<T>): T
```

Yields the promise to execute the async action and return its result. Note that this is the only function other than `stream.next` that yields.

#### Parameters

- `promise` ([LuaPromise<T>](#type.LuaPromise<T>)): The promise to yield.


#### Returns

- `T` ([T](#type.T)): The result of executing the promise.

---

## Promise Execution Cycle

When you create a promise, it does not do anything (in essence, it acts like a *future*). You must yield the promise to actually execute the async action and get the result. This is because the Lua VM is single-threaded and cannot execute things concurrently so your code must yield to allow the Promises' internal code to run and return the result back, which *resumes* your code.

```lua
local promise = someAsyncAction() -- A LuaPromise<T> is returned
local result = promise.yield(promise) -- Now, result is a ``T``!
```

While usually not very useful, the created promise can also be re-used multiple times, as it is not consumed by yielding it.

```lua
local promise = someAsyncAction() -- A LuaPromise<T> is returned
local result1 = promise.yield(promise) -- Now, result1 is a ``T``!
local result2 = promise.yield(promise) -- Now, result2 is a ``T``!
```

---
# @antiraid/stings

List, get, create, update and delete stings on Anti-Raid.

## Types

<div id="type.StingCreate" />

### StingCreate

A type representing a new sting to be created.

```json
{
  "src": "test",
  "stings": 10,
  "reason": "test",
  "void_reason": null,
  "guild_id": "128384",
  "creator": "system",
  "target": "user:1945824",
  "state": "active",
  "duration": {
    "secs": 60,
    "nanos": 0
  },
  "sting_data": {
    "a": "b"
  }
}
```

#### Fields

- `src` ([string?](#type.string)): The source of the sting.
- `stings` ([number](#type.number)): The number of stings.
- `reason` ([string?](#type.string)): The reason for the stings.
- `void_reason` ([string?](#type.string)): The reason the stings were voided.
- `guild_id` ([string](#type.string)): The guild ID the sting targets. **MUST MATCH THE GUILD ID THE TEMPLATE IS RUNNING ON**
- `creator` ([StingTarget](#type.StingTarget)): The creator of the sting.
- `target` ([StingTarget](#type.StingTarget)): The target of the sting.
- `state` ([string](#type.string)): The state of the sting. Must be one of 'active', 'voided' or 'handled'
- `duration` ([Duration?](#type.Duration)): When the sting expires as a duration.
- `sting_data` ([any?](#type.any)): The data/metadata present within the sting, if any.


<div id="type.Sting" />

### Sting

Represents a sting on AntiRaid

```json
{
  "id": "470a2958-3827-4e59-8b97-928a583a37a3",
  "src": "test",
  "stings": 10,
  "reason": "test",
  "void_reason": null,
  "guild_id": "128384",
  "creator": "system",
  "target": "user:1945824",
  "state": "active",
  "created_at": "2025-01-13T04:57:43.488668165Z",
  "duration": {
    "secs": 60,
    "nanos": 0
  },
  "sting_data": {
    "a": "b"
  },
  "handle_log": {
    "a": "b"
  }
}
```

#### Fields

- `id` ([string](#type.string)): The sting ID.
- `src` ([string?](#type.string)): The source of the sting.
- `stings` ([number](#type.number)): The number of stings.
- `reason` ([string?](#type.string)): The reason for the stings.
- `void_reason` ([string?](#type.string)): The reason the stings were voided.
- `guild_id` ([string](#type.string)): The guild ID the sting targets. **MUST MATCH THE GUILD ID THE TEMPLATE IS RUNNING ON**
- `creator` ([StingTarget](#type.StingTarget)): The creator of the sting.
- `target` ([StingTarget](#type.StingTarget)): The target of the sting.
- `state` ([StingState](#type.StingState)): The state of the sting.
- `duration` ([Duration?](#type.Duration)): When the sting expires as a duration.
- `sting_data` ([any?](#type.any)): The data/metadata present within the sting, if any.
- `handle_log` ([any](#type.any)): The handle log encountered while handling the sting.
- `created_at` ([string](#type.string)): When the sting was created at.


<div id="type.StingExecutor" />

### StingExecutor

An sting executor is used to execute actions related to stings from Lua templates



#### Methods

##### StingExecutor:list

```lua
function StingExecutor:list(page: number): {Sting}
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `page` ([number](#type.number)): The page number to fetch.


###### Returns

- `stings` ([{Sting}](#type.Sting)): The list of stings.
##### StingExecutor:get

```lua
function StingExecutor:get(id: string): Sting
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `id` ([string](#type.string)): The sting ID.


###### Returns

- `sting` ([Sting](#type.Sting)): The sting.
##### StingExecutor:create

```lua
function StingExecutor:create(data: StingCreate): string
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([StingCreate](#type.StingCreate)): The sting data.


###### Returns

- `id` ([string](#type.string)): The sting ID of the created sting.
##### StingExecutor:update

```lua
function StingExecutor:update(data: Sting)
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([Sting](#type.Sting)): The sting to update to. Note that if an invalid ID is used, this method may either do nothing or error out.

##### StingExecutor:delete

```lua
function StingExecutor:delete(id: string)
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `id` ([string](#type.string)): The sting ID.



## Enums

<div id="type.StingTarget" />

### StingTarget

The target of the sting.

There are two variants: ``system`` (A system target/no associated user) and ``user:{user_id}`` (A user target)

---
# @antiraid/typesext

Extra types used by Anti-Raid Lua templating subsystem to either add in common functionality such as streams or handle things like u64/i64 types performantly.

## Types

<div id="type.MultiOption" />

### MultiOption<T>

MultiOption allows distinguishing between `null` and empty fields. Use the value to show both existence and value (`Some(Some(value))`) an empty object to show existence (``Some(None)``) or null to show neither (`None`)



<div id="type.U64" />

### U64

U64 is a 64-bit unsigned integer type. Implements Add/Subtract/Multiply/Divide/Modulus/Power/Integer Division/Equality/Comparison (Lt/Le and its complements Gt/Ge) and ToString with a type name of U64



#### Methods

##### U64:to_ne_bytes

```lua
function U64:to_ne_bytes(): {u8}
```

Converts the U64 to a little-endian byte array.

###### Returns

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.
##### U64:from_ne_bytes

```lua
function U64:from_ne_bytes(bytes: {u8}): U64
```

Converts a little-endian byte array to a U64.

###### Parameters

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.


###### Returns

- `u64` ([U64](#type.U64)): The U64 value.
##### U64:to_le_bytes

```lua
function U64:to_le_bytes(): {u8}
```

Converts the U64 to a little-endian byte array.

###### Returns

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.
##### U64:from_le_bytes

```lua
function U64:from_le_bytes(bytes: {u8}): U64
```

Converts a little-endian byte array to a U64.

###### Parameters

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.


###### Returns

- `u64` ([U64](#type.U64)): The U64 value.
##### U64:to_be_bytes

```lua
function U64:to_be_bytes(): {u8}
```

Converts the U64 to a big-endian byte array.

###### Returns

- `bytes` ([{u8}](#type.u8)): The big-endian byte array.
##### U64:from_be_bytes

```lua
function U64:from_be_bytes(bytes: {u8}): U64
```

Converts a big-endian byte array to a U64.

###### Parameters

- `bytes` ([{u8}](#type.u8)): The big-endian byte array.


###### Returns

- `u64` ([U64](#type.U64)): The U64 value.
##### U64:to_i64

```lua
function U64:to_i64(): I64
```

Converts the U64 to an i64.

###### Returns

- `i64` ([I64](#type.I64)): The i64 value.


<div id="type.I64" />

### I64

I64 is a 64-bit signed integer type. Implements Add/Subtract/Multiply/Divide/Modulus/Power/Integer Division/Equality/Comparison (Lt/Le and its complements Gt/Ge) and ToString with a type name of I64



#### Methods

##### I64:to_ne_bytes

```lua
function I64:to_ne_bytes(): {u8}
```

Converts the I64 to a little-endian byte array.

###### Returns

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.
##### I64:from_ne_bytes

```lua
function I64:from_ne_bytes(bytes: {u8}): I64
```

Converts a little-endian byte array to a I64.

###### Parameters

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.


###### Returns

- `i64` ([I64](#type.I64)): The I64 value.
##### I64:to_le_bytes

```lua
function I64:to_le_bytes(): {u8}
```

Converts the I64 to a little-endian byte array.

###### Returns

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.
##### I64:from_le_bytes

```lua
function I64:from_le_bytes(bytes: {u8}): I64
```

Converts a little-endian byte array to a I64.

###### Parameters

- `bytes` ([{u8}](#type.u8)): The little-endian byte array.


###### Returns

- `i64` ([I64](#type.I64)): The I64 value.
##### I64:to_be_bytes

```lua
function I64:to_be_bytes(): {u8}
```

Converts the I64 to a big-endian byte array.

###### Returns

- `bytes` ([{u8}](#type.u8)): The big-endian byte array.
##### I64:from_be_bytes

```lua
function I64:from_be_bytes(bytes: {u8}): I64
```

Converts a big-endian byte array to a I64.

###### Parameters

- `bytes` ([{u8}](#type.u8)): The big-endian byte array.


###### Returns

- `i64` ([I64](#type.I64)): The I64 value.
##### I64:to_u64

```lua
function I64:to_u64(): U64
```

Converts the I64 to a U64.

###### Returns

- `u64` ([U64](#type.U64)): The U64 value.


<div id="type.bitu64" />

### bitu64

[bit32](https://luau.org/library#bit32-library) but for U64 datatype. Note that bit64 is experimental and may not be properly documented at all times. When in doubt, reach for Luau's bit32 documentation and simply replace 31's with 63's



#### Methods

##### bitu64:band

```lua
function bitu64:band(values: {U64}): U64
```

Performs a bitwise AND operation on the given values.

###### Parameters

- `values` ([{U64}](#type.U64)): The values to perform the operation on.


###### Returns

- `result` ([U64](#type.U64)): The result of the operation.
##### bitu64:bnor

```lua
function bitu64:bnor(n: U64): U64
```

Performs a bitwise NOR operation on the given value.

###### Parameters

- `n` ([U64](#type.U64)): The value to perform the operation on.


###### Returns

- `result` ([U64](#type.U64)): The result of the operation.
##### bitu64:bor

```lua
function bitu64:bor(values: {U64}): U64
```

Performs a bitwise OR operation on the given values.

###### Parameters

- `values` ([{U64}](#type.U64)): The values to perform the operation on.


###### Returns

- `result` ([U64](#type.U64)): The result of the operation.
##### bitu64:bxor

```lua
function bitu64:bxor(values: {U64}): U64
```

Performs a bitwise XOR operation on the given values.

###### Parameters

- `values` ([{U64}](#type.U64)): The values to perform the operation on.


###### Returns

- `result` ([U64](#type.U64)): The result of the operation.
##### bitu64:btest

```lua
function bitu64:btest(values: {U64}): bool
```

Tests if the bitwise AND of the given values is not zero.

###### Parameters

- `values` ([{U64}](#type.U64)): The values to perform the operation on.


###### Returns

- `result` ([bool](#type.bool)): True if the bitwise AND of the values is not zero, false otherwise.
##### bitu64:extract

```lua
function bitu64:extract(n: U64, f: u64, w: u64): U64
```

Extracts a field from a value.

###### Parameters

- `n` ([U64](#type.U64)): The value to extract the field from.
- `f` ([u64](#type.u64)): The field to extract.
- `w` ([u64](#type.u64)): The width of the field to extract.


###### Returns

- `result` ([U64](#type.U64)): The extracted field.
##### bitu64:lrotate

```lua
function bitu64:lrotate(n: U64, i: i64): U64
```

Rotates a value left or right.

###### Parameters

- `n` ([U64](#type.U64)): The value to rotate.
- `i` ([i64](#type.i64)): The amount to rotate by.


###### Returns

- `result` ([U64](#type.U64)): The rotated value.
##### bitu64:lshift

```lua
function bitu64:lshift(n: U64, i: i64): U64
```

Shifts a value left or right.

###### Parameters

- `n` ([U64](#type.U64)): The value to shift.
- `i` ([i64](#type.i64)): The amount to shift by.


###### Returns

- `result` ([U64](#type.U64)): The shifted value.
##### bitu64:replace

```lua
function bitu64:replace(n: U64, v: U64, f: u64, w: u64): U64
```

Replaces a field in a value.

###### Parameters

- `n` ([U64](#type.U64)): The value to replace the field in.
- `v` ([U64](#type.U64)): The value to replace the field with.
- `f` ([u64](#type.u64)): The field to replace.
- `w` ([u64](#type.u64)): The width of the field to replace.


###### Returns

- `result` ([U64](#type.U64)): The value with the field replaced.
##### bitu64:rrotate

```lua
function bitu64:rrotate(n: U64, i: i64): U64
```

Rotates a value left or right.

###### Parameters

- `n` ([U64](#type.U64)): The value to rotate.
- `i` ([i64](#type.i64)): The amount to rotate by.


###### Returns

- `result` ([U64](#type.U64)): The rotated value.
##### bitu64:rshift

```lua
function bitu64:rshift(n: U64, i: i64): U64
```

Shifts a value left or right.

###### Parameters

- `n` ([U64](#type.U64)): The value to shift.
- `i` ([i64](#type.i64)): The amount to shift by.


###### Returns

- `result` ([U64](#type.U64)): The shifted value.


## Methods

### U64

```lua
function U64(value: u64): U64
```

Creates a new U64.

#### Parameters

- `value` ([u64](#type.u64)): The value of the U64.


#### Returns

- `u64` ([U64](#type.U64)): The U64 value.

### I64

```lua
function I64(value: i64): I64
```

Creates a new I64.

#### Parameters

- `value` ([i64](#type.i64)): The value of the I64.


#### Returns

- `i64` ([I64](#type.I64)): The I64 value.

---
# @antiraid/userinfo

This plugin allows for templates to interact with user's core information on AntiRaid (permissions etc)

## Types

<div id="type.UserInfo" />

### UserInfo

A user info object

```json
{
  "discord_permissions": "2111062325329919",
  "kittycat_staff_permissions": {
    "user_positions": [],
    "perm_overrides": [
      {
        "namespace": "global",
        "perm": "*",
        "negator": false
      }
    ]
  },
  "kittycat_resolved_permissions": [
    {
      "namespace": "moderation",
      "perm": "kick",
      "negator": false
    },
    {
      "namespace": "moderation",
      "perm": "ban",
      "negator": false
    }
  ],
  "guild_owner_id": "1234567890",
  "guild_roles": [],
  "member_roles": [
    "1234567890"
  ]
}
```

#### Fields

- `discord_permissions` ([string](#type.string)): The discord permissions of the user
- `kittycat_staff_permissions` ([StaffPermissions](#type.StaffPermissions)): The staff permissions of the user
- `kittycat_resolved_permissions` ([{Permission}](#type.Permission)): The resolved permissions of the user
- `guild_owner_id` ([string](#type.string)): The guild owner id
- `guild_roles` ([{[string]: Serenity.Role}](#type.[string]: Serenity.Role)): The roles of the guild
- `member_roles` ([{string}](#type.string)): The roles of the member


<div id="type.UserInfoExecutor" />

### UserInfoExecutor

UserInfoExecutor allows templates to access/use user infos not otherwise sent via events.



#### Methods

##### UserInfoExecutor:get

```lua
function UserInfoExecutor:get(user: string): 
```

Gets the user info of a user.

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `user` ([string](#type.string)): The user id to get the info of.


###### Returns

- `UserInfo` ([](#type.)): The user info of the user.


## Methods

### new

```lua
function new(token: TemplateContext): UserInfoExecutor
```

#### Parameters

- `token` ([TemplateContext](#type.TemplateContext)): The token of the template to use.


#### Returns

- `executor` ([UserInfoExecutor](#type.UserInfoExecutor)): A userinfo executor.
---
# Primitives

<div id="type.u8" />

## u8

```lua
type u8 = number
```

An unsigned 8-bit integer. **Note: u8 arrays (`{u8}`) are often used to represent an array of bytes in AntiRaid**

### Constraints

- **range**: The range of values this number can take on (accepted values: 0-255)

---

<div id="type.u16" />

## u16

```lua
type u16 = number
```

An unsigned 16-bit integer.

### Constraints

- **range**: The range of values this number can take on (accepted values: 0-65535)

---

<div id="type.u32" />

## u32

```lua
type u32 = number
```

An unsigned 32-bit integer.

### Constraints

- **range**: The range of values this number can take on (accepted values: 0-4294967295)

---

<div id="type.u64" />

## u64

```lua
type u64 = number
```

An unsigned 64-bit integer. **Note that most, if not all, cases of `i64` in the actual API are either `string` or the `I64` custom type from typesext**

### Constraints

- **range**: The range of values this number can take on (accepted values: 0-18446744073709551615)

---

<div id="type.i8" />

## i8

```lua
type i8 = number
```

A signed 8-bit integer.

### Constraints

- **range**: The range of values this number can take on (accepted values: -128-127)

---

<div id="type.i16" />

## i16

```lua
type i16 = number
```

A signed 16-bit integer.

### Constraints

- **range**: The range of values this number can take on (accepted values: -32768-32767)

---

<div id="type.i32" />

## i32

```lua
type i32 = number
```

A signed 32-bit integer.

### Constraints

- **range**: The range of values this number can take on (accepted values: -2147483648-2147483647)

---

<div id="type.i64" />

## i64

```lua
type i64 = number
```

A signed 64-bit integer. **Note that most, if not all, cases of `i64` in the actual API are either `string` or the `I64` custom type from typesext**

### Constraints

- **range**: The range of values this number can take on (accepted values: -9223372036854775808-9223372036854775807)

---

<div id="type.f32" />

## f32

```lua
type f32 = number
```

A 32-bit floating point number.

### Constraints

- **range**: The range of values this number can take on (accepted values: IEEE 754 single-precision floating point)

---

<div id="type.f64" />

## f64

```lua
type f64 = number
```

A 64-bit floating point number.

### Constraints

- **range**: The range of values this number can take on (accepted values: IEEE 754 double-precision floating point)

---

<div id="type.byte" />

## byte

```lua
type byte = number
```

An unsigned 8-bit integer that semantically stores a byte of information

### Constraints

- **range**: The range of values this number can take on (accepted values: 0-255)

---

<div id="type.bool" />

## bool

```lua
type bool = boolean
```

A boolean value.

---

<div id="type.char" />

## char

```lua
type char = string
```

A single Unicode character.

### Constraints

- **length**: The length of the string (accepted values: 1)

---

<div id="type.string" />

## string

```lua
type string = string
```

A UTF-8 encoded string.

### Constraints

- **encoding**: Accepted character encoding (accepted values: UTF-8 *only*)

---

<div id="type.function" />

## function

```lua
type function = function
```

A Lua function.

---

# Types

<div id="type.Event" />

## Event

An event that has been dispatched to the template. This is what `args` is in the template.



### Fields

- `base_name` ([string](#type.string)): The base name of the event.
- `name` ([string](#type.string)): The name of the event.
- `data` ([unknown](#type.unknown)): The data of the event.
- `can_respond` ([boolean](#type.boolean)): Whether the event can be responded to.
- `response` ([unknown](#type.unknown)): The current response of the event. This can be overwritten by the template by just setting it to a new value.
- `author` ([string?](#type.string)): The author of the event, if any. If there is no known author, this field will either be `nil` or `null`.


<div id="type.Template" />

## Template

`Template` is a struct that represents the data associated with a template. Fields are still being documented and subject to change.

```json
{
  "guild_id": "0",
  "name": "",
  "description": null,
  "shop_name": null,
  "shop_owner": null,
  "events": [],
  "error_channel": null,
  "content": "",
  "lang": "luau",
  "allowed_caps": [],
  "created_by": "",
  "created_at": "1970-01-01T00:00:00Z",
  "updated_by": "",
  "updated_at": "1970-01-01T00:00:00Z"
}
```

### Fields

- `language` ([string](#type.string)): The language of the template.
- `allowed_caps` ([{string}](#type.string)): The allowed capabilities provided to the template.


<div id="type.TemplateContext" />

## TemplateContext

`TemplateContext` is a struct that represents the context of a template. Stores data including the templates data, pragma and what capabilities it should have access to. Passing a TemplateContext is often required when using AntiRaid plugins for security purposes.



### Fields

- `template_data` ([TemplateData](#type.TemplateData)): The data associated with the template.
- `guild_id` ([string](#type.string)): The current guild ID the template is running on.
- `current_user` ([Serenity.User](#type.Serenity.User)): Returns AntiRaid's discord user object [the current discord bot user driving the template].

---
