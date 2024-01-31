<script lang="ts">
	import Icon from "@iconify/svelte";

    export let type: "loading" | "loading-big" | "error" | "success";
</script>

{#if type == "loading"}
    <div class="text-white text-2xl">
        <Icon icon="mdi:loading" class="animate-spin inline-block text-2xl" />
        <slot />
    </div>
{:else if type == "loading-big"}
    <div class="text-white text-4xl text-center">
        <Icon icon="mdi:loading" class="animate-spin block text-center m-auto" />
        <slot />
    </div>
{:else if type == "error"}
    <div class="text-red-500 text-2xl">
        <Icon icon="mdi:alert" class="inline-block text-3xl align-top" />
        <slot />
    </div>
{:else if type == "success"}
    <div class="text-green-500 text-2xl">
        <Icon icon="mdi:check" class="inline-block text-3xl align-top" />
        <slot />
    </div>
{/if}