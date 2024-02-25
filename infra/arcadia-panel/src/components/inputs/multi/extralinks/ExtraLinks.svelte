<!--Wrapper around kv multi-input to provide a Link[]-->
<script lang="ts">
	import type { Link } from '$lib/generated/arcadia/Link';
	import { onMount } from 'svelte';
	import KvMultiInput from '../kv/KVMultiInput.svelte';
	import logger from '$lib/logger';

	export let id: string = 'extra-links';
	export let title: string = 'Links';
	export let label = title;
	export let values: Link[];
	export let placeholder: string = 'Link';
	export let minlength: number = 5;
	export let showErrors: boolean = false;
	export let required: boolean = true;
	export let disabled: boolean = false;

	let internalValues: [string, string][] = values?.map(({ name, value }) => [name, value]) || [];

	$: {
		values = internalValues.map(([k, v]) => ({ name: k, value: v }));
		logger.info('ExtraLinks.onMount', values, internalValues);
	}
</script>

<KvMultiInput
	{id}
	bind:values={internalValues}
	{title}
	{label}
	{placeholder}
	{minlength}
	{showErrors}
	{required}
	{disabled}
/>
