<script lang="ts">
	import type { StaffPosition } from '$lib/generated/arcadia/StaffPosition';
	import { title } from '$lib/strings';
	import Icon from '@iconify/svelte';
	import ObjectRender from '../../../../components/ObjectRender.svelte';
	import SmallCard from '../../../../components/SmallCard.svelte';
	import GreyText from '../../../../components/GreyText.svelte';
	import Select from '../../../../components/inputs/select/Select.svelte';
	import { panelState } from '$lib/panelState';
	import { build, hasPerm } from '$lib/perms';
	import { error, success } from '$lib/toast';
	import logger from '$lib/logger';
	import { panelQuery } from '$lib/fetch';
	import { panelAuthState } from '$lib/panelAuthState';
	import ButtonReact from '../../../../components/button/ButtonReact.svelte';
	import { Color } from '../../../../components/button/colors';
	import InputText from '../../../../components/inputs/InputText.svelte';
	import MultiInput from '../../../../components/inputs/multi/simple/MultiInput.svelte';
	import ExtraLinks from '../../../../components/inputs/multi/extralinks/ExtraLinks.svelte';
	import IndexChooser from './IndexChooser.svelte';

	const allActions = {
		swap_index: ['ph:swap', 'Change Position'],
		set_index: ['mdi:code', 'Set Index'],
		edit: ['mdi:edit', 'Edit']
	} as const;

	type Action = keyof typeof allActions;

	let openAction: Action | null = null;

	const open = (action: Action) => {
		if (openAction == action) {
			openAction = null;
			return;
		}
		openAction = action;
	};

	const getTopPosition = (spl: StaffPosition[]) => {
		let topPosition: StaffPosition | null = null;

		for (let sp of spl) {
			if (!topPosition) {
				topPosition = sp;
				continue;
			}

			if (sp.index < topPosition.index) {
				topPosition = sp;
			}
		}

		return topPosition;
	};

	export let staffPositionList: StaffPosition[];
	export let staffPosition: StaffPosition;

	const getAllActions = (): Action[] => {
		if (!topUserPosition) {
			topUserPosition = getTopPosition($panelState?.staff_member?.positions || []);
		}

		let available: Action[] = [];

		for (let perm of Object.keys(allActions)) {
			if (topUserPosition?.index && staffPosition.index <= topUserPosition?.index) {
				continue;
			}
			if (
				hasPerm($panelState?.staff_member?.resolved_perms || [], build('staff_positions', perm))
			) {
				available.push(perm as Action);
			}
		}

		return available;
	};

	let topUserPosition: StaffPosition | null = getTopPosition(
		$panelState?.staff_member?.positions || []
	);
	let availableActions: Action[] = getAllActions();

	// Actions

	let swapIndex: number | null = null;
	let swapIndexProposed: string | undefined;
	const swapIndexExecute = async () => {
		if (!swapIndex) {
			error('Please select a position to swap with');
			return false;
		}

		let b = staffPositionList.find((sp) => sp.index == swapIndex);

		if (!b) {
			error('Invalid index');
			return false;
		}

		let res = await panelQuery({
			UpdateStaffPositions: {
				login_token: $panelAuthState?.loginToken || '',
				action: {
					SwapIndex: {
						a: staffPosition.id,
						b: b.id
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to swap index: ${err?.toString() || 'Unknown error'}`);
		}

		success('Swapped index!');
		return true;
	};

	// Set Index
	let staffPositionUpdateIndex: number | undefined = undefined;
	const setIndexExecute = async () => {
		if (!staffPositionUpdateIndex) {
			error('Please select an index');
			return false;
		}

		let res = await panelQuery({
			UpdateStaffPositions: {
				login_token: $panelAuthState?.loginToken || '',
				action: {
					SetIndex: {
						id: staffPosition.id,
						index: staffPositionUpdateIndex
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to set index: ${err?.toString() || 'Unknown error'}`);
		}

		success('Set index!');
		return true;
	};

	// Edit Position
	let editPosition = staffPosition;
	const editPositionExecute = async () => {
		let res = await panelQuery({
			UpdateStaffPositions: {
				login_token: $panelAuthState?.loginToken || '',
				action: {
					EditPosition: {
						id: editPosition.id || staffPosition.id,
						name: editPosition.name || staffPosition.name,
						role_id: editPosition.role_id || staffPosition.role_id,
						perms: editPosition.perms || staffPosition.perms,
						corresponding_roles:
							editPosition.corresponding_roles || staffPosition.corresponding_roles || []
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to edit position: ${err?.toString() || 'Unknown error'}`);
		}

		success('Edited position!');
		return true;
	};

	// Bindings
	$: {
		logger.info('swapIndexProposed', swapIndexProposed);
		if (swapIndexProposed) {
			let pInt = parseInt(swapIndexProposed);
			if (isNaN(pInt)) {
				error('Invalid index');
				swapIndex = null;
			} else {
				swapIndex = pInt;
			}
		} else {
			swapIndex = null;
		}
	}

	$: {
		topUserPosition = getTopPosition($panelState?.staff_member?.positions || []);
		availableActions = getAllActions();
	}
</script>

<SmallCard>
	<h1 class="text-2xl font-semibold">
		{title(staffPosition.name.replaceAll('_', ' '))}
		<span class="opacity-80">({staffPosition.name})</span>
	</h1>
	<details>
		<summary class="hover:cursor-pointer">View Advanced Information</summary>
		<ObjectRender object={staffPosition}></ObjectRender>
	</details>

	{#if availableActions.length > 0}
		<h2 class="mt-3 mb-1 text-xl">Actions</h2>
		<div class="mb-3 border rounded-md">
			{#each availableActions as action}
				<button
					on:click={() => {
						open(action);
					}}
					class="text-white hover:text-gray-300 focus:outline-none px-2 py-3 border-r"
				>
					<Icon icon={allActions[action][0]} class={'text-2xl inline-block align-bottom'} />
					{openAction == action ? 'Close' : allActions[action][1]}
				</button>
			{/each}
		</div>
	{/if}

	{#if openAction == 'swap_index'}
		<h1 class="text-2xl">Swap With Index</h1>
		<GreyText
			>A simple way to change position hierarchy is by swapping the permissions index with one that
			is lower (lower index = higher in hierarchy)</GreyText
		>

		<Select
			bind:value={swapIndexProposed}
			id="indexswap"
			label="Choose Permission To Swap With"
			choices={staffPositionList
				?.filter(
					(sp) =>
						sp.id != staffPosition.id &&
						(topUserPosition?.index || topUserPosition?.index == 0) &&
						topUserPosition.index < sp.index
				)
				.map((sp) => {
					return {
						label: `${title(sp.name.replaceAll('_', ' '))} (${sp.name}) [${sp.index}]`,
						value: sp.index.toString(),
						id: sp.id
					};
				})}
		/>

		<ButtonReact
			color={Color.Themable}
			icon="ph:swap"
			onClick={swapIndexExecute}
			states={{
				loading: 'Swapping index...',
				success: 'Swapped index!',
				error: 'Failed to swap index!'
			}}
			text="Swap Index"
		/>
	{:else if openAction == 'set_index'}
		<h1 class="text-2xl">Set Index</h1>
		<GreyText>
			You can set the index of a position to a specific number. This will change the hierarchy of
			the position.
		</GreyText>

		<IndexChooser {staffPositionList} bind:index={staffPositionUpdateIndex} />

		<ButtonReact
			color={Color.Themable}
			icon="mdi:code"
			onClick={setIndexExecute}
			states={{
				loading: 'Setting index...',
				success: 'Set index!',
				error: 'Failed to set index!'
			}}
			text="Set Index"
		/>
	{:else if openAction == 'edit'}
		<h1 class="text-2xl">Edit Position</h1>

		<InputText
			id="id"
			label="ID"
			value={editPosition.id}
			placeholder="ID cannot be changed"
			disabled
			minlength={0}
			showErrors={false}
		/>
		<InputText
			id="name"
			label="Name"
			bind:value={editPosition.name}
			placeholder="New name of the position"
			minlength={1}
			showErrors={false}
		/>
		<InputText
			id="role_id"
			label="Role ID"
			bind:value={editPosition.role_id}
			placeholder="New role id on the staff server of the position to set"
			minlength={1}
			showErrors={true}
		/>
		<MultiInput
			id="perms"
			title="Permissions"
			placeholder="Choose which permissions to add"
			label="Permission"
			showLabel={true}
			bind:values={editPosition.perms}
			minlength={1}
			showErrors={true}
		/>

		<ExtraLinks
			id="corresponding_roles"
			title="Corresponding Roles"
			placeholder="Server ID: Role ID"
			label="Corresponding Roles"
			bind:values={editPosition.corresponding_roles}
			minlength={1}
			showErrors={true}
		/>

		<ButtonReact
			color={Color.Themable}
			icon="mdi:edit"
			onClick={editPositionExecute}
			states={{
				loading: 'Editing position...',
				success: 'Edited position!',
				error: 'Failed to edit position!'
			}}
			text="Edit Position"
		/>
	{/if}
</SmallCard>
