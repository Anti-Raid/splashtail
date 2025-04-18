<div id="@antiraid/promise"></div>

# @antiraid/promise

<div id="Types"></div>

## Types

<div id="LuaPromise"></div>

## LuaPromise

Opaque promise type returned by antiraid

<details>
<summary>Raw Type</summary>

```luau
--- Opaque promise type returned by antiraid
type LuaPromise<T> = {
	--- Note: this will always actually be nil but is required to enforce nominal typing properly
	__phantom: T
}
```

</details>

<div id="__phantom"></div>

### __phantom

Note: this will always actually be nil but is required to enforce nominal typing properly

[T](#T)

<div id="Functions"></div>

# Functions

<div id="yield"></div>

## yield

Yields the coroutine and resumes it returning the end value of the promise when it resolves

<details>
<summary>Function Signature</summary>

```luau
--- Yields the coroutine and resumes it returning the end value of the promise when it resolves
function yield<T>(promise: LuaPromise<T>) -> T end
```

</details>

<div id="Generics"></div>

## Generics

<div id="T"></div>

### T

This generic is unconstrained and can be any type

<div id="Arguments"></div>

## Arguments

<div id="promise"></div>

### promise

[LuaPromise](#LuaPromise)&lt;[T](#T)&gt;<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[T](#T)