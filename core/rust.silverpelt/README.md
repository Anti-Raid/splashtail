# Silverpelt

Silverpelt provides a standard library for all Anti-Raid modules.

To create a new Anti-Raid bot making use of Anti-Raid modules, one must first create ``silverpelt::Module``s. These modules must then be added to a ``SilverpeltCache`` which is then inserted into ``silverpelt::Data``.

Most things in silverpelt are abstracted out through traits. This includes per-module executors etc. This allows silverpelt to be used as an abstract interface allowing for Anti-Raid to quickly evolve and change/adapt to different targets

## Interfaces 

### Sting Sources
### Punishments

Note that silverpelt only provides interfaces and general structures for punishments. A dedicated ``punishments`` module is needed for actually handling punishments using these interfaces