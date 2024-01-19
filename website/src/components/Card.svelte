<script lang="ts">
	interface ButtonType {
		name: String;
		click: () => void;
	}

	export let name: string;
	export let title: string;
	export let description: string;
	export let image: string | null;
	export let button: ButtonType | null;
</script>

<div
	id={name}
	class="p-6 max-w-sm bg-white rounded-md border border-gray-200 shadow-md dark:bg-gray-800 dark:border-gray-700"
>
	<h5 class="mb-2 text-2xl font-bold tracking-light text-gray-900 dark:text-white">{title}</h5>

	{#if image}
		<!-- svelte-ignore a11y-img-redundant-alt -->
		<img class="h-20 rounded-half" src={image} height="80px" width="80px" alt="Card Image" />
	{/if}

	<p class="mb-3 font-normal text-gray-700 dark:text-gray-400">{description}</p>

	{#if button}
		<button
			on:click={button.click}
			class="inline-flex items-center py-2 px-3 text-sm font-medium text-center text-white bg-emerald-700 rounded-md focus:ring-4 focus:outline-none focus:ring-emerald-300 dark:bg-emerald-700 dark:focus:ring-emerald-800"
			>{button.name}</button
		>
	{/if}
</div>

<div class="p-3" />
