<script lang="ts">
	import BoolInput from '../inputs/BoolInput.svelte';
	import InputNumber from '../inputs/InputNumber.svelte';
	import InputText from '../inputs/InputText.svelte';
	import InputTextArea from '../inputs/InputTextArea.svelte';
	import ExtraLinks from '../inputs/multi/extralinks/ExtraLinks.svelte';
	import KvMultiInput from '../inputs/multi/kv/KVMultiInput.svelte';
	import MultiInput from '../inputs/multi/simple/MultiInput.svelte';
	import Select from '../inputs/select/Select.svelte';
	import FileUploadElement from './FileUploadElement.svelte';
	import type { Field, Schema } from './types';

	export let field: Field<any>;

	export let data: { [key: string]: any };
	export let fileData: { [key: string]: File };
</script>

{#if field.renderMethod != 'none'}
	{#if field.type == 'text'}
		<InputText
			id={field.id}
			bind:value={data[field.id]}
			label={field.label}
			placeholder={field.helpText}
			minlength={0}
			showErrors={false}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'textarea'}
		<InputTextArea
			id={field.id}
			bind:value={data[field.id]}
			label={field.label}
			placeholder={field.helpText}
			minlength={0}
			showErrors={false}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'text[]'}
		<MultiInput
			id={field.id}
			title={field.label}
			label={field.arrayLabel ? field.arrayLabel : field.label}
			bind:values={data[field.id]}
			placeholder={field.helpText}
			minlength={0}
			showErrors={false}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'text[kv]'}
		<KvMultiInput
			id={field.id}
			title={field.label}
			label={field.arrayLabel ? field.arrayLabel : field.label}
			bind:values={data[field.id]}
			placeholder={field.helpText}
			minlength={0}
			showErrors={false}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'ibl:link'}
		<ExtraLinks
			id={field.id}
			title={field.label}
			label={field.arrayLabel ? field.arrayLabel : field.label}
			bind:values={data[field.id]}
			placeholder={field.helpText}
			minlength={0}
			showErrors={false}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'text[choice]'}
		<Select
			id={field.id}
			label={field.label}
			bind:value={data[field.id]}
			choices={field.selectMenuChoices || []}
			description={field.helpText}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'number'}
		<InputNumber
			id={field.id}
			bind:value={data[field.id]}
			label={field.label}
			placeholder={field.helpText}
			minlength={0}
			showErrors={false}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'boolean'}
		<BoolInput
			id={field.id}
			bind:value={data[field.id]}
			label={field.label}
			description={field.helpText}
			required={field.required}
			disabled={field.disabled}
		/>
	{:else if field.type == 'file'}
		<FileUploadElement bind:outputFile={fileData[field.id]} {field} cap="update" />
	{:else}
		<p class="text-red-500">Unsupported field type {field.type}: {JSON.stringify(field)}</p>
	{/if}
{/if}

{#if ['custom', 'custom[html'].includes(field.renderMethod) && field.disabled}
	{#if field.renderMethod == 'custom'}
		{#if field?.customRenderer}
			{#await field?.customRenderer('update', data)}
				<p class="animate-pulse">Loading {field.id}</p>
			{:then data}
				{data}
			{/await}
		{:else}
			{data[field.id]}
		{/if}
	{:else if field.renderMethod == 'custom[html]'}
		{#if field?.customRenderer}
			{#await field?.customRenderer('update', data)}
				<p class="animate-pulse">Loading {field.id}</p>
			{:then data}
				{@html data}
			{/await}
		{:else}
			{@html data[field.id]}
		{/if}
	{/if}
{/if}
