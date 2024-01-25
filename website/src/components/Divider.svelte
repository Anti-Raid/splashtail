<script lang="ts">
	export let text: string;
	export let margin: string;
</script>

<div class={`relative mt-${margin}`}>
	<div class="absolute inset-0 flex items-center">
		<div class="w-full m-10 border-b border-gray-300" />
	</div>

	<div class="relative flex justify-center">
		<span class="px-4 bg-slate-900 text-sm text-white font-bold">{text}</span>
	</div>
</div>
