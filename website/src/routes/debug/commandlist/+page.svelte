<script>
	import { makeSharedRequest, opGetClusterHealth } from "$lib/fetch/ext";
import CommandList from "../../../components/common/CommandList.svelte";
import Message from "../../../components/Message.svelte";

</script>
{#await makeSharedRequest(opGetClusterHealth)}
    <Message type="loading">
        Loading cluster data...
    </Message>
{:then data}
    <CommandList instanceList={data} />
{:catch error}
    <Message type="error">
        {error?.message?.toString() || error?.toString() || "Unknown error"}
    </Message>
{/await}
