<div id="@antiraid/luau"></div>

# @antiraid/luau

<div id="Types"></div>

## Types

<div id="Chunk"></div>

## Chunk

<details>
<summary>Raw Type</summary>

```luau
type Chunk = {
	--- Requires the ``luau:eval.set_environment`` capability to modify
	environment: {
		[any]: any
	}?,

	--- Requires the ``luau:eval.set_optimization_level`` capability to modify
	optimization_level: number?,

	--- Requires the ``luau:eval.modify_set_code`` capability to modify
	code: string,

	--- Requires the ``luau:eval.set_chunk_name`` capability to modify
	chunk_name: string?,

	--- Requires the ``luau:eval.call`` capability to use. Takes in args and returns the 
	--- returned values from the ``code`` being evaluated.
	call: (self: Chunk, args: any) -> any,

	--- Requires the ``luau:eval.call_async`` capability to use. Takes in args and returns the
	--- returned values from the ``code`` being evaluated.
	---
	--- This runs the code asynchronously
	call_async: (self: Chunk, args: any) -> Promise.LuaPromise<any>
}
```

</details>

<div id="call"></div>

### call

Requires the ``luau:eval.call`` capability to use. Takes in args and returns the

returned values from the ``code`` being evaluated.

<details>
<summary>Function Signature</summary>

```luau
--- Requires the ``luau:eval.call`` capability to use. Takes in args and returns the 
--- returned values from the ``code`` being evaluated.
call: (self: Chunk, args: any) -> any
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="args"></div>

##### args

[any](#any)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[any](#any)<div id="call_async"></div>

### call_async

Requires the ``luau:eval.call_async`` capability to use. Takes in args and returns the

returned values from the ``code`` being evaluated.



This runs the code asynchronously

<details>
<summary>Function Signature</summary>

```luau
--- Requires the ``luau:eval.call_async`` capability to use. Takes in args and returns the
--- returned values from the ``code`` being evaluated.
---
--- This runs the code asynchronously
call_async: (self: Chunk, args: any) -> Promise.LuaPromise<any>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="args"></div>

##### args

[any](#any)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[any](#any)&gt;<div id="environment"></div>

### environment

Requires the ``luau:eval.set_environment`` capability to modify

*This field is optional and may not be specified*

{[any]: [any](#any)}?

<div id="optimization_level"></div>

### optimization_level

Requires the ``luau:eval.set_optimization_level`` capability to modify

*This field is optional and may not be specified*

[number](#number)?

<div id="code"></div>

### code

Requires the ``luau:eval.modify_set_code`` capability to modify

[string](#string)

<div id="chunk_name"></div>

### chunk_name

Requires the ``luau:eval.set_chunk_name`` capability to modify

*This field is optional and may not be specified*

[string](#string)?

<div id="Functions"></div>

# Functions

<div id="load"></div>

## load

Requires the ``luau:eval`` capability to use. Be careful as this allows

for arbitrary code execution.

<details>
<summary>Function Signature</summary>

```luau
--- Requires the ``luau:eval`` capability to use. Be careful as this allows
--- for arbitrary code execution.
function load(token: Primitives.TemplateContext, code: string) -> Chunk end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="token"></div>

### token

[Primitives](./primitives.md).[TemplateContext](./primitives.md#TemplateContext)

<div id="code"></div>

### code

[string](#string)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[Chunk](#Chunk)<div id="format"></div>

## format

Formats a set of values to a string

<details>
<summary>Function Signature</summary>

```luau
--- Formats a set of values to a string
function format(...: any) -> string end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="..."></div>

### ...

[any](#any)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[string](#string)