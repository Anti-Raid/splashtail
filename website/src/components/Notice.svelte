<script lang="ts">
	export let Description: string;
</script>

<div class="mt-3 p-2 bg-indigo-400 rounded-md">
	<h1 class="flex text-xl text-white font-bold tracking-tight">
		<i class="fa-regular fa-lightbulb mt-1 pr-2" /> NOTE
	</h1>
	<p class="mt-2 block text-white font-semibold xl:inline">
		{@html Description}
	</p>
</div>
