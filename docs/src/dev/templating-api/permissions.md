<div id="@antiraid/permissions"></div>

# @antiraid/permissions

<div id="Types"></div>

## Types

<div id="Permission"></div>

## Permission

<details>
<summary>Raw Type</summary>

```luau
type Permission = kittycat.Permission
```

</details>

[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)

<div id="StaffPermissions"></div>

## StaffPermissions

<details>
<summary>Raw Type</summary>

```luau
type StaffPermissions = kittycat.StaffPermissions
```

</details>

[kittycat](./kittycat.md).[StaffPermissions](./kittycat.md#StaffPermissions)

<div id="CheckPatchChangesError"></div>

## CheckPatchChangesError

<details>
<summary>Raw Type</summary>

```luau
type CheckPatchChangesError = kittycat.CheckPatchChangesError
```

</details>

[kittycat](./kittycat.md).[CheckPatchChangesError](./kittycat.md#CheckPatchChangesError)

<div id="Functions"></div>

# Functions

<div id="permission_from_string"></div>

## permission_from_string

Returns a Permission object from a string.

<details>
<summary>Function Signature</summary>

```luau
--- Returns a Permission object from a string.
function permission_from_string(str: string) -> kittycat.Permission end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="str"></div>

### str

[string](#string)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)<div id="permission_to_string"></div>

## permission_to_string

Returns a string from a Permission object.

<details>
<summary>Function Signature</summary>

```luau
--- Returns a string from a Permission object.
function permission_to_string(perm: kittycat.Permission) -> string end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="perm"></div>

### perm

[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[string](#string)<div id="has_perm"></div>

## has_perm

Checks if a list of permissions in Permission object form contains a specific permission.

<details>
<summary>Function Signature</summary>

```luau
-- Checks if a list of permissions in Permission object form contains a specific permission.
function has_perm(permissions: {kittycat.Permission}, permission: kittycat.Permission) -> boolean end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="permissions"></div>

### permissions

{[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)}

<div id="permission"></div>

### permission

[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[boolean](#boolean)<div id="has_perm_str"></div>

## has_perm_str

Checks if a list of permissions in canonical string form contains a specific permission.

<details>
<summary>Function Signature</summary>

```luau
--- Checks if a list of permissions in canonical string form contains a specific permission.
function has_perm_str(permissions: {string}, permission: string) -> boolean end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="permissions"></div>

### permissions

{[string](#string)}

<div id="permission"></div>

### permission

[string](#string)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[boolean](#boolean)<div id="staff_permissions_resolve"></div>

## staff_permissions_resolve

Resolves a StaffPermissions object into a list of Permission objects.

<details>
<summary>Function Signature</summary>

```luau
--- Resolves a StaffPermissions object into a list of Permission objects.
function staff_permissions_resolve(sp: kittycat.StaffPermissions) -> {kittycat.Permission} end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="sp"></div>

### sp

[kittycat](./kittycat.md).[StaffPermissions](./kittycat.md#StaffPermissions)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

{[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)}<div id="check_patch_changes"></div>

## check_patch_changes

Checks if a list of permissions can be patched to another list of permissions.

<details>
<summary>Function Signature</summary>

```luau
--- Checks if a list of permissions can be patched to another list of permissions.
function check_patch_changes(manager_perms: {kittycat.Permission}, current_perms: {kittycat.Permission}, new_perms: {kittycat.Permission}) -> (boolean, kittycat.CheckPatchChangesError?) end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="manager_perms"></div>

### manager_perms

{[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)}

<div id="current_perms"></div>

### current_perms

{[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)}

<div id="new_perms"></div>

### new_perms

{[kittycat](./kittycat.md).[Permission](./kittycat.md#Permission)}

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[boolean](#boolean)

<div id="ret2"></div>

### ret2

*This field is optional and may not be specified*

[kittycat](./kittycat.md).[CheckPatchChangesError](./kittycat.md#CheckPatchChangesError)?

