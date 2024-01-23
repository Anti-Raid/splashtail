<script lang="ts">
	import type { StaffPosition } from '$lib/generated/arcadia/StaffPosition';
	import { panelState } from '$lib/panelState';
	import { error } from '$lib/toast';
	import InputNumber from '../../../../components/inputs/InputNumber.svelte';
	import Label from '../../../../components/inputs/Label.svelte';
	import Select from '../../../../components/inputs/select/Select.svelte';

	export let index: number | undefined = undefined;
	export let staffPositionList: StaffPosition[];

	type IndexSelectChoice = 'manual' | 'above' | 'below' | '';
	let indexSelectChoice: IndexSelectChoice = '';

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

	let topUserPosition = getTopPosition($panelState?.staff_member?.positions || []);

	let above: string;
	let below: string;

	$: {
		if (above && indexSelectChoice == 'above') {
			let aboveInt = parseInt(above);
			if (isNaN(aboveInt)) {
				error('Above is not a number');
			}
			index = aboveInt;
		}

		if (below && indexSelectChoice == 'below') {
			let belowInt = parseInt(below);
			if (isNaN(belowInt)) {
				error('Below is not a number');
			}
			index = belowInt + 1;
		}
	}
</script>

<Label id="index" label="Index" />

<div id="index">
	<Select
		id="index-select-choice"
		label="Index Select Choice"
		bind:value={indexSelectChoice}
		choices={[
			{
				id: 'manual',
				value: 'manual',
				label: 'Manual Input'
			},
			{
				id: 'above',
				value: 'above',
				label: 'Above Position'
			},
			{
				id: 'below',
				value: 'below',
				label: 'Below Position'
			}
		]}
	/>

	{#if indexSelectChoice == 'manual'}
		<InputNumber
			id="index-manual"
			label="Index"
			placeholder="Lower means higher in hierarchy"
			minlength={0}
			showErrors={false}
			bind:value={index}
		/>
	{:else if indexSelectChoice == 'above'}
		<Select
			id="index-above"
			label="Above"
			bind:value={above}
			choices={staffPositionList
				.filter((sp) => sp.index > (topUserPosition?.index || 0))
				.map((sp) => {
					return {
						id: sp.id,
						value: sp.index.toString(),
						label: sp.name
					};
				})}
		/>
	{:else if indexSelectChoice == 'below'}
		<Select
			id="index-below"
			label="Below"
			bind:value={below}
			choices={staffPositionList
				.filter((sp) => sp.index >= (topUserPosition?.index || 0))
				.map((sp) => {
					return {
						id: sp.id,
						value: sp.index.toString(),
						label: sp.name
					};
				})}
		/>
	{/if}
	<p><span class="font-semibold">Selected Index</span> {index}</p>
</div>
