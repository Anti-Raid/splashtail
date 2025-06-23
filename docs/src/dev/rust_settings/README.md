# Settings

**Note: this page is specifically w.r.t creating settings on the Rust side of things. For more information on how to create settings within Lua templates, please see the documentation specific to Lua templates (when that feature is released)**

**Warning: this page only documents the new AntiRaid Settings v2 API which is simpler and with fewer footguns. The old API is not documented due to how complicated it was with an inenumerable number of footguns.**

# Rust Settings Developer Docs

This folder documents the settings system for the Rust components of AntiRaid.

## Overview
- The settings API allows for safe, flexible, and dynamic configuration of bot and service behavior.
- This documentation covers the new Settings v2 API, which is simpler and safer than the legacy system.

## Files
- `README.md`: Overview and usage of the settings API from Rust.

## Key Points
- Settings are defined in Rust and exposed to templates and the API.
- For Lua template settings, see the templating documentation.

---

For more, see the main Rust service code and this folder's files.