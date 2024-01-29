<script lang="ts">
	import { permissionMapper, rewindPerms, unwindPerm } from "$lib/ui/permMap";
	import BoolInput from "../inputs/BoolInput.svelte";
	import InputText from "../inputs/InputText.svelte";
    import Select from "../inputs/select/Select.svelte";

    // The set permission
    export let perm: string;
    export let validPerm: boolean = false;

    // The following are derived from `perm`
    let namespace: string;
    let permission: string;
    let scope: string;
    let negator: boolean;
    
    $: {
        const unwoundPerm = unwindPerm(perm);
        namespace = unwoundPerm.namespace;
        permission = unwoundPerm.permission;
        scope = unwoundPerm.scope;
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
                choices={permissionMapper.map((preselectablePermission) => {
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
                        ...permissionMapper.find((preselectablePermission) => preselectablePermission.namespace_id == namespace)?.permissions.map((preselectablePermissionPermission) => {
                            return {
                                id: preselectablePermissionPermission.id,
                                value: preselectablePermissionPermission.id,
                                label: preselectablePermissionPermission.label
                            }
                        }) || []
                    ]
                )}
                onChange={() => {
                    perm = rewindPerms(namespace, permission, scope, negator)
                    if(namespace && permission) {
                        validPerm = true;
                    }
                }}
            />
        </div>
    </div>
    <div class="mt-3 text-white">
        <BoolInput 
            id="negator"
            label="Negate Permission"
            description="Whether or not the permission should be *removed*. This overrides 'All Permissions On Namespace' thus allowing for a easy permission blacklist on certain namespaces/modules."
            bind:value={negator}
            onChange={(_) => {
                perm = rewindPerms(namespace, permission, scope, negator)
            }}
            disabled={false}
            required={false}
        />
    </div>
    <div class="mt-3">
        <InputText 
            id="scope"
            bind:value={scope}
            label="Scope"
            placeholder="Fine-grained permission controls."
            minlength={1}
            showErrors={false}
        />
        <small>Scopes are fine-grained permission controls. Not all permissions support scopes. The usage of scopes are module-dependent</small>

        <InputText 
            id="perm"
            bind:value={perm}
            onChange={() => perm = rewindPerms(namespace, permission, scope, negator)}
            label="Permission String"
            placeholder="server_backups.list"
            minlength={1}
            showErrors={false}
        />
    </div>
</div>