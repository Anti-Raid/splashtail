<script lang="ts">
	import logger from '$lib/logger';

	// The index that selected should be set to when a bot is selected
	export let index: number;

	export let selected: number | null;

	let checked: boolean;

	$: checked = selected === index;
</script>

<button
	class="flex justify-center hover:opacity-75 focus:outline-none mt-3 w-full rounded-lg bg-black/90 p-3 text-center text-white"
	on:click={() => {
		if (checked === false) {
			logger.info('Select', 'Selected bot', index);
			selected = index;
		} else {
			logger.info('Select', 'Deselected bot', index);
			selected = null;
		}
	}}
>
	{#if checked}
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			stroke-width="1.5"
			stroke="currentColor"
			class="w-6 h-6 mr-1"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
			/>
		</svg>
	{:else}
		<svg
			xmlns="http://www.w3.org/2000/svg"
			fill="none"
			viewBox="0 0 24 24"
			stroke-width="1.5"
			stroke="currentColor"
			class="w-6 h-6 mr-1"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
			/>
		</svg>
	{/if}

	Select{checked ? 'ed' : ''}
</button>
