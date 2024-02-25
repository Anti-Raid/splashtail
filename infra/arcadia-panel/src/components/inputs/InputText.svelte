<script lang="ts">
	import Label from './Label.svelte';

	export let id: string;
	export let label: string;
	export let placeholder: string;
	export let minlength: number;
	export let value: string = '';
	export let showErrors: boolean = true;
	export let description: string = '';
	export let inpClass: string = 'mb-4';
	export let required: boolean = true;
	export let disabled: boolean = false;

	let success: boolean | null = null;

	let errorMsg = '';

	function checkLength() {
		if (!showErrors) return;

		if (!value) {
			success = null;
			return;
		}

		if (value.length < minlength) {
			success = false;
			errorMsg = `Must be at least ${minlength} characters long`;
		} else {
			success = true;
		}
	}
</script>

<div class={inpClass}>
	<Label {id} {label} />
	{#if description}
		<p class="text-md mb-2 opacity-80">{@html description}</p>
	{/if}

	<input
		on:change={checkLength}
		{minlength}
		type="text"
		{id}
		class={disabled
			? 'w-full mx-auto mt-2 flex bg-black bg-opacity-30 text-gray-100 rounded-xl border border-themable-200 opacity-75 py-4 px-6 disabled cursor-not-allowed'
			: 'w-full mx-auto mt-2 flex transition duration-200 hover:bg-slate-900 bg-black bg-opacity-100 text-white focus:text-themable-400 rounded-xl border border-themable-200 focus:border-themable-400 focus:outline-none py-4 px-6'}
		{placeholder}
		{required}
		{disabled}
		aria-disabled={disabled}
		aria-required={required}
		bind:value
	/>

	{#if success == true}
		<p class="mt-2 text-sm text-green-600 dark:text-green-500">
			<span class="font-medium">Looks good!</span>
		</p>
	{:else if success == false}
		<p class="mt-2 text-sm text-red-600 dark:text-red-500">
			<span class="font-medium">{errorMsg}</span>
		</p>
	{/if}

	<slot />
</div>
