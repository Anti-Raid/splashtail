<script lang="ts">
	export let Text: string;
	export let Margin: string;
</script>

<div class="relative mt-{Margin}">
	<div class="absolute inset-0 flex items-center">
		<div class="w-full border-b border-gray-300" />
	</div>

	<div class="relative flex justify-center">
		<span class="px-4 bg-slate-900 text-sm text-white font-bold">{Text}</span>
	</div>
</div>
