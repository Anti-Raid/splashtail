<script lang="ts">
	import Label from '../Label.svelte';
	import type { SMOption } from './select';

	export let id: string;
	export let label: string;
	export let description: string = '';
	export let choices: SMOption[];
	export let required: boolean = true;
	export let disabled: boolean = false;
	export let inpClass: string = 'mb-4';
	export let defaultLabel: string = 'Select an action';

	export let value: string = '';
</script>

<div class={inpClass}>
	<Label {id} {label} />
	{#if description}
		<p class="text-md mb-2 opacity-80">{@html description}</p>
	{/if}
	<select
		{id}
		class={disabled
			? 'w-full mx-auto mt-2 flex bg-black bg-opacity-50 text-gray-100 rounded-xl border border-white/10 focus:outline-none py-4 px-6'
			: 'w-full mx-auto mt-2 flex transition duration-200 hover:bg-slate-900 bg-black bg-opacity-100 text-white focus:text-themable-400 rounded-xl border border-themable-200 focus:border-themable-400 focus:outline-none py-4 px-6'}
		bind:value
		{required}
		{disabled}
		aria-disabled={disabled}
		aria-required={required}
	>
		<option value="">{defaultLabel}</option>
		{#each choices as choice}
			<option id={choice.id} value={choice.value}>{choice.label}</option>
		{/each}
	</select>
</div>
