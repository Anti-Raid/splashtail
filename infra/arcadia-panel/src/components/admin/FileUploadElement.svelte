<script lang="ts">
	import Icon from '@iconify/svelte';
	import FileUpload from '../inputs/FileUpload.svelte';
	import type { Field, Capability } from './types';

	export let field: Field<any>;
	export let cap: Capability;

	let file: File;
	let fileMimeType: string;
	export let fileUploaded: boolean = false;

	let filePreviewBox: HTMLDivElement;

	export let outputFile: File | undefined = undefined;

	$: {
		if (fileUploaded) outputFile = file;
	}
</script>

<FileUpload
	id={field.id}
	label={field.label}
	bind:file
	bind:fileMimeType
	bind:fileUploaded
	acceptableTypes={field.fileUploadData?.acceptableMimeTypes || []}
/>

{#if fileUploaded && file}
	{#if field.fileUploadData?.renderPreview}
		<p class="font-semibold">File Preview ({fileMimeType.split('/')[1]})</p>

		{#await field.fileUploadData?.renderPreview(cap, file, filePreviewBox)}
			<Icon icon="mdi:loading" class="inline animate-spin text-2xl" />
			<span class="text-xl">Loading Preview</span>
		{:catch err}
			<p class="text-red-500">{err?.toString()}</p>
		{/await}
		<div bind:this={filePreviewBox} />
	{:else}
		<p class="font-semibold">File Uploaded [{file.name}] ({fileMimeType.split('/')[1]})</p>
	{/if}
{/if}
