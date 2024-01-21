<script lang="ts">
	import BoolInput from "../inputs/BoolInput.svelte";
	import InputText from "../inputs/InputText.svelte";
    import Select from "../inputs/select/Select.svelte";

    // The set permission
    export let perm: string;
    export let validPerm: boolean = false;

    // The following are derived from `perm`
    let namespace: string;
    let permission: string;
    let negator: boolean;

    interface PreselectablePermission {
        namespace_id: string; // The namespace ID (backup/limits etc.)
        namespace_label: string; // The namespace label (Backups, Limits etc.)
        permissions: {
            id: string; // The permission ID (create, delete etc.)
            label: string; // The permission label (Create, Delete etc.)
        }[]
    }

    let preselectablePermissions: PreselectablePermission[] = [
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

    const unwindPerm = (perm: string) => {
        const split = perm.split('.');
        
        namespace = split[0] // Namespace is always the first part of the permission

        let negator: boolean;
        let permission: string;
        let validPerm: boolean;

        if(namespace.startsWith("~")) {
            negator = true
            namespace = namespace.substring(1)
        } else {
            negator = false
        }
        
        if(split.length == 2) {
            permission = split[1];
            validPerm = true;
        } else {
            permission = ""
        }

        return {
            namespace,
            permission,
            negator,
            validPerm
        }
    }

    const rewindPerms = (namespace: string, permission: string, negator: boolean) => {
        if(permission) {
            return `${negator ? "~" : ""}${namespace}.${permission}`
        } else {
            return `${negator ? "~" : ""}${namespace}`
        }
    }
    
    $: {
        const unwoundPerm = unwindPerm(perm);
        namespace = unwoundPerm.namespace;
        permission = unwoundPerm.permission;
        negator = unwoundPerm.negator;
        validPerm = unwoundPerm.validPerm;
    }
</script>

<div class="flex flex-col">
    <div class="flex flex-row">
        <div class="lg:pt-0 mr-5 block w-1/2">
            <Select 
                id="namespace"
                label="Namespace"
                bind:value={namespace}
                disabledDefaultInput={true}
                choices={preselectablePermissions.map((preselectablePermission) => {
                    return {
                        id: preselectablePermission.namespace_id,
                        value: preselectablePermission.namespace_id,
                        label: preselectablePermission.namespace_label
                    }
                })}
                onChange={() => {
                    permission = ""
                    validPerm = false;
                }}
            />
        </div>
        <div class="lg:pt-0 block w-1/2">
            <Select
                id="permission"
                label="Permission"
                bind:value={permission}
                disabledDefaultInput={true}
                choices={(
                    [
                        {
                            id: "*",
                            value: "*",
                            label: "All Permissions On Namespace"
                        },
                        ...preselectablePermissions.find((preselectablePermission) => preselectablePermission.namespace_id == namespace)?.permissions.map((preselectablePermissionPermission) => {
                            return {
                                id: preselectablePermissionPermission.id,
                                value: preselectablePermissionPermission.id,
                                label: preselectablePermissionPermission.label
                            }
                        }) || []
                    ]
                )}
                onChange={() => {
                    perm = rewindPerms(namespace, permission, negator)
                    if(namespace && permission) {
                        validPerm = true;
                    }
                }}
            />
        </div>
    </div>
    <div class="mt-3">
        <BoolInput 
            id="negator"
            label="Negate Permission"
            description="Whether or not the permission should be *removed*. This overrides 'All Permissions On Namespace' thus allowing for a easy permission blacklist on certain namespaces/modules."
            bind:value={negator}
            onChange={(_) => {
                perm = rewindPerms(namespace, permission, negator)
            }}
            disabled={false}
            required={false}
        />
    </div>
    <div class="mt-3">
        <InputText 
            id="perm"
            bind:value={perm}
            label="Permission String"
            placeholder="server_backups.list"
            minlength={1}
            showErrors={false}
        />
    </div>
</div>