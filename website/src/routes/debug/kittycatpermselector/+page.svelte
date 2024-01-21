<script lang="ts">
	import KittycatPermSelector from "../../../components/dashboard/KittycatPermSelector.svelte";

    let validPerm: boolean = false;
</script>

<div class="px-4">
    <KittycatPermSelector perm="server_backups.create" bind:validPerm />

    <small class="text-white font-semibold">validPerm: {validPerm}</small>
</div>