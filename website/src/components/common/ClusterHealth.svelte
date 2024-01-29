<script lang="ts">
	import { Instance, InstanceList } from "$lib/generated/mewld/proc";
	import moment from "moment";
	import Modal from "../Modal.svelte";
    import ObjectRender from "../ObjectRender.svelte";

    export let instanceList: InstanceList;

    const getClusterGuildCount = (i: Instance) => {
        let count: number = 0;
        for(let ch of (i?.ClusterHealth || [])) {
            count += ch?.guilds
        }
        return count;
    }

    const getClusterUserCount = (i: Instance) => {
        let count: number = 0;
        for(let ch of (i?.ClusterHealth || [])) {
            count += ch?.users
        }
        return count;
    }

    let openCluster: number | undefined;

    let showModal: boolean = false;

    $: if (!showModal) openCluster = undefined;
</script>

{#if openCluster != undefined && showModal}
    <Modal bind:showModal>
        <h1 slot="header" class="font-semibold text-2xl">
            Cluster {openCluster} - {instanceList?.Map?.find((cluster) => cluster.ID == openCluster)?.Name}
        </h1>

        <h2 class="text-xl font-semibold">Cluster Map</h2>
        <ObjectRender object={instanceList?.Map?.find((cluster) => cluster.ID == openCluster)} />
        <h2 class="mt-2 text-xl font-semibold">Instance</h2>
        <ObjectRender object={instanceList?.Instances?.find((instance) => instance?.ClusterID == openCluster)} />   
    </Modal>
{/if}

<div class="flex flex-col mt-4">
    <div class="overflow-x-auto sm:-mx-6 lg:-mx-8">
        <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
            <div class="overflow-hidden shadow-xl sm:rounded-lg">
                <table class="min-w-full">
                    <thead class="bg-slate-800 text-gray-50">
                        <tr>
                            <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left uppercase">
                                Cluster
                            </th>
                            <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left uppercase">
                                Shards
                            </th>
                            <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left uppercase">
                                Guilds
                            </th>
                            <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left uppercase">
                                Users
                            </th>
                            <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left uppercase">
                                Last Started
                            </th>
                            <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left uppercase">
                                Last Health Check
                            </th>
                            <th role="" scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left uppercase">
                                
                            </th>
                        </tr>
                    </thead>
                    <tbody class={"bg-black text-white divide-y divide-gray-700"}>
                        {#each instanceList.Instances as instance}
                            <tr class="hover:bg-slate-950">
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="text-sm font-medium">
                                        <strong>{instance?.ClusterID} </strong> ({instanceList?.Map?.find((cluster) => cluster.ID == instance?.ClusterID)?.Name})
                                    </div>
                                    <span class={
                                        instance?.Active ? "text-sm text-green-500" : "text-sm text-red-500"
                                    }>
                                        {instance?.Active ? "Active" : "Inactive"}
                                    </span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-300">
                                    {instance?.Shards?.join(", ")} 
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-300">
                                    {#if instance}
                                        {getClusterGuildCount(instance)}
                                    {:else}
                                        Unknown
                                    {/if}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-300">
                                    {#if instance}
                                        {getClusterUserCount(instance)}
                                    {:else}
                                        Unknown
                                    {/if}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-300">
                                    {moment(instance?.StartedAt).fromNow()}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-300">
                                    {moment(instance?.LastChecked).fromNow()}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                    <button on:click={() => {
                                        openCluster = instance?.ClusterID;
                                        showModal = true;
                                    }} class="text-indigo-400 hover:text-indigo-600">View Advanced</button>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</div>