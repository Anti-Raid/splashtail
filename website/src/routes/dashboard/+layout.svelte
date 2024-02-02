<script lang="ts">
	import AuthBoundary from "../../components/dashboard/AuthBoundary.svelte";

</script>

<AuthBoundary>
    <slot />
</AuthBoundary>