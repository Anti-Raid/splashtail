# Tuesday, June 11th 2024

## Botv2

### Internal changes

Lots of changes to Settings API (which will be finished before moving on with the rest of the bot to allow for easier testing and configuring of the bot)

- _getcols now correctly handles column_ids in operation specific data
- _parse_row now executes actions (will remove lua soon)
- _query_bind_value has been added to allow binding the Value enum provided by silverpelt to sqlx queries
- _post_op_colset has been added to run post operation column sets (like ``last_updated``, ``updated_by`` etc)
- settings_view has been split into the database bit which handles collecting the values, running everything and the poise display bit which displays the values
- Timestamp and Timestamptz have been added as native types to Value, when collecting from database using sqlx, these will be used instead of String, likewise, query binding against them has also been added