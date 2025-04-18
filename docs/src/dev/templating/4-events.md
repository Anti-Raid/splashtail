# Events

AntiRaid makes use of events for all communication between templates. Templates can either recieve events (e.g. from Discord), generate their own events (such as ScheduledExecution's) or expose shared callback hooks within the VM using ``store(ctx)``. This is what makes Anti-Raid so flexible and powerful, especially in templating.

## A special event: OnStartup

The ``OnStartup`` event is a special event that may be fired at some time when the server's Lua VM is initialized. While exactly when it happens is intentionally undefined to allow changes/optimizations there, it is guaranteed to at least be fired upon an update of any script etc.