<script lang="ts">
	import '../app.postcss';
	import '$lib/styles/customColors.css';
	import Header from '../components/Header.svelte';
</script>

<Header />

<main class="p-3 mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
	<slot />
</main>	

