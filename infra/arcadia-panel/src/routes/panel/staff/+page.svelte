<script lang="ts">
	import QuickMenuOption from '../_core/QuickMenuOption.svelte';
	import { staffQuickActions } from './quickActions';
</script>

<h1 class="text-3xl font-semibold">Staff Management</h1>

<div class="bg-gray-500 bg-opacity-10 border border-themable-700/50 rounded-lg mt-3 md:p-4">
	{#each staffQuickActions as action, index}
		{#if action.enabled()}
			<QuickMenuOption {index} {action} actionsLength={staffQuickActions.length} />
		{/if}
	{/each}
</div>
