<script lang="ts">
	import type { QuickAction, Option } from './QuickAction';

	export let subMenuOpened: boolean = false;

	export let index: number;
	export let action: QuickAction;
	export let actionsLength: number;

	let options: Option[] | null = null;

	$: if (action.options) {
		options = action.options();
	}
</script>

{#if options}
	<button
		class="w-full border border-themable-700/50 p-3 text-xl bg-black hover:bg-slate-800 {index === 0
			? 'rounded-t-md'
			: ''} {index === actionsLength - 1 ? 'rounded-b-md' : ''}"
		on:click={() => {
			subMenuOpened = !subMenuOpened;
		}}
	>
		{action.name}
		<small class="text-sm text-gray-400 block">{action.description}</small>
	</button>

	{#if subMenuOpened}
		{#each options as a}
			{#if a.enabled()}
				<a
					class="block text-center w-full border border-red-300 border-opacity-50 p-3 text-xl bg-black hover:bg-slate-800"
					href={a.link}
				>
					{a.name}
					<small class="text-sm text-gray-400 block">{a.description}</small>
				</a>
			{/if}
		{/each}
	{/if}
{:else}
	<a
		class="block text-center w-full border border-themable-700/50 p-3 text-xl bg-black hover:bg-slate-800 {index ===
		0
			? 'rounded-t-md'
			: ''} {index === actionsLength - 1 ? 'rounded-b-md' : ''}"
		href={action.link}
	>
		{action.name}
		<small class="text-sm text-gray-400 block">{action.description}</small>
	</a>
{/if}
