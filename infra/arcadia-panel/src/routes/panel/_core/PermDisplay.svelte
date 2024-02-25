<script lang="ts">
	import ListItem from '../../../components/ListItem.svelte';

	export let perm: string;
</script>

{#if perm.startsWith('~')}
	<ListItem className="text-red-600 line-through ml-3">{perm}</ListItem>
{:else if perm.endsWith('.*')}
	<ListItem className="text-green-600 font-bold ml-3">{perm}</ListItem>
{:else}
	<ListItem className="ml-3">{perm}</ListItem>
{/if}
