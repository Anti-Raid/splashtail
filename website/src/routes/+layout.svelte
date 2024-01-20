<script lang="ts">
	import '../app.css';
	import 'sweetalert2/dist/sweetalert2.min.css';
	import Header from '../components/Header.svelte';
</script>

<Header />

<div class="m-4">
	<slot />
</div>
