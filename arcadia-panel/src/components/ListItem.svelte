<script lang="ts">
	export let className: string | null = '';
</script>

<li class="mb-2 border-gray-900 break-all break-words {className}">
	<slot />
</li>
