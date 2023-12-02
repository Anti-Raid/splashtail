<script lang="ts">
  import ActionLogEvent from "$lib/ActionLogEvent.svelte";
	import { onMount } from "svelte";
    
    const basePath = `/mewld`

    export let instances: any;
    export let actionLogs: any;

    let clusterInfo: { [key: string]: any } = {};

    async function getInstanceData() {
        let res = await fetch(`${basePath}/api/instance-list`);

        if (!res.ok) {
            throw new Error(`Could not load clusters: ${await res.text()}`)
        }

        let aRes = await fetch(`${basePath}/api/action-logs`);

        if (!aRes.ok) {
            throw new Error(`Could not load action logs: ${await aRes.text()}`)
        }

        actionLogs = await aRes.json();
        instances = await res.json();
    }

    async function renderClusterExt(cid: number) {
        let res = await fetch(`${basePath}/api/cluster-health?cid=${cid}`);

        if (!res.ok) {
            alert(await res.text());
            return
        }

        let cluster: { [key: string]: any } = await res.json();

        clusterInfo[cid] = cluster;

        clusterInfo = clusterInfo
    }

    async function restartMewld() {
        let p = prompt("Are you sure you want to restart mewdl (Yes/No)");

        if(p != "Yes") {
            return;
        }

        let res = await fetch(`${basePath}/api/redis/pub`, {
            method: "POST",
            body: JSON.stringify({
                "scope": "launcher",
                "action": "restartproc"
            })
        });
        if (res.ok) {
            alert("Restarting mewld");
        } else {
            alert("Failed to restart mewld");
        }
    }

    async function rollRestart() {
        let p = prompt("Are you sure you want to roll restart (Yes/No)");

        if(p != "Yes") {
            return;
        }

        let res = await fetch(`${basePath}/api/redis/pub`, {
            method: "POST",
            body: JSON.stringify({
                "scope": "launcher",
                "action": "rollingrestart",
            })
        });
        if (res.ok) {
            alert("Roll restarting");
        } else {
            alert("Failed to roll restart");
        }
    }

    async function restartCluster(id: number) {
        let p = prompt("Are you sure you want to restart mewdl (Yes/No)");

        if(p != "Yes") {
            return;
        }

        let res = await fetch(`${basePath}/api/redis/pub`, {
            method: "POST",
            body: JSON.stringify({
                "scope": "launcher",
                "action": "restart",
                "args": {
                    id: id
                }
            })
        });
        if (res.ok) {
            alert("Restarting mewld");
        } else {
            alert("Failed to restart mewld");
        }
    }

    onMount(() => {
        setInterval(async () => {
            let aRes = null;
            try {
                aRes = await fetch(`${basePath}/api/action-logs`);
            } catch (err) {
                console.log(err)
                return
            }

            if (!aRes.ok) {
                console.error(await aRes.text());
                return
            }

            actionLogs = await aRes.json();
        }, 5000)
    })
</script>

{#await getInstanceData()}
    <p>Loading...</p>
{:then _}
    <h2>Action Logs</h2>

    <div id="action-logs">
    <ActionLogEvent data={actionLogs} />
    </div>

    <h2>Clusters</h2>

    <div id="cluster-list">
    {#each instances.Map as cluster, i}
        <div role="button" tabindex="-1" class="cluster" on:click={() => renderClusterExt(cluster.ID)} on:keydown={(event) => {
            if (event.key == "Enter") {
                renderClusterExt(cluster.ID)
            }
        }}>
            <p class="cluster-para">{cluster.ID}. {cluster.Name}</p>
            <div class="cluster-pane clickable" id="c-{cluster.ID}">
                <strong>Session ID:</strong> {instances.Instances[i].SessionID}<br/>
                <strong>Shards:</strong> {instances.Instances[i].Shards.join(', ')}<br/>
                <strong>Started At:</strong> {instances.Instances[i].StartedAt}<br/>
                <strong>Active:</strong> {instances.Instances[i].Active}<br/>
                
                <div id="c-{cluster.ID}-health" style="margin-bottom: 10px">
                    {#if !clusterInfo[cluster.ID]}
                        <span style="font-weight: bold">Click here to manage this cluster and fetch health information about it</span>
                    {:else}
                        <strong>Locked:</strong> {clusterInfo[cluster.ID].locked}<br/>
                        {#each clusterInfo[cluster.ID].health as shard, i}
                            <h3>Shard {i}</h3>
                            <strong>Latency: {Math.round(shard.latency)} ms</strong><br/>
                            <strong>Guilds: {shard.guilds}</strong><br/>
                        {/each}
                        <button on:click={() => restartCluster(cluster.ID)}>Restart Cluster</button>
                    {/if}
                </div>    
            </div>
        </div>
    {/each}
    </div>

    <h2>Advanced</h2>

    <details>
    <summary>instance-list JSON</summary>
        <code>
        {JSON.stringify(instances)}
        </code>
    </details>
    <details>
    <summary>action-logs JSON</summary>
        <code>
        {JSON.stringify(actionLogs)}
        </code>
    </details>

    <button on:click={() => rollRestart()}>Rolling Restart All Clusters</button>

    <button on:click={() => restartMewld()}>Restart Mewld (DANGEROUS)</button>
{:catch error}
    <p style="color: red">{error.message}</p>
{/await}
