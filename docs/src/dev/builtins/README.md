# Builtins Developer Docs

This folder documents built-in modules, types, and helpers available across the AntiRaid codebase. These builtins are available to templates, modules, and core services for common tasks and abstractions.

## Key Builtins
- **Types:** Common data types (User, Server, Channel, etc.) used throughout the system.
- **Helpers:** Utility functions for string manipulation, date/time, permissions, etc.
- **Template APIs:** Built-in APIs exposed to user templates (see templating-api docs).

## Usage
- Builtins are imported or available by default in most services and templates.
- Extend builtins by adding new types or helpers to this folder and registering them in the host service.

Builtins are a replacement for the current "bot" commands. these are currently in works and prone to change

---

For more, see the templating-api docs and the main project README.
