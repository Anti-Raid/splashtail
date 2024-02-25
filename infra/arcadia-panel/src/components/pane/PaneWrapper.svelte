<script lang="ts">
	import PaddedPage from './PaddedPage.svelte';
</script>

<PaddedPage>
	<div class="w-full h-max grid grid-cols-1 lg:grid-cols-12 lg:gap-x-4"><slot /></div>
</PaddedPage>
