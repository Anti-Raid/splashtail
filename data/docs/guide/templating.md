# Templating

To allow further customizing the bot. Anti-Raid supports templating. Templating allows you to customize embeds and messages to the extreme through for-loops, if statements, variables and some basic functions.

To do so, Anti-Raid uses [tera](https://keats.github.io/tera/docs/). See its docs for the full list of features. Note that the following extra changes apply in Anti-Raid:

- Dangerous functions such as ``get_env`` do not exist.
- ``__tera_context_raw`` provides the Tera context as an object. This complements ``__tera_context`` which provides the context as a string for debugging.
- All templates have a (reasonable) time limit for execution to protect against abuse and DDOS attacks.
- **When using templates to construct a message, the output of the template itself is ignored. For messages, you must use ``Message Helpers`` to construct the message. See example 1 below:**

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