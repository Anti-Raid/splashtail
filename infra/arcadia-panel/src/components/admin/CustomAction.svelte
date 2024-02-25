<script lang="ts">
	import GreyText from '../GreyText.svelte';
	import ButtonReact from '../button/ButtonReact.svelte';
	import type { Capability, CustomAction, Field, Schema } from './types';
	import { setupWarning, type WarningBox as WB } from '../warningbox/warningBox';
	import { error } from '$lib/toast';

	export let data: any;
	export let action: CustomAction<any>;
	export let cap: Capability;
	export let showContaining: boolean;

	let div: HTMLDivElement;

	let warningBox: WB | undefined;
	let showWarningBox: boolean = false;
</script>

<h2 class="mt-4 text-xl font-semibold">{action.label}</h2>
<GreyText>{action.helpText}</GreyText>

<ButtonReact
	color={action.button.color}
	icon={action.button.icon}
	states={action.button.states}
	text={action.button.states.normal}
	onClick={async () => {
		if (action?.warningBox) {
			let warningBox = action?.warningBox(cap, data, div, async () => {
				return await action.action(cap, data, div);
			});
			if (!warningBox) {
				error('Internal error: no warningBoxDelete found');
				return false;
			}
			setupWarning(warningBox);
			showContaining = false;
			showWarningBox = true;
			return true;
		}

		return await action.action(cap, data, div);
	}}
/>

<div class="mt-4" bind:this={div} />
