<!--Showcase/debug all cluster related things-->
<script>
	import { makeSharedRequest, opGetClusterHealth } from "$lib/fetch/ext";
	import ClusterHealth from "../../../components/common/ClusterHealth.svelte";
	import CommandList from "../../../components/common/CommandList.svelte";
import Message from "../../../components/Message.svelte";

</script>
{#await makeSharedRequest(opGetClusterHealth)}
    <Message type="loading">
        Loading cluster data...
    </Message>
{:then data}
    <h1 class="text-2xl font-semibold">Cluster Health</h1>
    <ClusterHealth instanceList={data} />
    <div class="mb-6" /> <!--TODO: Experiment with this a bit more-->
    <h1 class="text-2xl font-semibold">Command List</h1>
    <CommandList instanceList={data} />
{:catch error}
    <Message type="error">
        {error?.message?.toString() || error?.toString() || "Unknown error"}
    </Message>
{/await}
