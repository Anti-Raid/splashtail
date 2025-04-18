<div id="@antiraid/lazy"></div>

# @antiraid/lazy

<div id="Types"></div>

## Types

<div id="Lazy"></div>

## Lazy

A lazy value is a value that is only serialized when accessed

<details>
<summary>Raw Type</summary>

```luau
--- A lazy value is a value that is only serialized when accessed
type Lazy<T> = {
	--- The inner value. Only serialized when accessed and then cached.
	data: T,

	--- Always returns true
	lazy: boolean
}
```

</details>

<div id="data"></div>

### data

The inner value. Only serialized when accessed and then cached.

[T](#T)

<div id="lazy"></div>

### lazy

Always returns true

[boolean](#boolean)

<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

<details>
<summary>Function Signature</summary>

```luau
function new<T>(value: T) -> Lazy<T> end
```

</details>

<div id="Generics"></div>

## Generics

<div id="T"></div>

### T

This generic is unconstrained and can be any type

<div id="Arguments"></div>

## Arguments

<div id="value"></div>

### value

[T](#T)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[Lazy](#Lazy)&lt;[T](#T)&gt;