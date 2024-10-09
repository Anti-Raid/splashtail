# Silverpelt

Silverpelt provides a standard library for all Anti-Raid modules.

To create a new Anti-Raid bot making use of Anti-Raid modules, simply implement the trait ``silverpelt::module::Module``. These modules must then be added to a ``SilverpeltCache`` which is then inserted into ``silverpelt::Data``.

Most things in silverpelt are abstracted out through traits or dispatched via events. This allows silverpelt to be used as an abstract interface allowing for Anti-Raid to quickly evolve and change/adapt to different targets.

**Note:** AntiRaid uses a event-driven architecture. This means that modules+the main bot process make events that are dispatched to modules. The event system is currently passive (meaning there is no continously running event loop), however this is subject to change in the future.

## Interfaces 

### Sting

Silverpelt provides concrete structures, utilities and special events for handling stings.

### Punishments

Silverpelt provides concrete structures, utilities and special events for handling punishments.

## STDEvents

See the ``rust.stdevent`` crate first if there's a existing standardized custom event for you to use directly.

## Some extra misc points

- A command is the base unit for access control. This means that all operations with differing access controls must have commands associated with them.

- This means that all operations (list backup, create/restore backup, delete backup) *MUST* have associated commands

- Sometimes, an operation (such as a web-only operation) may not have a module/command associated with it. In such cases, a 'virtual' module should be used. Virtual modules are modules with commands that are not registered via Discord's API. They are used to group commands together for access control purposes and to ensure that each operation is tied to a command
