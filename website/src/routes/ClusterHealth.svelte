<script lang="ts">
	import { get } from "$lib/configs/functions/services";
	import { fetchClient } from "$lib/fetch/fetch";
	import { InstanceList, Instance } from "$lib/generated/mewld/proc";
	import moment from "moment";
	import Message from "../components/Message.svelte";
	import Modal from "../components/Modal.svelte";
    import ObjectRender from "../components/ObjectRender.svelte";

    const getClusterHealth = async () => {
        const response = await fetchClient(`${get('splashtail')}/clusters/health`);
        const data: InstanceList = await response.json();
        return data;
    }

    const getClusterGuildCount = (i: Instance) => {
        let count: number = 0;
        for(let ch of i?.ClusterHealth) {
            count += ch?.guilds
        }
        return count;
    }

    const getClusterUserCount = (i: Instance) => {
        let count: number = 0;
        for(let ch of i?.ClusterHealth) {
            count += ch?.users
        }
        return count;
    }

    let openCluster: number | undefined;

    let showModal: boolean = false;

    $: if (!showModal) openCluster = undefined;
</script>

<h2 class="text-4xl font-bold tracking-tight text-gray-900 sm:text-5xl md:text-6xl">
    <span class="block text-white xl:inline">Cluster Health</span>
</h2>

{#await getClusterHealth()}
    <Message type="loading">Fetching cluster data...</Message>
{:then data}
    {#if openCluster != undefined && showModal}
        <Modal bind:showModal>
            <h1 slot="header" class="font-semibold text-2xl">
                Cluster {openCluster} - {data?.Map?.find((cluster) => cluster.ID == openCluster)?.Name}
            </h1>

            <h2 class="text-xl font-semibold">Cluster Map</h2>
            <ObjectRender object={data?.Map?.find((cluster) => cluster.ID == openCluster)} />
            <h2 class="mt-2 text-xl font-semibold">Instance</h2>
            <ObjectRender object={data?.Instances?.find((instance) => instance?.ClusterID == openCluster)} />   
        </Modal>
    {/if}

    <div class="flex flex-col mt-4">
        <div class="overflow-x-auto sm:-mx-6 lg:-mx-8">
            <div class="py-2 align-middle inline-block min-w-full sm:px-6 lg:px-8">
                <div class="overflow-hidden border-b border-gray-200 shadow sm:rounded-lg">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left text-gray-500 uppercase">
                                    Cluster
                                </th>
                                <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left text-gray-500 uppercase">
                                    Shards
                                </th>
                                <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left text-gray-500 uppercase">
                                    Guilds
                                </th>
                                <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left text-gray-500 uppercase">
                                    Users
                                </th>
                                <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left text-gray-500 uppercase">
                                    Last Started
                                </th>
                                <th scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left text-gray-500 uppercase">
                                    Last Health Check
                                </th>
                                <th role="" scope="col" class="px-6 py-3 text-xs font-medium tracking-wider text-left text-gray-500 uppercase">
                                    -
                                </th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            {#each data.Instances as instance}
                                <tr>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <div class="text-sm font-medium text-gray-900">
                                            <strong>{instance?.ClusterID} </strong> ({data?.Map?.find((cluster) => cluster.ID == instance?.ClusterID)?.Name})
                                        </div>
                                        <span class={
                                            instance?.Active ? "text-sm text-green-600" : "text-sm text-red-600"
                                        }>
                                            {instance?.Active ? "Active" : "Inactive"}
                                        </span>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                                            {instance?.Shards?.join(", ")} 
                                        </span>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {#if instance}
                                            {getClusterGuildCount(instance)}
                                        {:else}
                                            Unknown
                                        {/if}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {#if instance}
                                            {getClusterUserCount(instance)}
                                        {:else}
                                            Unknown
                                        {/if}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {moment(instance?.StartedAt).fromNow()}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {moment(instance?.LastChecked).fromNow()}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                        <button on:click={() => {
                                            openCluster = instance?.ClusterID;
                                            showModal = true;
                                        }} class="text-indigo-600 hover:text-indigo-900">View</button>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    </div>
{:catch err}
    <Message type="error">Error loading cluster data: {err}</Message>
{/await}