<script lang="ts">
	import Icon from '@iconify/svelte';

	export let double: boolean = false;
	export let disabled: boolean = false;
	export let link: string | null = null;
	export let onClick: () => void = () => {};
	export let icon: string | null = null;

	let extClass = '';
	let className = '';

	$: {
		if (disabled) {
			extClass += 'opacity-50 cursor-not-allowed ';
		} else {
			extClass += 'hover:opacity-75 focus:outline-none ';
		}

		if (double) {
			className = 'mt-3 w-1/2 rounded-lg bg-black/90 p-4 text-center text-white';
		} else {
			className = 'mt-3 w-full block rounded-lg bg-black/90 p-4 text-center text-white';
		}
	}
</script>

{#if disabled}
	<button
		on:click={(e) => {
			e.preventDefault();
			e.stopPropagation();
		}}
		class={className}
		disabled={true}
		aria-disabled={true}
	>
		{#if icon}
			<Icon inline={true} {icon} class="text-white mr-1 inline-block" />
		{/if}
		<slot />
	</button>
{/if}

{#if link}
	<a href={link} class={className}>
		{#if icon}
			<Icon inline={true} {icon} class="text-white mr-1 inline-block" />
		{/if}
		<slot />
	</a>
{:else}
	<button
		on:click={(e) => {
			if (e && e.preventDefault) e.preventDefault();
			onClick();
		}}
		class={className}
	>
		{#if icon}
			<Icon inline={true} {icon} class="text-white mr-1 inline-block" />
		{/if}
		<slot />
	</button>
{/if}
