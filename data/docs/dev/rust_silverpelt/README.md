# Silverpelt

Silverpelt provides a standard library for all Anti-Raid modules.

To create a new Anti-Raid bot making use of Anti-Raid modules, simply implement the trait ``silverpelt::module::Module``. These modules must then be added to a ``SilverpeltCache`` which is then inserted into ``silverpelt::Data``.

Most things in silverpelt are abstracted out through traits or dispatched via events. This allows silverpelt to be used as an abstract interface allowing for Anti-Raid to quickly evolve and change/adapt to different targets

## Interfaces 

### Sting

Silverpelt provides concrete structures, utilities and special events for handling stings.

### Punishments

Silverpelt provides concrete structures, utilities and special events for handling punishments.

## Custom Events

### Targets

Current (std) target classes are:

- `0x0` -> auditlogs
