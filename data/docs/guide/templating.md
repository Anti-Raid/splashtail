# Templating

To allow further customizing the bot. Anti-Raid supports templating. Templating allows you to customize embeds and messages to the extreme through for-loops, if statements, variables and some basic functions.

To do so, Anti-Raid uses [tera](https://keats.github.io/tera/docs/). See its docs for the full list of features. Note that the following extra changes apply in Anti-Raid:

- Dangerous functions such as ``get_env`` do not exist.
- ``__tera_context_raw`` provides the Tera context as an object. This complements ``__tera_context`` which provides the context as a string for debugging.
- All templates have a (reasonable) time limit for execution to protect against abuse and DDOS attacks.
- Dividing by zero will error instead of returning ``NaN``.
- **When using templates, the output of the template itself is ignored. For messages, you must use ``Message Helpers`` to construct the message and for permission checking, you must use ``Permission Check Helpers``. See example 1 below:**
- Bitwise operators are also supported. ``N bitor N`` (bitwise OR / ``|``), ``N bitand N`` (bitwise AND / ``&``), ``N bitxor N`` (bitwise XOR / ``^``), ``N << N`` (bitwise shift left / ``<<``), ``N >> N`` (bitwise shift right / ``>>``), ``bitnot N`` (bitwise NOT / ``~``) are supported. ``N`` is a number. For example, ``8 bitor 4`` will return ``12``. 

## Example 1:

The below second template will have no effect when constructing a message

```
Hello world
```

However, the below second template will construct a message with the content "Hello world"

```
{% filter content %}
Hello world
{% endfilter %}
```

Note that this only applies to templates used to construct messages such as ``Audit Long Sink`` templates.

## Gateway event structure

All gateway events are tagged

## Common Functions And Filters

### Base filters

- The ``bettertitle`` filter provides a potentially better title-ing filter than the ``title`` filter pre-provided by Tera
- The ``merge`` filter merges two objects together with the second object being defined by ``with``. The second object overwrites the first one in the event of a conflict

## Situational Functions and Filters

These functions and filters are only available in certain contexts listed by the "Works on" section.

### Gateway Event Helpers

The following functions can be used on Gateway Event related templates.

Works on:
- Audit Log Sink Embeds

- The ``{gwevent::field::Field} | formatter__gwevent_field`` filter can be used to format a gateway event field

### Message Helpers

The following functions can be used to create embeds/messages.

Works on:
- Audit Log Sink Embeds

- The ``embed_title(title=TITLE)`` function can be used to set the title of an embed.
- The ``embed_field(name=NAME, value=VALUE, inline=INLINE [default: false])`` function can be used to add fields to embeds.
- The ``embed_description`` filter can be used to set the description of an embed. This uses a filter to make multi-line descriptions easier.
- The ``content`` filter can be used to set the content of a message. This uses a filter to make multi-line content easier.
- The ``new_embed(title=TITLE [optional], description=DESCRIPTION [optional])`` function can be used to create a new embed.


**Note that not calling ``new_embed`` before calling ``embed_title`` or ``embed_field`` will automatically make a new embed in state.**

**Example**

```
{{ embed_title(title="My cool embed") }}
{{ embed_field(name="Field 1", value="Value 1") }}
{{ embed_field(name="Field 2", value="Value 2") }}
{{ embed_field(name="Field 3", value="Value 3", inline=true) }}

{% filter embed_description %}
This is a cool embed
{% endfilter %}

{% filter content %}
# Hello world

This is message content
{% endfilter %}
```

### Permission Check Helpers

Works on:
- Permission Checking

- The ``run_permission_check(kittycat_perms = string[], native_permissions = Permissions, and = BOOLEAN)`` function can be used to run a single permission check against the members permission returning a boolean. This returns an object containing ``code`` and ``ok`` fields.
- The ``permission_result(result=PermissionResult)`` filter can be used to return a permission result early on. For example:

For example, the below template will return "Ok" if the user has the permission "moderation.prune_user" and Administrator on Discord:

```jinja2
{% set perm_res = run_permission_check(kittycat_perms = ["moderation.prune_user"], native_permissions = 8, and = true) %}

{% if perm_res.ok %}
    {% filter permission_result %}
        {"Ok": {}}
    {% endfilter %}
{% endif %}
```

## Available Fields

### Messages

{message_fields}

### Permission Checking

- ``user_id``: The user ID of the user being checked
- ``guild_id``: The guild ID of the guild the user is being checked in
- ``guild_owner_id``: The user ID of the owner of the guild the user is being checked in
- ``native_permissions``: The native (Discord) permissions of the user
- ``kittycat_permissions``: The kittycat (custom) permissions of the user (`Vec<String>`)
- ``channel_id``: The channel ID of the channel the user is being checked in (if the command is executed in a channel context), may be `None`