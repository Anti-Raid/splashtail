<script lang="ts">
	import { error } from '$lib/toast';
	import Modal from '../Modal.svelte';
	import ButtonReact from '../button/ButtonReact.svelte';
	import { Color } from '../button/colors';
	import InputText from '../inputs/InputText.svelte';
	import type { WarningBox } from './warningBox';

	export let show: boolean = false;
	export let warningBox: WarningBox | null = null;
</script>

{#if show && warningBox && warningBox.nonce}
	<Modal bind:showModal={show}>
		<h1 slot="header" class="font-semibold text-2xl">{warningBox.header}</h1>

		<p class="font-semibold text-xl">{warningBox.text}</p>

		<p>
			To confirm, please type the following: <code class="select-none cursor-pointer"
				>{warningBox.nonce}</code
			>
		</p>

		<div class="mb-5" />

		<InputText
			id="wb-input"
			label="Are you sure? This is dangerous"
			placeholder="Dangerous nilly!"
			bind:value={warningBox.inputtedText}
			minlength={1}
			showErrors={false}
		/>

		<div class="mb-5" />

		<ButtonReact
			color={Color.Red}
			states={{
				loading: warningBox.buttonStates?.loading,
				success: warningBox.buttonStates?.success,
				error: warningBox.buttonStates?.error
			}}
			onClick={async () => {
				if (!warningBox) {
					error('Internal error: no warningBox found');
					return false;
				}

				if (warningBox.inputtedText != warningBox.nonce) {
					error('Please type the nonce correctly');
					return false;
				}

				let res = await warningBox.onConfirm();

				if (res) {
					show = false;
				}
				return res;
			}}
			icon="mdi:trash-can-outline"
			text={warningBox.buttonStates?.normal}
		/>
	</Modal>
{/if}
