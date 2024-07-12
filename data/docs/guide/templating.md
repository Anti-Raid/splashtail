# Templating

To allow further customizing the bot. Anti-Raid supports templating. Templating allows you to customize embeds and messages to the extreme through for-loops, if statements, variables and some basic functions.

To do so, Anti-Raid uses [tera](https://keats.github.io/tera/docs/). See its docs for the full list of features. Note that the following extra changes apply in Anti-Raid:

- Dangerous functions such as ``get_env`` do not exist.
- ``__tera_context_raw`` provides the Tera context as an object. This complements ``__tera_context`` which provides the context as a string for debugging.
- All templates have a (reasonable) time limit for execution to protect against abuse and DDOS attacks.

## Common Functions And Filters

### Base filters

- The ``bettertitle`` filter provides a potentially better title-ing filter than the ``title`` filter pre-provided by Tera

### Embed helpers

- The ``title(title=TITLE)`` function can be used to set the title of an embed.
- The ``field(name=NAME, value=VALUE, inline=INLINE [default: false])`` function can be used to add fields to embeds.

## Situational Functions and Filters

These functions and filters are only available in certain contexts listed by the "Works on" section.

### Gateway Event Helpers

The following functions can be used on Gateway Event related templates.

Works on:
- Audit Log Sink Embeds

- The ``{gwevent::field::Field} | formatter__gwevent_field`` filter can be used to format a gateway event field