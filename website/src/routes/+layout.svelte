<script lang="ts">
	import '../app.postcss';
	import '$lib/styles/customColors.css';
	import Header from '../components/Header.svelte';
</script>

<Header />

<main class="p-3">
	<slot />
</main>	

