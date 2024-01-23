<!--https://github.com/InfinityBotList/sysmanage-web/blob/main/core/frontend/src/lib/components/ObjectRender.svelte-->
<script lang="ts">
	export let object: any;
</script>

{#if object}
	{#each Object.entries(object) as [key, value]}
		{#if typeof value === 'object' && value !== null}
			<span class="font-semibold">{key}:</span>
			<div class="ml-4">
				<svelte:self object={value} />
			</div>
		{:else}
			<p>
				<span class="font-semibold">{key}:</span>
				{value}
			</p>
		{/if}
	{/each}
{:else}
	<p class="text-red-500">No information can be provided about this entity</p>
{/if}
