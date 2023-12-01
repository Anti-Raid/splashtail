<script lang="ts">
    export let data: any[];

    function showAld(i: number) {
        let el = (document.querySelector(`#ald-${i}`) as HTMLInputElement)

        if (el.style.display == "none") {
            el.style.display = "initial"
        } else {
            el.style.display = "none"
        }
    }
</script>

{#each data as logentry, i}
    {#if logentry.event == "shards_launched"}
        {#if logentry.cluster != undefined}
            <p class="clickable" on:click={() => {showAld(i)}} id="alp-{i}">Cluster <span class="cluster-id">{logentry.cluster}</span> launched successfully</p>
        {/if}
    {:else if logentry.event == "rolling_restart"}
        <p class="clickable" on:click={() => {showAld(i)}} id="alp-{i}">Rolling restart begun (instance wide)</p>
    {:else if logentry.event == "ping_failure"}
        <p class="clickable" on:click={() => {showAld(i)}} id="alp-{i}">Cluster <span class="cluster-id">{logentry.id}</span> failed to respond and is/was restarted</p>
    {:else}
        <p class="clickable" on:click={() => {showAld(i)}} id="alp-{i}">Unknown event: {JSON.stringify(logentry)}</p>
    {/if}

    <div class="ald" style="display: none" id="ald-{i}">
        <strong>Timestamp: </strong><span class="ts">{new Date(logentry.ts/1000)}</span><br/>

        {#if logentry.event == "shards_launched"}
            <strong>Cluster:</strong> {logentry.cluster}<br/><strong>From shard:</strong> {logentry.from}<br/><strong>To shard:</strong> {logentry.to}
        {:else if logentry.event == "ping_failure"}
        <strong>Cluster:</strong> {logentry.id}
        {/if}
    </div>
{/each}

<style>
    .cluster-id {
        font-weight: bold;
    }
</style>