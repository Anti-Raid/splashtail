<script lang="ts">
	export let value: boolean = false;

	export let disabled: boolean;
	export let required: boolean = false;
	export let id: string;
	export let label: string;
	export let description: string;
</script>

<div class="mt-4 col-span-9 pt-2 lg:pt-0 justify-middle items-center align-bottom text-lg flex">
	<input
		{id}
		name={id}
		type="checkbox"
		bind:checked={value}
		aria-disabled={disabled ? true : false}
		disabled={disabled ? true : false}
		required={required ? true : false}
		aria-required={required ? true : false}
		class={disabled ? 'cursor-not-allowed bg-slate-400' : 'cursor-pointer'}
	/>
	<label for={id} class="text-white text-left">
		<span class="text-white text-left ml-1">
			<span class="text-yellow-200 hover:cursor-pointer font-semibold">
				{label}
			</span>
		</span>
	</label>
</div>
<p class="opacity-80 text-sm">{description}</p>
