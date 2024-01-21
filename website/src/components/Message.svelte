<script lang="ts">
	import Icon from "@iconify/svelte";

    export let type: "loading" | "error" | "success";
</script>

{#if type == "loading"}
    <div class="text-white">
        <Icon icon="mdi:loading" class="animate-spin inline-block" />
        <slot />
    </div>
{:else if type == "error"}
    <div class="text-red-500">
        <Icon icon="mdi:alert" class="inline-block" />
        <slot />
    </div>
{:else if type == "success"}
    <div class="text-green-500">
        <Icon icon="mdi:check" class="inline-block" />
        <slot />
    </div>
{/if}