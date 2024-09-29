# Modifiers

There are many cases where you want to either target or override settings for users/channels/roles/globally/other targets. This is where modifiers come in. Modifiers can be used to target users/channels/role/globally.

Each modifier is a string that can be used to target a specific user, channel, role, or globally. The following modifiers are available:

- **User ID** - target a specific user (``user/{id}``, specificity = ``3``)
- **Channel ID** - target a specific channel (``channel/{id}``, specificity = ``2``)
- **Role ID** - target a specific role (``role/{id}``, specificity = ``1``)
- **Custom Variable** - target a specific variable (``custom/{key}/{value}/{specificity}``, specificity = ``{specificity}``)
- **Global** - target everything globally (guild-wide) (``global``, specificity = ``0``)

To allow handling conflicts between modifiers, each modifier has a specificity which is essentially a number as seen above.