<script lang="ts">
	export let msg: string;
</script>

<section>
	<h3 aria-live="polite" class="text-center text-6xl font-semibold p-10">
		Welcome to Infinity Panel
	</h3>
	<svg
		aria-labelledby="___panelLoad"
		class="ml-auto mr-auto text-center animate-spin h-48 w-48"
		viewBox="0 0 66 66"
		xmlns="http://www.w3.org/2000/svg"
	>
		<circle
			class="path"
			fill="transparent"
			stroke-width="6.5"
			cx="33"
			cy="33"
			r="30"
			stroke="url(#gradient)"
		/>
		<linearGradient id="gradient">
			<stop offset="1%" stop-color="#42d179" stop-opacity="1" />
			<stop offset="80%" stop-color="#42d179" stop-opacity="0" />
		</linearGradient>
		<title id="___panelLoad">Loading the panel, please wait</title>
	</svg>
	<p class="mt-3 text-4xl font-semibold text-center dark:text-white">{msg}</p>
</section>
