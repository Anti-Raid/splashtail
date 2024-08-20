## Common state variables:

- {__author} => the user id of the user running the operation
- {__guild_id} => the guild id of the guild the operation is being run in
- {__now} always returns the current timestamp (TimestampTz), {__now_naive} returns the current timestamp in naive form (Timestamp)
- Note that these special variables do not need to live in state and may instead be special cased
- For sending a info message etc on save, the {__message} can be set

## Special variables:

- ``__limit`` and ``__offset``: These are used for limit-offset pagination and are used in the query to limit the number of results returned and the offset to start from

### View operations:

- ``__count``: Counts the number of rows instead of returning them