<script lang="ts">
	import InputSm from '../../InputText.svelte';

	export let title: string;
	export let placeholder: string;
	export let minlength: number;
	export let showErrors: boolean = false;
	export let i: number;
	export let value: [string, string];
	export let required: boolean = true;
	export let disabled: boolean = false;

	// Get key and value from value
	let key = value[0];
	let val = value[1];

	// Update value when key or value changes
	$: value = [key, val];
</script>

<div class="flex flex-row items-center align-items justify-center">
	<div class="lg:pt-0 mr-5 block w-1/2">
		<InputSm
			id={i.toString() + '-k'}
			label={title + ' ' + (i + 1) + ' Key'}
			{placeholder}
			bind:value={key}
			{minlength}
			{showErrors}
			{required}
			{disabled}
		/>
	</div>
	<div class="lg:pt-0 block w-1/2">
		<InputSm
			id={i.toString() + '-v'}
			label={title + ' ' + (i + 1) + ' Value'}
			{placeholder}
			bind:value={val}
			{minlength}
			{showErrors}
			{required}
			{disabled}
		/>
	</div>
</div>
