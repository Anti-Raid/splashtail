<script lang="ts">
	import { InstanceList } from "$lib/generated/mewld/proc";
	import NavButton from "../NavButton.svelte";

    export let instanceList: InstanceList;

    // Which clister is currently open for viewing
    let openCluster: number | undefined = 0; // Defaults to 0
</script>

<!--Cluster Menu at the right of the page-->
<article class="command-list-article">
    <small class="text-red-600 word-wrap block mb-1">
        Different clusters may have different available modules due to outages, A/B testing and other reasons.
    </small>
    <section class="command-list flex flex-grow">
        <nav class="cluster-map flex-none border-r border-slate-500 w-28">
            {#each instanceList?.Instances as instance}
                <NavButton 
                    current={openCluster == instance?.ClusterID} 
                    title={`Cluster ${instance?.ClusterID}`} 
                    onClick={() => openCluster = instance?.ClusterID}
                    extClass="block mb-2"
                />
            {/each}
        </nav>
        <div class="cluster-map-content flex-1 px-2">
            Content
        </div>
    </section>
</article>