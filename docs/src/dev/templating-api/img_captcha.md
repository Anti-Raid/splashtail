<div id="@antiraid/img_captcha"></div>

# @antiraid/img_captcha

<div id="Types"></div>

## Types

<div id="Geometry"></div>

## Geometry

Represents the area which contains text in a CAPTCHA.

<details>
<summary>Raw Type</summary>

```luau
--- Represents the area which contains text in a CAPTCHA.
type Geometry = {
	--- The minimum x coordinate of the area which contains text (inclusive).
	left: Primitives.u32,

	--- The maximum x coordinate of the area which contains text (inclusive).
	right: Primitives.u32,

	--- The minimum y coordinate of the area which contains text (inclusive).
	top: Primitives.u32,

	--- The maximum y coordinate of the area which contains text (inclusive).
	bottom: Primitives.u32
}
```

</details>

<div id="left"></div>

### left

The minimum x coordinate of the area which contains text (inclusive).

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="right"></div>

### right

The maximum x coordinate of the area which contains text (inclusive).

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="top"></div>

### top

The minimum y coordinate of the area which contains text (inclusive).

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="bottom"></div>

### bottom

The maximum y coordinate of the area which contains text (inclusive).

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="SerdeColor"></div>

## SerdeColor

Represents a color in RGB format.

<details>
<summary>Raw Type</summary>

```luau
--- Represents a color in RGB format.
type SerdeColor = {
	--- The red component of the color.
	r: Primitives.u8,

	--- The green component of the color.
	g: Primitives.u8,

	--- The blue component of the color.
	b: Primitives.u8
}
```

</details>

<div id="r"></div>

### r

The red component of the color.

[Primitives](./primitives.md).[u8](./primitives.md#u8)

<div id="g"></div>

### g

The green component of the color.

[Primitives](./primitives.md).[u8](./primitives.md#u8)

<div id="b"></div>

### b

The blue component of the color.

[Primitives](./primitives.md).[u8](./primitives.md#u8)

<div id="ColorInvertFilter"></div>

## ColorInvertFilter

Filter that inverts the colors of an image.

<details>
<summary>Raw Type</summary>

```luau
--- Filter that inverts the colors of an image.
type ColorInvertFilter = {
	filter: "ColorInvert"
}
```

</details>

<div id="filter"></div>

### filter

```luau
"ColorInvert"
```

<div id="CowFilter"></div>

## CowFilter

Filter that generates a CAPTCHA with a specified number of cows (circles/other curvical noise)

<details>
<summary>Raw Type</summary>

```luau
--- Filter that generates a CAPTCHA with a specified number of cows (circles/other curvical noise)
type CowFilter = {
	filter: "Cow",

	--- The minimum radius of the cows
	min_radius: Primitives.u32,

	--- The maximum radius of the cows
	max_radius: Primitives.u32,

	--- The number of cows to generate
	n: Primitives.u32,

	--- Whether to allow duplicate cows
	allow_duplicates: boolean,

	--- The geometry of the area which contains text
	geometry: Geometry?
}
```

</details>

<div id="filter"></div>

### filter

```luau
"Cow"
```

<div id="min_radius"></div>

### min_radius

The minimum radius of the cows

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="max_radius"></div>

### max_radius

The maximum radius of the cows

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="n"></div>

### n

The number of cows to generate

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="allow_duplicates"></div>

### allow_duplicates

Whether to allow duplicate cows

[boolean](#boolean)

<div id="geometry"></div>

### geometry

The geometry of the area which contains text

*This field is optional and may not be specified*

[Geometry](#Geometry)?

<div id="DotFilter"></div>

## DotFilter

Filter that creates a specified number of dots

<details>
<summary>Raw Type</summary>

```luau
--- Filter that creates a specified number of dots
type DotFilter = {
	filter: "Dot",

	--- The number of dots to generate
	n: Primitives.u32,

	--- The minimum radius of the dots
	min_radius: Primitives.u32,

	--- The maximum radius of the dots
	max_radius: Primitives.u32
}
```

</details>

<div id="filter"></div>

### filter

```luau
"Dot"
```

<div id="n"></div>

### n

The number of dots to generate

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="min_radius"></div>

### min_radius

The minimum radius of the dots

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="max_radius"></div>

### max_radius

The maximum radius of the dots

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="GridFilter"></div>

## GridFilter

Filter that creates a grid (horizontal/vertical lines with a specified gap in X and Y direction)



(think graph paper)

<details>
<summary>Raw Type</summary>

```luau
--- Filter that creates a grid (horizontal/vertical lines with a specified gap in X and Y direction)
---
--- (think graph paper)
type GridFilter = {
	filter: "Grid",

	--- The Y gap between the vertical lines
	y_gap: Primitives.u32,

	--- The X gap between the horizontal lines
	x_gap: Primitives.u32
}
```

</details>

<div id="filter"></div>

### filter

```luau
"Grid"
```

<div id="y_gap"></div>

### y_gap

The Y gap between the vertical lines

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="x_gap"></div>

### x_gap

The X gap between the horizontal lines

[Primitives](./primitives.md).[u32](./primitives.md#u32)

<div id="LineFilter"></div>

## LineFilter

Draw lines/rectangles on the screen

<details>
<summary>Raw Type</summary>

```luau
--- Draw lines/rectangles on the screen
type LineFilter = {
	filter: "Line",

	--- Point 1, must be an array of two numbers
	p1: {Primitives.f32},

	--- Point 2, must be an array of two numbers
	p2: {Primitives.f32},

	--- The thickness of the line
	thickness: Primitives.f32,

	--- The color of the line
	color: SerdeColor
}
```

</details>

<div id="filter"></div>

### filter

```luau
"Line"
```

<div id="p1"></div>

### p1

Point 1, must be an array of two numbers

{[Primitives](./primitives.md).[f32](./primitives.md#f32)}

<div id="p2"></div>

### p2

Point 2, must be an array of two numbers

{[Primitives](./primitives.md).[f32](./primitives.md#f32)}

<div id="thickness"></div>

### thickness

The thickness of the line

[Primitives](./primitives.md).[f32](./primitives.md#f32)

<div id="color"></div>

### color

The color of the line

[SerdeColor](#SerdeColor)

<div id="NoiseFilter"></div>

## NoiseFilter

Adds some random noise at a specified probability

<details>
<summary>Raw Type</summary>

```luau
--- Adds some random noise at a specified probability
type NoiseFilter = {
	filter: "Noise",

	--- The probability of adding noise
	prob: Primitives.f32
}
```

</details>

<div id="filter"></div>

### filter

```luau
"Noise"
```

<div id="prob"></div>

### prob

The probability of adding noise

[Primitives](./primitives.md).[f32](./primitives.md#f32)

<div id="RandomLineFilter"></div>

## RandomLineFilter

Creates a random line somewhere

<details>
<summary>Raw Type</summary>

```luau
--- Creates a random line somewhere
type RandomLineFilter = {
	filter: "RandomLine"
}
```

</details>

<div id="filter"></div>

### filter

```luau
"RandomLine"
```

<div id="WaveFilter"></div>

## WaveFilter

Creates a wave in a given direction (horizontal/vertical)

<details>
<summary>Raw Type</summary>

```luau
--- Creates a wave in a given direction (horizontal/vertical)
type WaveFilter = {
	filter: "Wave",

	--- The frequency of the wave
	f: Primitives.f32,

	--- The amplitude of the wave
	amp: Primitives.f32,

	--- The direction of the wave
	d: "horizontal" | "vertical"
}
```

</details>

<div id="filter"></div>

### filter

```luau
"Wave"
```

<div id="f"></div>

### f

The frequency of the wave

[Primitives](./primitives.md).[f32](./primitives.md#f32)

<div id="amp"></div>

### amp

The amplitude of the wave

[Primitives](./primitives.md).[f32](./primitives.md#f32)

<div id="d"></div>

### d

The direction of the wave

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"horizontal"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"vertical"
```

</details>

<div id="Filter"></div>

## Filter

Represents a filter that can be applied to an image.

<details>
<summary>Raw Type</summary>

```luau
--- Represents a filter that can be applied to an image.
type Filter = ColorInvertFilter | CowFilter | DotFilter | GridFilter | LineFilter | NoiseFilter | RandomLineFilter | WaveFilter
```

</details>

Union with variants:

<details>
<summary>Variant 1</summary>

[ColorInvertFilter](#ColorInvertFilter)

</details>

<details>
<summary>Variant 2</summary>

[CowFilter](#CowFilter)

</details>

<details>
<summary>Variant 3</summary>

[DotFilter](#DotFilter)

</details>

<details>
<summary>Variant 4</summary>

[GridFilter](#GridFilter)

</details>

<details>
<summary>Variant 5</summary>

[LineFilter](#LineFilter)

</details>

<details>
<summary>Variant 6</summary>

[NoiseFilter](#NoiseFilter)

</details>

<details>
<summary>Variant 7</summary>

[RandomLineFilter](#RandomLineFilter)

</details>

<details>
<summary>Variant 8</summary>

[WaveFilter](#WaveFilter)

</details>

<div id="CaptchaConfig"></div>

## CaptchaConfig

Captcha configuration

<details>
<summary>Raw Type</summary>

```luau
--- Captcha configuration
type CaptchaConfig = {
	--- The number of characters the CAPTCHA should have.
	char_count: Primitives.u8,

	--- The list of filters
	filters: {Filter},

	--- The size of the viewbox to render the CAPTCHA in.
	--- (first element is width, second element is height)
	viewbox_size: {Primitives.u32},

	--- At what index of CAPTCHA generation should a viewbox be created at.
	set_viewbox_at_idx: Primitives.u8
}
```

</details>

<div id="char_count"></div>

### char_count

The number of characters the CAPTCHA should have.

[Primitives](./primitives.md).[u8](./primitives.md#u8)

<div id="filters"></div>

### filters

The list of filters

{[Filter](#Filter)}

<div id="viewbox_size"></div>

### viewbox_size

The size of the viewbox to render the CAPTCHA in.

(first element is width, second element is height)

{[Primitives](./primitives.md).[u32](./primitives.md#u32)}

<div id="set_viewbox_at_idx"></div>

### set_viewbox_at_idx

At what index of CAPTCHA generation should a viewbox be created at.

[Primitives](./primitives.md).[u8](./primitives.md#u8)

<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

Creates a new CAPTCHA with the given configuration.

<details>
<summary>Function Signature</summary>

```luau
--- Creates a new CAPTCHA with the given configuration.
function new(config: CaptchaConfig) -> promise.LuaPromise<{Primitives.u8}> end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="config"></div>

### config

[CaptchaConfig](#CaptchaConfig)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;{[Primitives](./primitives.md).[u8](./primitives.md#u8)}&gt;