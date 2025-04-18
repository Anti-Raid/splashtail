<div id="@antiraid/kv"></div>

# @antiraid/kv

<div id="Types"></div>

## Types

<div id="KvRecord"></div>

## KvRecord

KvRecord represents a key-value record with metadata.

@class KvRecord

<details>
<summary>Raw Type</summary>

```luau
--- KvRecord represents a key-value record with metadata.
---@class KvRecord
type KvRecord = {
	--- The key of the record.
	key: string,

	--- The value of the record. This can be any type, depending on what was stored.
	value: any,

	--- Indicates whether the record exists in the key-value store.
	exists: boolean,

	--- The timestamp when the record was created, in ISO 8601 format (e.g., "2023-10-01T12:00:00Z").
	created_at: string,

	--- The timestamp when the record was last updated, in ISO 8601 format (e.g., "2023-10-01T12:00:00Z").
	last_updated_at: string
}
```

</details>

<div id="key"></div>

### key

The key of the record.

[string](#string)

<div id="value"></div>

### value

The value of the record. This can be any type, depending on what was stored.

[any](#any)

<div id="exists"></div>

### exists

Indicates whether the record exists in the key-value store.

[boolean](#boolean)

<div id="created_at"></div>

### created_at

The timestamp when the record was created, in ISO 8601 format (e.g., "2023-10-01T12:00:00Z").

[string](#string)

<div id="last_updated_at"></div>

### last_updated_at

The timestamp when the record was last updated, in ISO 8601 format (e.g., "2023-10-01T12:00:00Z").

[string](#string)

<div id="KvRecordList"></div>

## KvRecordList

A list of KvRecords

<details>
<summary>Raw Type</summary>

```luau
--- A list of KvRecords
type KvRecordList = {KvRecord}
```

</details>

{[KvRecord](#KvRecord)}

<div id="KvExecutor"></div>

## KvExecutor

KvExecutor allows templates to get, store and find persistent data within a server.

@class KvExecutor

<details>
<summary>Raw Type</summary>

```luau
--- KvExecutor allows templates to get, store and find persistent data within a server.
---@class KvExecutor
type KvExecutor = {
	--- The guild ID the executor will perform key-value operations on.
	guild_id: string,

	--- The originating guild ID (the guild ID of the template itself).
	origin_guild_id: string,

	--- The scope of the executor.
	scope: ExecutorScope.ExecutorScope,

	--- Returns a list of all key-value scopes/
	--- This is only guaranteed to return scopes that actually have at least one key inside it.
	---
	--- Needs the `kv.meta:list_scopes` capability to use
	list_scopes: (self: KvExecutor) -> {string},

	--- Finds matching records in this key-value scope.
	--- @param query string The key to search for. % matches zero or more characters; _ matches a single character. To search anywhere in a string, surround {KEY} with %, e.g. %{KEY}%
	--- @return {KvRecord} The records.
	find: (self: KvExecutor, query: string) -> Promise.LuaPromise<KvRecordList>,

	--- Gets a value from this key-value scope.
	--- @param key string The key of the record.
	--- @return any The value of the record.
	get: (self: KvExecutor, key: string) -> Promise.LuaPromise<any>,

	--- Gets a record from this key-value scope.
	--- @param key string The key of the record.
	--- @return KvRecord The record.
	getrecord: (self: KvExecutor, key: string) -> Promise.LuaPromise<KvRecord>,

	--- Returns all keys present in this key-value scope.
	---
	--- This needs the ``kv.meta:{SCOPE}:keys`` capability where `{SCOPE}` is the name of the scope
	--- being accessed
	keys: (self: KvExecutor) -> Promise.LuaPromise<KvRecordList>,

	--- Sets a record in the key-value store.
	--- @param key string The key of the record.
	--- @param value any The value of the record.
	--- @return KvRecord The record.
	set: (self: KvExecutor, key: string, value: any) -> Promise.LuaPromise<KvRecord>,

	--- Deletes a record from this key-value scope
	--- @param key string The key of the record.
	delete: (self: KvExecutor, key: string) -> Promise.LuaPromise<nil>
}
```

</details>

<div id="list_scopes"></div>

### list_scopes

Returns a list of all key-value scopes/

This is only guaranteed to return scopes that actually have at least one key inside it.



Needs the `kv.meta:list_scopes` capability to use

<details>
<summary>Function Signature</summary>

```luau
--- Returns a list of all key-value scopes/
--- This is only guaranteed to return scopes that actually have at least one key inside it.
---
--- Needs the `kv.meta:list_scopes` capability to use
list_scopes: (self: KvExecutor) -> {string}
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

{[string](#string)}<div id="find"></div>

### find

Finds matching records in this key-value scope.

<details>
<summary>Function Signature</summary>

```luau
--- Finds matching records in this key-value scope.
--- @param query string The key to search for. % matches zero or more characters; _ matches a single character. To search anywhere in a string, surround {KEY} with %, e.g. %{KEY}%
--- @return {KvRecord} The records.
find: (self: KvExecutor, query: string) -> Promise.LuaPromise<KvRecordList>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="query"></div>

##### query

string The key to search for. % matches zero or more characters; _ matches a single character. To search anywhere in a string, surround {KEY} with %, e.g. %{KEY}%

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="{KvRecord}"></div>

##### {KvRecord}

The records.

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[KvRecordList](#KvRecordList)&gt;<div id="get"></div>

### get

Gets a value from this key-value scope.

<details>
<summary>Function Signature</summary>

```luau
--- Gets a value from this key-value scope.
--- @param key string The key of the record.
--- @return any The value of the record.
get: (self: KvExecutor, key: string) -> Promise.LuaPromise<any>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="key"></div>

##### key

string The key of the record.

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="any"></div>

##### any

The value of the record.

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[any](#any)&gt;<div id="getrecord"></div>

### getrecord

Gets a record from this key-value scope.

<details>
<summary>Function Signature</summary>

```luau
--- Gets a record from this key-value scope.
--- @param key string The key of the record.
--- @return KvRecord The record.
getrecord: (self: KvExecutor, key: string) -> Promise.LuaPromise<KvRecord>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="key"></div>

##### key

string The key of the record.

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="KvRecord"></div>

##### KvRecord

The record.

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[KvRecord](#KvRecord)&gt;<div id="keys"></div>

### keys

Returns all keys present in this key-value scope.



This needs the ``kv.meta:{SCOPE}:keys`` capability where `{SCOPE}` is the name of the scope

being accessed

<details>
<summary>Function Signature</summary>

```luau
--- Returns all keys present in this key-value scope.
---
--- This needs the ``kv.meta:{SCOPE}:keys`` capability where `{SCOPE}` is the name of the scope
--- being accessed
keys: (self: KvExecutor) -> Promise.LuaPromise<KvRecordList>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[KvRecordList](#KvRecordList)&gt;<div id="set"></div>

### set

Sets a record in the key-value store.

<details>
<summary>Function Signature</summary>

```luau
--- Sets a record in the key-value store.
--- @param key string The key of the record.
--- @param value any The value of the record.
--- @return KvRecord The record.
set: (self: KvExecutor, key: string, value: any) -> Promise.LuaPromise<KvRecord>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="key"></div>

##### key

string The key of the record.

[string](#string)

<div id="value"></div>

##### value

any The value of the record.

[any](#any)

<div id="Returns"></div>

#### Returns

<div id="KvRecord"></div>

##### KvRecord

The record.

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[KvRecord](#KvRecord)&gt;<div id="delete"></div>

### delete

Deletes a record from this key-value scope

<details>
<summary>Function Signature</summary>

```luau
--- Deletes a record from this key-value scope
--- @param key string The key of the record.
delete: (self: KvExecutor, key: string) -> Promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="key"></div>

##### key

string The key of the record.

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="guild_id"></div>

### guild_id

The guild ID the executor will perform key-value operations on.

[string](#string)

<div id="origin_guild_id"></div>

### origin_guild_id

The originating guild ID (the guild ID of the template itself).

[string](#string)

<div id="scope"></div>

### scope

The scope of the executor.

[ExecutorScope](./executorscope.md).[ExecutorScope](./executorscope.md#ExecutorScope)

<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

Creates a new key-value executor with the desired executor scope and the desired kv scope



KV scopes allow for fast access to getting/setting/listing keys within a scope without needing expensive and potentially

heavily internally ratelimited operations like find() etc. as well as allowing for isolated

get/sets (as operations within one scope do not affect other scopes)



Note that the all keys are scoped. The default scope with the name `unscoped` is used when an explicit named

scope is not provided

<details>
<summary>Function Signature</summary>

```luau
--- Creates a new key-value executor with the desired executor scope and the desired kv scope
---
--- KV scopes allow for fast access to getting/setting/listing keys within a scope without needing expensive and potentially
--- heavily internally ratelimited operations like find() etc. as well as allowing for isolated
--- get/sets (as operations within one scope do not affect other scopes)
---
--- Note that the all keys are scoped. The default scope with the name `unscoped` is used when an explicit named
--- scope is not provided
function new(token: Primitives.TemplateContext, scope: ExecutorScope.ExecutorScope?, kv_scope: string?) -> KvExecutor end
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

<div id="kv_scope"></div>

### kv_scope

*This field is optional and may not be specified*

[string](#string)?

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[KvExecutor](#KvExecutor)