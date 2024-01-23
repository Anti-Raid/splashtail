<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { CdnAssetItem } from '$lib/generated/arcadia/CdnAssetItem';
	import { cdnStateStore, cdnDataStore } from './cdnStateStore';
	import FileModal from './FileModal.svelte';
	import { prettifyBytes } from '$lib/fileutils';

	export let scope: string;
	export let index: number;

	let file: CdnAssetItem;

	const fileIcon = (): string => {
		// Directory
		if (file.is_dir) {
			return 'mdi:folder';
		}

		// No file extension
		if (!file.name.includes('.')) {
			return 'mdi:file';
		}

		return 'bx:file';
	};

	let showFileModal: boolean = false;

	$: file = $cdnDataStore.files[index];
</script>

{#key file}
	<button
		class={`rounded-t-md w-full text-left block text-white hover:bg-slate-800 p-4 ${
			index < $cdnDataStore.files.length - 1 ? 'border-b' : 'rounded-md'
		}`}
		on:click={() => {
			if (file.is_dir) {
				if ($cdnStateStore.path) {
					$cdnStateStore.path = `${$cdnStateStore.path}/${file.name}`;
				} else {
					$cdnStateStore.path = file.name;
				}
			} else {
				showFileModal = true;
			}
		}}
	>
		<Icon icon={fileIcon()} class="text-2xl inline-block align-bottom" />
		{file.name}
		<div class="mt-2 text-gray-400"><span class="font-semibold">Location: </span>{file.path}</div>
		<div class="mt-2 text-gray-400">
			<span class="font-semibold">Size: </span>{prettifyBytes(file.size)}
		</div>
	</button>
{/key}

{#if showFileModal}
	<FileModal bind:showModal={showFileModal} {file} {scope} />
{/if}
