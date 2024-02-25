<script lang="ts">
	import TabButton from './TabButton.svelte';

	interface Tab {
		id: string;
		label: string;
	}

	export let tabs: Tab[];

	export let visibleTab: string;

	$: {
		if (!visibleTab && tabs) visibleTab = tabs[0].id;
	}
</script>

{#each tabs as tab}
	<TabButton
		visible={tab.id == visibleTab}
		text={tab.label}
		onClick={() => (visibleTab = tab.id)}
	/>
{/each}
