# Events

AntiRaid makes use of events for all communication between modules. Modules can either recieve events (e.g. from Discord) or generate their own events (such as PunishmentCreate/PunishmentExpire). This is what makes Anti-Raid so flexible and powerful, especially in templating.

## Discord Events

Discord Events are special. AntiRaid uses Serenity for handling Discord events. As such, please see [Serenity's Documentation](https://docs.rs/serenity/latest/serenity/client/enum.FullEvent.html#variants) for what fields are available in each event. It's much better documentation than what we can come up with. 

Some notes:

- AntiRaid does not modify the events in any way, so you can expect the same fields as what Serenity provides. Note, however, that only the fields are available to templates, not the methods provided by Serenity.
- All event data will be wrapped in a outer table containing the enum variant name. For example, message data can be found in ``event.data["Message"]`` and not just ``event.data``. This is for performance reasons.

## Other Events

TODO: Just ask on our support server for now