# Friday, August 2nd, 2024

## Settings

- Replaced guild_id with common_filters/default_common_filters in settings. This allows for global config options to properly be supported without requiring a ``published_from`` extra table. This also opens the door for more advanced use-cases internally in the future.

# Thursday, July 25th 2024

## Permissions

- Refactored templating a bit

# Wednesday, July 24th 2024

## Permissions

- Removed ``PermissionResult::MissingMinChecks``
- Removed ``checks_needed`` from ``PermissionChecks`` in favor of just using templates
- Added ``PermissionChecks::Template`` to allow for more advanced permission checks. This is a work in progress and may not be fully functional yet.

# Tuesday, July 23rd 2024

## Settings

- Refactored out the database code into two traits. The first, ``CreateDataStore`` has one function ``create`` which is passed all the common arguments/state. The second, ``DataStore`` includes functions for fetching/creating/updating/deleting entries.
- Refactored the ``_validate_and_parse_value`` function into two seperate functions ``_parse_value`` and ``_validate_value``. This avoids performing useless operations and helps improve maintenance and readability of the codebase at the cost of potential performance.

## GWEvent

- Flatten ``Field``s and tag it with the value stored in ``value``. This helps improve user experience in templates.

## Settings (module)

- Moved guild roles to settings. This also removes the (now useless) ``perms list`` and ``perms modrole`` commands.

# Thursday, July 18th 2024

## Settings

- Added ``allowed_types`` and ``needed_bot_permissions`` to StringKind::Channel to allow for more granular control over the type of channel that can be selected complete with parsing and unified permission checks.

# Wednesday, July 17th 2024

## GWEvents

- Begun working on CI for automatically generating the template docs and data for gateway events. This will, in the future, allow for interactive documentation of gateway events and their fields in the website

## Settings

- Added ``TemplateKind`` to ``StringKind::Template`` to allow displaying message template builder in the site

## Website

- Added support for displaying message template builder in site settings. This is a work in progress and may not be fully functional yet.

# Tuesday, July 16th 2024

## Templating
- Have begun working on new TemplateBuilder component on the website
- Support for multiple embed + message content has been added

# Monday, July 15th 2024

## Settings

- Several ``sqlx`` bugs have been fixed including an error where int4 was decoded to i64 instead of i32 and ``columns()`` being used over ``try_columns()``

## Bot

- Maintenances have been moved to the database officially
- Guild Limits have been moved to settings. Note that other subsystems within limits will be moved later on.

# Sunday, July 14th 2024

## Settings

- The settings animus magic interface now processes fields based on column order instead of field order to allow older browsers to properly handle the data. 
- ``settings_create`` no longer treats ``null`` values as being omitted from validation.
- ``settings_update`` no longer treats ``null`` values as the weird special case where it was omitted from validation yet also not an ``unchanged_field``
- `settings_update` now correctly handles ``ignored_for`` by ensuring that it returns in the output while also not being updated itself. This also fixes a potential bug where secrets may not be updated correctly.

# Saturday, July 13th 2024

## Settings

- Website now supports basic listing of inputs with some reliability although this is still a work in progress
- ``__[column_id]_displaytype`` has been replaced with a proper `Dynamic` column type. This improves website UX and makes it easier to handle different display types for different columns. **This is a breaking change however as dynamic field types must be sent after the `clause` it relies on**

# Friday, July 12th 2024

## Templating

- Templating has been improved and should now handle template timeout across for-loops/if-statements correctly

## Bot

- ``guild_command_configurations`` now stores audit information (`created_at`, `created_by`, `last_updated_at`, `last_updated_by`) for each command configuration. This is useful for auditing purposes and tracking changes to command configurations. Note that to avoid unneeded data retrievals and to preserve backwards compatibility, this audit info has been spun out into ``FullGuildCommandConfiguration``.

## Webserver

- The ``get_all_command_configurations`` API now returns a ``FullGuildCommandConfiguration`` inline with the change to the bot. Similarly, the `patch_command_configuration` API now also returns a `FullGuildCommandConfiguration`. This is needed for the website to properly display audit information for command configurations.
- The ``get_command_configuration`` API has been removed in favor of the ``get_all_command_configurations`` API. This endpoint was redundant anyways.

# Thursday, July 11th 2024

## Audit Logs

- Audit Logs now support template-based embeds. Note that this is still a work in progress and may not be fully functional yet.

# Wednesday, July 10th 2024

## GWEvent

- A ``name`` property was added to field type for template formatting support
- A CI check was added to GWEvent through ``build.rs`` to ensure all fields (events only for now) are expanded and that none are omitted

## Website

- Major bug fixes to website. Specifically, a bug was fixed in which ``getCommandConfigurations`` continued to use a hack that was needed before for older API but is no longer needed and now instead causes crashes when executing a command outside of the first/second modules in the bot.


# Monday, July 8th 2024

## Bot

- Added support for custom templating. The templating engine used is ``tera``. The ``rust.templating`` helper crate has been added to allow unified and consistent handling of templating. In particular, the ``field`` function is a builtin function to custom templating allowing for setting embed fields

# Sunday, July 7th 2024

## Bot

- ``get_best_command_configuration`` now makes use of the parents permissions/disabled status if explicitly set on parent but not explicitly set on the command itself. This means that if ``web`` is disabled explicitly but ``web use`` does not have an explicit override for disabled, it will be disabled as well. This is useful for setting a base state for all commands in a module.
- Simplified the CommandDisabled error to no longer include the useless ``inherited from`` as it is useless and confusing to boot.

## Website

- Improved command extended data logic to properly handle the lack of permission checks etc. This has been achieved through the use of a new API ``(parsedCommands: ParsedCanonicalCommandData[], command: string): CommandExtendedData`` in the website ``commands.ts`` library. All callers should switch their use of ``commands.find()`` to this new API to ensure proper handling of command extended data.
- The ``settings_get_suggestions`` API has been added to the webserver to allow the site to provide column suggestions
- Begun working on the settings section of the site. Currently only column suggestions are supported.

# Wednesday, July 3rd 2024

## Settings

- Made ``ConfigOptions.columns`` an ``Arc<Vec<Column>>`` from ``Vec<Column>`` to make cloning cheaper
- Removed lots of useless clones by making `_query_bind_value` accept a reference versus a value and changing `validate_and_parse_value` to consume the `Value` versus taking a reference to it.
- The settings API no longer guarantees that the state returned will be in any particular order.
- Fixed a bug in which delete checked nullability incorrectly. This is now hardcoded to `false` to better acommodate invalid data deletions
- Fixed a bug where delete called `validate_and_parse_value` with the primary key column name instead of the actual column name being parsed. This led to inproper errors being returned

# Monday, July 1st 2024

## Settings

- Added support for secret fields. Fields marked with ``secret: Some(length)`` will be hidden in ``view`` operations and if null/unset, will be set to a random string of the specified length. This is useful for API keys, webhook secrets etc. that should not be disclosed where possible. 
- Fixed several bugs throughout the codebase. Specifically, internal fields are no longer considered in ``settings_create`` and ``settings_update``. Primary key schema checks are no longer applied on ``settings_delete`` to allow deleting invalid data that may have been added prior to a more stricter schema etc.
- Common fields such as created_at/created_by/last_updated_at/last_updated_by have been added to the ``column_columns.rs`` module of settings.

## Bot

- The ``gitlogs`` module has been moved entirely to settings. In addition, many fields such as created_at/created_by/last_updated_at/last_updated_by have been added to all ``gitlogs`` structures.
- ``auditlogs`` settings schemas have been improved slightly
- Module parsing now also validates the ``CommandExtendedDataMap`` to ensure that all submodules are also present in the map. A test has been added for module parsing to allow testing this without running the bot.

# Sunday, June 30th 2024

- Fully restructuring the bot. Here is the new structure:

=> ``core/{lang}.{module}`` contains all core data
=> ``services/{lang}.{module}`` contains all services

In particular, the rust side has changed significantly. AnimusMagicClient has been refactored using traits to clone less and allow returning an opaque trait object for use in modules. This also allows for other services (such as a premium only bot) to also make use of the same base modules.

- Fields linked to ``mewld`` have been moved to the ``props.statistics()`` function (``Statistics`` struct).
- Cargo workspaces now work properly :D

*Note that build times have increased temporarily as our Makefiles are not very efficient. This should be resolved later*

# Saturday, June 29th 2024

## Webserver

- ``Patch Module Configuration`` and ``Patch Command Configuration`` now returns the updated configuration. This is needed for properly updating the website after a successful update.

# Thursday, June 27th 2024

## Bot

- The bot now runs permission checks for reset toggle on both commands and modules based on the default state of the command/module. In addition, commands/modules must be toggleable to be able to reset toggles.
- Refactored permission checking a bit internally. Users should see no changes beyond potentially improved performance due to using sandwich state more.
- Added `virtual_command` to `CommandExtendedData` struct. Virtual commands, like virtual modules, are not actually loaded into the bot but can be used for permission checks etc.
- The ``acl__{module}_defaultperms_check`` command has been added to better handle default permission checks
- ``custom_module_configuration`` has been added to ``can_run_command``
- The `AmCheckCommandOptions` struct used for permission checks has been changed significantly. It now uses a ``flags`` bitfield (`u8`) instead of 5-6 bool fields to save memory. **These changes have also been ported to the webserver as well**
- Added ``Module.parse`` to allow performing some checks on the modules before starting up the bot in an invalid state
- ``ensure_custom_kittycat_perms`` has been replaced with ``SKIP_CUSTOM_RESOLVED_FIT_CHECKS``. The new API is opt-in hence ensuring that all permission limits are checked by default unless explicitly overriden instead of the less secure opt-out approach of `ensure_custom_kittycat_perms`.
- Several bug fixes including the removal of ``ignore_module/command_disabled`` in several useless places.

## Website

- The ``patch_module_configuration`` API now supports clearing toggles and default permissions of modules. This brings it up-to-speed / inline with the equivalent bot command ``modules modperms``

# Wednesday, June 26th 2024

## Website

- Several website updates. ``commandLookup`` now properly uses ``full_name`` like the rest of the command handling code. Module info editting has also been significantly improved with better TypeScript typing and a more consistent UI. Several other UI/UX changes have also been made including a footer including version information for debugging as well as a new logger that provides more details to developers.

## Bot And Webserver

- Permission checks are now validated through a standardized function: ``silverpelt::validators::parse_permission_checks`` (botv2) and ``webutils.ParsePermissionChecks`` (webserver). These functions provide a consistent and standardized data validation system for permission checks that also limit abusive use of Anti-Raid services. In the Webserver, a new ``bigint`` type was added directly to ``splashcore`` in preparation for Discord increasing the permission bits beyond 64 which is something serenity is also looking into handling.
- ``PermissionChecks::ModuleNotFound`` has been added to avoid an unwrap in can_run_command. While this invariant should not actually happen, it is better to be safe than sorry.
- ``PermissionCheck::SudoNotGranted`` has also been added for removing the genericerror previously used for the root module.
- The ``commands_configurable`` option for modules has been renamed to ``commands_toggleable`` and has been changed to only apply to toggling commands. Other cases are safe anyways as owners can always bypass permissions anyways.

## Animus Magic

- The ``GetSerenityPermissionList`` endpoint has been removed. The already generated ``serenity_perms.json`` should be used instead and embedded into the compiled binaries/any other place using them. This also avoids several useless Animus Magic calls which may improve performance.

# Monday, June 24th 2024

## Bot

- Commands now also support ``web_hidden``

## Website

- Refactored theming
- Command related code has been moved to its own library (`$lib/ui/commands`)

# Sunday, June 23rd 2024

## Website

- The website has been redesigned slightly to use tab buttons to switch between different sections of a module. Command editting has been improved to use a dedicated component to avoid rendering issues. Command configurations are now properly parsed from the base command and the command configuration list itself

## API

- The ``toggle_module`` endpoint has been replaced by ``patch_module_configuration``. This is needed for default permissions on modules

## Bot

- Renamed the ``web`` virtual module to ``acl`` to better reflect its purpose.
- Module modperms permissions can now be granularly controlled through the ``acl__modules_modperms {module}`` command on the ``acl`` virtual module.
- Modules can now have a set of default permissions defined by the ``default_perms`` field in the module configuration. This is useful for setting a base set of permissions for all commands in a module. Note that if a command has an explicit set of permissions defined/overriden for it in the command configuration, the commands permissions are used. This ensures both broad and specific control over permissions while avoiding cascading effects between ``default_perms`` inherited from the module and the command's set permissions.

# Friday, June 21st 2024

- The presense of operation specific data is now used to determine if an operation is supported or not.
- ``module_id_cache`` has been renamed to ``module_cache`` in consistency with ``canonical_module_cache``
- The Settings API has been integrated into animus magic through the ``SettingsOperation`` op. This is a work in progress and may not be fully optimal yet.

# Wednesday, June 19th 2024

- ``_validate_value`` has been removed in favor of ``_validate_and_parse_value``. The ``_validate_and_parse_value`` API also handles parsing types to take into account user/developer error and returns the parsed type (hence why ``validate_value`` was renamed). This is a breaking change.
- The ``User``/``Message``/``Channel``/``Role``/``Emoji`` column types have been removed in favor of a new ``kind`` property inside of the ``string`` inner type. This is how they were parsed/used internally as well and it makes things easier to maintain. This is a breaking change.
- Added ``src`` to ``SettingsError::MissingOrInvalidField`` to improve debugging and better communicate the source of such an error.
- Poise implementation of settings now handles corresponding command permissions. This will most likely anyways be in cache for poise commands but it is good to enforce everywhere.
- Added WIP ``settings_delete`` for the Delete operation. This is a work in progress and may not be fully optimal yet.

# Tuesday, June 18th, 2024

- ``settings_create`` has been significantly optimized to reduce the number of needed hash maps. The ``SettingsError`` enum has been moved to ``config_opts.rs`` with the other type definitions. General bug fixes and improvements. Notably, unique constaints on columns are now enforced.
- **All native functions and conditions must now return SettingsError. Previously, they were free to return crate::Error**. This is to ensure that all errors are reported in a consistent manner
- ``ColumnAction::Check`` has been removed. Native actions are way more powerful and the performance difference between async and sync functions on rust are close to negligible. This also reduces the maintainence burden of the codebase.
- ``MissingField`` has been renamed to ``MissingOrInvalidField`` to better reflect the error type. This is a breaking change. ``SchemaCheckValidationError`` now contains a ``error`` field to better communicate exact errors.
- Added WIP ``settings_update`` for the Update operation. This is a work in progress and may not be fully optimal yet.
- Several misc refactors throughout the Settings API.

# Wednesday, June 12th 2024

## Botv2

### Internal changes 

Lots of changes to Settings API (which will be finished before moving on with the rest of the bot to allow for easier testing and configuring of the bot)

- ``columns_to_set`` no longer supports cross table column sets as this took way too much code and made the entire settings API a clunky and spaghetti code mess
- Significant code improvements to the ``settings_view`` and ``settings_create`` API's in general.
- ``on_condition`` has been added to the settings action API to allow for conditional actions. This is useful for cases where different checks need to be performed based on the value of a field (such as channel checks for when the sink type is ``channel``)
- Both ``add_channel`` and ``add_webhook`` from the Audit Logs module has been moved to the new settings API. This marks a key milestone for the settings API and further suggests that using audit log sinks as a guinea pig and starting point was in fact a good decision.

# Tuesday, June 11th 2024

## Botv2

### Internal changes

Lots of changes to Settings API (which will be finished before moving on with the rest of the bot to allow for easier testing and configuring of the bot)

- _getcols now correctly handles column_ids in operation specific data
- _parse_row now executes actions (will remove lua soon)
- _query_bind_value has been added to allow binding the Value enum provided by silverpelt to sqlx queries
- _post_op_colset has been added to run post operation column sets (like ``last_updated``, ``updated_by`` etc)
- settings_view has been split into the database bit which handles collecting the values, running everything and the poise display bit which displays the values
- Poise display code now uses column names for the display key and tries to format the value for the user instead of simply using the column id and raw value. This should improve UX significantly for the user
- Timestamp and TimestampTz have been added as native types to Value, when collecting from database using sqlx, these will be used instead of String, likewise, query binding against them has also been added
- Likewise, the canonical typings for config options now includes both Timestamp and TimestampTz as opposed to just Timestamp to avoid confusion and mixing incompatible types
- Variables have been improved. Instead of storing system variables like ``user_id`` and ``now`` in the state map, they are now special-cased and prefixed with ``__``. This both improves performance, reduces the chance of collision with user-defined variables and avoids display errors (e.g. with the poise layer)
- For display purposes, the special case variable {[__column_id]_displaytype} can be set to allow displaying in a different format. This is useful in cases like sinks where the display type can be changed based on a condition
- **Lua has been removed entirely in favor of native Rust async functions due to both performance and implementation reasons (extra dependencies, relying on ``mlua`` and `C` code). A microservice can be spun out if needed for performance/load reasons**
- The readonly and column_ids options have been removed in favor of ``ignored_for``. Post operation column sets have been optimized as well.

## Animus Magic

- Animus Magic now uses JSON instead of CBOR for payloads due to issues on the Go side with deserializing CBOR data on external structs (such as ordered maps etc.). This may be temporary or permanent. To compensate for performance (and because stdlib json serde on Go is slow), the ``github.com/bytedance/sonic`` library is now used on the go side instead of stdlib (`encoding/json`). Likewise, the ``eureka`` library used by the Anti-Raid webserver for handling requests + responding to them etc. has also been updated to use ``github.com/bytedance/sonic``. **This change also applies to the webserver**

## Webserver

- Silverpelt typings have been updated to include the external representation of the Settings API structures. Note that actions may be moved from lua scripts + actions enum to native Rust structures (potentially)
