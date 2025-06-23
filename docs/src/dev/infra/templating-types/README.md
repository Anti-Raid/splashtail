# Templating-Types Infra Docs

This component provides the type system and type extensions for the AntiRaid templating system.

## Key Features
- **Type Extensions:** Adds custom types and helpers to the template environment.
- **Safety:** Ensures only safe, serializable types are exposed to templates.
- **Extensibility:** New types can be added for advanced template logic.

## Configuration
- Types are registered at service startup.
- Controlled via the template host and service config.

## Extension Points
- Add new types or helpers by extending the type registry.

## Deployment
- Used as a library by the bot, template worker, and API services.

---

See the templating-api docs for more details on available types and helpers.
