# Introduction

**NOTE:** Just adding AntiRaid (or any discord bot) to your server will not do much unless you configure it first!

AntiRaid is a discord bot that takes a very unique approach to protecting servers. Instead of providing a single-size-fits-all solution, AntiRaid makes use of "structured templating" and a (work in progress/coming soon) template shop to allow servers to protect themselves in a way that is tailored to them.

---

## Documentation Index

### Developer Docs
- [Core Libraries](./dev/core/)
- [Infrastructure](./dev/infra/)
- [Bot Service](./dev/bot/)
- [API Service](./dev/api/)
- [Jobserver](./dev/jobserver/)
- [Services](./dev/services/)
- [Templating System](./dev/templating/)
- [Templating API Reference](./dev/templating-api/)
- [Rust Settings](./dev/rust_settings/)
- [Go Jobserver](./dev/go_jobs/)
- [Builtins](./dev/builtins/)
- [Command-line Tools](./dev/cmd/)
- [Data & Assets](./dev/data/)
- [Hosting](./dev/hosting/)

### User Docs
- [User Guides](../user/)

---

## Our Philosophy

While many bots like Wick provide a managed anti-raid/anti-nuke solution, these systems often provide suboptimal user experience to many servers as they may not cater to specific needs of the server. Wick, for example, is often criticized for limiting the abilities of server moderators in a way that makes moderation harder. While Wick is useful in many servers and does work quite well in the anti-raid/anti-nuke field from our experience, it may not be the best solution for all servers.

Templating, on the other hand, allows servers to tailor their needs directly without needing to go through the pain of making a custom discord bot, hosting it and then navigating the complexities of the discord API manually while reinventing concepts like stings/punishments/permissions/captchas all over again.