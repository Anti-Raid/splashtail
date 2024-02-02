<script lang="ts">
    export let size: "normal" | "small" | "xs" = "normal";
</script>

{#if size == "normal"}
    <div class="col-span-8 grid grid-cols-1 gap-6 lg:grid-cols-2"><slot /></div>
{:else if size == "small"}
    <div class="col-span-4 grid grid-cols-1 gap-6 lg:grid-cols-3"><slot /></div>
{:else if size == "xs"}
    <div class="col-span-4 grid grid-cols-1 gap-6 lg:grid-cols-4"><slot /></div>
{/if}