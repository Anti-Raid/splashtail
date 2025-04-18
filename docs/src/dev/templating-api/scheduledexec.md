<div id="@antiraid/scheduledexec"></div>

# @antiraid/scheduledexec

<div id="Types"></div>

## Types

<div id="ScheduledExecution"></div>

## ScheduledExecution

Represents a scheduled execution

<details>
<summary>Raw Type</summary>

```luau
--- Represents a scheduled execution
type ScheduledExecution = {
	id: string,

	data: any,

	template_name: string,

	--- String representation of the time the event is scheduled to run at
	---
	--- This can be converted to an ``@antiraid/datetime`` using Timezone:fromString
	run_at: string
}
```

</details>

<div id="id"></div>

### id

[string](#string)

<div id="data"></div>

### data

[any](#any)

<div id="template_name"></div>

### template_name

[string](#string)

<div id="run_at"></div>

### run_at

String representation of the time the event is scheduled to run at



This can be converted to an ``@antiraid/datetime`` using Timezone:fromString

[string](#string)

<div id="ScheduledExecutionList"></div>

## ScheduledExecutionList

<details>
<summary>Raw Type</summary>

```luau
type ScheduledExecutionList = {ScheduledExecution}
```

</details>

{[ScheduledExecution](#ScheduledExecution)}

<div id="CreateScheduledExecution"></div>

## CreateScheduledExecution

<details>
<summary>Raw Type</summary>

```luau
type CreateScheduledExecution = {
	id: string,

	data: any,

	--- String representation of the time the event is scheduled to run at
	---
	--- This can be converted from an ``@antiraid/datetime`` using tostring
	run_at: string
}
```

</details>

<div id="id"></div>

### id

[string](#string)

<div id="data"></div>

### data

[any](#any)

<div id="run_at"></div>

### run_at

String representation of the time the event is scheduled to run at



This can be converted from an ``@antiraid/datetime`` using tostring

[string](#string)

<div id="ScheduledExecExecutor"></div>

## ScheduledExecExecutor

Allows templates to schedule executions of themselves with a specific id and data sometime

in the future



A scheduled execution is a task that is executed once at (or after) a specific time or interval.

When a scheduled execution errors, the caller should *not* remove the scheduled execution

and should instead retry it at the next interval.



All scheduled executions have an ID and data associated with them for use by the caller.

<details>
<summary>Raw Type</summary>

```luau
--- Allows templates to schedule executions of themselves with a specific id and data sometime
--- in the future
---
--- A scheduled execution is a task that is executed once at (or after) a specific time or interval.
--- When a scheduled execution errors, the caller should *not* remove the scheduled execution
--- and should instead retry it at the next interval.
---
--- All scheduled executions have an ID and data associated with them for use by the caller.
type ScheduledExecExecutor = {
	--- Lists all scheduled executions. If ``id`` is provided, then all scheduled executions
	--- with that ID will be returned. See ``create`` for more information on same ID caveats.
	list: (self: ScheduledExecExecutor, id: string?) -> Promise.LuaPromise<ScheduledExecutionList>,

	--- Creates a new scheduled execution
	---
	--- Note that scheduled executions are not guaranteed to be run at the exact time specified and
	--- may be slightly late. 
	---
	--- Note that reusing the same ID may store both scheduled executions and will dispatch an 
	--- set of created scheduled executions with the same ID. While the number of elements in this set is 
	--- undefined, the set is guaranteed to not be the empty/null set
	---
	--- The ordering in which scheduled executions are triggered about a small time period is undefined.
	--- It is recommended to use the ``task`` library for small time periods.
	add: (self: ScheduledExecExecutor, data: CreateScheduledExecution) -> Promise.LuaPromise<nil>,

	--- Deletes all scheduled execution with a given ID
	remove: (self: ScheduledExecExecutor, id: string) -> Promise.LuaPromise<nil>
}
```

</details>

<div id="list"></div>

### list

Lists all scheduled executions. If ``id`` is provided, then all scheduled executions

with that ID will be returned. See ``create`` for more information on same ID caveats.

<details>
<summary>Function Signature</summary>

```luau
--- Lists all scheduled executions. If ``id`` is provided, then all scheduled executions
--- with that ID will be returned. See ``create`` for more information on same ID caveats.
list: (self: ScheduledExecExecutor, id: string?) -> Promise.LuaPromise<ScheduledExecutionList>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="id"></div>

##### id

*This field is optional and may not be specified*

[string](#string)?

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[ScheduledExecutionList](#ScheduledExecutionList)&gt;<div id="add"></div>

### add

Creates a new scheduled execution



Note that scheduled executions are not guaranteed to be run at the exact time specified and

may be slightly late.



Note that reusing the same ID may store both scheduled executions and will dispatch an

set of created scheduled executions with the same ID. While the number of elements in this set is

undefined, the set is guaranteed to not be the empty/null set



The ordering in which scheduled executions are triggered about a small time period is undefined.

It is recommended to use the ``task`` library for small time periods.

<details>
<summary>Function Signature</summary>

```luau
--- Creates a new scheduled execution
---
--- Note that scheduled executions are not guaranteed to be run at the exact time specified and
--- may be slightly late. 
---
--- Note that reusing the same ID may store both scheduled executions and will dispatch an 
--- set of created scheduled executions with the same ID. While the number of elements in this set is 
--- undefined, the set is guaranteed to not be the empty/null set
---
--- The ordering in which scheduled executions are triggered about a small time period is undefined.
--- It is recommended to use the ``task`` library for small time periods.
add: (self: ScheduledExecExecutor, data: CreateScheduledExecution) -> Promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="data"></div>

##### data

[CreateScheduledExecution](#CreateScheduledExecution)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="remove"></div>

### remove

Deletes all scheduled execution with a given ID

<details>
<summary>Function Signature</summary>

```luau
--- Deletes all scheduled execution with a given ID
remove: (self: ScheduledExecExecutor, id: string) -> Promise.LuaPromise<nil>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="id"></div>

##### id

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[nil](#nil)&gt;<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

<details>
<summary>Function Signature</summary>

```luau
function new(token: Primitives.TemplateContext) -> ScheduledExecExecutor end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="token"></div>

### token

[Primitives](./primitives.md).[TemplateContext](./primitives.md#TemplateContext)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[ScheduledExecExecutor](#ScheduledExecExecutor)