<div id="@antiraid/discord"></div>

# @antiraid/discord

<div id="Types"></div>

## Types

<div id="GetAuditLogOptions"></div>

## GetAuditLogOptions

Options for getting audit logs in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for getting audit logs in Discord
type GetAuditLogOptions = {
	--- The action type to filter by
	action_type: discord.AuditLogEventType?,

	--- The user ID to filter by
	user_id: discord.Snowflake?,

	--- The audit log entry ID to filter
	before: discord.Snowflake?,

	--- The number of entries to return
	limit: number?
}
```

</details>

<div id="action_type"></div>

### action_type

The action type to filter by

*This field is optional and may not be specified*

[discord](./discord.md).[AuditLogEventType](./discord.md#AuditLogEventType)?

<div id="user_id"></div>

### user_id

The user ID to filter by

*This field is optional and may not be specified*

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)?

<div id="before"></div>

### before

The audit log entry ID to filter

*This field is optional and may not be specified*

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)?

<div id="limit"></div>

### limit

The number of entries to return

*This field is optional and may not be specified*

[number](#number)?

<div id="GetAutoModerationRuleOptions"></div>

## GetAutoModerationRuleOptions

Options for getting an auto moderation rule in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for getting an auto moderation rule in Discord
type GetAutoModerationRuleOptions = {
	--- The rule ID
	rule_id: discord.Snowflake
}
```

</details>

<div id="rule_id"></div>

### rule_id

The rule ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="CreateAutoModerationRuleOptions"></div>

## CreateAutoModerationRuleOptions

Options for creating an auto moderation rule in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for creating an auto moderation rule in Discord
type CreateAutoModerationRuleOptions = {
	--- The reason for creating the rule
	reason: string,

	--- The data to create the rule with
	data: discordRest.CreateAutoModerationRuleRequest
}
```

</details>

<div id="reason"></div>

### reason

The reason for creating the rule

[string](#string)

<div id="data"></div>

### data

The data to create the rule with

[discordRest](./discordrest.md).[CreateAutoModerationRuleRequest](./discordrest.md#CreateAutoModerationRuleRequest)

<div id="EditAutoModerationRuleOptions"></div>

## EditAutoModerationRuleOptions

Options for editing an auto moderation rule in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for editing an auto moderation rule in Discord
type EditAutoModerationRuleOptions = {
	--- The rule ID
	rule_id: discord.Snowflake,

	--- The reason for editing the rule
	reason: string,

	--- The data to edit the rule with
	data: discordRest.ModifyAutoModerationRuleRequest
}
```

</details>

<div id="rule_id"></div>

### rule_id

The rule ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for editing the rule

[string](#string)

<div id="data"></div>

### data

The data to edit the rule with

[discordRest](./discordrest.md).[ModifyAutoModerationRuleRequest](./discordrest.md#ModifyAutoModerationRuleRequest)

<div id="DeleteAutoModerationRuleOptions"></div>

## DeleteAutoModerationRuleOptions

Options for deleting an auto moderation rule in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for deleting an auto moderation rule in Discord
type DeleteAutoModerationRuleOptions = {
	--- The rule ID
	rule_id: discord.Snowflake,

	--- The reason for deleting the rule
	reason: string
}
```

</details>

<div id="rule_id"></div>

### rule_id

The rule ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for deleting the rule

[string](#string)

<div id="GetChannelOptions"></div>

## GetChannelOptions

Options for getting a channel in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for getting a channel in Discord
type GetChannelOptions = {
	--- The channel ID
	channel_id: discord.Snowflake
}
```

</details>

<div id="channel_id"></div>

### channel_id

The channel ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="EditChannelOptions"></div>

## EditChannelOptions

Options for editing a channel in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for editing a channel in Discord
type EditChannelOptions = {
	--- The channel ID
	channel_id: discord.Snowflake,

	--- The reason for the edit
	reason: string,

	--- The data to edit the channel with
	data: discordRest.ModifyChannelRequest
}
```

</details>

<div id="channel_id"></div>

### channel_id

The channel ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for the edit

[string](#string)

<div id="data"></div>

### data

The data to edit the channel with

[discordRest](./discordrest.md).[ModifyChannelRequest](./discordrest.md#ModifyChannelRequest)

<div id="DeleteChannelOptions"></div>

## DeleteChannelOptions

Options for deleting a channel in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for deleting a channel in Discord
type DeleteChannelOptions = {
	--- The channel ID
	channel_id: discord.Snowflake,

	--- The reason for the deletion
	reason: string
}
```

</details>

<div id="channel_id"></div>

### channel_id

The channel ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for the deletion

[string](#string)

<div id="EditChannelPermissionsOptions"></div>

## EditChannelPermissionsOptions

Options for editting channel permissions

<details>
<summary>Raw Type</summary>

```luau
--- Options for editting channel permissions
type EditChannelPermissionsOptions = {
	--- The channel ID
	channel_id: discord.Snowflake,

	--- The target ID to edit permissions of
	target_id: discord.Snowflake,

	--- The allow permissions
	allow: typesext.MultiOption<string>,

	--- The deny permissions
	deny: typesext.MultiOption<string>,

	--- The type of the target
	kind: discord.OverwriteObjectType,

	--- The reason for the edit
	reason: string
}
```

</details>

<div id="channel_id"></div>

### channel_id

The channel ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="target_id"></div>

### target_id

The target ID to edit permissions of

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="allow"></div>

### allow

The allow permissions

[typesext](./typesext.md).[MultiOption](./typesext.md#MultiOption)&lt;[string](#string)&gt;

<div id="deny"></div>

### deny

The deny permissions

[typesext](./typesext.md).[MultiOption](./typesext.md#MultiOption)&lt;[string](#string)&gt;

<div id="kind"></div>

### kind

The type of the target

[discord](./discord.md).[OverwriteObjectType](./discord.md#OverwriteObjectType)

<div id="reason"></div>

### reason

The reason for the edit

[string](#string)

<div id="CreateChannelOptions"></div>

## CreateChannelOptions

Options for editing a channel in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for editing a channel in Discord
type CreateChannelOptions = {
	--- The reason for the create
	reason: string,

	--- The data to edit the channel with
	data: discordRest.CreateGuildChannelRequest
}
```

</details>

<div id="reason"></div>

### reason

The reason for the create

[string](#string)

<div id="data"></div>

### data

The data to edit the channel with

[discordRest](./discordrest.md).[CreateGuildChannelRequest](./discordrest.md#CreateGuildChannelRequest)

<div id="AddGuildMemberRoleOptions"></div>

## AddGuildMemberRoleOptions

Options for adding a role to a member

<details>
<summary>Raw Type</summary>

```luau
--- Options for adding a role to a member
type AddGuildMemberRoleOptions = {
	--- The member ID
	user_id: discord.Snowflake,

	--- The role ID
	role_id: discord.Snowflake,

	--- The reason for adding the role
	reason: string
}
```

</details>

<div id="user_id"></div>

### user_id

The member ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="role_id"></div>

### role_id

The role ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for adding the role

[string](#string)

<div id="RemoveGuildMemberRoleOptions"></div>

## RemoveGuildMemberRoleOptions

Options for removing a role from a member

<details>
<summary>Raw Type</summary>

```luau
--- Options for removing a role from a member
type RemoveGuildMemberRoleOptions = {
	--- The member ID
	user_id: discord.Snowflake,

	--- The role ID
	role_id: discord.Snowflake,

	--- The reason for adding the role
	reason: string
}
```

</details>

<div id="user_id"></div>

### user_id

The member ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="role_id"></div>

### role_id

The role ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for adding the role

[string](#string)

<div id="RemoveGuildMemberOptions"></div>

## RemoveGuildMemberOptions

Options for removing a member from a guild

<details>
<summary>Raw Type</summary>

```luau
--- Options for removing a member from a guild
type RemoveGuildMemberOptions = {
	--- The member ID
	user_id: discord.Snowflake,

	--- The reason for removing the member
	reason: string
}
```

</details>

<div id="user_id"></div>

### user_id

The member ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for removing the member

[string](#string)

<div id="GetGuildBansOptions"></div>

## GetGuildBansOptions

Options for getting guild bans



Note: If both `before` and `after` are provided, `before` will take precedence.

<details>
<summary>Raw Type</summary>

```luau
--- Options for getting guild bans
---
--- Note: If both `before` and `after` are provided, `before` will take precedence.
type GetGuildBansOptions = {
	--- The limit of bans to get (max 100)
	limit: number?,

	--- Before a certain user ID
	before: discord.Snowflake?,

	--- After a certain user ID
	after: discord.Snowflake?
}
```

</details>

<div id="limit"></div>

### limit

The limit of bans to get (max 100)

*This field is optional and may not be specified*

[number](#number)?

<div id="before"></div>

### before

Before a certain user ID

*This field is optional and may not be specified*

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)?

<div id="after"></div>

### after

After a certain user ID

*This field is optional and may not be specified*

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)?

<div id="CreateMessageOptions"></div>

## CreateMessageOptions

Options for sending a message to a channel in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for sending a message to a channel in Discord
type CreateMessageOptions = {
	--- The channel ID
	channel_id: discord.Snowflake,

	--- The data to send the message with
	data: discordRest.CreateMessageRequest
}
```

</details>

<div id="channel_id"></div>

### channel_id

The channel ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="data"></div>

### data

The data to send the message with

[discordRest](./discordrest.md).[CreateMessageRequest](./discordrest.md#CreateMessageRequest)

<div id="CreateCommandOptions"></div>

## CreateCommandOptions

Options for creating a command in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for creating a command in Discord
type CreateCommandOptions = {
	--- The data to create the command with
	data: discordRest.CreateGuildApplicationCommandRequest
}
```

</details>

<div id="data"></div>

### data

The data to create the command with

[discordRest](./discordrest.md).[CreateGuildApplicationCommandRequest](./discordrest.md#CreateGuildApplicationCommandRequest)

<div id="CreateCommandsOptions"></div>

## CreateCommandsOptions

Options for creating multiple command in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for creating multiple command in Discord
type CreateCommandsOptions = {
	--- The data to create the command with
	data: {discordRest.CreateGuildApplicationCommandRequest}
}
```

</details>

<div id="data"></div>

### data

The data to create the command with

{[discordRest](./discordrest.md).[CreateGuildApplicationCommandRequest](./discordrest.md#CreateGuildApplicationCommandRequest)}

<div id="CreateInteractionResponseOptions"></div>

## CreateInteractionResponseOptions

Options for creating an interaction response in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for creating an interaction response in Discord
type CreateInteractionResponseOptions = {
	--- The interaction ID
	interaction_id: discord.Snowflake,

	--- The interaction token
	interaction_token: string,

	--- The data to create the interaction response with
	data: discordRest.CreateInteractionRequest
}
```

</details>

<div id="interaction_id"></div>

### interaction_id

The interaction ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="interaction_token"></div>

### interaction_token

The interaction token

[string](#string)

<div id="data"></div>

### data

The data to create the interaction response with

[discordRest](./discordrest.md).[CreateInteractionRequest](./discordrest.md#CreateInteractionRequest)

<div id="CreateFollowupMessageOptions"></div>

## CreateFollowupMessageOptions

Options for creating a followup message in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for creating a followup message in Discord
type CreateFollowupMessageOptions = {
	--- The interaction token
	interaction_token: string,

	--- The data to create the followup message with
	data: discordRest.CreateFollowupMessageRequest
}
```

</details>

<div id="interaction_token"></div>

### interaction_token

The interaction token

[string](#string)

<div id="data"></div>

### data

The data to create the followup message with

[discordRest](./discordrest.md).[CreateFollowupMessageRequest](./discordrest.md#CreateFollowupMessageRequest)

<div id="MessagePagination"></div>

## MessagePagination

A message pagination object

<details>
<summary>Raw Type</summary>

```luau
--- A message pagination object
type MessagePagination = {
	type: "After" | "Around" | "Before",

	id: discord.Snowflake
}
```

</details>

<div id="type"></div>

### type

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"After"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Around"
```

</details>

<details>
<summary>Variant 3</summary>

```luau
"Before"
```

</details>

<div id="id"></div>

### id

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="GetMessagesOptions"></div>

## GetMessagesOptions

Options for getting messages in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for getting messages in Discord
type GetMessagesOptions = {
	--- The channel ID
	channel_id: discord.Snowflake,

	--- The target message
	target: MessagePagination?,

	--- The limit of messages to get
	limit: number?
}
```

</details>

<div id="channel_id"></div>

### channel_id

The channel ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="target"></div>

### target

The target message

*This field is optional and may not be specified*

[MessagePagination](#MessagePagination)?

<div id="limit"></div>

### limit

The limit of messages to get

*This field is optional and may not be specified*

[number](#number)?

<div id="GetMessageOptions"></div>

## GetMessageOptions

Options for getting a message in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for getting a message in Discord
type GetMessageOptions = {
	--- The channel ID
	channel_id: discord.Snowflake,

	--- The message ID
	message_id: discord.Snowflake
}
```

</details>

<div id="channel_id"></div>

### channel_id

The channel ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="message_id"></div>

### message_id

The message ID

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="CreateGuildBanOptions"></div>

## CreateGuildBanOptions

Options for creating a guild ban in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for creating a guild ban in Discord
type CreateGuildBanOptions = {
	--- The user ID to ban
	user_id: discord.Snowflake,

	--- The reason for the ban
	reason: string,

	--- The number of seconds to delete messages from
	delete_message_seconds: number?
}
```

</details>

<div id="user_id"></div>

### user_id

The user ID to ban

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for the ban

[string](#string)

<div id="delete_message_seconds"></div>

### delete_message_seconds

The number of seconds to delete messages from

*This field is optional and may not be specified*

[number](#number)?

<div id="RemoveGuildBanOptions"></div>

## RemoveGuildBanOptions

Options for removing a guild ban in Discord

<details>
<summary>Raw Type</summary>

```luau
--- Options for removing a guild ban in Discord
type RemoveGuildBanOptions = {
	--- The user ID to unban
	user_id: discord.Snowflake,

	--- The reason for the unban
	reason: string
}
```

</details>

<div id="user_id"></div>

### user_id

The user ID to unban

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for the unban

[string](#string)

<div id="GetGuildMembersOptions"></div>

## GetGuildMembersOptions

Options for getting guild members

<details>
<summary>Raw Type</summary>

```luau
--- Options for getting guild members
type GetGuildMembersOptions = {
	--- The limit of members to get
	limit: number?,

	--- The user ID to get members after
	after: discord.Snowflake?
}
```

</details>

<div id="limit"></div>

### limit

The limit of members to get

*This field is optional and may not be specified*

[number](#number)?

<div id="after"></div>

### after

The user ID to get members after

*This field is optional and may not be specified*

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)?

<div id="SearchGuildMembersOptions"></div>

## SearchGuildMembersOptions

Options for searching guild members

<details>
<summary>Raw Type</summary>

```luau
--- Options for searching guild members
type SearchGuildMembersOptions = {
	--- The query to search for
	query: string,

	--- The limit of members to get
	limit: number?
}
```

</details>

<div id="query"></div>

### query

The query to search for

[string](#string)

<div id="limit"></div>

### limit

The limit of members to get

*This field is optional and may not be specified*

[number](#number)?

<div id="ModifyGuildMemberOptions"></div>

## ModifyGuildMemberOptions

Options for modifying a guild member

<details>
<summary>Raw Type</summary>

```luau
--- Options for modifying a guild member
type ModifyGuildMemberOptions = {
	--- The user ID to modify
	user_id: discord.Snowflake,

	--- The reason for the modification
	reason: string,

	--- The data to modify the member with
	data: discordRest.ModifyGuildMemberRequest
}
```

</details>

<div id="user_id"></div>

### user_id

The user ID to modify

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="reason"></div>

### reason

The reason for the modification

[string](#string)

<div id="data"></div>

### data

The data to modify the member with

[discordRest](./discordrest.md).[ModifyGuildMemberRequest](./discordrest.md#ModifyGuildMemberRequest)

<div id="ModifyGuildOptions"></div>

## ModifyGuildOptions

Options for modifying a guild

<details>
<summary>Raw Type</summary>

```luau
--- Options for modifying a guild
type ModifyGuildOptions = {
	data: discordRest.ModifyGuildRequest,

	reason: string
}
```

</details>

<div id="data"></div>

### data

[discordRest](./discordrest.md).[ModifyGuildRequest](./discordrest.md#ModifyGuildRequest)

<div id="reason"></div>

### reason

[string](#string)

<div id="AntiRaidCheckPermissionsOptions"></div>

## AntiRaidCheckPermissionsOptions

Options for checking if a user has the needed Discord permissions to perform an action

<details>
<summary>Raw Type</summary>

```luau
--- Options for checking if a user has the needed Discord permissions to perform an action
type AntiRaidCheckPermissionsOptions = {
	--- The user ID to check permissions for
	user_id: discord.Snowflake,

	--- The needed permissions
	needed_permissions: typesext.MultiOption<string>
}
```

</details>

<div id="user_id"></div>

### user_id

The user ID to check permissions for

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="needed_permissions"></div>

### needed_permissions

The needed permissions

[typesext](./typesext.md).[MultiOption](./typesext.md#MultiOption)&lt;[string](#string)&gt;

<div id="AntiRaidCheckPermissionsAndHierarchyOptions"></div>

## AntiRaidCheckPermissionsAndHierarchyOptions

Options for checking if a user has the needed Discord permissions to perform an action

and is above the target user in terms of hierarchy

<details>
<summary>Raw Type</summary>

```luau
--- Options for checking if a user has the needed Discord permissions to perform an action 
--- and is above the target user in terms of hierarchy
type AntiRaidCheckPermissionsAndHierarchyOptions = {
	--- The user ID to check permissions for
	user_id: discord.Snowflake,

	--- The target ID to check permissions for
	target_id: discord.Snowflake,

	--- The needed permissions
	needed_permissions: typesext.MultiOption<string>
}
```

</details>

<div id="user_id"></div>

### user_id

The user ID to check permissions for

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="target_id"></div>

### target_id

The target ID to check permissions for

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="needed_permissions"></div>

### needed_permissions

The needed permissions

[typesext](./typesext.md).[MultiOption](./typesext.md#MultiOption)&lt;[string](#string)&gt;

<div id="AntiRaidCheckPermissionsResponse"></div>

## AntiRaidCheckPermissionsResponse

Extra/additional success response for checking if a user has the needed Discord permissions to perform an action

<details>
<summary>Raw Type</summary>

```luau
--- Extra/additional success response for checking if a user has the needed Discord permissions to perform an action
type AntiRaidCheckPermissionsResponse = {
	--- The partial guild
	partial_guild: discord.Partial<discord.GuildObject>,

	--- The member
	member: discord.GuildMemberObject,

	--- The permissions
	permissions: typesext.MultiOption<string>
}
```

</details>

<div id="partial_guild"></div>

### partial_guild

The partial guild

[discord](./discord.md).[Partial](./discord.md#Partial)&lt;[discord](./discord.md).[GuildObject](./discord.md#GuildObject)&gt;

<div id="member"></div>

### member

The member

[discord](./discord.md).[GuildMemberObject](./discord.md#GuildMemberObject)

<div id="permissions"></div>

### permissions

The permissions

[typesext](./typesext.md).[MultiOption](./typesext.md#MultiOption)&lt;[string](#string)&gt;

<div id="CreateGuildRoleOptions"></div>

## CreateGuildRoleOptions

Options for creating a guild role

<details>
<summary>Raw Type</summary>

```luau
--- Options for creating a guild role
type CreateGuildRoleOptions = {
	--- The reason for the creation
	reason: string,

	--- The data to create the role with
	data: discordRest.CreateGuildRoleRequest
}
```

</details>

<div id="reason"></div>

### reason

The reason for the creation

[string](#string)

<div id="data"></div>

### data

The data to create the role with

[discordRest](./discordrest.md).[CreateGuildRoleRequest](./discordrest.md#CreateGuildRoleRequest)

<div id="ModifyRolePositionOptions"></div>

## ModifyRolePositionOptions

Options for modifying a guild role position

<details>
<summary>Raw Type</summary>

```luau
--- Options for modifying a guild role position
type ModifyRolePositionOptions = {
	--- The data to modify the role position with
	data: {discordRest.ModifyGuildRolePositionsRequest},

	--- The reason for the modification
	reason: string
}
```

</details>

<div id="data"></div>

### data

The data to modify the role position with

{[discordRest](./discordrest.md).[ModifyGuildRolePositionsRequest](./discordrest.md#ModifyGuildRolePositionsRequest)}

<div id="reason"></div>

### reason

The reason for the modification

[string](#string)

<div id="DiscordExecutor"></div>

## DiscordExecutor

DiscordExecutor allows templates to access/use the Discord API in a sandboxed form.

<details>
<summary>Raw Type</summary>

```luau
--- DiscordExecutor allows templates to access/use the Discord API in a sandboxed form.
type DiscordExecutor = {
	-- Antiraid helpers
	-- Checks an audit log reason for validity, errors out if invalid
	antiraid_check_reason: (self: DiscordExecutor, reason: string) -> nil,

	-- Checks if a specified user with an ID of `data.user_id` has the specified permissions in the server
	antiraid_check_permissions: (self: DiscordExecutor, data: AntiRaidCheckPermissionsOptions) -> promise.LuaPromise<AntiRaidCheckPermissionsResponse>,

	-- Checks if a specified user with an ID of `data.user_id` has the specified permissions in the server and is above the target user with an ID of `data.target_id` in terms of hierarchy
	antiraid_check_permissions_and_hierarchy: (self: DiscordExecutor, data: AntiRaidCheckPermissionsAndHierarchyOptions) -> promise.LuaPromise<AntiRaidCheckPermissionsResponse>,

	-- Discord API
	--- Gets the audit logs
	get_audit_logs: (self: DiscordExecutor, data: GetAuditLogOptions) -> promise.LuaPromise<LazyAuditLogObject>,

	--- Lists the auto moderation rules available
	list_auto_moderation_rules: (self: DiscordExecutor) -> promise.LuaPromise<LazyAutomoderationRuleObjectList>,

	--- Gets an auto moderation rule by ID
	get_auto_moderation_rule: (self: DiscordExecutor, data: GetAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>,

	--- Creates an auto moderation rule
	create_auto_moderation_rule: (self: DiscordExecutor, data: CreateAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>,

	--- Edits an auto moderation rule
	edit_auto_moderation_rule: (self: DiscordExecutor, data: EditAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>,

	--- Deletes an auto moderation rule
	delete_auto_moderation_rule: (self: DiscordExecutor, data: DeleteAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>,

	--- Gets a channel
	get_channel: (self: DiscordExecutor, data: GetChannelOptions) -> promise.LuaPromise<LazyChannelObject>,

	--- Edits a channel
	edit_channel: (self: DiscordExecutor, data: EditChannelOptions) -> promise.LuaPromise<LazyChannelObject>,

	--- Deletes a channel
	delete_channel: (self: DiscordExecutor, data: DeleteChannelOptions) -> promise.LuaPromise<LazyChannelObject>,

	--- Edits channel permissions for a target
	edit_channel_permissions: (self: DiscordExecutor, data: EditChannelPermissionsOptions) -> promise.LuaPromise<nil>,

	-- Guild
	--- Gets the guild
	get_guild: (self: DiscordExecutor) -> promise.LuaPromise<LazyGuildObject>,

	--- Gets the guilds preview
	get_guild_preview: (self: DiscordExecutor) -> promise.LuaPromise<LazyGuildPreviewObject>,

	--- Edits the guild
	modify_guild: (self: DiscordExecutor, data: ModifyGuildOptions) -> promise.LuaPromise<LazyGuildObject>,

	--- Gets the guild channels
	get_guild_channels: (self: DiscordExecutor) -> promise.LuaPromise<LazyChannelsObject>,

	--- Creates a guild channel
	create_guild_channel: (self: DiscordExecutor, data: CreateChannelOptions) -> promise.LuaPromise<LazyChannelObject>,

	--- Modify guild channel positions. Only channels to be modified are required to be passed in `data`.
	modify_guild_channel_positions: (self: DiscordExecutor, data: {discordRest.ModifyGuildChannelPositionsRequest}) -> promise.LuaPromise<nil>,

	--- List active guild threads
	list_active_guild_threads: (self: DiscordExecutor) -> promise.LuaPromise<LazyActiveGuildThreadsResponse>,

	--- Gets a guild member by ID
	get_guild_member: (self: DiscordExecutor, user_id: string) -> promise.LuaPromise<LazyGuildMemberObject>,

	--- List all guild members
	list_guild_members: (self: DiscordExecutor, data: GetGuildMembersOptions) -> promise.LuaPromise<LazyGuildMembersObject>,

	--- Search guild members
	search_guild_members: (self: DiscordExecutor, data: SearchGuildMembersOptions) -> promise.LuaPromise<LazyGuildMembersObject>,

	--- Modify guild member (this includes timing out a member using `communication_disabled_until`)
	modify_guild_member: (self: DiscordExecutor, data: ModifyGuildMemberOptions) -> promise.LuaPromise<LazyGuildMemberObject>,

	--- Adds a role to a member
	add_guild_member_role: (self: DiscordExecutor, data: AddGuildMemberRoleOptions) -> promise.LuaPromise<nil>,

	--- Removes a role from a member
	remove_guild_member_role: (self: DiscordExecutor, data: RemoveGuildMemberRoleOptions) -> promise.LuaPromise<nil>,

	-- Removes a member from a guild
	remove_guild_member: (self: DiscordExecutor, data: RemoveGuildMemberOptions) -> promise.LuaPromise<nil>,

	--- Gets guild bans
	get_guild_bans: (self: DiscordExecutor, data: GetGuildBansOptions) -> promise.LuaPromise<LazyBanObjectList>,

	--- Gets a guild ban for a user or nil if it does not exist
	get_guild_ban: (self: DiscordExecutor, user_id: discord.Snowflake) -> promise.LuaPromise<LazyBanOptionalObject>,

	--- Creates a guild ban
	create_guild_ban: (self: DiscordExecutor, data: CreateGuildBanOptions) -> promise.LuaPromise<nil>,

	--- Removes a guild ban
	remove_guild_ban: (self: DiscordExecutor, data: RemoveGuildBanOptions) -> promise.LuaPromise<nil>,

	--- Returns the guild roles of a guild
	get_guild_roles: (self: DiscordExecutor, guild_id: discord.Snowflake) -> promise.LuaPromise<LazyRolesObject>,

	--- Returns a guild role by ID
	get_guild_role: (self: DiscordExecutor, role_id: discord.Snowflake) -> promise.LuaPromise<LazyRoleObject>,

	--- Creates a guild role
	create_guild_role: (self: DiscordExecutor, data: CreateGuildRoleOptions) -> promise.LuaPromise<LazyRoleObject>,

	--- Modify guild role positions
	modify_guild_role_positions: (self: DiscordExecutor, data: ModifyRolePositionOptions) -> promise.LuaPromise<LazyRolesObject>,

	-- Messages
	--- Gets messages from a channel
	get_channel_messages: (self: DiscordExecutor, data: GetMessagesOptions) -> promise.LuaPromise<LazyMessagesObject>,

	--- Gets a message
	get_channel_message: (self: DiscordExecutor, data: GetMessageOptions) -> promise.LuaPromise<LazyMessageObject>,

	--- Creates a message
	create_message: (self: DiscordExecutor, data: CreateMessageOptions) -> promise.LuaPromise<LazyMessageObject>,

	-- Uncategorized (for now)
	--- Creates an interaction response
	create_interaction_response: (self: DiscordExecutor, data: CreateInteractionResponseOptions) -> promise.LuaPromise<nil>,

	--- Creates a followup interaction response
	create_followup_message: (self: DiscordExecutor, data: CreateFollowupMessageOptions) -> promise.LuaPromise<LazyMessageObject>,

	--- Gets the original interaction response
	get_original_interaction_response: (self: DiscordExecutor, interaction_token: string) -> promise.LuaPromise<LazyMessageObject>,

	--- Gets all guild commands currently registered
	get_guild_commands: (self: DiscordExecutor) -> promise.LuaPromise<LazyApplicationCommandObject>,

	--- Creates a guild command
	create_guild_command: (self: DiscordExecutor, data: CreateCommandOptions) -> promise.LuaPromise<LazyApplicationCommandObject>,

	--- Creates multiple guild commands
	create_guild_commands: (self: DiscordExecutor, data: CreateCommandsOptions) -> promise.LuaPromise<LazyApplicationCommandsObject>
}
```

</details>

<div id="antiraid_check_reason"></div>

### antiraid_check_reason

Antiraid helpers

Checks an audit log reason for validity, errors out if invalid

<details>
<summary>Function Signature</summary>

```luau
-- Antiraid helpers
-- Checks an audit log reason for validity, errors out if invalid
antiraid_check_reason: (self: DiscordExecutor, reason: string) -> nil
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="reason"></div>

##### reason

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[nil](#nil)<div id="antiraid_check_permissions"></div>

### antiraid_check_permissions

Checks if a specified user with an ID of `data.user_id` has the specified permissions in the server

<details>
<summary>Function Signature</summary>

```luau
-- Checks if a specified user with an ID of `data.user_id` has the specified permissions in the server
antiraid_check_permissions: (self: DiscordExecutor, data: AntiRaidCheckPermissionsOptions) -> promise.LuaPromise<AntiRaidCheckPermissionsResponse>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[AntiRaidCheckPermissionsOptions](#AntiRaidCheckPermissionsOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[AntiRaidCheckPermissionsResponse](#AntiRaidCheckPermissionsResponse)&gt;<div id="antiraid_check_permissions_and_hierarchy"></div>

### antiraid_check_permissions_and_hierarchy

Checks if a specified user with an ID of `data.user_id` has the specified permissions in the server and is above the target user with an ID of `data.target_id` in terms of hierarchy

<details>
<summary>Function Signature</summary>

```luau
-- Checks if a specified user with an ID of `data.user_id` has the specified permissions in the server and is above the target user with an ID of `data.target_id` in terms of hierarchy
antiraid_check_permissions_and_hierarchy: (self: DiscordExecutor, data: AntiRaidCheckPermissionsAndHierarchyOptions) -> promise.LuaPromise<AntiRaidCheckPermissionsResponse>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[AntiRaidCheckPermissionsAndHierarchyOptions](#AntiRaidCheckPermissionsAndHierarchyOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[AntiRaidCheckPermissionsResponse](#AntiRaidCheckPermissionsResponse)&gt;<div id="get_audit_logs"></div>

### get_audit_logs

Discord API

Gets the audit logs

<details>
<summary>Function Signature</summary>

```luau
-- Discord API
--- Gets the audit logs
get_audit_logs: (self: DiscordExecutor, data: GetAuditLogOptions) -> promise.LuaPromise<LazyAuditLogObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[GetAuditLogOptions](#GetAuditLogOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyAuditLogObject](#LazyAuditLogObject)&gt;<div id="list_auto_moderation_rules"></div>

### list_auto_moderation_rules

Lists the auto moderation rules available

<details>
<summary>Function Signature</summary>

```luau
--- Lists the auto moderation rules available
list_auto_moderation_rules: (self: DiscordExecutor) -> promise.LuaPromise<LazyAutomoderationRuleObjectList>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyAutomoderationRuleObjectList](#LazyAutomoderationRuleObjectList)&gt;<div id="get_auto_moderation_rule"></div>

### get_auto_moderation_rule

Gets an auto moderation rule by ID

<details>
<summary>Function Signature</summary>

```luau
--- Gets an auto moderation rule by ID
get_auto_moderation_rule: (self: DiscordExecutor, data: GetAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[GetAutoModerationRuleOptions](#GetAutoModerationRuleOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyAutomoderationRuleObject](#LazyAutomoderationRuleObject)&gt;<div id="create_auto_moderation_rule"></div>

### create_auto_moderation_rule

Creates an auto moderation rule

<details>
<summary>Function Signature</summary>

```luau
--- Creates an auto moderation rule
create_auto_moderation_rule: (self: DiscordExecutor, data: CreateAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateAutoModerationRuleOptions](#CreateAutoModerationRuleOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyAutomoderationRuleObject](#LazyAutomoderationRuleObject)&gt;<div id="edit_auto_moderation_rule"></div>

### edit_auto_moderation_rule

Edits an auto moderation rule

<details>
<summary>Function Signature</summary>

```luau
--- Edits an auto moderation rule
edit_auto_moderation_rule: (self: DiscordExecutor, data: EditAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[EditAutoModerationRuleOptions](#EditAutoModerationRuleOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyAutomoderationRuleObject](#LazyAutomoderationRuleObject)&gt;<div id="delete_auto_moderation_rule"></div>

### delete_auto_moderation_rule

Deletes an auto moderation rule

<details>
<summary>Function Signature</summary>

```luau
--- Deletes an auto moderation rule
delete_auto_moderation_rule: (self: DiscordExecutor, data: DeleteAutoModerationRuleOptions) -> promise.LuaPromise<LazyAutomoderationRuleObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[DeleteAutoModerationRuleOptions](#DeleteAutoModerationRuleOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyAutomoderationRuleObject](#LazyAutomoderationRuleObject)&gt;<div id="get_channel"></div>

### get_channel

Gets a channel

<details>
<summary>Function Signature</summary>

```luau
--- Gets a channel
get_channel: (self: DiscordExecutor, data: GetChannelOptions) -> promise.LuaPromise<LazyChannelObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[GetChannelOptions](#GetChannelOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyChannelObject](#LazyChannelObject)&gt;<div id="edit_channel"></div>

### edit_channel

Edits a channel

<details>
<summary>Function Signature</summary>

```luau
--- Edits a channel
edit_channel: (self: DiscordExecutor, data: EditChannelOptions) -> promise.LuaPromise<LazyChannelObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[EditChannelOptions](#EditChannelOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyChannelObject](#LazyChannelObject)&gt;<div id="delete_channel"></div>

### delete_channel

Deletes a channel

<details>
<summary>Function Signature</summary>

```luau
--- Deletes a channel
delete_channel: (self: DiscordExecutor, data: DeleteChannelOptions) -> promise.LuaPromise<LazyChannelObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[DeleteChannelOptions](#DeleteChannelOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyChannelObject](#LazyChannelObject)&gt;<div id="edit_channel_permissions"></div>

### edit_channel_permissions

Edits channel permissions for a target

<details>
<summary>Function Signature</summary>

```luau
--- Edits channel permissions for a target
edit_channel_permissions: (self: DiscordExecutor, data: EditChannelPermissionsOptions) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[EditChannelPermissionsOptions](#EditChannelPermissionsOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="get_guild"></div>

### get_guild

Guild

Gets the guild

<details>
<summary>Function Signature</summary>

```luau
-- Guild
--- Gets the guild
get_guild: (self: DiscordExecutor) -> promise.LuaPromise<LazyGuildObject>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyGuildObject](#LazyGuildObject)&gt;<div id="get_guild_preview"></div>

### get_guild_preview

Gets the guilds preview

<details>
<summary>Function Signature</summary>

```luau
--- Gets the guilds preview
get_guild_preview: (self: DiscordExecutor) -> promise.LuaPromise<LazyGuildPreviewObject>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyGuildPreviewObject](#LazyGuildPreviewObject)&gt;<div id="modify_guild"></div>

### modify_guild

Edits the guild

<details>
<summary>Function Signature</summary>

```luau
--- Edits the guild
modify_guild: (self: DiscordExecutor, data: ModifyGuildOptions) -> promise.LuaPromise<LazyGuildObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[ModifyGuildOptions](#ModifyGuildOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyGuildObject](#LazyGuildObject)&gt;<div id="get_guild_channels"></div>

### get_guild_channels

Gets the guild channels

<details>
<summary>Function Signature</summary>

```luau
--- Gets the guild channels
get_guild_channels: (self: DiscordExecutor) -> promise.LuaPromise<LazyChannelsObject>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyChannelsObject](#LazyChannelsObject)&gt;<div id="create_guild_channel"></div>

### create_guild_channel

Creates a guild channel

<details>
<summary>Function Signature</summary>

```luau
--- Creates a guild channel
create_guild_channel: (self: DiscordExecutor, data: CreateChannelOptions) -> promise.LuaPromise<LazyChannelObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateChannelOptions](#CreateChannelOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyChannelObject](#LazyChannelObject)&gt;<div id="modify_guild_channel_positions"></div>

### modify_guild_channel_positions

Modify guild channel positions. Only channels to be modified are required to be passed in `data`.

<details>
<summary>Function Signature</summary>

```luau
--- Modify guild channel positions. Only channels to be modified are required to be passed in `data`.
modify_guild_channel_positions: (self: DiscordExecutor, data: {discordRest.ModifyGuildChannelPositionsRequest}) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

{[discordRest](./discordrest.md).[ModifyGuildChannelPositionsRequest](./discordrest.md#ModifyGuildChannelPositionsRequest)}

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="list_active_guild_threads"></div>

### list_active_guild_threads

List active guild threads

<details>
<summary>Function Signature</summary>

```luau
--- List active guild threads
list_active_guild_threads: (self: DiscordExecutor) -> promise.LuaPromise<LazyActiveGuildThreadsResponse>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyActiveGuildThreadsResponse](#LazyActiveGuildThreadsResponse)&gt;<div id="get_guild_member"></div>

### get_guild_member

Gets a guild member by ID

<details>
<summary>Function Signature</summary>

```luau
--- Gets a guild member by ID
get_guild_member: (self: DiscordExecutor, user_id: string) -> promise.LuaPromise<LazyGuildMemberObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="user_id"></div>

##### user_id

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyGuildMemberObject](#LazyGuildMemberObject)&gt;<div id="list_guild_members"></div>

### list_guild_members

List all guild members

<details>
<summary>Function Signature</summary>

```luau
--- List all guild members
list_guild_members: (self: DiscordExecutor, data: GetGuildMembersOptions) -> promise.LuaPromise<LazyGuildMembersObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[GetGuildMembersOptions](#GetGuildMembersOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyGuildMembersObject](#LazyGuildMembersObject)&gt;<div id="search_guild_members"></div>

### search_guild_members

Search guild members

<details>
<summary>Function Signature</summary>

```luau
--- Search guild members
search_guild_members: (self: DiscordExecutor, data: SearchGuildMembersOptions) -> promise.LuaPromise<LazyGuildMembersObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[SearchGuildMembersOptions](#SearchGuildMembersOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyGuildMembersObject](#LazyGuildMembersObject)&gt;<div id="modify_guild_member"></div>

### modify_guild_member

Modify guild member (this includes timing out a member using `communication_disabled_until`)

<details>
<summary>Function Signature</summary>

```luau
--- Modify guild member (this includes timing out a member using `communication_disabled_until`)
modify_guild_member: (self: DiscordExecutor, data: ModifyGuildMemberOptions) -> promise.LuaPromise<LazyGuildMemberObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[ModifyGuildMemberOptions](#ModifyGuildMemberOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyGuildMemberObject](#LazyGuildMemberObject)&gt;<div id="add_guild_member_role"></div>

### add_guild_member_role

Adds a role to a member

<details>
<summary>Function Signature</summary>

```luau
--- Adds a role to a member
add_guild_member_role: (self: DiscordExecutor, data: AddGuildMemberRoleOptions) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[AddGuildMemberRoleOptions](#AddGuildMemberRoleOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="remove_guild_member_role"></div>

### remove_guild_member_role

Removes a role from a member

<details>
<summary>Function Signature</summary>

```luau
--- Removes a role from a member
remove_guild_member_role: (self: DiscordExecutor, data: RemoveGuildMemberRoleOptions) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[RemoveGuildMemberRoleOptions](#RemoveGuildMemberRoleOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="remove_guild_member"></div>

### remove_guild_member

Removes a member from a guild

<details>
<summary>Function Signature</summary>

```luau
-- Removes a member from a guild
remove_guild_member: (self: DiscordExecutor, data: RemoveGuildMemberOptions) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[RemoveGuildMemberOptions](#RemoveGuildMemberOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="get_guild_bans"></div>

### get_guild_bans

Gets guild bans

<details>
<summary>Function Signature</summary>

```luau
--- Gets guild bans
get_guild_bans: (self: DiscordExecutor, data: GetGuildBansOptions) -> promise.LuaPromise<LazyBanObjectList>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[GetGuildBansOptions](#GetGuildBansOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyBanObjectList](#LazyBanObjectList)&gt;<div id="get_guild_ban"></div>

### get_guild_ban

Gets a guild ban for a user or nil if it does not exist

<details>
<summary>Function Signature</summary>

```luau
--- Gets a guild ban for a user or nil if it does not exist
get_guild_ban: (self: DiscordExecutor, user_id: discord.Snowflake) -> promise.LuaPromise<LazyBanOptionalObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="user_id"></div>

##### user_id

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyBanOptionalObject](#LazyBanOptionalObject)&gt;<div id="create_guild_ban"></div>

### create_guild_ban

Creates a guild ban

<details>
<summary>Function Signature</summary>

```luau
--- Creates a guild ban
create_guild_ban: (self: DiscordExecutor, data: CreateGuildBanOptions) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateGuildBanOptions](#CreateGuildBanOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="remove_guild_ban"></div>

### remove_guild_ban

Removes a guild ban

<details>
<summary>Function Signature</summary>

```luau
--- Removes a guild ban
remove_guild_ban: (self: DiscordExecutor, data: RemoveGuildBanOptions) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[RemoveGuildBanOptions](#RemoveGuildBanOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="get_guild_roles"></div>

### get_guild_roles

Returns the guild roles of a guild

<details>
<summary>Function Signature</summary>

```luau
--- Returns the guild roles of a guild
get_guild_roles: (self: DiscordExecutor, guild_id: discord.Snowflake) -> promise.LuaPromise<LazyRolesObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="guild_id"></div>

##### guild_id

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyRolesObject](#LazyRolesObject)&gt;<div id="get_guild_role"></div>

### get_guild_role

Returns a guild role by ID

<details>
<summary>Function Signature</summary>

```luau
--- Returns a guild role by ID
get_guild_role: (self: DiscordExecutor, role_id: discord.Snowflake) -> promise.LuaPromise<LazyRoleObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="role_id"></div>

##### role_id

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyRoleObject](#LazyRoleObject)&gt;<div id="create_guild_role"></div>

### create_guild_role

Creates a guild role

<details>
<summary>Function Signature</summary>

```luau
--- Creates a guild role
create_guild_role: (self: DiscordExecutor, data: CreateGuildRoleOptions) -> promise.LuaPromise<LazyRoleObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateGuildRoleOptions](#CreateGuildRoleOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyRoleObject](#LazyRoleObject)&gt;<div id="modify_guild_role_positions"></div>

### modify_guild_role_positions

Modify guild role positions

<details>
<summary>Function Signature</summary>

```luau
--- Modify guild role positions
modify_guild_role_positions: (self: DiscordExecutor, data: ModifyRolePositionOptions) -> promise.LuaPromise<LazyRolesObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[ModifyRolePositionOptions](#ModifyRolePositionOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyRolesObject](#LazyRolesObject)&gt;<div id="get_channel_messages"></div>

### get_channel_messages

Messages

Gets messages from a channel

<details>
<summary>Function Signature</summary>

```luau
-- Messages
--- Gets messages from a channel
get_channel_messages: (self: DiscordExecutor, data: GetMessagesOptions) -> promise.LuaPromise<LazyMessagesObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[GetMessagesOptions](#GetMessagesOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyMessagesObject](#LazyMessagesObject)&gt;<div id="get_channel_message"></div>

### get_channel_message

Gets a message

<details>
<summary>Function Signature</summary>

```luau
--- Gets a message
get_channel_message: (self: DiscordExecutor, data: GetMessageOptions) -> promise.LuaPromise<LazyMessageObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[GetMessageOptions](#GetMessageOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyMessageObject](#LazyMessageObject)&gt;<div id="create_message"></div>

### create_message

Creates a message

<details>
<summary>Function Signature</summary>

```luau
--- Creates a message
create_message: (self: DiscordExecutor, data: CreateMessageOptions) -> promise.LuaPromise<LazyMessageObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateMessageOptions](#CreateMessageOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyMessageObject](#LazyMessageObject)&gt;<div id="create_interaction_response"></div>

### create_interaction_response

Uncategorized (for now)

Creates an interaction response

<details>
<summary>Function Signature</summary>

```luau
-- Uncategorized (for now)
--- Creates an interaction response
create_interaction_response: (self: DiscordExecutor, data: CreateInteractionResponseOptions) -> promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateInteractionResponseOptions](#CreateInteractionResponseOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="create_followup_message"></div>

### create_followup_message

Creates a followup interaction response

<details>
<summary>Function Signature</summary>

```luau
--- Creates a followup interaction response
create_followup_message: (self: DiscordExecutor, data: CreateFollowupMessageOptions) -> promise.LuaPromise<LazyMessageObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateFollowupMessageOptions](#CreateFollowupMessageOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyMessageObject](#LazyMessageObject)&gt;<div id="get_original_interaction_response"></div>

### get_original_interaction_response

Gets the original interaction response

<details>
<summary>Function Signature</summary>

```luau
--- Gets the original interaction response
get_original_interaction_response: (self: DiscordExecutor, interaction_token: string) -> promise.LuaPromise<LazyMessageObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="interaction_token"></div>

##### interaction_token

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyMessageObject](#LazyMessageObject)&gt;<div id="get_guild_commands"></div>

### get_guild_commands

Gets all guild commands currently registered

<details>
<summary>Function Signature</summary>

```luau
--- Gets all guild commands currently registered
get_guild_commands: (self: DiscordExecutor) -> promise.LuaPromise<LazyApplicationCommandObject>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyApplicationCommandObject](#LazyApplicationCommandObject)&gt;<div id="create_guild_command"></div>

### create_guild_command

Creates a guild command

<details>
<summary>Function Signature</summary>

```luau
--- Creates a guild command
create_guild_command: (self: DiscordExecutor, data: CreateCommandOptions) -> promise.LuaPromise<LazyApplicationCommandObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateCommandOptions](#CreateCommandOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyApplicationCommandObject](#LazyApplicationCommandObject)&gt;<div id="create_guild_commands"></div>

### create_guild_commands

Creates multiple guild commands

<details>
<summary>Function Signature</summary>

```luau
--- Creates multiple guild commands
create_guild_commands: (self: DiscordExecutor, data: CreateCommandsOptions) -> promise.LuaPromise<LazyApplicationCommandsObject>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateCommandsOptions](#CreateCommandsOptions)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[LazyApplicationCommandsObject](#LazyApplicationCommandsObject)&gt;<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

<details>
<summary>Function Signature</summary>

```luau
function new(token: Primitives.TemplateContext, scope: ExecutorScope.ExecutorScope?) -> DiscordExecutor end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="token"></div>

### token

[Primitives](./primitives.md).[TemplateContext](./primitives.md#TemplateContext)

<div id="scope"></div>

### scope

*This field is optional and may not be specified*

[ExecutorScope](./executorscope.md).[ExecutorScope](./executorscope.md#ExecutorScope)?

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[DiscordExecutor](#DiscordExecutor)