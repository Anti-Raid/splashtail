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

## Animus Magic

- Animus Magic now uses JSON instead of CBOR for payloads due to issues on the Go side with deserializing CBOR data on external structs (such as ordered maps etc.). This may be temporary or permanent. To compensate for performance (and because stdlib json serde on Go is slow), the ``github.com/bytedance/sonic`` library is now used on the go side instead of stdlib (`encoding/json`). Likewise, the ``eureka`` library used by the Anti-Raid webserver for handling requests + responding to them etc. has also been updated to use ``github.com/bytedance/sonic``. **This change also applies to the webserver**

## Webserver

- Silverpelt typings have been updated to include the external representation of the Settings API structures. Note that actions may be moved from lua scripts + actions enum to native Rust structures (potentially)
