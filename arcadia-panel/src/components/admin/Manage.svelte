<script lang="ts">
	import { error, success } from '$lib/toast';
	import GreyText from '../GreyText.svelte';
	import Modal from '../Modal.svelte';
	import ButtonReact from '../button/ButtonReact.svelte';
	import { Color } from '../button/colors';
	import {
		commonButtonReactStates,
		setupWarning,
		type WarningBox as WB
	} from '../warningbox/warningBox';
	import WarningBox from '../warningbox/WarningBox.svelte';
	import type { ManageSchema } from './types';
	import { title } from '$lib/strings';
	import { fetchCustomActions, fetchFields } from './logic';
	import Loading from '../Loading.svelte';
	import OrderedList from '../OrderedList.svelte';
	import ListItem from '../ListItem.svelte';
	import InputHandler from './InputHandler.svelte';
	import Icon from '@iconify/svelte';
	import CustomAction from './CustomAction.svelte';

	export let show: boolean = true;

	export let data: ManageSchema<any>;
	let pkey: string = '';
	let editData: { [key: string]: any } = data?.manageData || {};

	// Files support
	let fileKeys: string[];
	let fileData: { [key: string]: File } = {};

	let status: string[] = [];

	let warningBoxDelete: WB | undefined;
	let showWarningBoxDelete: boolean = false;

	$: {
		pkey = data?.schema?.getPrimaryKey('update');
		data?.schema?.onOpen('update', 'showComponent', editData);
		editData = data?.manageData || {};
	}

	const addStatus = (s: string) => {
		status.push(s);
		status = status;
	};

	const deleteObject = async () => {
		addStatus(`=> Deleting ${data?.schema?.name}`);
		try {
			await data?.schema?.delete({
				data: editData,
				files: fileData,
				addStatus
			});
		} catch (err) {
			addStatus(`Could not delete ${data?.schema?.name}: ${err}`);
			error(`Could not delete ${data?.schema?.name}: ${err}`);
			return false;
		}

		success(`Successfully deleted ${data?.schema?.name}`);
		return true;
	};

	const editObject = async () => {
		addStatus(`=> Editting ${data?.schema?.name}`);
		try {
			await data?.schema?.update({
				data: editData,
				files: fileData,
				addStatus
			});
		} catch (err) {
			addStatus(`Could not update ${data?.schema?.name}: ${err}`);
			error(`Could not update ${data?.schema?.name}: ${err}`);
			return false;
		}

		addStatus(`Successfully updated ${data?.schema?.name}`);
		success(`Successfully updated ${data?.schema?.name}`);
		return true;
	};

	const fetchStateAndSetupEditData = async () => {
		let res = await fetchFields('update', data?.schema?.fields);
		fileKeys = res.filter((f) => f.type == 'file').map((f) => f.id);
		return res;
	};

	const setupCustomActions = async () => {
		let res = await fetchCustomActions('update', data?.schema?.customActions || []);
		return res;
	};
</script>

{#if show}
	<Modal bind:showModal={show}>
		<h1 slot="header" class="font-semibold text-2xl">
			Editting {data?.schema?.name} for {pkey}
			{data?.manageData?.[pkey]}
		</h1>

		<h2 class="text-xl font-semibold">Edit {title(data?.schema?.name)} Entry</h2>

		{#await fetchStateAndSetupEditData()}
			<Loading msg="Loading field list" />
		{:then fields}
			{#each fields as field}
				<InputHandler {field} bind:data={editData} bind:fileData />
			{/each}

			{#if data?.schema?.getCaps()?.includes('update')}
				<ButtonReact
					color={Color.Themable}
					onClick={editObject}
					icon="mdi:plus"
					text={`Edit ${title(data?.schema?.name)}`}
					states={{
						loading: 'Editting entry...',
						success: 'Entry editted!',
						error: 'Failed to edit entry!'
					}}
				/>
			{:else}
				<p class="text-red-500">You do not have permission to edit this entry</p>
			{/if}

			{#await setupCustomActions()}
				<Icon icon="mdi:loading" class="inline animate-spin text-2xl" />
				<span class="text-xl">Loading Custom Actions</span>
			{:then actions}
				{#each actions as action}
					<CustomAction data={data?.manageData} {action} cap="update" bind:showContaining={show} />
				{/each}
			{:catch err}
				<p class="text-red-500">{err?.toString()}</p>
			{/await}
		{:catch err}
			<p class="text-red-500">{err?.toString()}</p>
		{/await}

		<h2 class="mt-4 text-xl font-semibold">Delete {title(data?.schema?.name)} Entry</h2>
		<GreyText>Note that this is IRREVERSIBLE</GreyText>

		{#if data?.schema?.getCaps()?.includes('delete')}
			<ButtonReact
				color={Color.Red}
				states={commonButtonReactStates}
				onClick={async () => {
					warningBoxDelete = data.schema.warningBox('delete', data.manageData, deleteObject);
					if (!warningBoxDelete) {
						error('Internal error: no warningBoxDelete found');
						return false;
					}
					setupWarning(warningBoxDelete);
					show = false;
					showWarningBoxDelete = true;
					return true;
				}}
				icon="mdi:trash-can-outline"
				text="Delete Entry"
			/>
		{:else}
			<p class="text-red-500">You do not have permission to delete this entry</p>
		{/if}

		{#if status?.length > 0}
			<OrderedList>
				{#each status as s}
					<ListItem>{s}</ListItem>
				{/each}
			</OrderedList>
		{/if}
	</Modal>
{/if}

<WarningBox bind:warningBox={warningBoxDelete} bind:show={showWarningBoxDelete} />
