# AntiRaid Plugins

## Builtins

**Module Name:** ``@antiraid/builtins``

Provides some basic builtins for AntiRaid.

### Functions

- ``require(module_name: string) -> table<any>``

## Concurrency

**Module Name:** ``@antiraid/concurrency``

Not really useful right now. Will be expanded in the future.

**UNTESTED**

### Functions

- ``select_ok(coros: {coroutine}) -> coroutine``

Creates a new coroutine which will select the first successful coroutine over a list of coroutine. The created coroutine is then executed and the result returned

The returned coroutine will wait for any coroutine within the list to be ready and Ok. Unlike select_all (not yet implemented, todo), this will only return the first successful completion, or the last failure. This is useful in contexts where any success is desired and failures are ignored, unless all the coroutines fail.

## Interop

**Module Name:** ``@antiraid/interop``

Interop between Lua and the Rust core

### Values

- ``null``

While the Lua ``nil`` does work in many cases (and even when calling the SDK), its not the best choice. When querying AntiRaid SDK, the SDK will use the ``@antiraid/interop#null`` value to represent a null value. Lua templates can also use this value if desired.

One advantage of ``null`` vs ``nil`` is that ``null`` can be used to check whether a value is set but is null or is completely unset. ``nil`` can only be used to check if a value is unset. This is important when interfacing with Discord as Discord often has differing semantics between non-existence and existing-but-null such as in Gateway Events.

- ``array_metatable``

To pass arrays to modules within the AntiRaid SDK, you need to set the metatable to ``@antiraid/interop#array_metatable``. This will allow the SDK to convert the array to a Rust ``Vec`` internally.

```lua
local interop = require '@antiraid/interop'
setmetatable({a = 5}, interop.array_metatable)
```

This is required because tables in Lua can represent both a hashmap and an array so the metadata is required to know which to choose,

### Functions

- ``memusage() -> number``

While not strictly useful for interop, it is often desirable to know the memory usage of a Lua template as AntiRaid will kill your template if it exceeds the memory limit. For this, you can use the `@antiraid/interop#memusage` function.

```lua
local interop = require '@antiraid/interop'
print(interop.memusage())
```

This function returns the memory usage of the VM in bytes.

## Message

Functions for creating templated messages

### Functions

- ``new_message() -> table<message.Message>``

Creates a new message table

- ``new_message_embed() -> table<message.MessageEmbed>``

Creates a new message embed table

- ``new_message_embed_field() -> table<message.MessageEmbedField>``

Creates a new message embed field table

- ``format_gwevent_field(field: table<gwevent.field.Field>) -> String``

Formats a gwevent field into a string. These are exposed in places such as Audit Logs and other areas.

### Types

The following rust types are exposed in this module

```rust
/// Represents an embed field
pub struct MessageEmbedField {
    /// The name of the field
    pub name: String,
    /// The value of the field
    pub value: String,
    /// Whether the field is inline
    pub inline: bool,
}

/// Represents a message embed
pub struct MessageEmbed {
    /// The title set by the template
    pub title: Option<String>,
    /// The description set by the template
    pub description: Option<String>,
    /// The fields that were set by the template
    pub fields: Vec<MessageEmbedField>,
}

/// Represents a message that can be created by templates
pub struct Message {
    /// Embeds [current_index, embeds]
    pub embeds: Vec<MessageEmbed>,
    /// What content to set on the message
    pub content: Option<String>,
}
```

## Permissions

**Module Name:** ``@antiraid/permissions``

Provides functions for checking and handling permissions. Internally, this exposes (parts of) the ``kittycat`` and ``rust.permissions`` crates for Lua templating.

### Functions

- ``new_permission_check() -> table<permissions.PermissionCheck>``

Creates a new permission check table

- ``new_permission_checks() -> table<permissions.PermissionChecks>``

Creates a new permission checks table. This is not very useful (theres only two variants: ``Simple`` and ``Template``) but is exposed for the sake of completeness.

- ``new_permission(namespace: string, perm: string, negator: boolean) -> table<kittycat.perms.Permission>``

Creates a new kittycat permission table given the namespace, permission and negator.

- ``new_permission_from_string(perm: string) -> table<kittycat.perms.Permission>``

Given the string form of a kittycat permission, creates a new permission table.

- ``permission_to_string(perm: table<kittycat.perms.Permission>) -> string``

Converts a kittycat permission to its string form.

- ``has_perm(permissions: {table<kittycat.perms.Permission>}, perm: table<kittycat.perms.Permission>) -> boolean``

Checks if a list of permissions 'has' a specific permission. This corresponds to ``kittycat::perms::has_perm(permissions, perm)``.

- ``has_perm_str(permissions: {string}, perm: string) -> boolean``

The string variant of ``has_perm``. This is useful when you have a list of permissions in string form. This corresponds to ``kittycat::perms::has_perm_str(permissions, perm)``.

- ``check_perms_single(check: permissions.PermissionCheck, member_native_perms: serenity.all.Permissions, member_kittycat_perms: {kittycat.perms.Permission}) -> LuaPermissionResult``

Checks a single permission check. This corresponds to ``permissions::check_perms_single(check, member_native_perms, member_kittycat_perms)``.

- ``eval_checks(checks: {permissions.PermissionCheck}, member_native_perms: serenity.all.Permissions, member_kittycat_perms: {kittycat.perms.Permission}) -> LuaPermissionResult``

Checks a set of permission checks. This corresponds to ``permissions::eval_checks(checks, member_native_perms, member_kittycat_perms)``.

### Types

The following rust types are exposed in this module

```rust
pub struct LuaPermissionResult {
    /// The raw result of the permission check
    pub result: PermissionResult,
    /// Whether the permission result represents a success or a failure
    pub is_ok: bool,
    /// The code of the permission result
    pub code: String,
    /// The markdown representation of the permission result
    pub markdown: String,
}

#[serde(tag = "var")]
pub enum PermissionResult {
    Ok {},
    OkWithMessage { message: String },
    MissingKittycatPerms { check: PermissionCheck },
    MissingNativePerms { check: PermissionCheck },
    MissingAnyPerms { check: PermissionCheck },
    CommandDisabled { command: String },
    UnknownModule { module: String },
    ModuleNotFound {},
    ModuleDisabled { module: String },
    NoChecksSucceeded { checks: PermissionChecks },
    DiscordError { error: String },
    SudoNotGranted {},
    GenericError { error: String },
}

pub struct PermissionCheck {
    /// The kittycat permissions needed to run the command
    pub kittycat_perms: Vec<String>,
    /// The native permissions needed to run the command
    pub native_perms: Vec<serenity::all::Permissions>,
    /// Whether the next permission check should be ANDed (all needed) or OR'd (at least one) to the current
    pub outer_and: bool,
    /// Whether or not the perms are ANDed (all needed) or OR'd (at least one)
    pub inner_and: bool,
}

pub enum PermissionChecks {
    Simple {
        /// The list of permission checks
        checks: Vec<PermissionCheck>,
    },
    Template {
        /// The template string to use
        template: String,
    },
}
```
