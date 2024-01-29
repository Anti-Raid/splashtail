interface PermissionMapper {
    namespace_id: string; // The namespace ID (backup/limits etc.)
    namespace_label: string; // The namespace label (Backups, Limits etc.)
    permissions: {
        id: string; // The permission ID (create, delete etc.)
        label: string; // The permission label (Create, Delete etc.)
    }[]
}

export const permissionMapper: PermissionMapper[] = [
    {
        namespace_id: "server_backups",
        namespace_label: "Server Backups",
        permissions: [
            {
                id: "list",
                label: "List Backups",
            },
            {
                id: "create",
                label: "Create Backups",
            },
            {
                id: "restore",
                label: "Restore Backups",
            }
        ]
    },
    {
        namespace_id: "limits",
        namespace_label: "Server Limits",
        permissions: [
            {
                id: "view",
                label: "View Existing Limits",
            },
            {
                id: "add",
                label: "Create Limits",
            },
            {
                id: "remove",
                label: "Remove Limits",
            },
            {
                id: "hit",
                label: "View Hit Limits"
            }
        ]
    },
    {
        namespace_id: "global",
        namespace_label: "Global Permissions",
        permissions: []
    },
]

// Given a perm string, extract it to its components
export const unwindPerm = (perm: string) => {
    const split = perm.split('.');
    
    let namespace = split[0] // Namespace is always the first part of the permission

    let negator: boolean = false;
    let permission: string = "";
    let validPerm: boolean = false;
    let scope: string = "";

    if(namespace.startsWith("~")) {
        negator = true
        namespace = namespace.substring(1)
    }
    
    if(split.length == 2) {
        permission = split[1];
        validPerm = true;

        // Handle scope (perm#scope form)
        if(permission.includes("#")) {
            const splitPermission = permission.split("#");
            permission = splitPermission[0];
            scope = splitPermission[1];
        } else {
            scope = ""
        }
    }

    return {
        namespace,
        permission,
        scope,
        negator,
        validPerm
    }
}

// Given the parts of a permission, rewind it to a perm string
export const rewindPerms = (namespace: string, permission: string, scope: string, negator: boolean) => {
    let base: string;
    if(permission) {
        base = `${negator ? "~" : ""}${namespace}.${permission}`
    } else {
        base = `${negator ? "~" : ""}${namespace}`
    }

    // Handle scope (perm#scope form)
    if(scope && permission) {
        base = `${base}#${scope}`
    }

    return base
}