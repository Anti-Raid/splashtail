<script>
	import { makeSharedRequest, opGetClusterHealth } from "$lib/fetch/ext";
	import ClusterHealth from "../../../components/common/ClusterHealth.svelte";
import Message from "../../../components/Message.svelte";

</script>
{#await makeSharedRequest(opGetClusterHealth)}
    <Message type="loading">
        Loading cluster data...
    </Message>
{:then data}
    <ClusterHealth instanceList={data} />
{:catch error}
    <Message type="error">
        {error?.message?.toString() || error?.toString() || "Unknown error"}
    </Message>
{/await}
