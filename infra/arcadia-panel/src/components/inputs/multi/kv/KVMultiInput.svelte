<script lang="ts">
	import Label from '../../Label.svelte';
	import ButtonReact from '../Button.svelte';
	import DangerButton from '../DangerButton.svelte';
	import KvMultiInputElement from './KVMultiInputElement.svelte';

	export let id: string;
	export let values: [string, string][] = [];
	export let title: string;
	export let label: string = title;
	export let placeholder: string;
	export let minlength: number;
	export let showErrors: boolean = false;
	export let required: boolean = true;
	export let disabled: boolean = false;

	const deleteValue = (i: number) => {
		values = values.filter((_, index) => index !== i);
	};

	const addValue = (i: number) => {
		values = [...values.slice(0, i + 1), '', ...values.slice(i + 1)] as [string, string][];
	};
</script>

<Label {id} {label} />
<div {id}>
	{#each values as value, i}
		<KvMultiInputElement
			{title}
			{placeholder}
			{minlength}
			{showErrors}
			{i}
			bind:value
			{required}
			{disabled}
		/>
		<DangerButton onclick={() => deleteValue(i)}>Delete</DangerButton>
		<ButtonReact onclick={() => addValue(i)}>Add</ButtonReact>
	{/each}

	{#if values.length == 0}
		<ButtonReact onclick={() => addValue(-1)}>New {title}</ButtonReact>
	{/if}
</div>
