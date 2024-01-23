<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState } from '$lib/panelAuthState';
	import Icon from '@iconify/svelte';
	import Modal from '../../../../../components/Modal.svelte';
	import type { CdnAssetItem } from '$lib/generated/arcadia/CdnAssetItem';
	import ErrorComponent from '../../../../../components/Error.svelte';
	import type { CdnScopeData } from '$lib/generated/arcadia/CdnScopeData';
	import { error, success } from '$lib/toast';
	import InputText from '../../../../../components/inputs/InputText.svelte';
	import ButtonReact from '../../../../../components/button/ButtonReact.svelte';
	import { Color } from '../../../../../components/button/colors';
	import BoolInput from '../../../../../components/inputs/BoolInput.svelte';
	import { cdnDataStore, cdnStateStore } from './cdnStateStore';
	import { loadData, renderPreview } from '../../../../../lib/fileutils';

	export let showModal: boolean; // boolean, whether or not the modal is shown or not
	export let file: CdnAssetItem;
	export let scope: string;

	let previewBox: HTMLDivElement;

	// Get scope list
	const getScope = async () => {
		let res = await panelQuery({
			ListCdnScopes: {
				login_token: $panelAuthState?.loginToken || ''
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to get CDN scopes: ${err}`);
		}

		let scopes: { [key: string]: CdnScopeData } = await res.json();
		let scopeData = scopes[scope];

		if (!scopeData) {
			throw new Error(`Failed to find scope ${scope}`);
		}

		return {
			scopes,
			scopeData
		};
	};

	let copyFilePath: string;
	let copyFilePaneOpen: boolean = false;
	let copyFileSettingOverwrite: boolean = false;
	let copyFileSettingDeleteOriginal: boolean = true;
	const copyFile = async () => {
		if (!copyFilePath) {
			error('Please enter a new file path');
			return false;
		}

		if (copyFilePath?.startsWith('/')) {
			copyFilePath = copyFilePath.slice(1);
		}

		let path = file.path;

		if (path.startsWith('/')) {
			path = path.slice(1);
		}

		// Remove filename from path
		let pathSplit = path.split('/');
		pathSplit.pop();

		// Join path back together
		path = pathSplit.join('/');

		let res = await panelQuery({
			UpdateCdnAsset: {
				login_token: $panelAuthState?.loginToken || '',
				cdn_scope: scope || '',
				path,
				name: file.name,
				action: {
					CopyFile: {
						overwrite: copyFileSettingOverwrite,
						delete_original: copyFileSettingDeleteOriginal,
						copy_to: copyFilePath
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			error(`Failed to modify file: ${err}`);
			return false;
		}

		$cdnStateStore.triggerRefresh += 1;

		success('Successfully modified file');

		return true;
	};

	const deleteFile = async () => {
		let path = file.path;

		if (path.startsWith('/')) {
			path = path.slice(1);
		}

		// Remove filename from path
		let pathSplit = path.split('/');
		pathSplit.pop();

		// Join path back together
		path = pathSplit.join('/');

		let res = await panelQuery({
			UpdateCdnAsset: {
				login_token: $panelAuthState?.loginToken || '',
				cdn_scope: scope || '',
				path,
				name: file.name,
				action: 'Delete'
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to delete file: ${err}`);
		}

		$cdnStateStore.triggerRefresh += 1;
	};

	enum ButtonState {
		Idle,
		Loading,
		Success,
		Error
	}

	interface ButtonStateData {
		state: ButtonState;
		error: any;
		getIcon: () => string;
		getIconClass: () => string;
	}

	let downloadBtnState: ButtonStateData = {
		state: ButtonState.Idle,
		error: null,
		getIcon: () => {
			if (downloadBtnState?.state == ButtonState.Idle) {
				return 'mdi:download';
			} else if (downloadBtnState?.state == ButtonState.Loading) {
				return 'mdi:loading';
			} else if (downloadBtnState?.state == ButtonState.Success) {
				return 'mdi:check';
			} else if (downloadBtnState?.state == ButtonState.Error) {
				return 'mdi:alert-circle';
			}

			return 'mdi:download';
		},
		getIconClass: () => {
			if (downloadBtnState?.state == ButtonState.Idle) {
				return 'text-xl inline-block';
			} else if (downloadBtnState?.state == ButtonState.Loading) {
				return 'text-xl inline-block animate-spin';
			} else if (downloadBtnState?.state == ButtonState.Success) {
				return 'text-xl inline-block';
			} else if (downloadBtnState?.state == ButtonState.Error) {
				return 'text-xl inline-block';
			}

			return 'text-xl inline-block';
		}
	};

	let deleteBtnState: ButtonStateData = {
		state: ButtonState.Idle,
		error: null,
		getIcon: () => {
			if (deleteBtnState?.state == ButtonState.Idle) {
				return 'mdi:trash-can-outline';
			} else if (deleteBtnState?.state == ButtonState.Loading) {
				return 'mdi:loading';
			} else if (deleteBtnState?.state == ButtonState.Success) {
				return 'mdi:check';
			} else if (deleteBtnState?.state == ButtonState.Error) {
				return 'mdi:alert-circle';
			}

			return 'mdi:download';
		},
		getIconClass: () => {
			if (deleteBtnState?.state == ButtonState.Idle) {
				return 'text-xl inline-block';
			} else if (deleteBtnState?.state == ButtonState.Loading) {
				return 'text-xl inline-block animate-spin';
			} else if (deleteBtnState?.state == ButtonState.Success) {
				return 'text-xl inline-block';
			} else if (deleteBtnState?.state == ButtonState.Error) {
				return 'text-xl inline-block';
			}

			return 'text-xl inline-block';
		}
	};

	$: if (copyFilePath === undefined) copyFilePath = file.path;
</script>

{#if showModal}
	<Modal bind:showModal>
		<h1 slot="header" class="font-semibold text-2xl">{file.name}</h1>

		<div id="actions-box" class="mb-3">
			<button
				on:click={async () => {
					downloadBtnState.state = ButtonState.Loading;
					try {
						let data = await loadData(scope, file);
						let url = URL.createObjectURL(data);
						let a = document.createElement('a');
						a.href = url;
						a.download = file.name;
						a?.click();
						downloadBtnState.state = ButtonState.Success;
					} catch (e) {
						downloadBtnState.state = ButtonState.Error;
						downloadBtnState.error = e;
					}

					setTimeout(() => {
						downloadBtnState.state = ButtonState.Idle;
					}, 5000);
				}}
				class="text-white hover:text-gray-300 focus:outline-none mr-2"
			>
				<Icon icon={downloadBtnState?.getIcon()} class={downloadBtnState?.getIconClass()} />
				{#if downloadBtnState?.state == ButtonState.Idle}
					Download
				{:else if downloadBtnState?.state == ButtonState.Loading}
					Downloading...
				{:else if downloadBtnState?.state == ButtonState.Success}
					Downloaded!
				{:else if downloadBtnState?.state == ButtonState.Error}
					Failed to download: {downloadBtnState?.error?.toString() || 'Unknown error'}
				{/if}
			</button>
			{#await getScope()}
				<span class="opacity-70">Checking CDN</span>
			{:then scopes}
				<a
					href={`${scopes?.scopeData?.exposed_url}/${file.path}`}
					target="_blank"
					class="text-white hover:text-gray-300 focus:outline-none mr-2"
				>
					<Icon icon="mdi:open-in-new" class="text-xl inline-block" />
					Open in CDN
				</a>
			{:catch err}
				<ErrorComponent msg={err?.toString() || 'Failed to get CDN scopes'} />
			{/await}
			<button
				on:click={() => {
					copyFilePaneOpen = !copyFilePaneOpen;
				}}
				class="text-white hover:text-gray-300 focus:outline-none mr-2"
			>
				<Icon icon="mdi:rename" class="text-xl inline-block" />
				{#if copyFilePaneOpen}
					Close Copy Pane
				{:else}
					Copy/Move/Rename
				{/if}
			</button>
			<button
				on:click={async () => {
					deleteBtnState.state = ButtonState.Loading;
					try {
						await deleteFile();
					} catch (e) {
						deleteBtnState.state = ButtonState.Error;
						deleteBtnState.error = e;
					}

					setTimeout(() => {
						deleteBtnState.state = ButtonState.Idle;
					}, 5000);
				}}
				class="text-white hover:text-gray-300 focus:outline-none mr-2"
			>
				<Icon icon={deleteBtnState?.getIcon()} class={deleteBtnState?.getIconClass()} />
				{#if deleteBtnState?.state == ButtonState.Idle}
					Delete File
				{:else if deleteBtnState?.state == ButtonState.Loading}
					Deleting...
				{:else if deleteBtnState?.state == ButtonState.Success}
					Deleted!
				{:else if deleteBtnState?.state == ButtonState.Error}
					Failed to delete: {deleteBtnState?.error?.toString() || 'Unknown error'}
				{/if}
			</button>
		</div>

		{#if copyFilePaneOpen}
			<div id="copy-file">
				<InputText
					id="copy-file-name"
					label="New file name"
					placeholder="New file name"
					bind:value={copyFilePath}
					minlength={1}
					showErrors={false}
				/>
				<BoolInput
					id="copy-file-overwrite"
					label="Overwrite existing file"
					description="Overwrite any existing file with the same name"
					disabled={false}
					bind:value={copyFileSettingOverwrite}
				/>
				<BoolInput
					id="copy-file-delete-original"
					label="Delete original file"
					description="This is equivalent to a move operation, if enabled the original file will be deleted"
					disabled={false}
					bind:value={copyFileSettingDeleteOriginal}
				/>
				<ButtonReact
					color={Color.Themable}
					icon="mdi:rename-box"
					text="Copy/Move/Rename"
					onClick={copyFile}
					states={{
						loading: 'Modifying file...',
						success: 'Successfully modified file',
						error: 'Failed to modify file'
					}}
				/>
			</div>
		{/if}

		<h2 class="text-xl font-semibold">Preview</h2>
		{#await renderPreview(loadData, scope, file, previewBox)}
			<Icon icon="mdi:loading" class="inline animate-spin text-2xl" />
			<span class="text-xl">Loading Preview</span>
		{:catch err}
			<p class="text-red-500">{err?.toString()}</p>
		{/await}
		<div bind:this={previewBox} />

		<slot />
	</Modal>
{/if}
