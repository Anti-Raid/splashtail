<script lang="ts">
	import type { StaffPosition } from '$lib/generated/arcadia/StaffPosition';
	import { title } from '$lib/strings';
	import Icon from '@iconify/svelte';
	import ObjectRender from '../../../../components/ObjectRender.svelte';
	import SmallCard from '../../../../components/SmallCard.svelte';
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
	import type { StaffMember } from '$lib/generated/arcadia/StaffMember';
	import UnorderedList from '../../../../components/UnorderedList.svelte';
	import ListItem from '../../../../components/ListItem.svelte';
	import BoolInput from '../../../../components/inputs/BoolInput.svelte';

	const allActions = {
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

	export let staffMember: StaffMember;

	const getAllActions = (): Action[] => {
		if (!topUserPosition) {
			topUserPosition = getTopPosition($panelState?.staff_member?.positions || []);
		}

		let available: Action[] = [];

		for (let perm of Object.keys(allActions)) {
			if (hasPerm($panelState?.staff_member?.resolved_perms || [], build('staff_members', perm))) {
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
	let editMember = staffMember;
	const editMemberExecute = async () => {
		let res = await panelQuery({
			UpdateStaffMembers: {
				login_token: $panelAuthState?.loginToken || '',
				action: {
					EditMember: {
						user_id: editMember.user_id,
						perm_overrides: editMember.perm_overrides || [],
						no_autosync: editMember.no_autosync || false,
						unaccounted: editMember.unaccounted || false
					}
				}
			}
		});

		if (!res.ok) {
			throw new Error('Failed to edit staff member');
		}

        success('Edited staff member');
		return true;
	};

	// Bindings
	$: {
		topUserPosition = getTopPosition($panelState?.staff_member?.positions || []);
		availableActions = getAllActions();
	}
</script>

<SmallCard>
	<h1 class="text-2xl font-semibold">
		{staffMember?.user?.display_name}
		<span class="opacity-80">[{staffMember?.user?.username}]</span>
	</h1>

	<h2 class="text-xl font-semibold">User ID</h2>

	<p>{staffMember?.user_id}</p>

	<h2 class="text-xl font-semibold">Positions</h2>

	{#each staffMember?.positions?.sort((a, b) => {
		// Sort based on index in ascending order
		return a.index - b.index;
	}) || [] as p, i}
		<UnorderedList>
			<ListItem>
				{title(p.name.replaceAll('_', ' '))}
				<span class="opacity-80">({p.name})</span>
				{#if i == 0}
					<span class="text-green-500"> (Top Position)</span>
				{/if}
			</ListItem>
		</UnorderedList>
	{/each}

	<details>
		<summary class="hover:cursor-pointer">View Advanced Information</summary>
		<ObjectRender object={staffMember}></ObjectRender>
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

	{#if openAction == 'edit'}
		<h1 class="text-2xl">Edit Member</h1>

		<InputText
			id="user_id"
			value={editMember?.user?.id}
			label="User ID"
			disabled={true}
			placeholder="User ID"
			minlength={0}
			showErrors={false}
		/>
		<MultiInput
			id="perm_overrides"
			bind:values={editMember.perm_overrides}
			title="Permission Overrides"
			label="Permission Overrides"
			placeholder="Permission Overrides"
			minlength={0}
			showErrors={false}
		/>
		<BoolInput
			id="no_autosync"
			bind:value={editMember.no_autosync}
			description="Whether or not the staff positions of this member should be synced when they change on the staff server, Leave enabled if unsure"
			label="No Autosync"
			disabled={false}
		/>
		<BoolInput
			id="unaccounted"
			bind:value={editMember.unaccounted}
			description="Whether or not this member has been accounted for correctly during sync (if they leave server with perm_overrides, this is set), Do not change unless you know what you're doing!"
			label="Unaccounted"
			disabled={false}
		/>

		<ButtonReact
			color={Color.Themable}
			icon="mdi:edit"
			onClick={editMemberExecute}
			states={{
				loading: 'Editing member...',
				success: 'Edited member!',
				error: 'Failed to edit member!'
			}}
			text="Edit Member"
		/>
	{/if}
</SmallCard>
