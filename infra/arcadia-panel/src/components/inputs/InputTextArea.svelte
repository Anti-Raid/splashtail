<script lang="ts">
	import Label from './Label.svelte';

	export let id: string;
	export let label: string;
	export let placeholder: string;
	export let minlength: number;
	export let value: string = '';
	export let showErrors: boolean = true;
	export let description: string = '';
	export let inpClass: string = 'mb-6';
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
		<span class="text-md text-gray-500 dark:text-gray-400 mb-2">{{ description }}</span>
	{/if}
	<textarea
		on:change={checkLength}
		{minlength}
		{id}
		class={disabled
			? 'w-full mx-auto mt-2 flex bg-black bg-opacity-30 text-gray-100 rounded-xl border border-themable-200 opacity-75 py-4 px-6 disabled'
			: 'w-full mt-2 flex transition duration-200 hover:bg-opacity-50 bg-black bg-opacity-30 text-white focus:text-themable-400 rounded-xl border border-themable-200 focus:border-themable-400 focus:outline-none py-4 px-6'}
		{placeholder}
		{required}
		{disabled}
		aria-disabled={disabled}
		aria-required={required}
		rows={10}
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
