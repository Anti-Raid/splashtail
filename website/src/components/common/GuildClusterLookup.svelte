<!--Simple component that provides a guild id as input and returns cluster/shard info of the guild-->
<script lang="ts">
	import { InstanceList } from "$lib/generated/mewld/proc";
	import { getClusterOfShard, getShardIDFromGuildID } from "$lib/mewext/mewext";
	import logger from "$lib/ui/logger";
    import InputText from "../inputs/InputText.svelte";

    export let instanceList: InstanceList;

    export let guildId: string = "";

    interface ExpectedInfo {
        cluster: number;
        shard: number;
    }

    export let expectedInfo: ExpectedInfo | null = null

    $: {
        if(guildId) {
            let [shardId, err] = getShardIDFromGuildID(guildId, instanceList?.ShardCount);

            if(!err) {
                let clusterId = getClusterOfShard(shardId, instanceList?.Map);

                expectedInfo = {
                    cluster: clusterId,
                    shard: shardId
                }
            } else {
                logger.error("GuildClusterLookup", "Failed to get shard id from guild id", err)
            }
        }
    }
</script>

<InputText
    id="guildid-lookup-input"
    label="Server ID"
    placeholder="Guild ID"
    minlength={0}
    showErrors={false}
    bind:value={guildId}
/>

{#if expectedInfo}
    <div class="mt-2">
        <span class="font-semibold">Cluster:</span> {expectedInfo.cluster}
    </div>
    <div class="mt-2">
        <span class="font-semibold">Shard:</span> {expectedInfo.shard}
    </div>
{/if}