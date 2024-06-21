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
