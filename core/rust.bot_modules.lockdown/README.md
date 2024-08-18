# Lockdown

- Member Roles: the roles which are given to members

# Quick Lockdown

Quick Lockdown allows for quickly locking down a server given the following permission overwrite setup:

- If no member roles, the ``@everyone`` role must have View Channel, Send Messages, Send Messages In Threads. All other roles must not have View Channel, Send Messages, Send Messages In Threads
- Otherwise, all roles other than member roles (including ``@everyone``) must not have View Channel, Send Messages, Send Messages In Threads, member roles must have View Channel and Send Messages, Send Messages In Threads

In other words (for development purposes):

- One can define a set of critical roles which are either the member roles or the ``@everyone`` role, all other roles must not have View Channel, Send Messages and/or Send Messages In Threads permissions