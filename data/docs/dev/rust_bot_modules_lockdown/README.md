# Definitions

- Critical Roles: the roles which are given to members and should hence be locked down. In essence, one can define a set of critical roles (hence the name) which are either the critical roles and defaults to the ``@everyone`` role if not set.

# Lockdown Types

## Quick Server Lockdown

### Specificity

- ``0`` (Lowest specificity)

### Rationale

Quickly lockdown a server as fast as possible

### Syntax

- ``qsl``

### Description

Quick Lockdown allows for quickly locking down a server given the following permission overwrite setup:

- All critical roles must have View Channel and Send Messages. All other roles must not have View Channel and Send Messages

Internally, ``qsl`` modifies only the critical roles to the locked down set of permissions. This requires much fewer API calls and is hence much faster than traditional lockdowns.

## Traditional Server Lockdown

### Specificity

- ``1`` (TSL > QSL as it updates all channels in a server)

### Rationale

In many cases, the requirements for ``qsl`` are not feasible for servers to meet. In such a case, a traditional lockdown is needed.

### Syntax

- ``tsl``

### Description

Traditional Lockdown is a more traditional lockdown method. It is more flexible than ``qsl`` as it has no required prior setup. However, it is much slower and should be avoided if possible.

Internally, ``tsl`` works by iterating over all channels and setting the permission overwrites for all critical roles to the locked down set. This is a slow process and can take a long time for large servers. In addition, super large servers may have outages when using a ``tsl`` that a ``qsl`` may not lead to.


## Single-Channel Lockdown

### Specificity

- ``2`` (SCL > TSL as it updates a single channel)

### Rationale

In some cases, only a single channel needs to be locked down. In such a case, a single-channel lockdown is needed.

### Syntax

- ``scl/<channel_id>``

Where ``<channel_id>`` is the ID of the channel to lockdown

### Description

Single-Channel Lockdown is a lockdown method that locks down a single channel. 

Internally, ``scl/<channel_id>`` works by setting the permission overwrites for all critical roles to the locked down set for the specified channel. This is a fast process and is recommended for locking down a single channel.

# Specificity

When multiple lockdowns are made on the same item (which will now be called a ``handle`` from now on), there needs to be a way to know what lockdown owns/has the handle. In AntiRaid, this is controlled through specificity based on the rules:

- Rule 0: When a handle is locked, the priority is added without replacing older priorities. When a handle is unlocked, the priority is removed leading to its previous value.
- Rule 1: A handle is controlled unlocked by a lockdown A if the lockdown (say, lockdown B) corresponding to the largest specificity that has locked the handle is less than the specificity of lockdown A. Otherwise, it is considered locked and cannot be modified by lockdown A.
- Rule 2: The underlying permissions or permission overwrites of a role/channel are defined as the saved permissions/permission overwrites of the role/channel of the oldest possible lockdown which has saved said data.

As an example, consider a case where a ``tsl`` is first applied and then an ``scl/<channel_id>``. As per Rule 1, the ``tsl`` has a lower specificity than the ``scl/<channel_id>`` and so the ``scl/<channel_id>`` will also lock the channel handle. When the ``tsl`` is then removed, the channel is still locked by ``scl/<channel_id>`` which has a greater specificity. Hence, by Rule 1, the ``scl/<channel_id>`` locked channel will remain locked even after the ``tsl`` is removed as expected. 

Next, consider what happens when the ``scl/<channel_id>`` is removed. As ``tsl`` stores the original channel permission overwrites of all channels and was created before the ``scl/<channel_id>``, Rule 2 applies. Hence, the underlying permissions of the channel is considered to come from the ``tsl``'s stored data and NOT the ``scl/<channel_id>`` which was set during the lockdown. This means that when the ``scl/<channel_id>`` is removed, the channel will revert to the permissions it had before the ``tsl`` was applied which was the original channels permissions.

As such, using Rules 1 and 2, the following holds true:

``tsl + scl/<channel_id> - tsl - scl/<channel_id> = 0``