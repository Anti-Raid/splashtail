<div id="@antiraid/typesext"></div>

# @antiraid/typesext

<div id="Types"></div>

## Types

<div id="MultiOption"></div>

## MultiOption

A multi option is either T (Some(Some(T)), a empty table (Some(None)), or nil (None)

<details>
<summary>Raw Type</summary>

```luau
--- A multi option is either T (Some(Some(T)), a empty table (Some(None)), or nil (None)
type MultiOption<T> = T | {} | nil
```

</details>

Union with variants:

<details>
<summary>Variant 1</summary>

[T](#T)

</details>

<details>
<summary>Variant 2</summary>

*This is an inline table type with the following fields*

</details>

<details>
<summary>Variant 3</summary>

[nil](#nil)

</details>

<div id="U64Convertibles"></div>

## U64Convertibles

<details>
<summary>Raw Type</summary>

```luau
type U64Convertibles = U64 | string | number
```

</details>

Union with variants:

<details>
<summary>Variant 1</summary>

[U64](#U64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="U64"></div>

## U64

A U64 is a 64 bit unsigned integer. A U64 can be implicitly created from a number or a string



Note: A U64

<details>
<summary>Raw Type</summary>

```luau
--- A U64 is a 64 bit unsigned integer. A U64 can be implicitly created from a number or a string
---
--- Note: A U64
type U64 = {
	to_ne_bytes: (self: U64) -> {number},

	from_ne_bytes: (self: U64, bytes: {number}) -> U64,

	to_le_bytes: (self: U64) -> {number},

	from_le_bytes: (self: U64, bytes: {number}) -> U64,

	to_be_bytes: (self: U64) -> {number},

	from_be_bytes: (self: U64, bytes: {number}) -> U64,

	to_i64: (self: U64) -> I64,

	-- Metatable
	__add: (self: U64, other: U64Convertibles) -> U64,
	__sub: (self: U64, other: U64Convertibles) -> U64,
	__mul: (self: U64, other: U64Convertibles) -> U64,
	__div: (self: U64, other: U64Convertibles) -> U64,
	-- This is the same as IDiv right now
	__mod: (self: U64, other: number) -> U64,
	__pow: (self: U64, other: U64Convertibles) -> U64,
	__idiv: (self: U64, other: U64Convertibles) -> U64,
	-- This is the same as Div right now
	__eq: (self: U64, other: U64Convertibles) -> boolean,
	__lt: (self: U64, other: U64Convertibles) -> boolean,
	__le: (self: U64, other: U64Convertibles) -> boolean,
	__tostring: (self: U64) -> string,
	__type: "U64"
}
```

</details>

<div id="to_ne_bytes"></div>

### to_ne_bytes

<details>
<summary>Function Signature</summary>

```luau
to_ne_bytes: (self: U64) -> {number}
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

{[number](#number)}<div id="from_ne_bytes"></div>

### from_ne_bytes

<details>
<summary>Function Signature</summary>

```luau
from_ne_bytes: (self: U64, bytes: {number}) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="bytes"></div>

##### bytes

{[number](#number)}

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="to_le_bytes"></div>

### to_le_bytes

<details>
<summary>Function Signature</summary>

```luau
to_le_bytes: (self: U64) -> {number}
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

{[number](#number)}<div id="from_le_bytes"></div>

### from_le_bytes

<details>
<summary>Function Signature</summary>

```luau
from_le_bytes: (self: U64, bytes: {number}) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="bytes"></div>

##### bytes

{[number](#number)}

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="to_be_bytes"></div>

### to_be_bytes

<details>
<summary>Function Signature</summary>

```luau
to_be_bytes: (self: U64) -> {number}
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

{[number](#number)}<div id="from_be_bytes"></div>

### from_be_bytes

<details>
<summary>Function Signature</summary>

```luau
from_be_bytes: (self: U64, bytes: {number}) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="bytes"></div>

##### bytes

{[number](#number)}

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="to_i64"></div>

### to_i64

<details>
<summary>Function Signature</summary>

```luau
to_i64: (self: U64) -> I64
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="MetatableFields"></div>

### Metatable Fields

<div id="__add"></div>

#### __add

<details>
<summary>Function Signature</summary>

```luau
__add: (self: U64, other: U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="__sub"></div>

#### __sub

<details>
<summary>Function Signature</summary>

```luau
__sub: (self: U64, other: U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="__mul"></div>

#### __mul

<details>
<summary>Function Signature</summary>

```luau
__mul: (self: U64, other: U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="__div"></div>

#### __div

<details>
<summary>Function Signature</summary>

```luau
__div: (self: U64, other: U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="__mod"></div>

#### __mod

This is the same as IDiv right now

<details>
<summary>Function Signature</summary>

```luau
-- This is the same as IDiv right now
__mod: (self: U64, other: number) -> U64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[number](#number)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="__pow"></div>

#### __pow

<details>
<summary>Function Signature</summary>

```luau
__pow: (self: U64, other: U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="__idiv"></div>

#### __idiv

<details>
<summary>Function Signature</summary>

```luau
__idiv: (self: U64, other: U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="__eq"></div>

#### __eq

This is the same as Div right now

<details>
<summary>Function Signature</summary>

```luau
-- This is the same as Div right now
__eq: (self: U64, other: U64Convertibles) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__lt"></div>

#### __lt

<details>
<summary>Function Signature</summary>

```luau
__lt: (self: U64, other: U64Convertibles) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__le"></div>

#### __le

<details>
<summary>Function Signature</summary>

```luau
__le: (self: U64, other: U64Convertibles) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__tostring"></div>

#### __tostring

<details>
<summary>Function Signature</summary>

```luau
__tostring: (self: U64) -> string
```

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[string](#string)<div id="__type"></div>

#### __type

```luau
"U64"
```

<div id="I64Convertibles"></div>

## I64Convertibles

<details>
<summary>Raw Type</summary>

```luau
type I64Convertibles = U64 | I64 | string | number
```

</details>

Union with variants:

<details>
<summary>Variant 1</summary>

[U64](#U64)

</details>

<details>
<summary>Variant 2</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 3</summary>

[string](#string)

</details>

<details>
<summary>Variant 4</summary>

[number](#number)

</details>

<div id="I64"></div>

## I64

<details>
<summary>Raw Type</summary>

```luau
type I64 = {
	to_ne_bytes: (self: I64) -> {number},

	from_ne_bytes: (self: I64, bytes: {number}) -> I64,

	to_le_bytes: (self: I64) -> {number},

	from_le_bytes: (self: I64, bytes: {number}) -> I64,

	to_be_bytes: (self: I64) -> {number},

	from_be_bytes: (self: I64, bytes: {number}) -> I64,

	to_u64: (self: I64) -> U64,

	-- Metatable
	__add: (self: I64, other: I64 | string | number) -> I64,
	__sub: (self: I64, other: I64 | string | number) -> I64,
	__mul: (self: I64, other: I64 | string | number) -> I64,
	__div: (self: I64, other: I64 | string | number) -> I64,
	__mod: (self: I64, other: number) -> I64,
	__pow: (self: I64, other: I64 | string | number) -> I64,
	__idiv: (self: I64, other: I64 | string | number) -> I64,
	__eq: (self: I64, other: I64 | string | number) -> boolean,
	__lt: (self: I64, other: I64 | string | number) -> boolean,
	__le: (self: I64, other: I64 | string | number) -> boolean,
	__tostring: (self: I64) -> string,
	__type: "I64"
}
```

</details>

<div id="to_ne_bytes"></div>

### to_ne_bytes

<details>
<summary>Function Signature</summary>

```luau
to_ne_bytes: (self: I64) -> {number}
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

{[number](#number)}<div id="from_ne_bytes"></div>

### from_ne_bytes

<details>
<summary>Function Signature</summary>

```luau
from_ne_bytes: (self: I64, bytes: {number}) -> I64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="bytes"></div>

##### bytes

{[number](#number)}

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="to_le_bytes"></div>

### to_le_bytes

<details>
<summary>Function Signature</summary>

```luau
to_le_bytes: (self: I64) -> {number}
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

{[number](#number)}<div id="from_le_bytes"></div>

### from_le_bytes

<details>
<summary>Function Signature</summary>

```luau
from_le_bytes: (self: I64, bytes: {number}) -> I64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="bytes"></div>

##### bytes

{[number](#number)}

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="to_be_bytes"></div>

### to_be_bytes

<details>
<summary>Function Signature</summary>

```luau
to_be_bytes: (self: I64) -> {number}
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

{[number](#number)}<div id="from_be_bytes"></div>

### from_be_bytes

<details>
<summary>Function Signature</summary>

```luau
from_be_bytes: (self: I64, bytes: {number}) -> I64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="bytes"></div>

##### bytes

{[number](#number)}

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="to_u64"></div>

### to_u64

<details>
<summary>Function Signature</summary>

```luau
to_u64: (self: I64) -> U64
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="MetatableFields"></div>

### Metatable Fields

<div id="__add"></div>

#### __add

<details>
<summary>Function Signature</summary>

```luau
__add: (self: I64, other: I64 | string | number) -> I64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="__sub"></div>

#### __sub

<details>
<summary>Function Signature</summary>

```luau
__sub: (self: I64, other: I64 | string | number) -> I64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="__mul"></div>

#### __mul

<details>
<summary>Function Signature</summary>

```luau
__mul: (self: I64, other: I64 | string | number) -> I64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="__div"></div>

#### __div

<details>
<summary>Function Signature</summary>

```luau
__div: (self: I64, other: I64 | string | number) -> I64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="__mod"></div>

#### __mod

<details>
<summary>Function Signature</summary>

```luau
__mod: (self: I64, other: number) -> I64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

[number](#number)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="__pow"></div>

#### __pow

<details>
<summary>Function Signature</summary>

```luau
__pow: (self: I64, other: I64 | string | number) -> I64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="__idiv"></div>

#### __idiv

<details>
<summary>Function Signature</summary>

```luau
__idiv: (self: I64, other: I64 | string | number) -> I64
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[I64](#I64)<div id="__eq"></div>

#### __eq

<details>
<summary>Function Signature</summary>

```luau
__eq: (self: I64, other: I64 | string | number) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__lt"></div>

#### __lt

<details>
<summary>Function Signature</summary>

```luau
__lt: (self: I64, other: I64 | string | number) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__le"></div>

#### __le

<details>
<summary>Function Signature</summary>

```luau
__le: (self: I64, other: I64 | string | number) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="other"></div>

##### other

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__tostring"></div>

#### __tostring

<details>
<summary>Function Signature</summary>

```luau
__tostring: (self: I64) -> string
```

</details>

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[string](#string)<div id="__type"></div>

#### __type

```luau
"I64"
```

<div id="BitU64"></div>

## BitU64

[[
/// Aim is to implement as many functions from https://luau.org/library#bit32-library as possible but for 64 bit unsigned integers
///
/// arshift, band, bnot, bor, bxor, btest, extract, lrotate, lshift, replace, rrotate, rshift, countlz
/// countrz, byteswap
///
/// Implemented:
/// - band
/// - bnot
/// - bor
/// - bxor
/// - btest
/// - extract
/// - lrotate
/// - lshift
/// - replace
/// - rrotate
/// - rshift
/// - countlz
/// - countrz
/// - byteswap
///
/// Not yet implemented due to spec difficulties:
/// - arshift
]]

<details>
<summary>Raw Type</summary>

```luau
--[[
/// Aim is to implement as many functions from https://luau.org/library#bit32-library as possible but for 64 bit unsigned integers
///
/// arshift, band, bnot, bor, bxor, btest, extract, lrotate, lshift, replace, rrotate, rshift, countlz
/// countrz, byteswap
///
/// Implemented:
/// - band
/// - bnot
/// - bor
/// - bxor
/// - btest
/// - extract
/// - lrotate
/// - lshift
/// - replace
/// - rrotate
/// - rshift
/// - countlz
/// - countrz
/// - byteswap
///
/// Not yet implemented due to spec difficulties:
/// - arshift
]]
type BitU64 = {
	band: (...: ...U64Convertibles) -> U64,

	bnot: (U64Convertibles) -> U64,

	bor: (...: ...U64Convertibles) -> U64,

	bxor: (...: ...U64Convertibles) -> U64,

	btest: (...: ...U64Convertibles) -> boolean,

	--- Untested and might not work yet. f must be less than 64
	extract: (n: U64Convertibles, f: number, w: number?) -> U64,

	lrotate: (n: U64Convertibles, i: number) -> U64,

	lshift: (n: U64Convertibles, i: number) -> U64,

	--- Untested and might not work yet. f must be less than 64
	replace: (n: U64Convertibles, r: number, f: number, w: number?) -> U64,

	rrotate: (n: U64Convertibles, i: number) -> U64,

	rshift: (n: U64Convertibles, i: number) -> U64,

	countlz: (n: U64Convertibles) -> number,

	countrz: (n: U64Convertibles) -> number,

	byteswap: (n: U64Convertibles) -> U64
}
```

</details>

<div id="band"></div>

### band

<details>
<summary>Function Signature</summary>

```luau
band: (...: ...U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="..."></div>

##### ...

...

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="bnot"></div>

### bnot

<details>
<summary>Function Signature</summary>

```luau
bnot: (U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="arg1"></div>

##### arg1

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="bor"></div>

### bor

<details>
<summary>Function Signature</summary>

```luau
bor: (...: ...U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="..."></div>

##### ...

...

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="bxor"></div>

### bxor

<details>
<summary>Function Signature</summary>

```luau
bxor: (...: ...U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="..."></div>

##### ...

...

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="btest"></div>

### btest

<details>
<summary>Function Signature</summary>

```luau
btest: (...: ...U64Convertibles) -> boolean
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="..."></div>

##### ...

...

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="extract"></div>

### extract

Untested and might not work yet. f must be less than 64

<details>
<summary>Function Signature</summary>

```luau
--- Untested and might not work yet. f must be less than 64
extract: (n: U64Convertibles, f: number, w: number?) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="f"></div>

##### f

[number](#number)

<div id="w"></div>

##### w

*This field is optional and may not be specified*

[number](#number)?

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="lrotate"></div>

### lrotate

<details>
<summary>Function Signature</summary>

```luau
lrotate: (n: U64Convertibles, i: number) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="i"></div>

##### i

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="lshift"></div>

### lshift

<details>
<summary>Function Signature</summary>

```luau
lshift: (n: U64Convertibles, i: number) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="i"></div>

##### i

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="replace"></div>

### replace

Untested and might not work yet. f must be less than 64

<details>
<summary>Function Signature</summary>

```luau
--- Untested and might not work yet. f must be less than 64
replace: (n: U64Convertibles, r: number, f: number, w: number?) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="r"></div>

##### r

[number](#number)

<div id="f"></div>

##### f

[number](#number)

<div id="w"></div>

##### w

*This field is optional and may not be specified*

[number](#number)?

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="rrotate"></div>

### rrotate

<details>
<summary>Function Signature</summary>

```luau
rrotate: (n: U64Convertibles, i: number) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="i"></div>

##### i

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="rshift"></div>

### rshift

<details>
<summary>Function Signature</summary>

```luau
rshift: (n: U64Convertibles, i: number) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="i"></div>

##### i

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="countlz"></div>

### countlz

<details>
<summary>Function Signature</summary>

```luau
countlz: (n: U64Convertibles) -> number
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[number](#number)<div id="countrz"></div>

### countrz

<details>
<summary>Function Signature</summary>

```luau
countrz: (n: U64Convertibles) -> number
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[number](#number)<div id="byteswap"></div>

### byteswap

<details>
<summary>Function Signature</summary>

```luau
byteswap: (n: U64Convertibles) -> U64
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="n"></div>

##### n

[U64Convertibles](#U64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[U64](#U64)<div id="Functions"></div>

# Functions

<div id="U64"></div>

## U64

Creates a U64 from a number or string



If nil is passed, 0 is used as default

<details>
<summary>Function Signature</summary>

```luau
--- Creates a U64 from a number or string
---
--- If nil is passed, 0 is used as default
function U64(value: U64Convertibles | nil) -> U64 end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="value"></div>

### value

Union with variants:

<details>
<summary>Variant 1</summary>

[U64Convertibles](#U64Convertibles)

</details>

<details>
<summary>Variant 2</summary>

[nil](#nil)

</details>

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[U64](#U64)<div id="I64"></div>

## I64

Creates a I64 from a number or string



If nil is passed, 0 is used as default

<details>
<summary>Function Signature</summary>

```luau
--- Creates a I64 from a number or string
---
--- If nil is passed, 0 is used as default
function I64(value: I64 | string | number | nil) -> I64 end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="value"></div>

### value

Union with variants:

<details>
<summary>Variant 1</summary>

[I64](#I64)

</details>

<details>
<summary>Variant 2</summary>

[string](#string)

</details>

<details>
<summary>Variant 3</summary>

[number](#number)

</details>

<details>
<summary>Variant 4</summary>

[nil](#nil)

</details>

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[I64](#I64)