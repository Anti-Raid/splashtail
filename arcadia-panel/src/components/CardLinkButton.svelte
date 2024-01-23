<script lang="ts">
	export let link: string;
	export let disabled: boolean = false;
	export let target: string = '_self';

	export let showArrow: boolean = true;
	export let double: boolean = true;
</script>

{#if disabled}
	<button
		disabled={true}
		aria-disabled={true}
		class="{!double
			? 'ml-3'
			: ''} hover:opacity-75 focus:outline-none mt-3 w-1/2 rounded-lg bg-black/90 p-3 text-center text-white"
	>
		<slot />

		{#if showArrow}
			<svg
				aria-hidden="true"
				class="ml-2 -mr-1 w-4 h-4"
				fill="currentColor"
				viewBox="0 0 20 20"
				xmlns="http://www.w3.org/2000/svg"
				><path
					fill-rule="evenodd"
					d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z"
					clip-rule="evenodd"
				/></svg
			>
		{/if}
	</button>
{:else}
	<a
		{target}
		href={link}
		class="{!double
			? 'hover:opacity-75 focus:outline-none mt-3 w-full rounded-lg bg-black/90 p-4 text-center text-white'
			: ''} hover:opacity-75 focus:outline-none mt-3 w-1/2 rounded-lg bg-black/90 p-4 text-center text-white"
	>
		<slot />

		{#if showArrow}
			<svg
				aria-hidden="true"
				class="ml-2 -mr-1 w-4 h-4"
				fill="currentColor"
				viewBox="0 0 20 20"
				xmlns="http://www.w3.org/2000/svg"
				><path
					fill-rule="evenodd"
					d="M10.293 3.293a1 1 0 011.414 0l6 6a1 1 0 010 1.414l-6 6a1 1 0 01-1.414-1.414L14.586 11H3a1 1 0 110-2h11.586l-4.293-4.293a1 1 0 010-1.414z"
					clip-rule="evenodd"
				/></svg
			>
		{/if}
	</a>
{/if}
