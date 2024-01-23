<script lang="ts">
	import { panelQuery } from '$lib/fetch';
	import type { StaffPosition } from '$lib/generated/arcadia/StaffPosition';
	import { panelAuthState } from '$lib/panelAuthState';
	import Loading from '../../../../components/Loading.svelte';
	import ErrorComponent from '../../../../components/Error.svelte';
	import StaffPositionCard from './StaffPositionCard.svelte';
	import { panelState } from '$lib/panelState';
	import { build, hasPerm } from '$lib/perms';
	import Icon from '@iconify/svelte';
	import InputText from '../../../../components/inputs/InputText.svelte';
	import MultiInput from '../../../../components/inputs/multi/simple/MultiInput.svelte';
	import IndexChooser from './IndexChooser.svelte';
	import { error, success } from '$lib/toast';
	import ButtonReact from '../../../../components/button/ButtonReact.svelte';
	import { Color } from '../../../../components/button/colors';
	import Select from '../../../../components/inputs/select/Select.svelte';
	import { title } from '$lib/strings';
	import ExtraLinks from '../../../../components/inputs/multi/extralinks/ExtraLinks.svelte';

	const allActions = {
		create: ['mdi:plus', 'Create Position'],
		delete: ['mdi:delete', 'Delete Position']
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

	const getAllActions = (): Action[] => {
		let available: Action[] = [];

		for (let perm of Object.keys(allActions)) {
			if (
				hasPerm($panelState?.staff_member?.resolved_perms || [], build('staff_positions', perm))
			) {
				available.push(perm as Action);
			}
		}

		return available;
	};

	let availableActions: Action[] = getAllActions();

	const fetchStaffPositionList = async () => {
		let res = await panelQuery({
			UpdateStaffPositions: {
				login_token: $panelAuthState?.loginToken || '',
				action: 'ListPositions'
			}
		});

		if (!res.ok) {
			throw new Error('Failed to fetch staff position list');
		}

		let staffPositionList: StaffPosition[] = await res.json();

		return {
			staffPositionList
		};
	};

	// Actions
	let createPosition = {
		name: '',
		role_id: '',
		index: 0,
		perms: [],
		corresponding_roles: []
	};
	const createPositionExecute = async () => {
		let res = await panelQuery({
			UpdateStaffPositions: {
				login_token: $panelAuthState?.loginToken || '',
				action: {
					CreatePosition: {
						name: createPosition.name,
						role_id: createPosition.role_id,
						index: createPosition.index,
						perms: createPosition.perms,
						corresponding_roles: createPosition.corresponding_roles || []
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to create staff position: ${err?.toString() || 'Unknown error'}`);
		}

		success('Created staff position!');
		return true;
	};

	let deletePosition: string = '';
	const deletePositionExecute = async () => {
		if (!deletePosition) {
			error('Please select a position to delete');
			return false;
		}

		let res = await panelQuery({
			UpdateStaffPositions: {
				login_token: $panelAuthState?.loginToken || '',
				action: {
					DeletePosition: {
						id: deletePosition
					}
				}
			}
		});

		if (!res.ok) {
			let err = await res.text();
			throw new Error(`Failed to delete staff position: ${err?.toString() || 'Unknown error'}`);
		}

		success('Deleted staff position!');
		return true;
	};
</script>

<h1 class="text-3xl font-semibold">Staff Positions</h1>

{#await fetchStaffPositionList()}
	<Loading msg={'Loading staff positions...'} />
{:then { staffPositionList }}
	{#if availableActions?.length}
		<h2 class="mt-3 mb-1 text-xl">Actions</h2>
		<div class="mb-7 border rounded-md">
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

	{#if openAction}
		<div class="mb-7 border rounded-md p-3">
			{#if openAction == 'create'}
				<h1 class="text-2xl">Create Position</h1>
				<InputText
					id="name"
					label="Name"
					bind:value={createPosition.name}
					placeholder="New name of the position"
					minlength={1}
					showErrors={false}
				/>
				<InputText
					id="role_id"
					label="Role ID"
					bind:value={createPosition.role_id}
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
					bind:values={createPosition.perms}
					minlength={1}
					showErrors={true}
				/>
				<IndexChooser bind:index={createPosition.index} {staffPositionList} />
				<ExtraLinks
					id="corresponding_roles"
					title="Corresponding Roles"
					placeholder="Server ID: Role ID"
					label="Corresponding Roles"
					bind:values={createPosition.corresponding_roles}
					minlength={1}
					showErrors={true}
				/>

				<ButtonReact
					color={Color.Themable}
					icon="mdi:plus"
					onClick={createPositionExecute}
					states={{
						loading: 'Creating position...',
						success: 'Created position!',
						error: 'Failed to create position!'
					}}
					text="Create Position"
				/>
			{:else if openAction == 'delete'}
				<h1 class="text-2xl">Delete Position</h1>
				<p class="text-red-500">
					Warning: This will delete the position and remove it from all staff members.
				</p>
				<Select
					id="delete-position"
					label="Position"
					bind:value={deletePosition}
					choices={staffPositionList.map((sp) => {
						return {
							id: sp.id,
							value: sp.id,
							label: `${title(sp.name.replaceAll('_', ' '))} (${sp.name}) [${sp.index}]`
						};
					})}
				/>

				<ButtonReact
					color={Color.Red}
					icon="mdi:delete"
					onClick={deletePositionExecute}
					states={{
						loading: 'Deleting position...',
						success: 'Deleted position!',
						error: 'Failed to delete position!'
					}}
					text="Delete Position"
				/>
			{/if}
		</div>
	{/if}

	{#each staffPositionList as staffPosition}
		<StaffPositionCard {staffPosition} {staffPositionList}></StaffPositionCard>
	{/each}
{:catch error}
	<ErrorComponent msg={error?.toString() || 'Unknown erro'} />
{/await}
