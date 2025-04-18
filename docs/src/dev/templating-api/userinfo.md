<div id="@antiraid/userinfo"></div>

# @antiraid/userinfo

<div id="Types"></div>

## Types

<div id="UserInfo"></div>

## UserInfo

@class UserInfo



Represents a summary of a users permission related

information on AntiRaid





<details>
<summary>Raw Type</summary>

```luau
--- @class UserInfo
---
--- Represents a summary of a users permission related 
--- information on AntiRaid
---
--- @field discord_permissions discord.Snowflake The users discord permissions
--- @field kittycat_staff_permissions Kittycat.StaffPermissions The users kittycat staff permissions
--- @field kittycat_resolved_permissions {Kittycat.Permission} The users resolved kittycat permissions
--- @field guild_owner_id discord.Snowflake The ID of the guild owner
--- @field guild_roles {[discord.Snowflake]: discord.GuildRoleObject} The users guild roles
--- @field member_roles {discord.Snowflake} The users member roles
---
type UserInfo = {
	discord_permissions: discord.Snowflake,

	kittycat_staff_permissions: Kittycat.StaffPermissions,

	kittycat_resolved_permissions: {Kittycat.Permission},

	guild_owner_id: discord.Snowflake,

	guild_roles: {
		[discord.Snowflake]: discord.GuildRoleObject
	},

	member_roles: {discord.Snowflake}
}
```

</details>

<div id="discord_permissions"></div>

### discord_permissions

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="kittycat_staff_permissions"></div>

### kittycat_staff_permissions

[Kittycat](./kittycat.md).[StaffPermissions](./kittycat.md#StaffPermissions)

<div id="kittycat_resolved_permissions"></div>

### kittycat_resolved_permissions

{[Kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)}

<div id="guild_owner_id"></div>

### guild_owner_id

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="guild_roles"></div>

### guild_roles

*This is an inline table type with the following fields*

<div id="[discord.Snowflake]"></div>

##### [discord.Snowflake]

[discord](./discord.md).[GuildRoleObject](./discord.md#GuildRoleObject)

<div id="member_roles"></div>

### member_roles

{[discord](./discord.md).[Snowflake](./discord.md#Snowflake)}

<div id="UserInfoExecutor"></div>

## UserInfoExecutor

@class UserInfoExecutor



Allows templates to get permission-related information about a user



<details>
<summary>Raw Type</summary>

```luau
--- @class UserInfoExecutor
---
--- Allows templates to get permission-related information about a user
---
--- @field get (user_id: discord.Snowflake): Promise.LuaPromise<UserInfo> Gets the UserInfo for a user
type UserInfoExecutor = {
	--- Gets the UserInfo for a user
	--- @param user_id discord.Snowflake The ID of the user to get the UserInfo for
	get: (self: UserInfoExecutor, user_id: discord.Snowflake) -> Promise.LuaPromise<UserInfo>
}
```

</details>

<div id="get"></div>

### get

Gets the UserInfo for a user

<details>
<summary>Function Signature</summary>

```luau
--- Gets the UserInfo for a user
--- @param user_id discord.Snowflake The ID of the user to get the UserInfo for
get: (self: UserInfoExecutor, user_id: discord.Snowflake) -> Promise.LuaPromise<UserInfo>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="user_id"></div>

##### user_id

discord.Snowflake The ID of the user to get the UserInfo for

[discord](./discord.md).[Snowflake](./discord.md#Snowflake)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[UserInfo](#UserInfo)&gt;<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

<details>
<summary>Function Signature</summary>

```luau
function new(token: Primitives.TemplateContext, scope: ExecutorScope.ExecutorScope?) -> UserInfoExecutor end
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

[UserInfoExecutor](#UserInfoExecutor)