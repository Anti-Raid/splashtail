<div id="@antiraid/pages"></div>

# @antiraid/pages

<div id="Types"></div>

## Types

<div id="SettingOperations"></div>

## SettingOperations

Supported setting operations for a setting

<details>
<summary>Raw Type</summary>

```luau
--- Supported setting operations for a setting
type SettingOperations = {
	--- @field boolean Can the setting be viewed?
	view: boolean,

	--- @field boolean Can the setting be created?
	create: boolean,

	--- @field boolean Can the setting be updated?
	update: boolean,

	--- @field boolean Can the setting be deleted?
	delete: boolean
}
```

</details>

<div id="view"></div>

### view

[boolean](#boolean)

<div id="create"></div>

### create

[boolean](#boolean)

<div id="update"></div>

### update

[boolean](#boolean)

<div id="delete"></div>

### delete

[boolean](#boolean)

<div id="ColumnSuggestion"></div>

## ColumnSuggestion

A suggestion for a column. Can either be a static set of suggestions or no suggestions at all.

<details>
<summary>Raw Type</summary>

```luau
--- A suggestion for a column. Can either be a static set of suggestions or no suggestions at all.
type ColumnSuggestion = {
	type: "None"
} | {
	type: "Static",

	suggestions: {string}
}
```

</details>

Union with variants:

<details>
<summary>Variant 1</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

```luau
"None"
```

</details>

<details>
<summary>Variant 2</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

```luau
"Static"
```

<div id="suggestions"></div>

#### suggestions

{[string](#string)}

</details>

<div id="ColumnType"></div>

## ColumnType

The type-specific data about a column

<details>
<summary>Raw Type</summary>

```luau
--- The type-specific data about a column
type ColumnType = {
	type: "Scalar" | "Array",

	inner: "String",

	--- @field number The minimum length of the string
	min_length: number?,

	--- @field number The maximum length of the string
	max_length: number?,

	--- @field {string} The allowed values for the string (will be rendered as either a select menu or otherwise)
	---
	--- If empty, all values are allowed.
	allowed_values: {string},

	--- @field string The kind of string this is. This will be used to determine how the string is rendered client-side.
	--- e.g. textarea, channel, user, role, kittycat-permission, uuid, interval, timestamp etc.
	kind: string
} | {
	type: "Scalar" | "Array",

	inner: "Integer"
} | {
	type: "Scalar" | "Array",

	inner: "Float"
} | {
	type: "Scalar" | "Array",

	inner: "BitFlag",

	--- @field {string} The bitflag values as a hashmap
	values: {
		[string]: number
	}
} | {
	type: "Scalar" | "Array",

	inner: "Boolean"
} | {
	type: "Scalar" | "Array",

	inner: "Json",

	--- @field The maximum size, in bytes, of the JSON object
	max_bytes: number?
}
```

</details>

Union with variants:

<details>
<summary>Variant 1</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"Scalar"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Array"
```

</details>

<div id="inner"></div>

#### inner

```luau
"String"
```

<div id="min_length"></div>

#### min_length

*This field is optional and may not be specified*

[number](#number)?

<div id="max_length"></div>

#### max_length

*This field is optional and may not be specified*

[number](#number)?

<div id="allowed_values"></div>

#### allowed_values



If empty, all values are allowed.

{[string](#string)}

<div id="kind"></div>

#### kind

e.g. textarea, channel, user, role, kittycat-permission, uuid, interval, timestamp etc.

[string](#string)

</details>

<details>
<summary>Variant 2</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"Scalar"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Array"
```

</details>

<div id="inner"></div>

#### inner

```luau
"Integer"
```

</details>

<details>
<summary>Variant 3</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"Scalar"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Array"
```

</details>

<div id="inner"></div>

#### inner

```luau
"Float"
```

</details>

<details>
<summary>Variant 4</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"Scalar"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Array"
```

</details>

<div id="inner"></div>

#### inner

```luau
"BitFlag"
```

<div id="values"></div>

#### values

*This is an inline table type with the following fields*

<div id="[string]"></div>

##### [string]

[number](#number)

</details>

<details>
<summary>Variant 5</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"Scalar"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Array"
```

</details>

<div id="inner"></div>

#### inner

```luau
"Boolean"
```

</details>

<details>
<summary>Variant 6</summary>

*This is an inline table type with the following fields*

<div id="type"></div>

#### type

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"Scalar"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Array"
```

</details>

<div id="inner"></div>

#### inner

```luau
"Json"
```

<div id="max_bytes"></div>

#### max_bytes

*This field is optional and may not be specified*

[number](#number)?

</details>

<div id="OperationType"></div>

## OperationType

<details>
<summary>Raw Type</summary>

```luau
type OperationType = "View" | "Create" | "Update" | "Delete"
```

</details>

Union with variants:

<details>
<summary>Variant 1</summary>

```luau
"View"
```

</details>

<details>
<summary>Variant 2</summary>

```luau
"Create"
```

</details>

<details>
<summary>Variant 3</summary>

```luau
"Update"
```

</details>

<details>
<summary>Variant 4</summary>

```luau
"Delete"
```

</details>

<div id="Column"></div>

## Column

A column in a setting

<details>
<summary>Raw Type</summary>

```luau
--- A column in a setting
type Column = {
	--- @field string The ID of the column
	id: string,

	--- @field string The friendly name of the column
	name: string,

	--- @field string The description of the column
	description: string,

	--- @field ColumnType The type of the column
	column_type: ColumnType,

	--- @field boolean Whether or not the column is nullable
	nullable: boolean,

	--- @field ColumnSuggestion The suggestions for the column
	suggestions: ColumnSuggestion,

	--- @field boolean Whether or not the column is a secret field
	secret: boolean,

	--- @field {string} The operations for which the field should be ignored (essentially, read only)
	---
	--- Semantics are defined by the template.
	ignored_for: {OperationType}
}
```

</details>

<div id="id"></div>

### id

[string](#string)

<div id="name"></div>

### name

[string](#string)

<div id="description"></div>

### description

[string](#string)

<div id="column_type"></div>

### column_type

[ColumnType](#ColumnType)

<div id="nullable"></div>

### nullable

[boolean](#boolean)

<div id="suggestions"></div>

### suggestions

[ColumnSuggestion](#ColumnSuggestion)

<div id="secret"></div>

### secret

[boolean](#boolean)

<div id="ignored_for"></div>

### ignored_for



Semantics are defined by the template.

{[OperationType](#OperationType)}

<div id="Setting"></div>

## Setting

<details>
<summary>Raw Type</summary>

```luau
type Setting = {
	--- @field string The ID of the option
	id: string,

	--- @field string The name of the option
	name: string,

	--- @field string The description of the option
	description: string,

	--- @field string The primary key of the table. This *should* be present in user responses to page settings
	--- as well but this is not guaranteed and must be checked for by the template.
	primary_key: string,

	--- @field string Title template, used for the title of the embed
	title_template: string,

	--- @field {Column} The columns for this option
	columns: {Column},

	--- @field SettingOperations The supported operations for this option
	supported_operations: SettingOperations
}
```

</details>

<div id="id"></div>

### id

[string](#string)

<div id="name"></div>

### name

[string](#string)

<div id="description"></div>

### description

[string](#string)

<div id="primary_key"></div>

### primary_key

as well but this is not guaranteed and must be checked for by the template.

[string](#string)

<div id="title_template"></div>

### title_template

[string](#string)

<div id="columns"></div>

### columns

{[Column](#Column)}

<div id="supported_operations"></div>

### supported_operations

[SettingOperations](#SettingOperations)

<div id="Page"></div>

## Page

<details>
<summary>Raw Type</summary>

```luau
type Page = {
	--- @field string The title of the page.
	title: string,

	--- @field string The description of the page.
	description: string,

	--- @field {Setting} The settings of the page.
	settings: {Setting}
}
```

</details>

<div id="title"></div>

### title

[string](#string)

<div id="description"></div>

### description

[string](#string)

<div id="settings"></div>

### settings

{[Setting](#Setting)}

<div id="PageExecutor"></div>

## PageExecutor

<details>
<summary>Raw Type</summary>

```luau
type PageExecutor = {
	--- @function () -> Page
	--- Returns the current page.
	get: (self: PageExecutor) -> Promise.LuaPromise<Page?>,

	--- @function () -> Promise<void>
	--- Sets a page to be the templates page. This will overwrite any existing page if one exists.
	set: (self: PageExecutor, page: Page) -> Promise.LuaPromise<{}>,

	--- @function () -> Promise<void>
	--- Deletes the templates page. This will not delete the page itself, but will remove it from the server's list of custom pages.
	delete: (self: PageExecutor) -> Promise.LuaPromise<{}>
}
```

</details>

<div id="get"></div>

### get

@function () -> Page

Returns the current page.

<details>
<summary>Function Signature</summary>

```luau
--- @function () -> Page
--- Returns the current page.
get: (self: PageExecutor) -> Promise.LuaPromise<Page?>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;[Page](#Page)?&gt;<div id="set"></div>

### set

@function () -> Promise<void>

Sets a page to be the templates page. This will overwrite any existing page if one exists.

<details>
<summary>Function Signature</summary>

```luau
--- @function () -> Promise<void>
--- Sets a page to be the templates page. This will overwrite any existing page if one exists.
set: (self: PageExecutor, page: Page) -> Promise.LuaPromise<{}>
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="page"></div>

##### page

[Page](#Page)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;{}&gt;<div id="delete"></div>

### delete

@function () -> Promise<void>

Deletes the templates page. This will not delete the page itself, but will remove it from the server's list of custom pages.

<details>
<summary>Function Signature</summary>

```luau
--- @function () -> Promise<void>
--- Deletes the templates page. This will not delete the page itself, but will remove it from the server's list of custom pages.
delete: (self: PageExecutor) -> Promise.LuaPromise<{}>
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[Promise](./promise.md).[LuaPromise](./promise.md#LuaPromise)&lt;{}&gt;<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

Creates a new PageExecutor

<details>
<summary>Function Signature</summary>

```luau
--- Creates a new PageExecutor
function new(token: Primitives.TemplateContext, scope: ExecutorScope.ExecutorScope?) -> PageExecutor end
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

[PageExecutor](#PageExecutor)