# Anti-Raid

- Member Roles: the roles which are given to members

# Quick Lockdown

Quick Lockdown allows for quickly locking down a server given the following permission overwrite setup:

- If no member roles, the ``@everyone`` role must have View Channel and Send Messages,
Send Messages In Threads
- Otherwise, all roles other than member roles (including ``@everyone``) must not have View Channel, Send Messages, Send Messages In Threads, member roles must have View Channel and Send Messages, Send Messages In Threads
