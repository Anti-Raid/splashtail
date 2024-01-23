<script lang="ts">
	import type { RPCField } from '$lib/generated/arcadia/RPCField';
	import InputNumber from '../inputs/InputNumber.svelte';

	export let value: [number, string] = [0, 'hour'];
	export let field: RPCField;
</script>

<div class="grid grid-cols-2 gap-4">
	<div>
		<InputNumber
			id={field.id}
			label={field.label}
			placeholder={`Number of ${value[1]}s`}
			bind:value={value[0]}
			minlength={5}
		/>
	</div>
	<div>
		<label for={field.id} class="text-white text-xl text-left">Unit</label>
		<select
			id={field.id}
			class="w-full mx-auto mt-2 flex transition duration-200 hover:bg-slate-900 bg-black bg-opacity-100 text-white focus:text-themable-400 rounded-xl border border-themable-200 focus:border-themable-400 focus:outline-none py-4 px-6"
			bind:value={value[1]}
		>
			<option value="hour">Hour</option>
			<option value="day">Day</option>
			<option value="week">Week</option>
			<option value="month">Month</option>
			<option value="year">Year</option>
		</select>
	</div>
</div>
