<script>
	import { page } from '$app/stores';
</script>

<div class="mt-8 h-full w-full p-2 rounded-md">
	{#if $page.error && $page.error.message}
		<h2 class="font-bold text-center text-white text-4xl">
			{$page.status}: {$page.error.message}
		</h2>
	{:else}
		<h2 class="font-semibold text-center text-white text-4xl">{$page.error}</h2>
	{/if}

	<p class="mt-2 text-center italic text-md text-white font-semibold">
		Oops! Something went wrong, This error has been logged!
	</p>

	<div class="flex justify-center items-center">
		<a class="mt-3 bg-white text-black font-bold p-2 rounded-md" href="/">Back Home!</a>
	</div>
</div>
