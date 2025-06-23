# Templating-Template Infra Docs

This component provides the infrastructure for the AntiRaid templating system, enabling user-defined logic for moderation, automation, and more.

## Key Features
- **Sandboxed Execution:** Runs user templates in a secure, resource-limited environment.
- **Language Support:** Primary support for Lua (Luau), with experiments for JS and WASM.
- **API Exposure:** Exposes core AntiRaid APIs (messages, permissions, captcha, etc.) to templates.

## Configuration
- Controlled via service config and environment variables.
- Template storage and loading is managed by the main bot and API services.

## Extension Points
- Add new APIs or helpers by extending the template host code.
- Support for additional languages can be added with new sandboxes.

## Deployment
- Used as a library by the bot, template worker, and API services.

---

See the templating and templating-api docs for more details on usage and extension.
