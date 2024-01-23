<script lang="ts">
	import { error, success } from '$lib/toast';
	import Modal from '../Modal.svelte';
	import ButtonReact from '../button/ButtonReact.svelte';
	import { Color } from '../button/colors';
	import type { Schema } from './types';
	import { title } from '$lib/strings';
	import { fetchFields } from './logic';
	import Loading from '../Loading.svelte';
	import OrderedList from '../OrderedList.svelte';
	import ListItem from '../ListItem.svelte';
	import InputHandler from './InputHandler.svelte';

	export let schema: Schema<any>;

	let showActionsModal: boolean = false;
	let createData: { [key: string]: any } = {};
	let pkey: string = '';

	// Files support
	let fileKeys: string[];
	let fileData: { [key: string]: File } = {};

	let status: string[] = [];
	const addStatus = (s: string) => {
		status.push(s);
		status = status;
	};

	const addObject = async () => {
		addStatus(`=> Adding a ${schema?.name}`);
		try {
			await schema?.create({
				data: createData,
				files: fileData,
				addStatus
			});
		} catch (err) {
			addStatus(`Could not add ${schema?.name}: ${err}`);
			error(`Could not add ${schema?.name}: ${err}`);
			return false;
		}

		addStatus(`Successfully added ${schema?.name}`);
		success(`Successfully added ${schema?.name}`);
		return true;
	};

	const fetchStateAndSetupAddData = async () => {
		let res = await fetchFields('create', schema?.fields);

		for (let k of res) {
			switch (k.type) {
				case 'text[]':
					createData[k.id] = [];
					break;
				case 'file':
					if (!fileKeys?.length) fileKeys = [];
					fileKeys.push(k.id);
			}
		}

		fileKeys = res.filter((f) => f.type == 'file').map((f) => f.id);

		return res;
	};

	$: {
		pkey = schema?.getPrimaryKey('create');
		schema?.onOpen('create', 'showComponent', null);
	}
</script>

<button
	class="inline-block py-4 px-3 text-xl text-themable-400 hover:bg-slate-700"
	on:click={() => {
		showActionsModal = true;
	}}
>
	Add {title(schema?.name)}
</button>

{#if showActionsModal}
	<Modal bind:showModal={showActionsModal}>
		<h1 slot="header" class="font-semibold text-2xl">Add a {schema?.name}</h1>

		<h2 class="text-xl font-semibold">Add {title(schema?.name)} Entry</h2>

		{#await fetchStateAndSetupAddData()}
			<Loading msg="Loading field list" />
		{:then fields}
			{#each fields as field}
				<InputHandler {field} bind:data={createData} bind:fileData />
			{/each}

			<ButtonReact
				color={Color.Themable}
				onClick={addObject}
				icon="mdi:plus"
				text={`Add ${title(schema?.name)}`}
				states={{
					loading: 'Adding entry...',
					success: 'Entry added!',
					error: 'Failed to add entry!'
				}}
			/>
		{:catch err}
			<p class="text-red-500">{err?.toString()}</p>
		{/await}

		{#if status?.length > 0}
			<OrderedList>
				{#each status as s}
					<ListItem>{s}</ListItem>
				{/each}
			</OrderedList>
		{/if}
	</Modal>
{/if}
