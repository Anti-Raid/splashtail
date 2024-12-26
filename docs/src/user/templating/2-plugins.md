# @antiraid/discord

This plugin allows for templates to interact with the Discord API

## Types

<div id="type.Serenity.User" />

### Serenity.User

A user object in Discord, as represented by AntiRaid. Internal fields are subject to change

**Refer to [serenity::model::user::User](https://docs.rs/serenity/latest/serenity/model/user/struct.User.html) for more documentation on what this type contains. Fields may be incomplete**

```json
{
  "id": "0",
  "username": "",
  "global_name": null,
  "avatar": null,
  "bot": false,
  "system": false,
  "mfa_enabled": false,
  "banner": null,
  "accent_color": null,
  "locale": null,
  "verified": null,
  "email": null,
  "flags": 0,
  "premium_type": 0,
  "public_flags": null,
  "member": null
}
```

<div id="type.Serenity.AuditLogs" />

### Serenity.AuditLogs

A audit log in Discord, as represented by AntiRaid. Internal fields are subject to change

**Refer to [serenity::model::guild::audit_log::AuditLogs](https://docs.rs/serenity/latest/serenity/model/guild/audit_log/struct.AuditLogs.html) for more documentation on what this type contains. Fields may be incomplete**



<div id="type.Serenity.AuditLogs.Action" />

### Serenity.AuditLogs.Action

An audit log action in Discord, as represented by AntiRaid. Internal fields are subject to change

**Refer to [serenity::model::guild::audit_log::Action](https://docs.rs/serenity/latest/serenity/model/guild/audit_log/struct.Action.html) for more documentation on what this type contains. Fields may be incomplete**

```json
1
```

<div id="type.Serenity.GuildChannel" />

### Serenity.GuildChannel

A guild channel in Discord, as represented by AntiRaid. Internal fields are subject to change

**Refer to [serenity::model::channel::GuildChannel](https://docs.rs/serenity/latest/serenity/model/channel/struct.GuildChannel.html) for more documentation on what this type contains. Fields may be incomplete**

```json
{
  "id": "0",
  "bitrate": null,
  "parent_id": null,
  "guild_id": "0",
  "type": 0,
  "owner_id": null,
  "last_message_id": null,
  "last_pin_timestamp": null,
  "name": "",
  "permission_overwrites": [],
  "position": 0,
  "topic": null,
  "user_limit": null,
  "nsfw": false,
  "rate_limit_per_user": null,
  "rtc_region": null,
  "video_quality_mode": null,
  "message_count": null,
  "member_count": null,
  "thread_metadata": null,
  "member": null,
  "default_auto_archive_duration": null,
  "permissions": null,
  "flags": 0,
  "total_message_sent": null,
  "available_tags": [],
  "applied_tags": [],
  "default_reaction_emoji": null,
  "default_thread_rate_limit_per_user": null,
  "status": null,
  "default_sort_order": null,
  "default_forum_layout": null
}
```

<div id="type.Serenity.PermissionOverwrite" />

### Serenity.PermissionOverwrite

A permission overwrite in Discord, as represented by AntiRaid. Internal fields are subject to change

**Refer to [serenity::model::channel::PermissionOverwrite](https://docs.rs/serenity/latest/serenity/model/channel/struct.PermissionOverwrite.html) for more documentation on what this type contains. Fields may be incomplete**

```json
{
  "allow": "2111062325329919",
  "deny": "2111062325329919",
  "id": "0",
  "type": 0
}
```

<div id="type.Serenity.ForumEmoji" />

### Serenity.ForumEmoji

A forum emoji in Discord, as represented by AntiRaid. Internal fields are subject to change

**Refer to [serenity::model::channel::ForumEmoji](https://docs.rs/serenity/latest/serenity/model/channel/struct.ForumEmoji.html) for more documentation on what this type contains. Fields may be incomplete**

```json
{
  "emoji_id": "0",
  "emoji_name": null
}
```

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


<div id="type.EditChannelOptions" />

### EditChannelOptions

Options for editing a channel in Discord

```json
{
  "channel_id": "0",
  "reason": "",
  "name": "my-channel",
  "type": 0,
  "position": 7,
  "topic": "My channel topic",
  "nsfw": true,
  "rate_limit_per_user": 5,
  "bitrate": null,
  "permission_overwrites": null,
  "parent_id": "0",
  "rtc_region": "us-west",
  "video_quality_mode": 1,
  "default_auto_archive_duration": 1440,
  "flags": 18,
  "available_tags": null,
  "default_reaction_emoji": {
    "emoji_id": "0",
    "emoji_name": null
  },
  "default_thread_rate_limit_per_user": null,
  "default_sort_order": null,
  "default_forum_layout": null
}
```

#### Fields

- `channel_id` ([string](#type.string)): The channel ID to edit
- `reason` ([string](#type.string)): The reason for editing the channel
- `name` ([string?](#type.string)): The name of the channel
- `type` ([string?](#type.string)): The type of the channel
- `position` ([number?](#type.number)): The position of the channel
- `topic` ([string?](#type.string)): The topic of the channel
- `nsfw` ([bool?](#type.bool)): Whether the channel is NSFW
- `rate_limit_per_user` ([number?](#type.number)): The rate limit per user/Slow mode of the channel
- `bitrate` ([number?](#type.number)): The bitrate of the channel
- `permission_overwrites` ([{Serenity.PermissionOverwrite}?](#type.Serenity.PermissionOverwrite)): The permission overwrites of the channel
- `parent_id` ([string??](#type.string)): The parent ID of the channel
- `rtc_region` ([string??](#type.string)): The RTC region of the channel
- `video_quality_mode` ([string?](#type.string)): The video quality mode of the channel
- `default_auto_archive_duration` ([string?](#type.string)): The default auto archive duration of the channel
- `flags` ([string?](#type.string)): The flags of the channel
- `available_tags` ([{Serenity.ForumTag}?](#type.Serenity.ForumTag)): The available tags of the channel
- `default_reaction_emoji` ([Serenity.ForumEmoji??](#type.Serenity.ForumEmoji)): The default reaction emoji of the channel
- `default_thread_rate_limit_per_user` ([number?](#type.number)): The default thread rate limit per user
- `default_sort_order` ([string?](#type.string)): The default sort order of the channel
- `default_forum_layout` ([string?](#type.string)): The default forum layout of the channel


<div id="type.EditThreadOptions" />

### EditThreadOptions

Options for editing a thread in Discord

```json
{
  "channel_id": "0",
  "reason": "",
  "name": "my-thread",
  "archived": false,
  "auto_archive_duration": 1440,
  "locked": false,
  "invitable": true,
  "rate_limit_per_user": 5,
  "flags": 18,
  "applied_tags": null
}
```

#### Fields

- `channel_id` ([string](#type.string)): The channel ID to edit
- `reason` ([string](#type.string)): The reason for editing the channel
- `name` ([string?](#type.string)): The name of the thread
- `archived` ([bool?](#type.bool)): Whether the thread is archived
- `auto_archive_duration` ([string?](#type.string)): The auto archive duration of the thread
- `locked` ([bool?](#type.bool)): Whether the thread is locked
- `invitable` ([bool?](#type.bool)): Whether the thread is invitable
- `rate_limit_per_user` ([number?](#type.number)): The rate limit per user/Slow mode of the thread
- `flags` ([string?](#type.string)): The flags of the thread
- `applied_tags` ([{Serenity.ForumTag}?](#type.Serenity.ForumTag)): The applied tags of the thread


<div id="type.DeleteChannelOption" />

### DeleteChannelOption

Options for deleting a channel in Discord

```json
{
  "channel_id": "0",
  "reason": "My reason here"
}
```

#### Fields

- `channel_id` ([string](#type.string)): The channel ID to delete
- `reason` ([string](#type.string)): The reason for deleting the channel


<div id="type.CreateMessageEmbedField" />

### CreateMessageEmbedField

A field in a message embed

```json
{
  "name": "",
  "value": "",
  "inline": false
}
```

#### Fields

- `name` ([string](#type.string)): The name of the field
- `value` ([string](#type.string)): The value of the field
- `inline` ([bool](#type.bool)): Whether the field is inline


<div id="type.CreateMessageEmbedAuthor" />

### CreateMessageEmbedAuthor

An author in a message embed

```json
{
  "name": "",
  "url": null,
  "icon_url": null
}
```

#### Fields

- `name` ([string](#type.string)): The name of the author
- `url` ([string?](#type.string)): The URL of the author
- `icon_url` ([string?](#type.string)): The icon URL of the author


<div id="type.CreateMessageEmbedFooter" />

### CreateMessageEmbedFooter

A footer in a message embed

```json
{
  "text": "",
  "icon_url": null
}
```

#### Fields

- `text` ([string](#type.string)): The text of the footer
- `icon_url` ([string?](#type.string)): The icon URL of the footer


<div id="type.CreateMessageEmbed" />

### CreateMessageEmbed

An embed in a message

```json
{
  "title": null,
  "description": null,
  "url": null,
  "timestamp": null,
  "color": null,
  "footer": null,
  "image": null,
  "thumbnail": null,
  "author": null,
  "fields": null
}
```

#### Fields

- `title` ([string?](#type.string)): The title of the embed
- `description` ([string?](#type.string)): The description of the embed
- `url` ([string?](#type.string)): The URL of the embed
- `timestamp` ([string?](#type.string)): The timestamp of the embed
- `color` ([string?](#type.string)): The color of the embed
- `footer` ([{Serenity.CreateMessageEmbedFooter}?](#type.Serenity.CreateMessageEmbedFooter)): The footer of the embed
- `image` ([string?](#type.string)): The image URL of the embed
- `thumbnail` ([string?](#type.string)): The thumbnail URL of the embed
- `author` ([{Serenity.CreateMessageEmbedAuthor}?](#type.Serenity.CreateMessageEmbedAuthor)): The author of the embed
- `fields` ([{Serenity.CreateMessageEmbedField}?](#type.Serenity.CreateMessageEmbedField)): The fields of the embed


<div id="type.CreateMessageAttachment" />

### CreateMessageAttachment

An attachment in a message

```json
{
  "filename": "",
  "description": null,
  "content": []
}
```

#### Fields

- `filename` ([string](#type.string)): The filename of the attachment
- `description` ([string?](#type.string)): The description (if any) of the attachment
- `content` ([{byte}](#type.byte)): The content of the attachment


<div id="type.CreateMessage" />

### CreateMessage

Options for creating a message in Discord

```json
{
  "embeds": null,
  "content": null,
  "attachments": null
}
```

#### Fields

- `embeds` ([{Serenity.CreateMessageEmbed}?](#type.Serenity.CreateMessageEmbed)): The embeds of the message
- `content` ([string?](#type.string)): The content of the message
- `attachments` ([{Serenity.CreateMessageAttachment}?](#type.Serenity.CreateMessageAttachment)): The attachments of the message


<div id="type.MessageHandle" />

### MessageHandle

A handle to a message in Discord, as represented by AntiRaid. Internal fields are subject to change



#### Methods

##### MessageHandle:data

```lua
function MessageHandle:data(): any
```

Gets the data of the message

###### Returns

- `data` ([any](#type.any)): The inner data of the message


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

- `Serenity.AuditLogs` ([](#type.)): The audit log entry
##### DiscordExecutor:get_channel

```lua
function DiscordExecutor:get_channel(data: GetChannelOptions): 
```

Gets a channel

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([GetChannelOptions](#type.GetChannelOptions)): Options for getting a channel.


###### Returns

- `Serenity.GuildChannel` ([](#type.)): The guild channel
##### DiscordExecutor:edit_channel

```lua
function DiscordExecutor:edit_channel(data: EditChannelOptions): 
```

Edits a channel

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([EditChannelOptions](#type.EditChannelOptions)): Options for editing a channel.


###### Returns

- `Serenity.GuildChannel` ([](#type.)): The guild channel
##### DiscordExecutor:edit_thread

```lua
function DiscordExecutor:edit_thread(data: EditThreadOptions): 
```

Edits a thread

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([EditThreadOptions](#type.EditThreadOptions)): Options for editing a thread.


###### Returns

- `Serenity.GuildChannel` ([](#type.)): The guild channel
##### DiscordExecutor:delete_channel

```lua
function DiscordExecutor:delete_channel(data: DeleteChannelOption): 
```

Deletes a channel

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([DeleteChannelOption](#type.DeleteChannelOption)): Options for deleting a channel.


###### Returns

- `Serenity.GuildChannel` ([](#type.)): The guild channel
##### DiscordExecutor:create_message

```lua
function DiscordExecutor:create_message(data: CreateMessage): 
```

Creates a message

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `data` ([CreateMessage](#type.CreateMessage)): Options for creating a message.


###### Returns

- `MessageHandle` ([](#type.)): The message


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

# @antiraid/interop

This plugin allows interoperability with AntiRaid and controlled interaction with the low-levels of AntiRaid templating subsystem.

## Types

<div id="type.null" />

### null

`null` is a special value that represents nothing. It is often used in AntiRaid instead of `nil` due to issues regarding existence etc. `null` is not equal to `nil` but is also an opaque type.



<div id="type.array_metatable" />

### array_metatable

`array_metatable` is a special metatable that is used to represent arrays across the Lua-AntiRaid templating subsystem boundary. This metatable must be set on all arrays over this boundary and is required to ensure AntiRaid knows the value you're sending it is actually an array and not an arbitrary Luau table.



## Methods

### array_metatable

```lua
function array_metatable(): table
```

Returns the array metatable.

#### Returns

- `array_metatable` ([table](#type.table)): The array metatable.

### memusage

```lua
function memusage(): f64
```

Returns the current memory usage of the Lua VM.

#### Returns

- `memory_usage` ([f64](#type.f64)): The current memory usage, in bytes, of the Lua VM.



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

- `filter` ([string](#type.string)): The name of the filter to use. See example for the parameters to pass for the filter as well as https://github.com/Anti-Raid/captcha.


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

KvExecutor allows templates to get, store and find persistent data within a server.



#### Methods

##### KvExecutor:find

```lua
function KvExecutor:find(key: string): {KvRecord}
```

**Note that this method returns a promise that must be yielded using [`promise.yield`](#type.promise.yield) to actually execute and return results.**



###### Parameters

- `key` ([string](#type.string)): The key to search for. % matches zero or more characters; _ matches a single character. To search anywhere in a string, surround {KEY} with %, e.g. %{KEY}%


###### Returns

- `records` ([{KvRecord}](#type.KvRecord)): The records found.
##### KvExecutor:get

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
function new(token: TemplateContext): KvExecutor
```

#### Parameters

- `token` ([TemplateContext](#type.TemplateContext)): The token of the template to use.


#### Returns

- `executor` ([KvExecutor](#type.KvExecutor)): A key-value executor.



---

# @antiraid/page

Create a page dedicated to your template on a server.

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
    "Scalar": {
      "inner": {
        "TimestampTz": {}
      }
    }
  },
  "nullable": false,
  "suggestions": {
    "None": {}
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
        "Scalar": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "Normal": {}
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Array": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "Token": {
                  "default_length": 10
                }
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Array": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "Textarea": {
                  "ctx": "anything"
                }
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Array": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "TemplateRef": {}
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Array": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "KittycatPermission": {}
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Array": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "User": {}
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Array": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "Role": {}
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Array": {
          "inner": {
            "String": {
              "min_length": 120,
              "max_length": 120,
              "allowed_values": [
                "allowed_value"
              ],
              "kind": {
                "Channel": {
                  "needed_bot_permissions": "2048",
                  "allowed_channel_types": [
                    0,
                    2
                  ]
                }
              }
            }
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Scalar": {
          "inner": {
            "Integer": {}
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Scalar": {
          "inner": {
            "Boolean": {}
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "Static": {
          "suggestions": [
            "suggestion"
          ]
        }
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
        "Scalar": {
          "inner": {
            "TimestampTz": {}
          }
        }
      },
      "nullable": false,
      "suggestions": {
        "None": {}
      },
      "secret": false,
      "ignored_for": [
        "Create",
        "Update"
      ]
    }
  ],
  "operations": []
}
```

#### Fields

- `id` ([string](#type.string)): The ID of the setting.
- `name` ([string](#type.string)): The name of the setting.
- `description` ([string](#type.string)): The description of the setting.
- `operations` ([{OperationType}](#type.OperationType)): The operations that can be performed on the setting. **Note that when using ``add_settings``, you must pass this as the second argument to settings and ignore this field.**
- `primary_key` ([string](#type.string)): The primary key of the setting that UNIQUELY identifies the row. When ``Delete`` is called, the value of this is what will be sent in the event. On ``Update``, this key MUST also exist (otherwise, the template MUST error out)
- `title_template` ([string](#type.string)): The template for the title of each row for the setting. This is a string that can contain placeholders for columns. The placeholders are in the form of ``{column_id}``. For example, if you have a column with ID ``col1`` and another with ID ``col2``, you can have a title template of ``{col1} - {col2}`` etc..
- `columns` ([{Setting.Column}](#type.Setting.Column)): The columns of the setting.


<div id="type.CreatePageSetting" />

### CreatePageSetting

A table containing a setting for a page



#### Fields

- `setting` ([Setting](#type.Setting)): The setting to add to the page.
- `operations` ([{string}](#type.string)): The operations to perform on the setting. Elements of the array can be either `View`, `Create`, `Update` or `Delete`.


<div id="type.CreatePage" />

### CreatePage

An intermediary structure for creating a page for a template



#### Fields

- `page_id` ([string](#type.string)): The ID of the page. This field **can be updated ONLY if the page is not created yet with no current settings.** The ID must not contain spaces, newlines, null characters, or tabs.
- `title` ([string](#type.string)): The title of the page. This field **can be updated ONLY if the page is not created yet.**
- `description` ([string](#type.string)): The description of the page. This field **can be updated ONLY if the page is not created yet.**
- `settings` ([table](#type.table)): The settings of the page. **This field is read-only.**
- `is_created` ([bool](#type.bool)): Whether the page is created. **This field is read-only.**
- `template` ([Template](#type.Template)): The template of the page. **This field is read-only.**


#### Methods

##### CreatePage:add_setting

```lua
function CreatePage:add_setting(setting: CreatePageSetting): nil
```

###### Parameters

- `setting` ([CreatePageSetting](#type.CreatePageSetting)): The setting to add to the page.


###### Returns

- `ret` ([nil](#type.nil)): 


## Enums

<div id="type.Setting.Column.InnerColumnType" />

### Setting.Column.InnerColumnType

The inner column type of the value



<div id="type.Setting.Column.ColumnType" />

### Setting.Column.ColumnType

The type of a setting column



#### Variants

#### Setting.Column.ColumnType::Scalar

A scalar column type.



##### Fields

- `inner` ([Setting.Column.InnerColumnType](#type.Setting.Column.InnerColumnType)): The inner type of the column.
#### Setting.Column.ColumnType::Array

An array column type.



##### Fields

- `inner` ([Setting.Column.InnerColumnType](#type.Setting.Column.InnerColumnType)): The array type of the column.


## Methods

### create_page

```lua
function create_page(token: TemplateContext): CreatePage
```

#### Parameters

- `token` ([TemplateContext](#type.TemplateContext)): The token of the template to use.


#### Returns

- `create_page` ([CreatePage](#type.CreatePage)): An empty created page.



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

- `module` ([string](#type.string)): The module name.
- `src` ([string?](#type.string)): The source of the sting.
- `stings` ([number](#type.number)): The number of stings.
- `reason` ([string?](#type.string)): The reason for the stings.
- `void_reason` ([string?](#type.string)): The reason the stings were voided.
- `guild_id` ([string](#type.string)): The guild ID the sting targets. **MUST MATCH THE GUILD ID THE TEMPLATE IS RUNNING ON**
- `creator` ([StingTarget](#type.StingTarget)): The creator of the sting.
- `target` ([StingTarget](#type.StingTarget)): The target of the sting.
- `state` ([StingState](#type.StingState)): The state of the sting.
- `created_at` ([string](#type.string)): When the sting was created as a chrono datetime.
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
  "created_at": "2024-12-25T12:46:36.832226484Z",
  "duration": {
    "secs": 60,
    "nanos": 0
  },
  "sting_data": {
    "a": "b"
  },
  "is_handled": false,
  "handle_log": {
    "a": "b"
  }
}
```

#### Fields

- `id` ([string](#type.string)): The sting ID.
- `module` ([string](#type.string)): The module name.
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
- `is_handled` ([boolean](#type.boolean)): Is Handled
- `handle_log` ([any](#type.any)): The handle log encountered while handling the sting.


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





---

# @antiraid/stream

Lua Streams, yield for a set of values using next.

## Types

<div id="type.LuaStream" />

### LuaStream<T>

LuaStream<T> provides a stream implementation. This is returned by MessageHandle's await_component_interaction for instance for handling button clicks/select menu choices etc.



## Methods

### next

```lua
function next(stream: LuaStream<T>): T
```

Returns the next value in the stream. Note that this is the only function other than `promise.yield` that yields.

#### Parameters

- `stream` ([LuaStream<T>](#type.LuaStream<T>)): The stream to get the next value from.


#### Returns

- `T` ([T](#type.T)): The next value in the stream, or nil if the stream is exhausted.



---

# @antiraid/typesext

Extra types used by Anti-Raid Lua templating subsystem to either add in common functionality such as streams or handle things like u64/i64 types performantly.

## Types

<div id="type.LuaStream" />

### LuaStream<T>

LuaStream<T> provides a stream implementation. This is returned by MessageHandle's await_component_interaction for instance for handling button clicks/select menu choices etc.



#### Methods

##### LuaStream:next

```lua
function LuaStream:next(): <T>?
```

Returns the next item in the stream.

###### Returns

- `item` ([<T>?](#type.<T>)): The next item in the stream.
##### LuaStream:for_each

```lua
function LuaStream:for_each(callback: function)
```

Executes a callback for every entry in the stream.

###### Parameters

- `callback` ([function](#type.function)): The callback to execute for each entry.



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

- `title` ([string](#type.string)): The title name of the event.
- `base_name` ([string](#type.string)): The base name of the event.
- `name` ([string](#type.string)): The name of the event.
- `data` ([unknown](#type.unknown)): The data of the event.
- `can_respond` ([boolean](#type.boolean)): Whether the event can be responded to.
- `response` ([unknown](#type.unknown)): The current response of the event. This can be overwritten by the template by just setting it to a new value.
- `uid` ([string](#type.string)): The unique identifier ID of the event. Will be guaranteed to be unique at a per-guild level.
- `author` ([string?](#type.string)): The author of the event, if any. If there is no known author, this field will either be `nil` or `null`.


<div id="type.TemplatePragma" />

## TemplatePragma

`TemplatePragma` contains the pragma of the template. Note that the list of fields below in non-exhaustive as templates can define extra fields on the pragma as well

```json
{
  "lang": "lua",
  "allowed_caps": []
}
```

### Fields

- `lang` ([string](#type.string)): The language of the template.
- `allowed_caps` ([{string}](#type.string)): The allowed capabilities provided to the template.


<div id="type.TemplateData" />

## TemplateData

`TemplateData` is a struct that represents the data associated with a template token. It is used to store the path and pragma of a template token.

```json
{
  "path": "test",
  "template": {
    "Named": "foo"
  },
  "pragma": {
    "lang": "lua",
    "allowed_caps": []
  }
}
```

### Fields

- `path` ([string](#type.string)): The path of the template token.
- `pragma` ([TemplatePragma](#type.TemplatePragma)): The pragma of the template.


<div id="type.TemplateContext" />

## TemplateContext

`TemplateContext` is a struct that represents the context of a template. Stores data including the templates data, pragma and what capabilities it should have access to. Passing a TemplateContext is often required when using AntiRaid plugins for security purposes.



### Fields

- `template_data` ([TemplateData](#type.TemplateData)): The data associated with the template.
- `guild_id` ([string](#type.string)): The current guild ID the template is running on.
- `current_user` ([Serenity.User](#type.Serenity.User)): Returns AntiRaid's discord user object [the current discord bot user driving the template].



